#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260502_082213_projects;
mod m20260502_092455_boards;
mod m20260502_140120_todos;
mod m20260503_124519_add_position_to_todos;
mod m20260503_212143_add_position_to_boards;
mod m20260506_140547_tags;
mod m20260615_000001_tags_project_scope;
mod m20260615_000002_add_locked_to_todos;
mod m20260615_000003_add_parent_id_to_todos;
mod m20260617_000001_comments;
mod m20260617_000002_project_settings;
mod m20260618_000001_project_roles;
mod m20260618_000002_comments_user_id;
mod m20260618_000003_time_entries;
mod m20260823_000001_projects_ai;
mod m20260823_000002_user_settings;
mod m20260823_000003_rename_ai_system_prompt;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260502_082213_projects::Migration),
            Box::new(m20260502_092455_boards::Migration),
            Box::new(m20260502_140120_todos::Migration),
            Box::new(m20260503_124519_add_position_to_todos::Migration),
            Box::new(m20260503_212143_add_position_to_boards::Migration),
            Box::new(m20260506_140547_tags::Migration),
            Box::new(m20260615_000001_tags_project_scope::Migration),
            Box::new(m20260615_000002_add_locked_to_todos::Migration),
            Box::new(m20260615_000003_add_parent_id_to_todos::Migration),
            Box::new(m20260617_000001_comments::Migration),
            Box::new(m20260617_000002_project_settings::Migration),
            Box::new(m20260618_000001_project_roles::Migration),
            Box::new(m20260618_000002_comments_user_id::Migration),
            Box::new(m20260618_000003_time_entries::Migration),
            Box::new(m20260823_000001_projects_ai::Migration),
            Box::new(m20260823_000002_user_settings::Migration),
            Box::new(m20260823_000003_rename_ai_system_prompt::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
