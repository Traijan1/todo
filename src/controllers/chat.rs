#![allow(clippy::missing_errors_doc)]

use std::{convert::Infallible, time::Duration};

use axum::response::sse::{Event, KeepAlive, Sse};
use loco_rs::prelude::*;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{
    models::users,
    services::chat::{self, ChatProgress, ChatRequest, ChatResult},
};

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ChatStreamEvent {
    Progress { progress: ChatProgress },
    Done { result: ChatResult },
    Error { message: String },
}

#[debug_handler]
pub async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(request): Json<ChatRequest>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let (sender, receiver) = mpsc::channel::<ChatStreamEvent>(64);

    tokio::spawn(async move {
        let progress_sender = sender.clone();
        let agent = chat::run(&ctx, &user, request, move |progress| {
            let _ = progress_sender.try_send(ChatStreamEvent::Progress { progress });
        });
        tokio::pin!(agent);

        // Dropping the browser stream cancels the in-flight provider request
        // and prevents later tool mutations from an abandoned chat turn.
        let result = tokio::select! {
            result = &mut agent => result,
            _ = sender.closed() => return,
        };

        let event = match result {
            Ok(result) => ChatStreamEvent::Done { result },
            Err(error) => ChatStreamEvent::Error {
                message: error.to_string(),
            },
        };
        let _ = sender.send(event).await;
    });

    let stream = ReceiverStream::new(receiver).map(|payload| {
        let event = Event::default()
            .event("chat")
            .json_data(payload)
            .unwrap_or_else(|_| {
                Event::default()
                    .event("chat")
                    .data(r#"{"type":"error","message":"Failed to serialize chat event"}"#)
            });
        Ok::<_, Infallible>(event)
    });
    let response = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("chat"),
    );

    Ok((
        [(
            axum::http::header::HeaderName::from_static("x-accel-buffering"),
            axum::http::HeaderValue::from_static("no"),
        )],
        response,
    )
        .into_response())
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/chat/").add("/", post(create))
}
