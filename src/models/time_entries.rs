pub use super::_entities::time_entries::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, QueryOrder};
use uuid::Uuid;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let mut this = self;
        if insert {
            this.pid = Set(Uuid::new_v4());
            this.created_at = Set(now);
        }
        this.updated_at = Set(now);
        Ok(this)
    }
}

impl Model {
    pub async fn find_active_for_todo<C>(db: &C, todo_id: i32) -> ModelResult<Option<Self>>
    where
        C: ConnectionTrait,
    {
        Entity::find()
            .filter(Column::TodoId.eq(todo_id))
            .filter(Column::StoppedAt.is_null())
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn find_all_for_todo<C>(db: &C, todo_id: i32) -> ModelResult<Vec<Self>>
    where
        C: ConnectionTrait,
    {
        Entity::find()
            .filter(Column::TodoId.eq(todo_id))
            .order_by_asc(Column::StartedAt)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// Returns total elapsed seconds across all completed entries + any running entry.
    pub fn total_seconds(entries: &[Self]) -> i64 {
        let now = chrono::Utc::now();
        entries
            .iter()
            .map(|e| {
                let stop = e.stopped_at.unwrap_or_else(|| now.into());
                (stop.timestamp() - e.started_at.timestamp()).max(0)
            })
            .sum()
    }
}

pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let rem_min = minutes % 60;
    if hours > 0 {
        format!("{}h {}m", hours, rem_min)
    } else {
        format!("{}m", minutes)
    }
}

impl Entity {
    pub async fn start<C>(
        db: &C,
        todo_id: i32,
        user_id: Option<i32>,
        is_ai: bool,
    ) -> ModelResult<Model>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            todo_id: Set(todo_id),
            user_id: Set(user_id),
            is_ai: Set(is_ai),
            started_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ModelError::Any(e.into()))
    }
}
