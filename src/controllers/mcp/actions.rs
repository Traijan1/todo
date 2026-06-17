use crate::models::{
    _entities::{
        boards as boards_entity, projects, tags as tags_entity, todos as todos_entity, users,
        users_projects,
    },
    boards::{self, BoardParams},
    comments as comments_model,
    tags as tags_model,
    todos::{self, TodoParams},
};

async fn project_for_board(ctx: &AppContext, board_id: i32) -> Result<projects::Model> {
    let board = boards_entity::Entity::find_by_id(board_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    projects::Entity::find_by_id(board.project_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}
use crate::controllers::ws;
use loco_rs::prelude::*;
use sea_orm::{QueryOrder, Set};
use serde_json::{json, Value};

/// Resolve tag names or PIDs → PIDs.
/// If an entry looks like a UUID, it's treated as a PID and looked up directly.
/// Otherwise it's treated as a name and auto-created if it doesn't exist.
async fn resolve_tag_names(
    ctx: &AppContext,
    names: &[String],
    project_id: i32,
) -> Result<Vec<String>> {
    let mut pids = Vec::new();
    for name in names {
        let tag = if sea_orm::prelude::Uuid::parse_str(name).is_ok() {
            tags_model::Model::find_by_pid(&ctx.db, name)
                .await
                .map_err(|_| Error::BadRequest(format!("Tag with PID '{}' not found", name)))?
        } else {
            tags_model::Model::find_or_create_by_title(&ctx.db, name, project_id).await?
        };
        pids.push(tag.pid.to_string());
    }
    Ok(pids)
}

// ── Projects ────────────────────────────────────────────────────────────────

pub async fn get_projects(ctx: &AppContext, user: &users::Model) -> Result<Value> {
    let project_ids: Vec<i32> = users_projects::Entity::find()
        .filter(users_projects::Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|up| up.project_id)
        .collect();
    let items = if project_ids.is_empty() {
        vec![]
    } else {
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids))
            .all(&ctx.db)
            .await?
    };
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&items).unwrap_or_default() }]
    }))
}

pub async fn add_project(ctx: &AppContext, args: &Value, user: &users::Model) -> Result<Value> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("title required".into()))?;
    let description = args.get("description").and_then(|v| v.as_str());

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
    let board = boards::Model::create(
        &ctx.db,
        &BoardParams {
            title: title.to_string(),
        },
        &project,
    )
    .await?;
    ws::broadcast(project_pid).await;
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
    let project_id = board.project_id;
    let mut active = board.into_active_model();

    if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }

    let board = active.update(&ctx.db).await?;
    ws::broadcast_for_project_id(&ctx.db, project_id).await;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&board).unwrap_or_default() }]
    }))
}

pub async fn delete_board(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args
        .get("board_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("board_pid required".into()))?;

    let board = boards::Model::find_by_pid(&ctx.db, pid).await?;
    let project_id = board.project_id;
    board.delete(&ctx.db).await?;
    ws::broadcast_for_project_id(&ctx.db, project_id).await;
    Ok(json!({ "content": [{ "type": "text", "text": "Board deleted" }] }))
}

pub async fn reorder_boards(ctx: &AppContext, args: &Value) -> Result<Value> {
    let board_pids: Vec<String> = args
        .get("board_pids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| Error::BadRequest("board_pids required".into()))?;

    let mut project_id: Option<i32> = None;
    for (index, pid) in board_pids.iter().enumerate() {
        let board = boards::Model::find_by_pid(&ctx.db, pid).await?;
        if project_id.is_none() { project_id = Some(board.project_id); }
        let mut active = board.into_active_model();
        active.position = Set(index as i32);
        active.update(&ctx.db).await?;
    }

    if let Some(pid) = project_id {
        ws::broadcast_for_project_id(&ctx.db, pid).await;
    }
    Ok(
        json!({ "content": [{ "type": "text", "text": format!("Reordered {} boards", board_pids.len()) }] }),
    )
}

pub async fn reorder_todos(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pids: Vec<String> = args
        .get("todo_pids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| Error::BadRequest("todo_pids required".into()))?;

    let mut board_id: Option<i32> = None;
    for (index, pid) in todo_pids.iter().enumerate() {
        let todo = todos::Model::find_by_pid(&ctx.db, pid).await?;
        if board_id.is_none() { board_id = Some(todo.board_id); }
        let mut active = todo.into_active_model();
        active.position = Set(index as i32);
        active.update(&ctx.db).await?;
    }

    if let Some(bid) = board_id {
        ws::broadcast_for_board_id(&ctx.db, bid).await;
    }
    Ok(
        json!({ "content": [{ "type": "text", "text": format!("Reordered {} todos", todo_pids.len()) }] }),
    )
}

// ── Todos ────────────────────────────────────────────────────────────────────

pub async fn get_todos(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let board_pid = args.get("board_pid").and_then(|v| v.as_str());
    let search = args.get("search").and_then(|v| v.as_str());

    let tag_pids: Option<Vec<String>> =
        args.get("tag_pids").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    let items = todos::Model::find_by_project_pid_with_tags(
        &ctx.db,
        project_pid,
        board_pid,
        tag_pids.as_deref(),
        search,
    )
    .await?;

    let response: Vec<Value> = items
        .into_iter()
        .filter(|(todo, _)| !todo.locked)
        .map(|(todo, todo_tags)| {
            let mut obj = serde_json::to_value(&todo).unwrap_or_default();
            if let Some(m) = obj.as_object_mut() {
                m.insert("tags".into(), json!(todo_tags));
            }
            obj
        })
        .collect();

    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&response).unwrap_or_default() }]
    }))
}

