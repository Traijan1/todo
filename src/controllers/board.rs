#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};

use crate::{
    controllers::ws,
    models::_entities::{
        boards::{self, ActiveModel, Entity, Model},
        projects, tags, todos,
    },
    views::todo::SubtaskItem,
};

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: i32,
    pub pid: Uuid,
    pub title: String,
    pub details: Option<String>,
    pub board_pid: Uuid,
    pub position: i32,
    pub locked: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub tags: Vec<tags::Model>,
    pub subtasks: Vec<SubtaskItem>,
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

fn build_subtasks_map(
    subtasks_raw: Vec<(todos::Model, Vec<tags::Model>)>,
) -> std::collections::HashMap<i32, Vec<SubtaskItem>> {
    let mut map: std::collections::HashMap<i32, Vec<SubtaskItem>> = std::collections::HashMap::new();
    for (subtask, stags) in subtasks_raw {
        if let Some(parent_id) = subtask.parent_id {
            map.entry(parent_id).or_default().push(SubtaskItem {
                pid: subtask.pid.to_string(),
                title: subtask.title,
                locked: subtask.locked,
                tags: stags,
            });
        }
    }
    map
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

    let boards_list = Entity::find()
        .filter(boards::Column::ProjectId.eq(project.id))
        .order_by_asc(boards::Column::Position)
        .all(&ctx.db)
        .await?;

    let board_ids: Vec<i32> = boards_list.iter().map(|b| b.id).collect();

    let todos_with_tags: Vec<(todos::Model, Vec<tags::Model>)> = if board_ids.is_empty() {
        vec![]
    } else {
        todos::Entity::find()
            .filter(todos::Column::BoardId.is_in(board_ids))
            .filter(todos::Column::ParentId.is_null())
            .order_by_asc(todos::Column::Position)
            .find_with_related(tags::Entity)
            .all(&ctx.db)
            .await?
    };

    let parent_ids: Vec<i32> = todos_with_tags.iter().map(|(t, _)| t.id).collect();
    let subtasks_raw = if parent_ids.is_empty() {
        vec![]
    } else {
        todos::Entity::find()
            .filter(todos::Column::ParentId.is_in(parent_ids))
            .order_by_asc(todos::Column::Position)
            .find_with_related(tags::Entity)
            .all(&ctx.db)
            .await?
    };
    let mut subtasks_by_parent = build_subtasks_map(subtasks_raw);

    let mut todos_by_board: std::collections::HashMap<i32, Vec<(todos::Model, Vec<tags::Model>)>> =
        std::collections::HashMap::new();
    for (todo, todo_tags) in todos_with_tags {
        todos_by_board.entry(todo.board_id).or_default().push((todo, todo_tags));
    }

    let mut response: Vec<BoardResponse> = Vec::new();
    for board in boards_list {
        let board_todos = todos_by_board.remove(&board.id).unwrap_or_default();
        let todo_count = board_todos.len();
        let board_pid = board.pid;
        let mut todos = Vec::new();
        for (t, todo_tags) in board_todos {
            let subtasks = subtasks_by_parent.remove(&t.id).unwrap_or_default();
            todos.push(TodoResponse {
                id: t.id,
                pid: t.pid,
                title: t.title,
                details: t.details,
                board_pid,
                position: t.position,
                locked: t.locked,
                created_at: t.created_at,
                updated_at: t.updated_at,
                tags: todo_tags,
                subtasks,
            });
        }
        response.push(BoardResponse {
            pid: board.pid,
            title: board.title,
            todos,
            todo_count,
        });
    }

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
    ws::broadcast(&project_pid.to_string()).await;

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
    let project_id = item.project_id;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    ws::broadcast_for_project_id(&ctx.db, project_id).await;

    format::json(BoardResponse {
        pid: item.pid,
        title: item.title,
        todos: vec![],
        todo_count: 0,
    })
}

#[debug_handler]
pub async fn remove(Path(pid): Path<Uuid>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item_by_pid(&ctx, &pid).await?;
    let project_id = item.project_id;
    item.delete(&ctx.db).await?;
    ws::broadcast_for_project_id(&ctx.db, project_id).await;
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
    let project_id = board.project_id;

    for (index, todo_pid) in params.todos.iter().enumerate() {
        let mut todo = todos::Model::find_by_pid(&ctx.db, &todo_pid)
            .await?
            .into_active_model();

        todo.position = Set(index as i32);
        todo.update(&ctx.db).await?;
    }

    let board_pid = board.pid;
    let todos_with_tags = todos::Entity::find()
        .filter(todos::Column::BoardId.eq(board.id))
        .filter(todos::Column::ParentId.is_null())
        .order_by_asc(todos::Column::Position)
        .find_with_related(tags::Entity)
        .all(&ctx.db)
        .await?;

    let parent_ids: Vec<i32> = todos_with_tags.iter().map(|(t, _)| t.id).collect();
    let subtasks_raw = if parent_ids.is_empty() {
        vec![]
    } else {
        todos::Entity::find()
            .filter(todos::Column::ParentId.is_in(parent_ids))
            .order_by_asc(todos::Column::Position)
            .find_with_related(tags::Entity)
            .all(&ctx.db)
            .await?
    };
    let mut subtasks_by_parent = build_subtasks_map(subtasks_raw);

    let todo_count = todos_with_tags.len();
    let mut todos = Vec::new();
    for (t, todo_tags) in todos_with_tags {
        let subtasks = subtasks_by_parent.remove(&t.id).unwrap_or_default();
        todos.push(TodoResponse {
            id: t.id,
            pid: t.pid,
            title: t.title,
            details: t.details,
            board_pid,
            position: t.position,
            locked: t.locked,
            created_at: t.created_at,
            updated_at: t.updated_at,
            tags: todo_tags,
            subtasks,
        });
    }

    ws::broadcast_for_project_id(&ctx.db, project_id).await;
    format::json(vec![BoardResponse {
        pid: board.pid,
        title: board.title,
        todos,
        todo_count,
    }])
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
