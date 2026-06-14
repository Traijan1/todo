use crate::models::_entities::tags::Column;

pub use super::_entities::tags::{ActiveModel, Entity, Model};
use crate::models::users;
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
pub type Tags = Entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagParams {
    pub title: String,
    pub color: Option<String>,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = Set(chrono::Utc::now().into());
            Ok(this)
        } else if insert {
            let mut this = self;
            this.pid = Set(Uuid::new_v4());

            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {
    pub async fn find_by_pid<C>(db: &C, pid: &str) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let pid = Uuid::parse_str(pid).map_err(|_| ModelError::EntityNotFound)?;
        let item = Entity::find().filter(Column::Pid.eq(pid)).one(db).await?;

        item.ok_or(ModelError::EntityNotFound)
    }

    pub async fn create<C>(db: &C, params: &TagParams, user: &users::Model) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            title: Set(params.title.clone()),
            color: Set(params.color.clone()),
            user_id: Set(user.id),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ModelError::Any(e.into()))
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
