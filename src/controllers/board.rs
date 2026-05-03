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
pub struct TodoResponse {
    pub id: i32,
    pub pid: Uuid,
    pub title: String,
    pub details: Option<String>,
    pub board_pid: Uuid,
    pub position: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct BoardResponse {
    pub pid: Uuid,
    pub title: String,
    pub todos: Vec<TodoResponse>,
    pub todo_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReorderParams {
    pub todos: Vec<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
    }
}

async fn load_item_by_pid(ctx: &AppContext, pid: &Uuid) -> Result<Model> {
    let pid_string = pid.to_string();
    let item = boards::Model::find_by_pid(&ctx.db, &pid_string).await?;
    Ok(item)
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
        .map(|(board, mut todos)| {
            todos.sort_by_key(|t| t.position);
            let todo_count = todos.len();
            let board_pid = board.pid;
            BoardResponse {
                pid: board.pid,
                title: board.title,
                todos: todos
                    .into_iter()
                    .map(|t| TodoResponse {
                        id: t.id,
                        pid: t.pid,
                        title: t.title,
                        details: t.details,
                        board_pid,
                        position: t.position,
                        created_at: t.created_at,
                        updated_at: t.updated_at,
                    })
                    .collect(),
                todo_count,
            }
        })
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
    
    format::json(BoardResponse {
        pid: item.pid,
        title: item.title,
        todos: vec![],
        todo_count: 0,
    })
}

#[debug_handler]
pub async fn update(
    Path(pid): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    
    format::json(BoardResponse {
        pid: item.pid,
        title: item.title,
        todos: vec![],
        todo_count: 0,
    })
}

#[debug_handler]
pub async fn remove(Path(pid): Path<Uuid>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item_by_pid(&ctx, &pid).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(pid): Path<Uuid>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    format::json(BoardResponse {
        pid: item.pid,
        title: item.title,
        todos: vec![],
        todo_count: 0,
    })
}

#[debug_handler]
pub async fn reorder(
    Path(pid): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<ReorderParams>,
) -> Result<Response> {
    let pid_string = pid.to_string();
    let board = boards::Model::find_by_pid(&ctx.db, &pid_string).await?;

    for (index, todo_pid) in params.todos.iter().enumerate() {
        let mut todo = todos::Model::find_by_pid(&ctx.db, &todo_pid)
            .await?
            .into_active_model();

        todo.position = Set(index as i32);
        todo.update(&ctx.db).await?;
    }

    let board_with_todos = Entity::find()
        .filter(boards::Column::Id.eq(board.id))
        .find_with_related(todos::Entity)
        .all(&ctx.db)
        .await?;

    let response: Vec<BoardResponse> = board_with_todos
        .into_iter()
        .map(|(board, mut todos)| {
            todos.sort_by_key(|t| t.position);
            let todo_count = todos.len();
            let board_pid = board.pid;
            BoardResponse {
                pid: board.pid,
                title: board.title,
                todos: todos
                    .into_iter()
                    .map(|t| TodoResponse {
                        id: t.id,
                        pid: t.pid,
                        title: t.title,
                        details: t.details,
                        board_pid,
                        position: t.position,
                        created_at: t.created_at,
                        updated_at: t.updated_at,
                    })
                    .collect(),
                todo_count,
            }
        })
        .collect();

    format::json(response)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/projects/{project_pid}/boards", get(list))
        .add("/projects/{project_pid}/boards", post(add))
        .add("/boards/{pid}", get(get_one))
        .add("/boards/{pid}", delete(remove))
        .add("/boards/{pid}", put(update))
        .add("/boards/{pid}", patch(update))
        .add("/boards/{pid}/reorder", post(reorder))
}
