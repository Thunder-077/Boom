use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "class_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub config_type: String,
    pub grade_name: String,
    pub class_name: String,
    pub building: String,
    pub floor: String,
    pub room_label: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::class_config_subjects::Entity")]
    ClassConfigSubjects,
}

impl Related<super::class_config_subjects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClassConfigSubjects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
