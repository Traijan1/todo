#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::tags::{ActiveModel, Entity, Model},
    tags::TagParams,
    users,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
    pub color: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.color = Set(self.color.clone());
    }
}

async fn load_item(ctx: &AppContext, pid: &str) -> Result<Model> {
    let item = Model::find_by_pid(&ctx.db, pid).await?;
    Ok(item)
}

#[debug_handler]
pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let tags = Entity::find()
        .filter(crate::models::_entities::tags::Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?;
    format::json(tags)
}

#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = Model::create(
        &ctx.db,
        &TagParams {
            title: params.title,
            color: params.color,
        },
        &user,
    )
    .await?;
    format::json(item)
}

#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, &pid).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_item(&ctx, &pid).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(
    auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(load_item(&ctx, &pid).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/tags")
        .add("/", get(list))
        .add("/", post(add))
        .add("/{pid}", get(get_one))
        .add("/{pid}", delete(remove))
        .add("/{pid}", put(update))
        .add("/{pid}", patch(update))
}
