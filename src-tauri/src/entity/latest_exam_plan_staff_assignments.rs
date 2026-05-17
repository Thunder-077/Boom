use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_exam_plan_staff_assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub session_id: i64,
    pub space_id: i64,
    pub teacher_name: String,
    pub assignment_type: String,
    pub note: Option<String>,
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
    #[sea_orm(
        belongs_to = "super::latest_exam_plan_spaces::Entity",
        from = "Column::SpaceId",
        to = "super::latest_exam_plan_spaces::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Space,
}

impl Related<super::latest_exam_plan_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<super::latest_exam_plan_spaces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Space.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
