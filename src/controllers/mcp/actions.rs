use crate::models::{
    _entities::{
        boards as boards_entity, projects, tags as tags_entity, todos as todos_entity,
        users, users_projects,
    },
    boards::{self, BoardParams},
    tags as tags_model,
    todos::{self, TodoParams},
};
use loco_rs::prelude::*;
use sea_orm::Set;
use serde_json::{json, Value};

fn first_user_err() -> Error {
    Error::NotFound
}

async fn get_user(ctx: &AppContext) -> Result<users::Model> {
    users::Entity::find()
        .one(&ctx.db)
        .await?
        .ok_or_else(first_user_err)
}

// ── Projects ────────────────────────────────────────────────────────────────

pub async fn get_projects(ctx: &AppContext) -> Result<Value> {
    let items = projects::Entity::find().all(&ctx.db).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&items).unwrap_or_default() }]
    }))
}

pub async fn add_project(ctx: &AppContext, args: &Value) -> Result<Value> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("title required".into()))?;
    let description = args.get("description").and_then(|v| v.as_str());

    let user = get_user(ctx).await?;
    let txn = ctx.db.begin().await?;

    let item = projects::ActiveModel {
        title: Set(title.to_string()),
        description: Set(description.map(|s| s.to_string())),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    users_projects::ActiveModel {
        user_id: Set(user.id),
        project_id: Set(item.id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&item).unwrap_or_default() }]
    }))
}

pub async fn update_project(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".into()))?;

    let item = projects::Model::find_by_pid(&ctx.db, pid).await?;
    let mut active = item.into_active_model();

    if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }
    if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
        active.description = Set(Some(desc.to_string()));
    }

    let item = active.update(&ctx.db).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&item).unwrap_or_default() }]
    }))
}

pub async fn delete_project(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".into()))?;

    projects::Model::find_by_pid(&ctx.db, pid)
        .await?
        .delete(&ctx.db)
        .await?;
    Ok(json!({ "content": [{ "type": "text", "text": "Project deleted" }] }))
}

// ── Boards ──────────────────────────────────────────────────────────────────

pub async fn get_boards(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let items = boards::Model::find_by_project_pid(&ctx.db, project_pid).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&items).unwrap_or_default() }]
    }))
}

pub async fn add_board(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".into()))?;
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let project = projects::Model::find_by_pid(&ctx.db, project_pid).await?;
    let board = boards::Model::create(&ctx.db, &BoardParams { title: title.to_string() }, &project).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&board).unwrap_or_default() }]
    }))
}

pub async fn update_board(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("board_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("board_pid required".into()))?;

    let board = boards::Model::find_by_pid(&ctx.db, pid).await?;
    let mut active = board.into_active_model();

    if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }

    let board = active.update(&ctx.db).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&board).unwrap_or_default() }]
    }))
}

pub async fn delete_board(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("board_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("board_pid required".into()))?;

    boards::Model::find_by_pid(&ctx.db, pid)
        .await?
        .delete(&ctx.db)
        .await?;
    Ok(json!({ "content": [{ "type": "text", "text": "Board deleted" }] }))
}

// ── Todos ────────────────────────────────────────────────────────────────────

pub async fn get_todos(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let items = todos::Model::find_by_project_pid(&ctx.db, project_pid).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&items).unwrap_or_default() }]
    }))
}

pub async fn get_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let todo = todos::Model::find_by_pid(&ctx.db, pid).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&todo).unwrap_or_default() }]
    }))
}

pub async fn add_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let board_pid = args
        .get("board_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("board_pid required".into()))?;
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
    let details = args.get("details").and_then(|v| v.as_str());
    let tags = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

    let user = get_user(ctx).await?;
    let board = boards::Model::find_by_pid(&ctx.db, board_pid).await?;
    let todo = todos::Model::create(
        &ctx.db,
        &TodoParams {
            title: title.to_string(),
            details: details.map(|s| s.to_string()),
            tags,
        },
        &board,
        &user,
    )
    .await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&todo).unwrap_or_default() }]
    }))
}

pub async fn update_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;

    let user = get_user(ctx).await?;
    let item = todos::Model::find_by_pid(&ctx.db, pid).await?;
    let mut active = item.clone().into_active_model();

    if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }
    if let Some(details) = args.get("details") {
        active.details = Set(details.as_str().map(|s| s.to_string()));
    }
    if let Some(board_pid) = args.get("board_pid").and_then(|v| v.as_str()) {
        let board = boards::Model::find_by_pid(&ctx.db, board_pid).await?;
        active.board_id = Set(board.id);
    }

    let _updated = active.update(&ctx.db).await?;

    if let Some(tags) = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
    {
        item.sync_tags(&ctx.db, tags, &user).await?;
    }

    let updated = todos::Model::find_by_pid(&ctx.db, pid).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&updated).unwrap_or_default() }]
    }))
}

pub async fn delete_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;

    todos::Model::find_by_pid(&ctx.db, pid)
        .await?
        .delete(&ctx.db)
        .await?;
    Ok(json!({ "content": [{ "type": "text", "text": "Todo deleted" }] }))
}

// ── Tags ─────────────────────────────────────────────────────────────────────

pub async fn get_tags(ctx: &AppContext) -> Result<Value> {
    let items = tags_entity::Entity::find().all(&ctx.db).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&items).unwrap_or_default() }]
    }))
}

pub async fn create_tag(ctx: &AppContext, args: &Value) -> Result<Value> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("title required".into()))?;
    let color = args.get("color").and_then(|v| v.as_str());

    let user = get_user(ctx).await?;
    let tag = tags_model::Model::create(
        &ctx.db,
        &tags_model::TagParams {
            title: title.to_string(),
            color: color.map(|s| s.to_string()),
        },
        &user,
    )
    .await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&tag).unwrap_or_default() }]
    }))
}

pub async fn update_tag(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".into()))?;

    let tag = tags_model::Model::find_by_pid(&ctx.db, pid).await?;
    let mut active = tag.into_active_model();

    if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }
    if let Some(color) = args.get("color").and_then(|v| v.as_str()) {
        active.color = Set(Some(color.to_string()));
    }

    let tag = active.update(&ctx.db).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&tag).unwrap_or_default() }]
    }))
}

pub async fn delete_tag(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".into()))?;

    tags_model::Model::find_by_pid(&ctx.db, pid)
        .await?
        .delete(&ctx.db)
        .await?;
    Ok(json!({ "content": [{ "type": "text", "text": "Tag deleted" }] }))
}

pub async fn add_tag_to_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;
    let tag_pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".into()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;
    let tag = tags_model::Model::find_by_pid(&ctx.db, tag_pid).await?;
    todo.add_tag(&ctx.db, &tag).await?;

    Ok(json!({
        "content": [{ "type": "text", "text": format!("Tag '{}' added to todo '{}'", tag.title, todo.title) }]
    }))
}

pub async fn remove_tag_from_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;
    let tag_pid = args
        .get("tag_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("tag_pid required".into()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;
    let tag = tags_model::Model::find_by_pid(&ctx.db, tag_pid).await?;
    todo.remove_tag(&ctx.db, &tag).await?;

    Ok(json!({
        "content": [{ "type": "text", "text": format!("Tag '{}' removed from todo '{}'", tag.title, todo.title) }]
    }))
}
