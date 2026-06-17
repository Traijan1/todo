mod actions;
mod schema;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::sse::{Event, Sse},
};
use futures::Stream;
use loco_rs::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::models::_entities::users;

lazy_static::lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<String, mpsc::Sender<Event>>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Deserialize)]
pub struct McpParams {
    pub session_id: String,
}

// ── Auth helper ──────────────────────────────────────────────────────────────

async fn extract_mcp_user(ctx: &AppContext, headers: &HeaderMap) -> Result<users::Model> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| Error::Unauthorized("Missing Authorization: Bearer <mcp_token> header".into()))?;
    users::Model::find_by_api_key(&ctx.db, token)
        .await
        .map_err(|_| Error::Unauthorized("Invalid MCP token".into()))
}

// ── Legacy SSE transport (Claude Code CLI) ───────────────────────────────────

pub async fn sse_handler() -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<Event>(100);

    let endpoint_url = format!("/api/mcp/messages?session_id={}", session_id);
    let _ = tx
        .send(Event::default().event("endpoint").data(endpoint_url))
        .await;

    let mut sessions = SESSIONS.lock().await;
    sessions.insert(session_id, tx);

    let stream = ReceiverStream::new(rx).map(Ok);
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

pub async fn message_handler(
    State(ctx): State<AppContext>,
    Query(params): Query<McpParams>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response> {
    let sessions = SESSIONS.lock().await;
    let tx = sessions
        .get(&params.session_id)
        .ok_or_else(|| Error::NotFound)?;

    let id = payload.get("id").cloned();
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let response = match method {
        "initialize" => handle_initialize(id, &payload),
        "tools/list" => handle_tools_list(id),
        "tools/call" => {
            let user = extract_mcp_user(&ctx, &headers).await?;
            handle_tools_call(&ctx, &user, id, &payload).await?
        }
        m if m.starts_with("notifications/") => {
            return Ok((axum::http::StatusCode::ACCEPTED, "").into_response());
        }
        _ => error_response(id, -32601, "Method not found"),
    };

    let event = Event::default()
        .event("message")
        .data(serde_json::to_string(&response).unwrap());
    let _ = tx.send(event).await;

    format::empty()
}

// ── Streamable HTTP transport (claude.ai web / Codex) ───────────────────────

/// GET /api/mcp/http — opens the server→client SSE stream.
/// rmcp clients (e.g. Codex) establish this channel after initialize.
/// We don't send server-initiated messages, but we must respond with SSE
/// so rmcp's worker doesn't crash when it can't connect (otherwise the
/// static-file fallback serves index.html, the worker dies, and the
/// initialized notification fails with "Transport channel closed").
pub async fn http_get_handler() -> impl IntoResponse {
    // Send an immediate comment so nginx/proxies flush the buffer and rmcp's
    // StreamableHttpClientWorker sees the connection as live before it tries
    // to send `notifications/initialized`. Without this initial event, the
    // 15-second keep-alive gap causes rmcp to consider the transport dead.
    let initial = futures::stream::once(async {
        Ok::<_, Infallible>(Event::default().comment("connected"))
    });
    let stream = initial.chain(futures::stream::pending());

    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(5))
            .text("ping"),
    );

    // X-Accel-Buffering: no tells nginx not to buffer SSE events,
    // which would otherwise delay them until its buffer fills.
    (
        [(
            axum::http::header::HeaderName::from_static("x-accel-buffering"),
            axum::http::HeaderValue::from_static("no"),
        )],
        sse,
    )
}

