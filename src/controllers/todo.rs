#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::todos::{self, ActiveModel, Entity, Model},
    boards,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Params {
    pub title: Option<String>,
    pub details: Option<Option<String>>,
    pub board_pid: Option<String>,
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
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

    let title = params
        .title
        .ok_or_else(|| Error::BadRequest("title is required".to_string()))?;

    let item = ActiveModel {
        title: Set(title),
        details: Set(params.details.flatten()),
        board_id: Set(board.id),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::json(item)
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
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
pub async fn remove(Path(id): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    todos::Model::find_by_pid(&ctx.db, &id)
        .await?
        .delete(&ctx.db)
        .await?;

    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/board/{board_pid}/todos", get(list_by_board))
        .add("/board/{board_pid}/todos", post(add))
        .add("/todos/{id}", get(get_one))
        .add("/todos/{id}", delete(remove))
        .add("/todos/{id}", put(update))
        .add("/todos/{id}", patch(update))
}
