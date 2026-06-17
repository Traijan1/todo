use crate::models::{_entities::tags, _entities::todos};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubtaskItem {
    pub pid: String,
    pub title: String,
    pub locked: bool,
    pub tags: Vec<tags::Model>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoResponse {
    pub pid: String,
    pub title: String,
    pub details: Option<String>,
    pub position: i32,
    pub locked: bool,
    pub parent_pid: Option<String>,
    pub subtasks: Vec<SubtaskItem>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<tags::Model>,
}

impl TodoResponse {
    pub fn from(
        todo: todos::Model,
        tags: Vec<tags::Model>,
        parent_pid: Option<String>,
        subtasks: Vec<SubtaskItem>,
    ) -> Self {
        Self {
            pid: todo.pid.to_string(),
            title: todo.title,
            details: todo.details,
            position: todo.position,
            locked: todo.locked,
            parent_pid,
            subtasks,
            created_at: todo.created_at.with_timezone(&chrono::Utc),
            updated_at: todo.updated_at.with_timezone(&chrono::Utc),
            tags,
        }
    }
}
