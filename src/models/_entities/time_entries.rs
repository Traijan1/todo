use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "time_entries")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub todo_id: i32,
    pub user_id: Option<i32>,
    pub is_ai: bool,
    pub started_at: DateTimeWithTimeZone,
    pub stopped_at: Option<DateTimeWithTimeZone>,
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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Users,
}

impl Related<super::todos::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Todos.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}
