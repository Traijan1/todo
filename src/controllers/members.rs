#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::PaginatorTrait;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::{projects, users_projects as up_entity},
    users, users_projects,
};

#[derive(Serialize)]
pub struct MemberEntry {
    pub pid: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct AddMemberParams {
    pub email: String,
}

async fn load_project(ctx: &AppContext, project_pid: &str) -> Result<projects::Model> {
    use crate::models::_entities::projects::Column as Col;
    let uuid = uuid::Uuid::parse_str(project_pid).map_err(|_| Error::NotFound)?;
    projects::Entity::find()
        .filter(Col::Pid.eq(uuid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

async fn require_owner(ctx: &AppContext, user_id: i32, project_id: i32) -> Result<()> {
    let link = up_entity::Entity::find()
        .filter(up_entity::Column::UserId.eq(user_id))
        .filter(up_entity::Column::ProjectId.eq(project_id))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if link.role != "owner" {
        return Err(Error::Unauthorized(
            "Only project owners can manage members".into(),
        ));
    }
    Ok(())
}

#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(project_pid): Path<String>,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let project = load_project(&ctx, &project_pid).await?;

    let links = up_entity::Entity::find()
        .filter(up_entity::Column::ProjectId.eq(project.id))
        .all(&ctx.db)
        .await?;

    let mut members = Vec::new();
    for link in links {
        if let Ok(u) = users::Entity::find_by_id(link.user_id).one(&ctx.db).await {
            if let Some(u) = u {
                members.push(MemberEntry {
                    pid: u.pid,
                    name: u.name,
                    email: u.email,
                    role: link.role,
                });
            }
        }
    }

    format::json(members)
}

#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(project_pid): Path<String>,
    Json(params): Json<AddMemberParams>,
) -> Result<Response> {
    let caller = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let project = load_project(&ctx, &project_pid).await?;
    require_owner(&ctx, caller.id, project.id).await?;

    let target = users::Model::find_by_email(&ctx.db, &params.email)
        .await
        .map_err(|_| Error::NotFound)?;

    // Already a member?
    let existing = up_entity::Entity::find()
        .filter(up_entity::Column::UserId.eq(target.id))
        .filter(up_entity::Column::ProjectId.eq(project.id))
        .one(&ctx.db)
        .await?;

    if existing.is_some() {
        return Err(Error::BadRequest(
            "User is already a member of this project".into(),
        ));
    }

    let link = users_projects::ActiveModel {
        user_id: Set(target.id),
        project_id: Set(project.id),
        role: Set("member".to_string()),
        ..Default::default()
    };
    link.insert(&ctx.db).await?;

    format::json(MemberEntry {
        pid: target.pid,
        name: target.name,
        email: target.email,
        role: "member".to_string(),
    })
}

#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((project_pid, user_pid)): Path<(String, String)>,
) -> Result<Response> {
    let caller = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let project = load_project(&ctx, &project_pid).await?;
    require_owner(&ctx, caller.id, project.id).await?;

    let target = users::Model::find_by_pid(&ctx.db, &user_pid).await?;

    // Can't remove the last owner
    if target.id != caller.id {
        let owner_count = up_entity::Entity::find()
            .filter(up_entity::Column::ProjectId.eq(project.id))
            .filter(up_entity::Column::Role.eq("owner"))
            .count(&ctx.db)
            .await?;
        if owner_count <= 1 {
            return Err(Error::BadRequest("Cannot remove the last owner".into()));
        }
    }

    up_entity::Entity::delete_many()
        .filter(up_entity::Column::UserId.eq(target.id))
        .filter(up_entity::Column::ProjectId.eq(project.id))
        .exec(&ctx.db)
        .await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/projects/{project_pid}/members")
        .add("/", get(list))
        .add("/", post(add))
        .add("{user_pid}", delete(remove))
}
