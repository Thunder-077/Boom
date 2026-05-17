use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "latest_exam_plan_meta")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub generated_at: String,
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub grade_count: i64,
    pub session_count: i64,
    pub warning_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
