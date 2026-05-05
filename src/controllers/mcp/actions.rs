use crate::models::{
    _entities::projects,
    boards::{self, BoardParams},
    todos::{self, TodoParams},
};
use loco_rs::prelude::*;
use serde_json::{json, Value};

pub async fn get_projects(ctx: &AppContext) -> Result<Value> {
    let projects = projects::Entity::find().all(&ctx.db).await.map_err(|e| {
        tracing::error!("Failed to fetch projects for MCP: {:?}", e);
        e
    })?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Projects found: {:?}", projects)
            }
        ]
    }))
}

pub async fn get_todos(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let todos = todos::Model::find_by_project_pid(&ctx.db, project_pid).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Todos found: {:?}", todos)
            }
        ]
    }))
}

pub async fn get_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args.get("todo_pid").and_then(|n| n.as_str()).unwrap_or("");

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Todo found: {:?}", todo)
            }
        ]
    }))
}

pub async fn get_boards(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let boards = boards::Model::find_by_project_pid(&ctx.db, project_pid).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Boards found: {:?}", boards)
            }
        ]
    }))
}

pub async fn add_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
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

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Todo created: {:?}", todo)
            }
        ]
    }))
}

pub async fn add_board(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".to_string()))?;
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let project = projects::Model::find_by_pid(&ctx.db, project_pid).await?;
    let board = boards::Model::create(
        &ctx.db,
        &BoardParams {
            title: title.to_string(),
        },
        &project,
    )
    .await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Board created: {:?}", board)
            }
        ]
    }))
}
