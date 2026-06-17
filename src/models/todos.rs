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
    pub locked: Option<bool>,
    pub parent_pid: Option<String>,
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

    /// Find todos for a project with optional board, tag, and text filters, including tag data.
    pub async fn find_by_project_pid_with_tags<C>(
        db: &C,
        project_pid: &str,
        board_pid: Option<&str>,
        tag_pids: Option<&[String]>,
        search: Option<&str>,
    ) -> ModelResult<Vec<(Self, Vec<tags::Model>)>>
    where
        C: ConnectionTrait,
    {
        let all_boards = boards::Model::find_by_project_pid(db, project_pid).await?;

        let board_ids: Vec<i32> = if let Some(bpid) = board_pid {
            let uuid = Uuid::parse_str(bpid).map_err(|_| ModelError::EntityNotFound)?;
            all_boards
                .iter()
                .filter(|b| b.pid == uuid)
                .map(|b| b.id)
                .collect()
        } else {
            all_boards.iter().map(|b| b.id).collect()
        };

        if board_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut select = todos::Entity::find()
            .filter(Column::BoardId.is_in(board_ids))
            .filter(Column::ParentId.is_null());

        if let Some(s) = search {
            if !s.is_empty() {
                select = select.filter(Column::Title.contains(s));
            }
        }

        let results = select.find_with_related(tags::Entity).all(db).await?;

        let filtered = match tag_pids {
            Some(pids) if !pids.is_empty() => {
                let uuids: Vec<Uuid> = pids
                    .iter()
                    .filter_map(|p| Uuid::parse_str(p).ok())
                    .collect();
                results
                    .into_iter()
                    .filter(|(_, todo_tags)| todo_tags.iter().any(|t| uuids.contains(&t.pid)))
                    .collect()
            }
            _ => results,
        };

        Ok(filtered)
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

        let parent_id = if let Some(ppid) = &params.parent_pid {
            let uuid = Uuid::parse_str(ppid).map_err(|_| ModelError::EntityNotFound)?;
            let parent = Entity::find()
                .filter(Column::Pid.eq(uuid))
                .one(db)
                .await?
                .ok_or(ModelError::EntityNotFound)?;
            Some(parent.id)
        } else {
            None
        };

        let item = ActiveModel {
            title: Set(params.title.clone()),
            details: Set(params.details.clone()),
            board_id: Set(board.id),
            position: Set(count as i32),
            parent_id: Set(parent_id),
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
        todos_tags::Entity::insert(todos_tags::ActiveModel {
            todo_id: Set(self.id),
            tag_id: Set(tag.id),
            ..Default::default()
        })
        .exec_without_returning(db)
        .await
        .map_err(|e| ModelError::Any(e.into()))?;
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

    /// Replaces all tags on this todo. `tag_pids` is a list of tag PIDs (UUIDs as strings).
    pub async fn sync_tags<C>(
        &self,
        db: &C,
        tag_pids: Vec<String>,
        _user: &users::Model,
    ) -> ModelResult<()>
    where
        C: ConnectionTrait,
    {
        todos_tags::Entity::delete_many()
            .filter(crate::models::_entities::todos_tags::Column::TodoId.eq(self.id))
            .exec(db)
            .await
            .map_err(|e| ModelError::Any(e.into()))?;

        for pid_str in tag_pids {
            let tag = tags::Model::find_by_pid(db, &pid_str).await?;
            todos_tags::Entity::insert(todos_tags::ActiveModel {
                todo_id: Set(self.id),
                tag_id: Set(tag.id),
                ..Default::default()
            })
            .exec_without_returning(db)
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
