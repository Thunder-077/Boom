use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "course_schedule_changes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub import_id: i64,
    pub source_entry_id: i64,
    pub change_type: String,
    pub status: String,
    pub target_date: String,
    pub source_teacher_name: String,
    pub actual_teacher_name: String,
    pub reason: String,
    pub remark: String,
    pub created_at: String,
    pub updated_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::course_schedule_imports::Entity",
        from = "Column::ImportId",
        to = "super::course_schedule_imports::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Import,
    #[sea_orm(
        belongs_to = "super::course_schedule_entries::Entity",
        from = "Column::SourceEntryId",
        to = "super::course_schedule_entries::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SourceEntry,
}

impl Related<super::course_schedule_imports::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Import.def()
    }
}

impl Related<super::course_schedule_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SourceEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
