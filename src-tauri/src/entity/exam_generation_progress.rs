use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "exam_generation_progress")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub status: String,
    pub stage: String,
    pub stage_label: String,
    pub percent: i64,
    pub message: String,
    pub current_grade: Option<String>,
    pub total_grades: i64,
    pub completed_grades: i64,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
