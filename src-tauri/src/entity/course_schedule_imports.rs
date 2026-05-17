use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "course_schedule_imports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub imported_at: String,
    pub source_file: String,
    pub entry_count: i64,
    pub teacher_count: i64,
    pub admin_class_count: i64,
    pub foreign_class_count: i64,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub start_week: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::course_schedule_classes::Entity")]
    Classes,
    #[sea_orm(has_many = "super::course_schedule_entries::Entity")]
    Entries,
    #[sea_orm(has_many = "super::course_schedule_periods::Entity")]
    Periods,
    #[sea_orm(has_many = "super::course_schedule_changes::Entity")]
    Changes,
}

impl Related<super::course_schedule_classes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Classes.def()
    }
}

impl Related<super::course_schedule_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entries.def()
    }
}

impl Related<super::course_schedule_periods::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Periods.def()
    }
}

impl Related<super::course_schedule_changes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Changes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
