use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "comments",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                ("todo_id", ColType::Integer),
                ("author", ColType::String),
                ("content", ColType::Text),
            ],
            &[("todos", "todo_id")],
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("comments"))
                .add_column(
                    ColumnDef::new(Alias::new("is_ai"))
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "comments").await
    }
}
