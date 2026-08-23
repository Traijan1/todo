pub use super::_entities::user_settings::{ActiveModel, Column, Entity, Model};
use loco_rs::model::ModelResult;
use sea_orm::{entity::prelude::*, ActiveValue::Set};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now: DateTimeWithTimeZone = chrono::Utc::now().into();
        let mut this = self;
        if insert {
            this.created_at = Set(now);
        }
        this.updated_at = Set(now);
        Ok(this)
    }
}

impl Model {
    pub async fn get_or_default<C>(db: &C, user_id: i32) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        if let Some(settings) = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?
        {
            Ok(settings)
        } else {
            let now: DateTimeWithTimeZone = chrono::Utc::now().into();
            Ok(Model {
                id: 0,
                user_id,
                ollama_url: String::new(),
                default_model: None,
                created_at: now,
                updated_at: now,
            })
        }
    }

    pub async fn upsert<C>(
        db: &C,
        user_id: i32,
        ollama_url: String,
        default_model: Option<String>,
    ) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let existing = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?;

        if let Some(existing) = existing {
            let mut active: ActiveModel = existing.into();
            active.ollama_url = Set(ollama_url);
            active.default_model = Set(default_model);
            active.update(db).await.map_err(Into::into)
        } else {
            let active = ActiveModel {
                user_id: Set(user_id),
                ollama_url: Set(ollama_url),
                default_model: Set(default_model),
                ..Default::default()
            };
            active.insert(db).await.map_err(Into::into)
        }
    }
}
