use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "invigilation_config_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub default_exam_room_required_count: i64,
    pub indoor_allowance_per_minute: f64,
    pub outdoor_allowance_per_minute: f64,
    pub middle_manager_default_enabled: i64,
    pub middle_manager_exception_teacher_ids_json: String,
    pub self_study_date: String,
    pub self_study_start_time: String,
    pub self_study_end_time: String,
    pub self_study_class_subjects_json: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
