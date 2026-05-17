use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "invigilation_custom_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub action_type: String,
    pub teacher_id: i64,
    pub teacher_name: String,
    pub time_scope_type: String,
    pub time_scope_ids_json: String,
    pub time_scope_labels_json: String,
    pub task_scope_type: String,
    pub target_scope_type: String,
    pub target_ids_json: String,
    pub target_labels_json: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
