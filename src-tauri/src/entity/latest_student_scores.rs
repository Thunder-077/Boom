use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_student_scores")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub admission_no: String,
    pub class_name: String,
    pub grade_name: String,
    pub student_name: String,
    pub subject_combination: String,
    pub language: String,
    pub total_score: f64,
    pub class_rank: i64,
    pub grade_rank: i64,
    pub selected_subject_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::latest_subject_scores::Entity")]
    SubjectScores,
}

impl Related<super::latest_subject_scores::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubjectScores.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
