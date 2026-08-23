use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("time_entries"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("pid"))
                        .uuid()
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(Alias::new("todo_id")).integer().not_null())
                .col(ColumnDef::new(Alias::new("user_id")).integer().null())
                .col(
                    ColumnDef::new(Alias::new("is_ai"))
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    ColumnDef::new(Alias::new("started_at"))
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("stopped_at"))
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Alias::new("time_entries"), Alias::new("todo_id"))
                        .to(Alias::new("todos"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Alias::new("time_entries"), Alias::new("user_id"))
                        .to(Alias::new("users"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Alias::new("time_entries")).to_owned())
            .await
    }
}
