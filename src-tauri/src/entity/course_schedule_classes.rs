use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "course_schedule_classes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub import_id: i64,
    pub class_name: String,
    pub display_name: String,
    pub class_type: String,
    pub sort_index: i64,
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
