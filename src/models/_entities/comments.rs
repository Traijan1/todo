//! `SeaORM` Entity for todo comments

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub todo_id: i32,
    pub author: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub is_ai: bool,
    pub user_id: Option<i32>, // FK→users, null for AI comments
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::todos::Entity",
        from = "Column::TodoId",
        to = "super::todos::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Todos,
}

impl Related<super::todos::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Todos.def()
    }
}
