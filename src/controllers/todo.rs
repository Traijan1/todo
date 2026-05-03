#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::todos::{self, Entity, Model},
    boards,
    todos::TodoParams,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Params {
    pub title: Option<String>,
    pub details: Option<Option<String>>,
    pub board_pid: Option<String>,
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

    format::json(
        Entity::find()
            .filter(todos::Column::BoardId.eq(board.id))
            .all(&ctx.db)
            .await?,
    )
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Path(board_pid): Path<String>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;
    let item = todos::Model::create(
        &ctx.db,
        &TodoParams {
            title: params.title.unwrap_or_default(),
            details: params.details.flatten(),
        },
        &board,
    )
    .await?;

    format::json(item)
}

#[debug_handler]
pub async fn update(
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    let mut item = item.into_active_model();

    if let Some(title) = params.title {
        item.title = Set(title);
    }

    if let Some(details) = params.details {
        item.details = Set(details);
    }

    if let Some(board_pid) = params.board_pid {
        let board = boards::Model::find_by_pid(&ctx.db, &board_pid).await?;
        item.board_id = Set(board.id);
    }

    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item_by_pid(&ctx, &pid).await?.delete(&ctx.db).await?;

    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item_by_pid(&ctx, &pid).await?)
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
