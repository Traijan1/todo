#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260502_082213_projects;
mod m20260502_092455_boards;
mod m20260502_140120_todos;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260502_082213_projects::Migration),
            Box::new(m20260502_092455_boards::Migration),
            Box::new(m20260502_140120_todos::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}