use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
};
use futures::Stream;
use lazy_static::lazy_static;
use loco_rs::prelude::*;
use std::convert::Infallible;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

lazy_static! {
    static ref BROADCAST_SENDER: broadcast::Sender<String> = {
        let (tx, _) = broadcast::channel(100);
        tx
    };
}

/// Broadcast a "refresh" event to all connected SSE & WebSocket clients.
pub async fn broadcast(project_pid: &str) {
    let msg = format!(r#"{{"event":"refresh","project_pid":"{}"}}"#, project_pid);
    let _ = BROADCAST_SENDER.send(msg);
}

/// Broadcast from a project integer id.
pub async fn broadcast_for_project_id(db: &sea_orm::DatabaseConnection, project_id: i32) {
    use crate::models::_entities::projects;
    use sea_orm::EntityTrait;
    if let Ok(Some(project)) = projects::Entity::find_by_id(project_id).one(db).await {
        broadcast(&project.pid.to_string()).await;
    }
}

/// Broadcast from a board integer id.
pub async fn broadcast_for_board_id(db: &sea_orm::DatabaseConnection, board_id: i32) {
    use crate::models::_entities::boards;
    use sea_orm::EntityTrait;
    if let Ok(Some(board)) = boards::Entity::find_by_id(board_id).one(db).await {
        broadcast_for_project_id(db, board.project_id).await;
    }
}

#[debug_handler]
pub async fn sse_events_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = BROADCAST_SENDER.subscribe();
    let (tx, mpsc_rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if tx.send(msg).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    let fallback = r#"{"event":"refresh_all"}"#.to_string();
                    if tx.send(fallback).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    let stream = ReceiverStream::new(mpsc_rx)
        .map(|text| Ok(Event::default().event("message").data(text)));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[debug_handler]
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = BROADCAST_SENDER.subscribe();

    loop {
        tokio::select! {
            msg_res = rx.recv() => {
                match msg_res {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let fallback = r#"{"event":"refresh_all"}"#.to_string();
                        if socket.send(Message::Text(fallback.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(Message::Ping(ping))) => {
                        if socket.send(Message::Pong(ping)).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/events", get(sse_events_handler))
        .add("/ws", get(ws_handler))
}
