#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::{
    controllers::ws,
    models::{
        _entities::todos::{self, Entity, Model},
        boards,
        todos::TodoParams,
        users,
    },
    views::todo::{SubtaskItem, TodoResponse},
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Params {
    pub title: Option<String>,
    pub details: Option<Option<String>>,
    pub board_pid: Option<String>,
    pub tags: Option<Vec<String>>,
    pub locked: Option<bool>,
    pub parent_pid: Option<String>,
}

async fn load_item_by_pid(ctx: &AppContext, pid: &str) -> Result<Model> {
    let item = todos::Model::find_by_pid(&ctx.db, pid).await?;
    Ok(item)
}

async fn load_subtasks(ctx: &AppContext, parent_id: i32) -> Result<Vec<SubtaskItem>> {
    let raw = Entity::find()
        .filter(todos::Column::ParentId.eq(parent_id))
        .order_by_asc(todos::Column::Position)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;
    Ok(raw
        .into_iter()
        .map(|(s, stags)| SubtaskItem {
            pid: s.pid.to_string(),
            title: s.title,
            locked: s.locked,
            tags: stags,
        })
        .collect())
}

#[debug_handler]
pub async fn list_by_board(
    State(ctx): State<AppContext>,
    Path(board_pid): Path<String>,
) -> Result<Response> {
    let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;

    let todos = Entity::find()
        .filter(todos::Column::BoardId.eq(board.id))
        .filter(todos::Column::ParentId.is_null())
        .order_by_asc(todos::Column::Position)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;

    let response: Vec<TodoResponse> = todos
        .into_iter()
        .map(|(todo, tags)| TodoResponse::from(todo, tags, None, vec![]))
        .collect();

    format::json(response)
}

#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(board_pid): Path<String>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;
    let item = todos::Model::create(
        &ctx.db,
        &TodoParams {
            title: params.title.unwrap_or_default(),
            details: params.details.flatten(),
            tags: params.tags,
            locked: None,
            parent_pid: params.parent_pid,
        },
        &board,
        &user,
    )
    .await?;

    // Reload to get tags
    let res = Entity::find_by_id(item.id)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;
    let (item, tags) = res.into_iter().next().ok_or_else(|| Error::NotFound)?;

    ws::broadcast_for_board_id(&ctx.db, board.id).await;
    format::json(TodoResponse::from(item, tags, None, vec![]))
}

#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item_by_pid(&ctx, &pid).await?;
    let mut item_active = item.clone().into_active_model();

    if let Some(title) = params.title {
        item_active.title = Set(title);
    }

    if let Some(details) = params.details {
        item_active.details = Set(details);
    }

    if let Some(board_pid) = params.board_pid {
        let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;
        item_active.board_id = Set(board.id);
    }

    if let Some(locked) = params.locked {
        item_active.locked = Set(locked);
    }

    if let Some(ppid) = &params.parent_pid {
        if ppid.is_empty() {
            item_active.parent_id = Set(None);
        } else {
            let parent = todos::Model::find_by_pid(&ctx.db, ppid).await?;
            item_active.parent_id = Set(Some(parent.id));
        }
    }

    let _item_active = item_active.update(&ctx.db).await?;

    if let Some(tags) = params.tags {
        item.sync_tags(&ctx.db, tags, &user).await?;
    }

    // Reload to get tags
    let res = Entity::find_by_id(item.id)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;
    let (item, tags) = res.into_iter().next().ok_or_else(|| Error::NotFound)?;
    ws::broadcast_for_board_id(&ctx.db, item.board_id).await;

    format::json(TodoResponse::from(item, tags, None, vec![]))
}

#[debug_handler]
pub async fn remove(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    let board_id = item.board_id;
    item.delete(&ctx.db).await?;
    ws::broadcast_for_board_id(&ctx.db, board_id).await;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    let res = Entity::find_by_id(item.id)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;
    let (item, tags) = res.into_iter().next().ok_or_else(|| Error::NotFound)?;

    let parent_pid = if let Some(parent_id) = item.parent_id {
        Entity::find_by_id(parent_id)
            .one(&ctx.db)
            .await?
            .map(|p| p.pid.to_string())
    } else {
        None
    };

    let subtasks = load_subtasks(&ctx, item.id).await?;

    format::json(TodoResponse::from(item, tags, parent_pid, subtasks))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/boards/{board_pid}/todos", get(list_by_board))
        .add("/boards/{board_pid}/todos", post(add))
        .add("/todos/{pid}", get(get_one))
        .add("/todos/{pid}", delete(remove))
        .add("/todos/{pid}", put(update))
        .add("/todos/{pid}", patch(update))
}
