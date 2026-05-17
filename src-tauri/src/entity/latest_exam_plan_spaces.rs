use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_exam_plan_spaces")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub session_id: i64,
    pub space_type: String,
    pub space_source: String,
    pub grade_name: String,
    pub subject: String,
    pub space_name: String,
    pub original_class_name: Option<String>,
    pub self_study_topic_kind: Option<String>,
    pub self_study_topic_subjects_json: Option<String>,
    pub self_study_topic_label: Option<String>,
    pub building: String,
    pub floor: String,
    pub capacity: Option<i64>,
    pub sort_index: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::latest_exam_plan_sessions::Entity",
        from = "Column::SessionId",
        to = "super::latest_exam_plan_sessions::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Session,
}

impl Related<super::latest_exam_plan_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
