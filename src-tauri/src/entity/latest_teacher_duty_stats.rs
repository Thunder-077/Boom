use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_teacher_duty_stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub teacher_id: i64,
    pub teacher_name: String,
    pub indoor_minutes: i64,
    pub outdoor_minutes: i64,
    pub total_minutes: i64,
    pub task_count: i64,
    pub exam_room_task_count: i64,
    pub self_study_task_count: i64,
    pub floor_rover_task_count: i64,
    pub is_middle_manager: i64,
    pub allowance_total: f64,
    pub indoor_allowance_total: f64,
    pub outdoor_allowance_total: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
