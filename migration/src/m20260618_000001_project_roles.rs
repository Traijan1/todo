use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // All existing members retroactively become owners (no history to distinguish)
        m.alter_table(
            Table::alter()
                .table(Alias::new("users_projects"))
                .add_column(
                    ColumnDef::new(Alias::new("role"))
                        .string()
                        .not_null()
                        .default("owner"),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("users_projects"))
                .drop_column(Alias::new("role"))
                .to_owned(),
        )
        .await
    }
}
