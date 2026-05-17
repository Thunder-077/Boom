use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_exam_staff_assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub task_id: i64,
    pub teacher_id: i64,
    pub teacher_name: String,
    pub assigned_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::latest_exam_staff_tasks::Entity",
        from = "Column::TaskId",
        to = "super::latest_exam_staff_tasks::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::latest_exam_staff_tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
