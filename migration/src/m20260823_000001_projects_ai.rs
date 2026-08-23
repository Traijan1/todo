use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .add_column(ColumnDef::new(Alias::new("ai_provider")).string().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .add_column(ColumnDef::new(Alias::new("ai_model")).string().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .add_column(ColumnDef::new(Alias::new("ai_prompt")).text().null())
                .to_owned(),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "projects", "ai_prompt").await?;
        remove_column(m, "projects", "ai_model").await?;
        remove_column(m, "projects", "ai_provider").await?;
        Ok(())
    }
}
