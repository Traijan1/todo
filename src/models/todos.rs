use crate::models::{
    _entities::todos::{self, Column},
    boards,
};

pub use super::_entities::todos::{ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
pub type Todos = Entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoParams {
    pub title: String,
    pub details: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct TodoValidator {
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(TodoValidator {
            title: self.title.as_ref().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.validate()?;
        let mut this = self;
        if !insert && this.updated_at.is_unchanged() {
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
        } else if insert {
            this.pid = Set(Uuid::new_v4());
        }
        Ok(this)
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

    pub async fn find_by_project_pid<C>(db: &C, pid: &str) -> ModelResult<Vec<Self>>
    where
        C: ConnectionTrait,
    {
        let mut items: Vec<Self> = vec![];
        let boards = boards::Model::find_by_project_pid(db, pid).await?;

        for board in boards {
            let mut todos = todos::Entity::find()
                .filter(todos::Column::BoardId.eq(board.id))
                .all(db)
                .await?;

            items.append(&mut todos);
        }

        Ok(items)
    }

    pub async fn create<C>(db: &C, params: &TodoParams, board: &boards::Model) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let count = board.todo_count(db).await?;
        ActiveModel {
            title: Set(params.title.clone()),
            details: Set(params.details.clone()),
            board_id: Set(board.id),
            position: Set(count as i32),
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
