#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use crate::models::{
    _entities::projects,
    boards::{self, BoardParams},
    todos::{self, TodoParams},
};
use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse},
};
use futures::Stream;
use loco_rs::prelude::*;
use serde::Deserialize;
use serde_json::json;
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
    Json(payload): Json<serde_json::Value>,
) -> Result<Response> {
    let sessions = SESSIONS.lock().await;
    let tx = sessions
        .get(&params.session_id)
        .ok_or_else(|| Error::NotFound)?;

    // Basic MCP JSON-RPC handling
    let id = payload.get("id").cloned();
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let response = match method {
        "initialize" => {
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
        "tools/list" => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "get_projects",
                            "description": "List all todo projects",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "get_todos",
                            "description": "List all todos of a project",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_pid": {
                                        "type": "string",
                                        "description": "The UUID (PID) of the project"
                                    }
                                }
                            }
                        },
                        {
                            "name": "get_todo",
                            "description": "Retrieve information about a specific todo",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "todo_pid": {
                                        "type": "string",
                                        "description": "The UUID (PID) of the todo"
                                    }
                                }
                            }
                        },
                        {
                            "name": "get_boards",
                            "description": "List all boards of a project",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_pid": {
                                        "type": "string",
                                        "description": "The UUID (PID) of the project"
                                    }
                                }
                            }
                        },
                        {
                            "name": "add_todo",
                            "description": "Create a todo for a project board",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "board_pid": {
                                        "type": "string",
                                        "description": "The UUID (PID) of the board"
                                    },
                                    "title": {
                                        "type": "string",
                                        "description": "The title of the todo"
                                    },
                                    "details": {
                                        "type": "string",
                                        "description": "The details of the todo (optional)"
                                    }
                                },
                                "required": ["board_pid", "title"]
                            }
                        },
                        {
                            "name": "add_board",
                            "description": "Create a board for a project",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_pid": {
                                        "type": "string",
                                        "description": "The UUID (PID) of the project"
                                    },
                                    "title": {
                                        "type": "string",
                                        "description": "The title of the board"
                                    },
                                }
                            }
                        },
                    ]
                }
            })
        }
        "tools/call" => {
            let tool_name = payload
                .get("params")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");

            match tool_name {
                "get_projects" => {
                    let projects = projects::Entity::find().all(&ctx.db).await.map_err(|e| {
                        tracing::error!("Failed to fetch projects for MCP: {:?}", e);
                        e
                    })?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Projects found: {:?}", projects)
                                }
                            ]
                        }
                    })
                }
                "get_todos" => {
                    let project_pid = payload
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .and_then(|p| p.get("project_pid"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let todos = todos::Model::find_by_project_pid(&ctx.db, project_pid).await?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Todos found: {:?}", todos)
                                }
                            ]
                        }
                    })
                }
                "get_todo" => {
                    let todo_pid = payload
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .and_then(|p| p.get("todo_pid"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let todos = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Todo found: {:?}", todos)
                                }
                            ]
                        }
                    })
                }
                "get_boards" => {
                    let project_pid = payload
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .and_then(|p| p.get("project_pid"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let boards = boards::Model::find_by_project_pid(&ctx.db, project_pid).await?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Boards found: {:?}", boards)
                                }
                            ]
                        }
                    })
                }
                "add_todo" => {
                    let args = payload
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .ok_or_else(|| Error::BadRequest("arguments required".to_string()))?;

                    let board_pid = args
                        .get("board_pid")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| Error::BadRequest("board_pid required".to_string()))?;
                    let title = args
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let details = args.get("details").and_then(|v| v.as_str());

                    let board = boards::Model::find_by_pid(&ctx.db, board_pid).await?;
                    let todo = todos::Model::create(
                        &ctx.db,
                        &TodoParams {
                            title: title.to_string(),
                            details: details.map(|s| s.to_string()),
                        },
                        &board,
                    )
                    .await?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Todo created: {:?}", todo)
                                }
                            ]
                        }
                    })
                }
                "add_board" => {
                    let args = payload
                        .get("params")
                        .and_then(|p| p.get("arguments"))
                        .ok_or_else(|| Error::BadRequest("arguments required".to_string()))?;

                    let project_pid = args
                        .get("project_pid")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| Error::BadRequest("project_pid required".to_string()))?;
                    let title = args
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    let board = projects::Model::find_by_pid(&ctx.db, project_pid).await?;
                    let todo = boards::Model::create(
                        &ctx.db,
                        &BoardParams {
                            title: title.to_string(),
                        },
                        &board,
                    )
                    .await?;

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Todo created: {:?}", todo)
                                }
                            ]
                        }
                    })
                }
                _ => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Method not found" }
                    })
                }
            }
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            })
        }
    };

    // Send the response back through the SSE stream
    let event = Event::default()
        .event("message")
        .data(serde_json::to_string(&response).unwrap());

    let _ = tx.send(event).await;

    // MCP SSE spec says the POST request should return 202 Accepted or similar
    // Here we just return an empty success response
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/mcp")
        .add("/sse", get(sse_handler))
        .add("/messages", post(message_handler))
}
