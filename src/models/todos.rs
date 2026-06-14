use crate::models::{
    _entities::todos::{self, Column},
    boards, tags, todos_tags, users,
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
    pub tags: Option<Vec<String>>,
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

    pub async fn create<C>(
        db: &C,
        params: &TodoParams,
        board: &boards::Model,
        user: &users::Model,
    ) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let count = board.todo_count(db).await?;
        let item = ActiveModel {
            title: Set(params.title.clone()),
            details: Set(params.details.clone()),
            board_id: Set(board.id),
            position: Set(count as i32),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ModelError::Any(e.into()))?;

        if let Some(tags) = &params.tags {
            item.sync_tags(db, tags.clone(), user).await?;
        }

        Ok(item)
    }

    pub async fn add_tag<C>(&self, db: &C, tag: &tags::Model) -> ModelResult<()>
    where
        C: ConnectionTrait,
    {
        let link = todos_tags::ActiveModel {
            todo_id: Set(self.id),
            tag_id: Set(tag.id),
            ..Default::default()
        };
        link.insert(db).await.map_err(|e| ModelError::Any(e.into()))?;
        Ok(())
    }

    pub async fn remove_tag<C>(&self, db: &C, tag: &tags::Model) -> ModelResult<()>
    where
        C: ConnectionTrait,
    {
        todos_tags::Entity::delete_many()
            .filter(crate::models::_entities::todos_tags::Column::TodoId.eq(self.id))
            .filter(crate::models::_entities::todos_tags::Column::TagId.eq(tag.id))
            .exec(db)
            .await
            .map_err(|e| ModelError::Any(e.into()))?;
        Ok(())
    }

    pub async fn sync_tags<C>(
        &self,
        db: &C,
        tag_titles: Vec<String>,
        user: &users::Model,
    ) -> ModelResult<()>
    where
        C: ConnectionTrait,
    {
        // Remove existing links
        todos_tags::Entity::delete_many()
            .filter(crate::models::_entities::todos_tags::Column::TodoId.eq(self.id))
            .exec(db)
            .await
            .map_err(|e| ModelError::Any(e.into()))?;

        for title in tag_titles {
            // Find or create tag
            let tag = tags::Entity::find()
                .filter(crate::models::_entities::tags::Column::Title.eq(title.clone()))
                .filter(crate::models::_entities::tags::Column::UserId.eq(user.id))
                .one(db)
                .await
                .map_err(|e| ModelError::Any(e.into()))?;

            let tag = if let Some(tag) = tag {
                tag
            } else {
                tags::ActiveModel {
                    title: Set(title),
                    user_id: Set(user.id),
                    ..Default::default()
                }
                .insert(db)
                .await
                .map_err(|e| ModelError::Any(e.into()))?
            };

            // Link tag to todo
            todos_tags::ActiveModel {
                todo_id: Set(self.id),
                tag_id: Set(tag.id),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| ModelError::Any(e.into()))?;
        }

        Ok(())
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
