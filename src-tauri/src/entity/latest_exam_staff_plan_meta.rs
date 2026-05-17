use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_exam_staff_plan_meta")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub generated_at: String,
    pub session_count: i64,
    pub task_count: i64,
    pub assigned_count: i64,
    pub unassigned_count: i64,
    pub warning_count: i64,
    pub imbalance_minutes: i64,
    pub solver_engine: String,
    pub optimality_status: String,
    pub solve_duration_ms: i64,
    pub fallback_reason: Option<String>,
    pub fallback_pool_assignments: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
