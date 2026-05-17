use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260517_000000_legacy_baseline_name::Migration),
            Box::new(m20260517_000001_baseline::Migration),
            Box::new(m20260517_000002_course_import_settings::Migration),
        ]
    }
}

mod m20260517_000000_legacy_baseline_name {
    use sea_orm_migration::prelude::*;

    const SCHEMA_SQL: &str = include_str!("../schema.sql");

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "migration"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .get_connection()
                .execute_unprepared(SCHEMA_SQL)
                .await?;
            Ok(())
        }

        async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }
}

mod m20260517_000001_baseline {
    use sea_orm_migration::prelude::*;

    const SCHEMA_SQL: &str = include_str!("../schema.sql");

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20260517_000001_baseline"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .get_connection()
                .execute_unprepared(SCHEMA_SQL)
                .await?;
            Ok(())
        }

        async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }
}

mod m20260517_000002_course_import_settings {
    use sea_orm_migration::prelude::*;

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20260517_000002_course_import_settings"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            if !manager
                .has_column("course_schedule_imports", "effective_start_date")
                .await?
            {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new("course_schedule_imports"))
                            .add_column(
                                ColumnDef::new(Alias::new("effective_start_date")).string(),
                            )
                            .to_owned(),
                    )
                    .await?;
            }
            if !manager
                .has_column("course_schedule_imports", "effective_end_date")
                .await?
            {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new("course_schedule_imports"))
                            .add_column(ColumnDef::new(Alias::new("effective_end_date")).string())
                            .to_owned(),
                    )
                    .await?;
            }
            if !manager
                .has_column("course_schedule_imports", "start_week")
                .await?
            {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new("course_schedule_imports"))
                            .add_column(
                                ColumnDef::new(Alias::new("start_week"))
                                    .integer()
                                    .not_null()
                                    .default(1),
                            )
                            .to_owned(),
                    )
                    .await?;
            }
            Ok(())
        }

        async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }
}
