pub use super::_entities::comments::{ActiveModel, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, QueryOrder};
use uuid::Uuid;

pub struct CreateComment {
    pub todo_id: i32,
    pub author: String,
    pub content: String,
    pub is_ai: bool,
}

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
    pub async fn create<C>(db: &C, params: CreateComment) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            todo_id: Set(params.todo_id),
            author: Set(params.author),
            content: Set(params.content),
            is_ai: Set(params.is_ai),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ModelError::Any(e.into()))
    }

    pub async fn find_by_todo_id<C>(db: &C, todo_id: i32) -> ModelResult<Vec<Self>>
    where
        C: ConnectionTrait,
    {
        use super::_entities::comments::Column;
        Entity::find()
            .filter(Column::TodoId.eq(todo_id))
            .order_by_asc(Column::CreatedAt)
            .all(db)
            .await
            .map_err(Into::into)
    }
}
