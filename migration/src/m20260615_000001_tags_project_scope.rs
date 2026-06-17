use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "todos_tags").await?;
        drop_table(m, "tags").await?;

        create_table(
            m,
            "tags",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                ("title", ColType::String),
                ("color", ColType::StringNull),
                ("project_id", ColType::Integer),
            ],
            &[("projects", "project_id")],
        )
        .await?;

        create_join_table(m, "todos_tags", &[], &[("todo", ""), ("tag", "")]).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "todos_tags").await?;
        drop_table(m, "tags").await?;

        create_table(
            m,
            "tags",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                ("title", ColType::String),
                ("color", ColType::StringNull),
                ("user_id", ColType::Integer),
            ],
            &[("users", "user_id")],
        )
        .await?;

        create_join_table(m, "todos_tags", &[], &[("todo", ""), ("tag", "")]).await?;

        Ok(())
    }
}
