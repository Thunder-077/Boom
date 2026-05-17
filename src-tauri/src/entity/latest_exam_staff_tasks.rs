use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_exam_staff_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub session_id: Option<i64>,
    pub space_id: Option<i64>,
    pub task_source: String,
    pub role: String,
    pub grade_name: String,
    pub subject: String,
    pub space_name: String,
    pub floor: String,
    pub start_at: String,
    pub end_at: String,
    pub duration_minutes: i64,
    pub recommended_self_study_topic_kind: Option<String>,
    pub recommended_self_study_topic_subjects_json: Option<String>,
    pub recommended_self_study_topic_label: Option<String>,
    pub priority_self_study_chain_json: String,
    pub assignment_tier: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub allowance_amount: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::latest_exam_staff_assignments::Entity")]
    Assignments,
}

impl Related<super::latest_exam_staff_assignments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assignments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
