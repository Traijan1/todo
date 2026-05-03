use std::str::FromStr;

use crate::models::{
    _entities::boards::{self, Column},
    projects::{self, Projects},
};

pub use super::_entities::boards::{ActiveModel, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
pub type Boards = Entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardParams {
    pub title: String,
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
    pub async fn todo_count<C>(&self, db: &C) -> ModelResult<u64>
    where
        C: ConnectionTrait,
    {
        self.find_related(crate::models::_entities::todos::Entity)
            .count(db)
            .await
            .map_err(ModelError::from)
    }

    pub async fn find_by_pid<C>(db: &C, pid: &str) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let pid = Uuid::parse_str(pid).map_err(|_| ModelError::EntityNotFound)?;
        let item = Entity::find().filter(Column::Pid.eq(pid)).one(db).await?;

        item.ok_or(ModelError::EntityNotFound)
    }

    pub async fn find_by_project_pid<C>(db: &C, pid: &str) -> ModelResult<Vec<Self>>
    where
        C: ConnectionTrait,
    {
        let project = projects::Entity::find()
            .filter(
                projects::Column::Pid
                    .eq(Uuid::from_str(pid).map_err(|_| ModelError::EntityNotFound)?),
            )
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)?;

        let items = boards::Entity::find()
            .filter(boards::Column::ProjectId.eq(project.id))
            .all(db)
            .await?;

        Ok(items)
    }

    pub async fn create<C>(
        db: &C,
        params: &BoardParams,
        project: &projects::Model,
    ) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            title: Set(params.title.clone()),
            project_id: Set(project.id),
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
