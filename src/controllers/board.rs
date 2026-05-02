#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::_entities::{
    boards::{self, ActiveModel, Entity, Model},
    projects, todos,
};

#[derive(Serialize)]
pub struct BoardResponse {
    #[serde(flatten)]
    pub board: boards::Model,
    pub todos: Vec<todos::Model>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    State(ctx): State<AppContext>,
    Path(project_pid): Path<Uuid>,
) -> Result<Response> {
    let project = projects::Entity::find()
        .filter(projects::Column::Pid.eq(project_pid))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let boards_with_todos = Entity::find()
        .filter(boards::Column::ProjectId.eq(project.id))
        .find_with_related(todos::Entity)
        .all(&ctx.db)
        .await?;

    let response: Vec<BoardResponse> = boards_with_todos
        .into_iter()
        .map(|(board, todos)| BoardResponse { board, todos })
        .collect();

    format::json(response)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Path(project_pid): Path<Uuid>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let project = projects::Entity::find()
        .filter(projects::Column::Pid.eq(project_pid))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut item = ActiveModel {
        pid: Set(Uuid::new_v4()),
        project_id: Set(project.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
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
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/projects/{project_pid}/boards", get(list))
        .add("/projects/{project_pid}/boards", post(add))
        .add("/boards/{id}", get(get_one))
        .add("/boards/{id}", delete(remove))
        .add("/boards/{id}", put(update))
        .add("/boards/{id}", patch(update))
}
