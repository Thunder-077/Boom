use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "course_schedule_periods")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub import_id: i64,
    pub week_index: i64,
    pub day_of_week: i64,
    pub day_label: String,
    pub period_index: i64,
    pub period_label: String,
    pub section_label: String,
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
}

impl Related<super::course_schedule_imports::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Import.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
