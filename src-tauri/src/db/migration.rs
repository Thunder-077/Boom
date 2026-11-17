use sea_orm_migration::prelude::*;

pub struct Migrator;

#[cfg(test)]
mod multi_grade_tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

    #[test]
    fn test_multigrade_migration_preserves_legacy_data() {
        tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            Migrator::up(&db, Some(4)).await.unwrap();
            db.execute_raw(Statement::from_sql_and_values(DbBackend::Sqlite,
                "INSERT INTO latest_teachers_v2 (teacher_name, remark, is_middle_manager) VALUES (?, ?, ?)",
                ["匿名教师".into(), "原备注".into(), 0.into()])).await.unwrap();
            db.execute_raw(Statement::from_sql_and_values(DbBackend::Sqlite,
                "INSERT INTO latest_student_scores (admission_no, class_name, grade_name, student_name, subject_combination, language, total_score, class_rank, grade_rank, selected_subject_count) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                ["old-1".into(), "高一1班".into(), "高一".into(), "匿名学生".into(), "".into(), "英语".into(), 10.into(), 1.into(), 1.into(), 1.into()])).await.unwrap();
            Migrator::up(&db, None).await.unwrap();
            Migrator::up(&db, None).await.unwrap();
            let manager = SchemaManager::new(&db);
            assert!(manager
                .has_column("latest_student_scores", "remark")
                .await
                .unwrap());
            assert!(manager
                .has_column("latest_student_scores", "exam_group")
                .await
                .unwrap());
            assert!(manager
                .has_column("latest_exam_plan_spaces", "exam_group")
                .await
                .unwrap());
            let row = db
                .query_one_raw(Statement::from_string(
                    DbBackend::Sqlite,
                    "SELECT teacher_name, remark, invigilation_grade_name FROM latest_teachers_v2"
                        .to_owned(),
                ))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                row.try_get::<String>("", "teacher_name").unwrap(),
                "匿名教师"
            );
            assert_eq!(row.try_get::<String>("", "remark").unwrap(), "原备注");
            assert_eq!(
                row.try_get::<Option<String>>("", "invigilation_grade_name")
                    .unwrap(),
                None
            );
            let student = db
                .query_one_raw(Statement::from_string(
                    DbBackend::Sqlite,
                    "SELECT exam_group, remark FROM latest_student_scores".to_owned(),
                ))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                student.try_get::<String>("", "exam_group").unwrap(),
                "regular"
            );
            assert_eq!(
                student.try_get::<Option<String>>("", "remark").unwrap(),
                None
            );
        });
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260517_000000_legacy_baseline_name::Migration),
            Box::new(m20260517_000001_baseline::Migration),
            Box::new(m20260517_000002_course_import_settings::Migration),
            Box::new(m20261009_000003_grade_capacity::Migration),
            Box::new(m20261116_000004_multi_grade::Migration),
            Box::new(m20261117_000005_export_current::Migration),
        ]
    }
}

mod m20261116_000004_multi_grade {
    use sea_orm_migration::prelude::*;
    pub struct Migration;
    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20261116_000004_multi_grade"
        }
    }
    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            // 旧教师归属不能从任课信息猜测；保留空值，生成前提示重新导入。
            for (table, column) in [
                ("latest_student_scores", "remark"),
                ("latest_teachers_v2", "invigilation_grade_name"),
                ("latest_exam_plan_spaces", "exam_group"),
            ] {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new(table))
                            .add_column(ColumnDef::new(Alias::new(column)).string())
                            .to_owned(),
                    )
                    .await?;
            }
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("latest_student_scores"))
                        .add_column(
                            ColumnDef::new(Alias::new("exam_group"))
                                .string()
                                .not_null()
                                .default("regular"),
                        )
                        .to_owned(),
                )
                .await
        }
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            for (table, column) in [
                ("latest_student_scores", "remark"),
                ("latest_student_scores", "exam_group"),
                ("latest_teachers_v2", "invigilation_grade_name"),
                ("latest_exam_plan_spaces", "exam_group"),
            ] {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Alias::new(table))
                            .drop_column(Alias::new(column))
                            .to_owned(),
                    )
                    .await?;
            }
            Ok(())
        }
    }
}

mod m20261009_000003_grade_capacity {
    use sea_orm_migration::prelude::*;

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20261009_000003_grade_capacity"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            // 仅新增覆盖配置表；空表表示所有年级继续继承旧的全局人数。
            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("exam_grade_capacity_settings"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("grade_name"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("default_capacity"))
                                .big_integer()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("max_capacity"))
                                .big_integer()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("updated_at")).string().not_null())
                        .to_owned(),
                )
                .await
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(
                    Table::drop()
                        .table(Alias::new("exam_grade_capacity_settings"))
                        .to_owned(),
                )
                .await
        }
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
                            .add_column(ColumnDef::new(Alias::new("effective_start_date")).string())
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

mod m20261117_000005_export_current {
    use sea_orm_migration::prelude::*;
    pub struct Migration;
    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20261117_000005_export_current"
        }
    }
    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("export_current"))
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .big_integer()
                                .not_null()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(Alias::new("directory")).string().not_null())
                        .to_owned(),
                )
                .await
        }
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(Table::drop().table(Alias::new("export_current")).to_owned())
                .await
        }
    }
}
