use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "exam_grade_capacity_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub grade_name: String,
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
