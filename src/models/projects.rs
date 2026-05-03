use crate::models::boards;

pub use super::_entities::projects::{ActiveModel, Column, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
pub type Projects = Entity;

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

    async fn after_save<C>(
        model: <Self::Entity as EntityTrait>::Model,
        db: &C,
        insert: bool,
    ) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        C: ConnectionTrait,
    {
        tracing::debug!("after_save hook triggered. insert={}", insert);
        if insert {
            tracing::debug!("Creating default boards for project id={}", model.id);
            boards::ActiveModel {
                project_id: Set(model.id),
                pid: Set(Uuid::new_v4()),
                title: Set("Todo".to_string()),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create Todo board: {:?}", e);
                e
            })?;

            boards::ActiveModel {
                project_id: Set(model.id),
                pid: Set(Uuid::new_v4()),
                title: Set("In Progress".to_string()),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create In Progress board: {:?}", e);
                e
            })?;

            boards::ActiveModel {
                project_id: Set(model.id),
                pid: Set(Uuid::new_v4()),
                title: Set("Done".to_string()),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create Done board: {:?}", e);
                e
            })?;
            tracing::debug!("Default boards created successfully.");
        }

        Ok(model)
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
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
