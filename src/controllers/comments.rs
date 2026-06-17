#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::models::{
    _entities::{comments as comments_entity, todos},
    comments::{self, CreateComment},
    users,
};

#[derive(Deserialize)]
pub struct AddCommentParams {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateCommentParams {
    pub content: String,
}

#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(todo_pid): Path<String>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;
    let items = comments::Model::find_by_todo_id(&ctx.db, todo.id).await?;
    format::json(items)
}

#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(todo_pid): Path<String>,
    Json(params): Json<AddCommentParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;
    let comment = comments::Model::create(
        &ctx.db,
        CreateComment {
            todo_id: todo.id,
            author: user.name,
            content: params.content,
            is_ai: false,
        },
    )
    .await?;
    format::json(comment)
}

#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((todo_pid, comment_pid)): Path<(String, String)>,
    Json(params): Json<UpdateCommentParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let _todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;

    let pid = uuid::Uuid::parse_str(&comment_pid).map_err(|_| Error::NotFound)?;
    let comment = comments_entity::Entity::find()
        .filter(comments_entity::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if comment.is_ai || comment.author != user.name {
        return Err(Error::Unauthorized(
            "You can only edit your own comments".into(),
        ));
    }

    let mut active = comment.into_active_model();
    active.content = Set(params.content);
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((todo_pid, comment_pid)): Path<(String, String)>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let _todo = todos::Model::find_by_pid(&ctx.db, &todo_pid).await?;

    let pid = uuid::Uuid::parse_str(&comment_pid).map_err(|_| Error::NotFound)?;
    let comment = comments_entity::Entity::find()
        .filter(comments_entity::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    if comment.is_ai || comment.author != user.name {
        return Err(Error::Unauthorized(
            "You can only delete your own comments".into(),
        ));
    }

    comment.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/todos/{todo_pid}/comments")
        .add("/", get(list))
        .add("/", post(add))
        .add("{comment_pid}", patch(update))
        .add("{comment_pid}", delete(remove))
}
