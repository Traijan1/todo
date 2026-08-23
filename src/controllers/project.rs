#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::models::{
    _entities::{
        boards,
        projects::{ActiveModel, Entity, Model},
        todos, users_projects as up_entity,
    },
    users, users_projects,
};

#[derive(Serialize)]
struct BoardCount {
    pid: Uuid,
    title: String,
    count: i64,
}

#[derive(Serialize)]
struct ProjectStats {
    pid: Uuid,
    title: String,
    board_counts: Vec<BoardCount>,
    total: i64,
}

#[derive(Serialize)]
struct RecentTodo {
    pid: Uuid,
    title: String,
    board_title: String,
    project_title: String,
    updated_at: DateTimeWithTimeZone,
}

#[derive(Serialize)]
struct UserStats {
    total: i64,
    by_project: Vec<ProjectStats>,
    recently_updated: Vec<RecentTodo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
    pub description: Option<String>,
    pub mcp_expose_comments: Option<bool>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.description = Set(self.description.clone());
        if let Some(v) = self.mcp_expose_comments {
            item.mcp_expose_comments = Set(v);
        }
    }
}

async fn load_item_by_pid(ctx: &AppContext, pid: &str) -> Result<Model> {
    use crate::models::_entities::projects::Column as ProjectCol;
    let uuid = uuid::Uuid::parse_str(pid).map_err(|_| Error::NotFound)?;
    Entity::find()
        .filter(ProjectCol::Pid.eq(uuid))
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
            "Only project owners can do this".into(),
        ));
    }
    Ok(())
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

    let project_user = users_projects::ActiveModel {
        user_id: Set(user.id),
        project_id: Set(item.id),
        role: Set("owner".to_string()),
        ..Default::default()
    };

    project_user.insert(&txn).await.map_err(|e| {
        tracing::error!("Failed to insert users_projects link: {:?}", e);
        e
    })?;

    txn.commit().await?;

    format::json(item)
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
    require_owner(&ctx, user.id, item.id).await?;
    let mut active = item.into_active_model();
    params.update(&mut active);
    let item = active.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    Path(pid): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item_by_pid(&ctx, &pid).await?;
    require_owner(&ctx, user.id, item.id).await?;
    item.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item_by_pid(&ctx, &pid).await?)
}

#[debug_handler]
pub async fn user_stats(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let projects_list = Entity::find()
        .inner_join(users_projects::Entity)
        .filter(users_projects::Column::UserId.eq(user.id))
        .all(&ctx.db)
        .await?;

    if projects_list.is_empty() {
        return format::json(UserStats {
            total: 0,
            by_project: vec![],
            recently_updated: vec![],
        });
    }

    let project_ids: Vec<i32> = projects_list.iter().map(|p| p.id).collect();

    let boards_list = boards::Entity::find()
        .filter(boards::Column::ProjectId.is_in(project_ids))
        .order_by_asc(boards::Column::Position)
        .all(&ctx.db)
        .await?;

    let board_ids: Vec<i32> = boards_list.iter().map(|b| b.id).collect();

    let all_todos = if board_ids.is_empty() {
        vec![]
    } else {
        todos::Entity::find()
            .filter(todos::Column::BoardId.is_in(board_ids))
            .filter(todos::Column::ParentId.is_null())
            .order_by_desc(todos::Column::UpdatedAt)
            .all(&ctx.db)
            .await?
    };

    let mut board_count_map: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
    for todo in &all_todos {
        *board_count_map.entry(todo.board_id).or_insert(0) += 1;
    }

    let board_to_project: std::collections::HashMap<i32, i32> =
        boards_list.iter().map(|b| (b.id, b.project_id)).collect();
    let board_map: std::collections::HashMap<i32, &boards::Model> =
        boards_list.iter().map(|b| (b.id, b)).collect();
    let project_map: std::collections::HashMap<i32, &Model> =
        projects_list.iter().map(|p| (p.id, p)).collect();

    let by_project: Vec<ProjectStats> = projects_list
        .iter()
        .map(|p| {
            let board_counts: Vec<BoardCount> = boards_list
                .iter()
                .filter(|b| b.project_id == p.id)
                .map(|b| {
                    let count = board_count_map.get(&b.id).copied().unwrap_or(0);
                    BoardCount {
                        pid: b.pid,
                        title: b.title.clone(),
                        count,
                    }
                })
                .collect();
            let total: i64 = board_counts.iter().map(|b| b.count).sum();
            ProjectStats {
                pid: p.pid,
                title: p.title.clone(),
                board_counts,
                total,
            }
        })
        .collect();

    let total = all_todos.len() as i64;

    let recently_updated: Vec<RecentTodo> = all_todos
        .iter()
        .take(5)
        .filter_map(|todo| {
            let board = board_map.get(&todo.board_id)?;
            let project_id = board_to_project.get(&todo.board_id)?;
            let project = project_map.get(project_id)?;
            Some(RecentTodo {
                pid: todo.pid,
                title: todo.title.clone(),
                board_title: board.title.clone(),
                project_title: project.title.clone(),
                updated_at: todo.updated_at,
            })
        })
        .collect();

    format::json(UserStats {
        total,
        by_project,
        recently_updated,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/projects/")
        .add("stats", get(user_stats))
        .add("/", get(list_by_user))
        .add("/", post(add))
        .add("{pid}", get(get_one))
        .add("{pid}", delete(remove))
        .add("{pid}", put(update))
        .add("{pid}", patch(update))
}
