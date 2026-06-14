mod actions;
mod schema;

use axum::{
    extract::{Query, State},
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

lazy_static::lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<String, mpsc::Sender<Event>>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Deserialize)]
pub struct McpParams {
    pub session_id: String,
}

pub async fn sse_handler() -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<Event>(100);

    // Initial event telling the client where to send POST messages
    let endpoint_url = format!("/api/mcp/messages?session_id={}", session_id);
    let _ = tx
        .send(Event::default().event("endpoint").data(endpoint_url))
        .await;

    // Store the sender so POST requests can find it
    let mut sessions = SESSIONS.lock().await;
    sessions.insert(session_id, tx);

    let stream = ReceiverStream::new(rx).map(Ok);
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

pub async fn message_handler(
    State(ctx): State<AppContext>,
    Query(params): Query<McpParams>,
    Json(payload): Json<Value>,
) -> Result<Response> {
    let sessions = SESSIONS.lock().await;
    let tx = sessions
        .get(&params.session_id)
        .ok_or_else(|| Error::NotFound)?;

    let id = payload.get("id").cloned();
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let response = match method {
        "initialize" => handle_initialize(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(&ctx, id, &payload).await?,
        _ => error_response(id, -32601, "Method not found"),
    };

    // Send the response back through the SSE stream
    let event = Event::default()
        .event("message")
        .data(serde_json::to_string(&response).unwrap());

    let _ = tx.send(event).await;

    format::empty()
}

fn handle_initialize(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "Loco Todo MCP",
                "version": "1.0.0"
            }
        }
    })
}

fn handle_tools_list(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": schema::get_tools_list()
        }
    })
}

async fn handle_tools_call(ctx: &AppContext, id: Option<Value>, payload: &Value) -> Result<Value> {
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
        "get_projects" => actions::get_projects(ctx).await?,
        "get_todos" => actions::get_todos(ctx, args).await?,
        "get_todo" => actions::get_todo(ctx, args).await?,
        "get_boards" => actions::get_boards(ctx, args).await?,
        "add_todo" => actions::add_todo(ctx, args).await?,
        "add_board" => actions::add_board(ctx, args).await?,
        "get_tags" => actions::get_tags(ctx).await?,
        "add_tag_to_todo" => actions::add_tag_to_todo(ctx, args).await?,
        "remove_tag_from_todo" => actions::remove_tag_from_todo(ctx, args).await?,
        _ => return Ok(error_response(id, -32601, "Tool not found")),
    };

    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
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
        .add("/sse", get(sse_handler))
        .add("/messages", post(message_handler))
}
