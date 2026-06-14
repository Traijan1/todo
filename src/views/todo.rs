use crate::models::{_entities::todos, _entities::tags};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoResponse {
    pub pid: String,
    pub title: String,
    pub details: Option<String>,
    pub position: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<tags::Model>,
}

impl TodoResponse {
    pub fn from(todo: todos::Model, tags: Vec<tags::Model>) -> Self {
        Self {
            pid: todo.pid.to_string(),
            title: todo.title,
            details: todo.details,
            position: todo.position,
            created_at: todo.created_at.with_timezone(&chrono::Utc),
            updated_at: todo.updated_at.with_timezone(&chrono::Utc),
            tags,
        }
    }
}
