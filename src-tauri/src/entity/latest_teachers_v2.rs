use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_teachers_v2")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub teacher_name: String,
    pub remark: Option<String>,
    pub is_middle_manager: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::latest_teacher_assignments_v2::Entity")]
    LatestTeacherAssignmentsV2,
    #[sea_orm(has_many = "super::latest_teacher_homerooms_v2::Entity")]
    LatestTeacherHomeroomsV2,
}

impl Related<super::latest_teacher_assignments_v2::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LatestTeacherAssignmentsV2.def()
    }
}

impl Related<super::latest_teacher_homerooms_v2::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LatestTeacherHomeroomsV2.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
