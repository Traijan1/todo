use crate::models::{
    _entities::{projects, users, tags},
    boards::{self, BoardParams},
    todos::{self, TodoParams},
    tags as tags_model,
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

    let user = users::Entity::find()
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let board = boards::Model::find_by_pid(&ctx.db, board_pid).await?;
    let todo = todos::Model::create(
        &ctx.db,
        &TodoParams {
            title: title.to_string(),
            details: details.map(|s| s.to_string()),
            tags: None,
        },
        &board,
        &user,
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

pub async fn get_tags(ctx: &AppContext) -> Result<Value> {
    let tags = tags::Entity::find().all(&ctx.db).await.map_err(|e| {
        tracing::error!("Failed to fetch tags for MCP: {:?}", e);
        e
    })?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Tags found: {:?}", tags)
            }
        ]
    }))
}

pub async fn add_tag_to_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".to_string()))?;
    let tag_pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".to_string()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;
    let tag = tags_model::Model::find_by_pid(&ctx.db, tag_pid).await?;

    todo.add_tag(&ctx.db, &tag).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Tag '{}' added to todo '{}'", tag.title, todo.title)
            }
        ]
    }))
}

pub async fn remove_tag_from_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".to_string()))?;
    let tag_pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".to_string()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;
    let tag = tags_model::Model::find_by_pid(&ctx.db, tag_pid).await?;

    todo.remove_tag(&ctx.db, &tag).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("Tag '{}' removed from todo '{}'", tag.title, todo.title)
            }
        ]
    }))
}