pub async fn http_handler(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response> {
    // Handle JSON-RPC batch (array of messages)
    if let Some(messages) = payload.as_array() {
        // Only extract user if there are tools/call messages to avoid unnecessary DB lookups
        let needs_auth = messages.iter().any(|m| m.get("method").and_then(|v| v.as_str()) == Some("tools/call"));
        let user = if needs_auth {
            Some(extract_mcp_user(&ctx, &headers).await?)
        } else {
            None
        };
        let mut responses = Vec::new();
        for msg in messages {
            let id = msg.get("id").cloned();
            let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
            match method {
                "initialize" => responses.push(handle_initialize(id, msg)),
                "tools/list" => responses.push(handle_tools_list(id)),
                "tools/call" => {
                    let u = user.as_ref().ok_or_else(|| Error::Unauthorized("Unreachable".into()))?;
                    responses.push(handle_tools_call(&ctx, u, id, msg).await?);
                }
                m if m.starts_with("notifications/") => { /* 202 sent below */ }
                _ => {
                    if id.is_some() {
                        responses.push(error_response(id, -32601, "Method not found"));
                    }
                }
            }
        }
        return if responses.is_empty() {
            Ok((axum::http::StatusCode::ACCEPTED, "").into_response())
        } else {
            format::json(Value::Array(responses))
        };
    }

    // Single message
    let id = payload.get("id").cloned();
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let response = match method {
        "initialize" => handle_initialize(id, &payload),
        "tools/list" => handle_tools_list(id),
        "tools/call" => {
            let user = extract_mcp_user(&ctx, &headers).await?;
            handle_tools_call(&ctx, &user, id, &payload).await?
        }
        m if m.starts_with("notifications/") => {
            return Ok((axum::http::StatusCode::ACCEPTED, "").into_response());
        }
        _ => error_response(id, -32601, "Method not found"),
    };

    format::json(response)
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn handle_initialize(id: Option<Value>, payload: &Value) -> Value {
    // Echo the client's requested protocol version if it's one we support,
    // so clients that send 2025-03-26 don't get a version mismatch error.
    let client_version = payload
        .get("params")
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");

    let version = match client_version {
        "2025-03-26" | "2025-06-18" => client_version,
        _ => "2024-11-05",
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": version,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "Todo MCP", "version": "1.0.0" }
        }
    })
}

fn handle_tools_list(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": schema::get_tools_list() }
    })
}

async fn handle_tools_call(ctx: &AppContext, user: &users::Model, id: Option<Value>, payload: &Value) -> Result<Value> {
    let tool_name = payload
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let default_args = json!({});
    let args = payload
        .get("params")
        .and_then(|p| p.get("arguments"))
        .unwrap_or(&default_args);

    let result = match tool_name {
        // Projects
        "get_projects"    => actions::get_projects(ctx, user).await?,
        "add_project"     => actions::add_project(ctx, args, user).await?,
        "update_project"  => actions::update_project(ctx, args).await?,
        "delete_project"  => actions::delete_project(ctx, args).await?,
        // Boards
        "get_boards"      => actions::get_boards(ctx, args).await?,
        "add_board"       => actions::add_board(ctx, args).await?,
        "update_board"    => actions::update_board(ctx, args).await?,
        "delete_board"    => actions::delete_board(ctx, args).await?,
        "reorder_boards"  => actions::reorder_boards(ctx, args).await?,
        // Todos
        "get_todos"       => actions::get_todos(ctx, args).await?,
        "get_todo"        => actions::get_todo(ctx, args).await?,
        "add_todo"        => actions::add_todo(ctx, args, user).await?,
        "update_todo"     => actions::update_todo(ctx, args, user).await?,
        "delete_todo"     => actions::delete_todo(ctx, args).await?,
        "reorder_todos"   => actions::reorder_todos(ctx, args).await?,
        // Tags
        "get_tags"             => actions::get_tags(ctx, args).await?,
        "create_tag"           => actions::create_tag(ctx, args).await?,
        "update_tag"           => actions::update_tag(ctx, args).await?,
        "delete_tag"           => actions::delete_tag(ctx, args).await?,
        "add_tag_to_todo"      => actions::add_tag_to_todo(ctx, args).await?,
        "remove_tag_from_todo" => actions::remove_tag_from_todo(ctx, args).await?,
        // Comments
        "get_comments"         => actions::get_comments(ctx, args).await?,
        "add_comment"          => actions::add_comment(ctx, args, user).await?,
        _ => return Ok(error_response(id, -32601, "Tool not found")),
    };

    Ok(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn error_response(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/mcp")
        .add("/http", get(http_get_handler).post(http_handler))  // Streamable HTTP (claude.ai / Codex)
        .add("/sse", get(sse_handler))                           // Legacy SSE (Claude Code CLI)
        .add("/messages", post(message_handler))
}
