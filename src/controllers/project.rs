#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::projects::{ActiveModel, Entity, Model},
    users, users_projects,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
    pub description: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.description = Set(self.description.clone());
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list_by_user(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let entities = Entity::find()
        .inner_join(users_projects::Entity)
        .filter(users_projects::Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?;

    format::json(entities)
}

#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    //ValidatorTrait::validate(&params)?;

    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find user by pid: {:?}", e);
            e
        })?;

    tracing::debug!("User found: id={}", user.id);

    let txn = ctx.db.begin().await?;

    let mut item = ActiveModel {
        ..Default::default()
    };

    params.update(&mut item);

    tracing::debug!("Inserting project...");
    let item = item.insert(&txn).await.map_err(|e| {
        tracing::error!("Failed to insert project: {:?}", e);
        e
    })?;

    tracing::debug!("Project inserted: id={}", item.id);

    // Create the many-to-many link
    let project_user = users_projects::ActiveModel {
        user_id: Set(user.id),
        project_id: Set(item.id),
        ..Default::default()
    };

    tracing::debug!("Inserting users_projects link...");
    project_user.insert(&txn).await.map_err(|e| {
        tracing::error!("Failed to insert users_projects link: {:?}", e);
        e
    })?;

    txn.commit().await?;
    tracing::debug!("Transaction committed.");

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
        .prefix("api/projects/")
        .add("/", get(list_by_user))
        .add("/", post(add))
        .add("{id}", get(get_one))
        .add("{id}", delete(remove))
        .add("{id}", put(update))
        .add("{id}", patch(update))
}
