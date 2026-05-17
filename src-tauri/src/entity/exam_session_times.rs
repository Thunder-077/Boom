use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "exam_session_times")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: i64,
    pub subject: String,
    pub start_at: String,
    pub end_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::latest_exam_plan_sessions::Entity",
        from = "Column::SessionId",
        to = "super::latest_exam_plan_sessions::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Session,
}

impl Related<super::latest_exam_plan_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
