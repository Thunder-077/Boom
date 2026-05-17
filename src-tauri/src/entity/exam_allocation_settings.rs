use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "exam_allocation_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub exam_title: String,
    pub exam_notices_json: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
