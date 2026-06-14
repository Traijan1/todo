#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        _entities::todos::{self, Entity, Model},
        boards,
        todos::TodoParams,
        users,
    },
    views::todo::TodoResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Params {
    pub title: Option<String>,
    pub details: Option<Option<String>>,
    pub board_pid: Option<String>,
    pub tags: Option<Vec<String>>,
}

async fn load_item_by_pid(ctx: &AppContext, pid: &str) -> Result<Model> {
    let item = todos::Model::find_by_pid(&ctx.db, pid).await?;
    Ok(item)
}

#[debug_handler]
pub async fn list_by_board(
    State(ctx): State<AppContext>,
    Path(board_pid): Path<String>,
) -> Result<Response> {
    let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;

    let todos = Entity::find()
        .filter(todos::Column::BoardId.eq(board.id))
        .order_by_asc(todos::Column::Position)
        .find_with_related(crate::models::_entities::tags::Entity)
        .all(&ctx.db)
        .await?;

    let response: Vec<TodoResponse> = todos
        .into_iter()
        .map(|(todo, tags)| TodoResponse::from(todo, tags))
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

    format::json(TodoResponse::from(item, tags))
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

    format::json(TodoResponse::from(item, tags))
}

#[debug_handler]
pub async fn remove(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item_by_pid(&ctx, &pid).await?.delete(&ctx.db).await?;

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

    format::json(TodoResponse::from(item, tags))
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
