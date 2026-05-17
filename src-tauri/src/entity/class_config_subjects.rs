use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "class_config_subjects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub config_id: i32,
    pub subject: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::class_configs::Entity",
        from = "Column::ConfigId",
        to = "super::class_configs::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ClassConfigs,
}

impl Related<super::class_configs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClassConfigs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
