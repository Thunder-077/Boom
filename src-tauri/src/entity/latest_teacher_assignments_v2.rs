use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_teacher_assignments_v2")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub teacher_id: i32,
    pub subject: String,
    pub class_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::latest_teachers_v2::Entity",
        from = "Column::TeacherId",
        to = "super::latest_teachers_v2::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    LatestTeachersV2,
}

impl Related<super::latest_teachers_v2::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LatestTeachersV2.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
