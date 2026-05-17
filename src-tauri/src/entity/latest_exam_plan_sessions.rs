use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_exam_plan_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub grade_name: String,
    pub subject: String,
    pub is_foreign_group: i64,
    pub foreign_order: Option<i64>,
    pub participant_count: i64,
    pub exam_room_count: i64,
    pub self_study_room_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::exam_session_times::Entity")]
    SessionTimes,
    #[sea_orm(has_many = "super::latest_exam_plan_spaces::Entity")]
    Spaces,
}

impl Related<super::exam_session_times::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SessionTimes.def()
    }
}

impl Related<super::latest_exam_plan_spaces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Spaces.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
