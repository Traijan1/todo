#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::Serialize;

use crate::models::{
    _entities::{time_entries as te_entity, todos},
    time_entries::{self as te_model, format_duration},
    users,
};

#[derive(Serialize)]
struct TimerEntry {
    pid: Uuid,
    started_at: chrono::DateTime<chrono::Utc>,
    stopped_at: Option<chrono::DateTime<chrono::Utc>>,
    duration_seconds: i64,
    duration_formatted: String,
    is_ai: bool,
}

#[derive(Serialize)]
struct TimerState {
    running: bool,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    total_seconds: i64,
    total_formatted: String,
    entries: Vec<TimerEntry>,
}

fn to_utc(dt: &chrono::DateTime<chrono::FixedOffset>) -> chrono::DateTime<chrono::Utc> {
    dt.with_timezone(&chrono::Utc)
}

#[debug_handler]
pub async fn status(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(todo_pid): Path<String>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;

    let entries = te_model::Model::find_all_for_todo(&ctx.db, todo.id).await?;
    let total_secs = te_model::Model::total_seconds(&entries);
    let running_entry = entries.iter().find(|e| e.stopped_at.is_none());

    let state = TimerState {
        running: running_entry.is_some(),
        started_at: running_entry.map(|e| to_utc(&e.started_at)),
        total_seconds: total_secs,
        total_formatted: format_duration(total_secs),
        entries: entries
            .iter()
            .filter(|e| e.stopped_at.is_some())
            .map(|e| {
                let secs = (e.stopped_at.unwrap().timestamp() - e.started_at.timestamp()).max(0);
                TimerEntry {
                    pid: e.pid,
                    started_at: to_utc(&e.started_at),
                    stopped_at: e.stopped_at.map(|t| to_utc(&t)),
                    duration_seconds: secs,
                    duration_formatted: format_duration(secs),
                    is_ai: e.is_ai,
                }
            })
            .collect(),
    };

    format::json(state)
}

#[debug_handler]
pub async fn start(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(todo_pid): Path<String>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;

    if te_model::Model::find_active_for_todo(&ctx.db, todo.id)
        .await?
        .is_some()
    {
        return Err(Error::BadRequest(
            "A timer is already running for this todo".into(),
        ));
    }

    te_entity::Entity::start(&ctx.db, todo.id, Some(user.id), false).await?;
    format::json(serde_json::json!({ "started": true }))
}

#[debug_handler]
pub async fn stop(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(todo_pid): Path<String>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;

    let entry = te_model::Model::find_active_for_todo(&ctx.db, todo.id)
        .await?
        .ok_or_else(|| Error::BadRequest("No running timer for this todo".into()))?;

    let mut active: te_model::ActiveModel = entry.into();
    active.stopped_at = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().into()));
    let saved = active.update(&ctx.db).await?;

    let elapsed = (saved.stopped_at.unwrap().timestamp() - saved.started_at.timestamp()).max(0);

    format::json(serde_json::json!({
        "stopped": true,
        "duration_seconds": elapsed,
        "duration_formatted": format_duration(elapsed),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/todos/{todo_pid}/timer")
        .add("/", get(status))
        .add("start", post(start))
        .add("stop", post(stop))
}