pub async fn get_todo(ctx: &AppContext, args: &Value) -> Result<Value> {
    let pid = args.get("todo_pid").and_then(|v| v.as_str()).unwrap_or("");
    let todo = todos::Model::find_by_pid(&ctx.db, pid).await?;

    if todo.locked {
        return Err(Error::NotFound);
    }

    let todo_tags = todo.find_related(tags_entity::Entity).all(&ctx.db).await?;

    let parent_pid = if let Some(parent_id) = todo.parent_id {
        todos_entity::Entity::find_by_id(parent_id)
            .one(&ctx.db)
            .await?
            .map(|p| p.pid.to_string())
    } else {
        None
    };

    let subtasks_raw = todos_entity::Entity::find()
        .filter(todos_entity::Column::ParentId.eq(todo.id))
        .order_by_asc(todos_entity::Column::Position)
        .find_with_related(tags_entity::Entity)
        .all(&ctx.db)
        .await?;
    let subtasks: Vec<Value> = subtasks_raw
        .into_iter()
        .filter(|(s, _)| !s.locked)
        .map(|(s, stags)| {
            let mut obj = serde_json::to_value(&s).unwrap_or_default();
            if let Some(m) = obj.as_object_mut() {
                m.insert("tags".into(), json!(stags));
            }
            obj
        })
        .collect();

    let project = project_for_board(ctx, todo.board_id).await?;
    let comments = if project.mcp_expose_comments {
        comments_model::Model::find_by_todo_id(&ctx.db, todo.id).await?
    } else {
        vec![]
    };

    let mut obj = serde_json::to_value(&todo).unwrap_or_default();
    if let Some(m) = obj.as_object_mut() {
        m.insert("tags".into(), json!(todo_tags));
        m.insert("parent_pid".into(), json!(parent_pid));
        m.insert("subtasks".into(), json!(subtasks));
        m.insert("comments".into(), json!(comments));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&obj).unwrap_or_default() }]
    }))
}

pub async fn add_todo(ctx: &AppContext, args: &Value, user: &users::Model) -> Result<Value> {
    let board_pid = args.get("board_pid").and_then(|v| v.as_str());
    let parent_pid = args.get("parent_pid").and_then(|v| v.as_str());
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let details = args.get("details").and_then(|v| v.as_str());

    let board = if let Some(bpid) = board_pid {
        boards::Model::find_by_pid(&ctx.db, bpid).await?
    } else if let Some(ppid) = parent_pid {
        let parent = todos::Model::find_by_pid(&ctx.db, ppid).await?;
        boards_entity::Entity::find_by_id(parent.board_id)
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?
    } else {
        return Err(Error::BadRequest(
            "board_pid required (or provide parent_pid to inherit board)".into(),
        ));
    };

    let tag_names: Vec<String> = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let tag_pids = if tag_names.is_empty() {
        None
    } else {
        Some(resolve_tag_names(ctx, &tag_names, board.project_id).await?)
    };
    let project_id = board.project_id;
    let todo = todos::Model::create(
        &ctx.db,
        &TodoParams {
            title: title.to_string(),
            details: details.map(|s| s.to_string()),
            tags: tag_pids,
            locked: None,
            parent_pid: parent_pid.map(|s| s.to_string()),
        },
        &board,
        &user,
    )
    .await?;
    ws::broadcast_for_project_id(&ctx.db, project_id).await;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&todo).unwrap_or_default() }]
    }))
}

