#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::{
        projects,
        tags::{ActiveModel, Column, Entity, Model},
    },
    tags::TagParams,
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
    Ok(Model::find_by_pid(&ctx.db, pid).await?)
}

#[debug_handler]
pub async fn list(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(project_pid): Path<Uuid>,
) -> Result<Response> {
    let project = projects::Entity::find()
        .filter(projects::Column::Pid.eq(project_pid))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let tags = Entity::find()
        .filter(Column::ProjectId.eq(project.id))
        .all(&ctx.db)
        .await?;

    format::json(tags)
}

#[debug_handler]
pub async fn add(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(project_pid): Path<Uuid>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let project = projects::Entity::find()
        .filter(projects::Column::Pid.eq(project_pid))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let item = Model::create(
        &ctx.db,
        &TagParams {
            title: params.title,
            color: params.color,
            project_id: project.id,
        },
    )
    .await?;

    format::json(item)
}

#[debug_handler]
pub async fn update(
    _auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, &pid).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(
    _auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    load_item(&ctx, &pid).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(
    _auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    format::json(load_item(&ctx, &pid).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/projects/{project_pid}/tags", get(list))
        .add("/projects/{project_pid}/tags", post(add))
        .add("/tags/{pid}", get(get_one))
        .add("/tags/{pid}", delete(remove))
        .add("/tags/{pid}", put(update))
        .add("/tags/{pid}", patch(update))
}
