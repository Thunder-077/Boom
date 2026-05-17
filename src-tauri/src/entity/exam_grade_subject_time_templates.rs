use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "exam_grade_subject_time_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub grade_name: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub subject: String,
    pub start_at: String,
    pub end_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