pub async fn update_todo(ctx: &AppContext, args: &Value, user: &users::Model) -> Result<Value> {
    let pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;

    let item = todos::Model::find_by_pid(&ctx.db, pid).await?;

    if item.locked {
        return Ok(
            json!({ "content": [{ "type": "text", "text": "Error: This todo is locked and invisible to AI. Skipping." }] }),
        );
    }

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

    if let Some(tag_names) = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>()
    }) {
        let board = boards_entity::Entity::find_by_id(item.board_id)
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;
        let tag_pids = resolve_tag_names(ctx, &tag_names, board.project_id).await?;
        item.sync_tags(&ctx.db, tag_pids, &user).await?;
    }

    ws::broadcast_for_board_id(&ctx.db, item.board_id).await;
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

    let todo = todos::Model::find_by_pid(&ctx.db, pid).await?;

    if todo.locked {
        return Ok(
            json!({ "content": [{ "type": "text", "text": "Error: This todo is locked and invisible to AI. Skipping." }] }),
        );
    }

    let board_id = todo.board_id;
    todo.delete(&ctx.db).await?;
    ws::broadcast_for_board_id(&ctx.db, board_id).await;
    Ok(json!({ "content": [{ "type": "text", "text": "Todo deleted" }] }))
}

// ── Tags ─────────────────────────────────────────────────────────────────────

pub async fn get_tags(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".into()))?;

    let project = projects::Model::find_by_pid(&ctx.db, project_pid).await?;

    let items = tags_entity::Entity::find()
        .filter(tags_entity::Column::ProjectId.eq(project.id))
        .all(&ctx.db)
        .await?;
    let clean: Vec<Value> = items
        .iter()
        .map(|t| json!({ "pid": t.pid, "name": t.title, "color": t.color }))
        .collect();
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&clean).unwrap_or_default() }]
    }))
}

pub async fn create_tag(ctx: &AppContext, args: &Value) -> Result<Value> {
    let project_pid = args
        .get("project_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("project_pid required".into()))?;
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("title required".into()))?;
    let color = args.get("color").and_then(|v| v.as_str());

    let project = projects::Model::find_by_pid(&ctx.db, project_pid).await?;
    let tag = tags_model::Model::create(
        &ctx.db,
        &tags_model::TagParams {
            title: title.to_string(),
            color: color.map(|s| s.to_string()),
            project_id: project.id,
        },
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

// ── Comments ──────────────────────────────────────────────────────────────────

pub async fn get_comments(ctx: &AppContext, args: &Value) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;

    if todo.locked {
        return Err(Error::NotFound);
    }

    let project = project_for_board(ctx, todo.board_id).await?;
    if !project.mcp_expose_comments {
        return Ok(json!({ "content": [{ "type": "text", "text": "Comments are disabled for MCP access on this project." }] }));
    }

    let comments = comments_model::Model::find_by_todo_id(&ctx.db, todo.id).await?;
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&comments).unwrap_or_default() }]
    }))
}

pub async fn add_comment(ctx: &AppContext, args: &Value, user: &users::Model) -> Result<Value> {
    let todo_pid = args
        .get("todo_pid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("todo_pid required".into()))?;
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("content required".into()))?;

    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;

    if todo.locked {
        return Ok(json!({ "content": [{ "type": "text", "text": "Error: This todo is locked and invisible to AI." }] }));
    }

    let project = project_for_board(ctx, todo.board_id).await?;
    if !project.mcp_expose_comments {
        return Ok(json!({ "content": [{ "type": "text", "text": "Error: Comments are disabled for MCP access on this project." }] }));
    }

    let comment = comments_model::Model::create(
        &ctx.db,
        comments_model::CreateComment {
            todo_id: todo.id,
            author: user.name.clone(),
            content: content.to_string(),
            is_ai: true,
        },
    )
    .await?;

    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&comment).unwrap_or_default() }]
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
