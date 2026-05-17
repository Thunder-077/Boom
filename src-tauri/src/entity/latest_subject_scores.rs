use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "latest_subject_scores")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub admission_no: String,
    pub subject: String,
    pub score: Option<f64>,
    pub is_selected: i64,
    pub is_absent: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::latest_student_scores::Entity",
        from = "Column::AdmissionNo",
        to = "super::latest_student_scores::Column::AdmissionNo",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Student,
}

impl Related<super::latest_student_scores::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Student.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
