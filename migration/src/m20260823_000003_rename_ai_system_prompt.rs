use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .rename_column(Alias::new("ai_prompt"), Alias::new("ai_system_prompt"))
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("projects"))
                .rename_column(Alias::new("ai_system_prompt"), Alias::new("ai_prompt"))
                .to_owned(),
        )
        .await
    }
}
