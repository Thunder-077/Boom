use sea_orm_migration::prelude::*;

pub struct Migrator;

#[cfg(test)]
mod multi_grade_tests {
    use super::*;
    use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DbBackend, EntityTrait, Statement};

    #[test]
    fn test_migration_removes_stale_middle_manager_exceptions() {
        tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            Migrator::up(&db, Some(7)).await.unwrap();
            crate::entity::invigilation_config_settings::ActiveModel {
                id: sea_orm::ActiveValue::Set(1),
                default_exam_room_required_count: sea_orm::ActiveValue::Set(2),
                indoor_allowance_per_minute: sea_orm::ActiveValue::Set(0.5),
                outdoor_allowance_per_minute: sea_orm::ActiveValue::Set(0.3),
                floor_rover_enabled: sea_orm::ActiveValue::Set(1),
                middle_manager_default_enabled: sea_orm::ActiveValue::Set(0),
                middle_manager_exception_teacher_ids_json: sea_orm::ActiveValue::Set("[]".into()),
                self_study_date: sea_orm::ActiveValue::Set(String::new()),
                self_study_start_time: sea_orm::ActiveValue::Set("12:10".into()),
                self_study_end_time: sea_orm::ActiveValue::Set("13:40".into()),
                self_study_class_subjects_json: sea_orm::ActiveValue::Set("[]".into()),
                updated_at: sea_orm::ActiveValue::Set("old".into()),
            }
            .insert(&db)
            .await
            .unwrap();
            db.execute_unprepared(
                "INSERT INTO latest_teachers_v2 (id, teacher_name, is_middle_manager) VALUES (449, '匿名中层', 1);
                 UPDATE invigilation_config_settings SET middle_manager_exception_teacher_ids_json = '[128,175,449,221,449]' WHERE id = 1;",
            )
            .await
            .unwrap();
            Migrator::up(&db, None).await.unwrap();
            let config = crate::entity::invigilation_config_settings::Entity::find_by_id(1)
                .one(&db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(config.middle_manager_exception_teacher_ids_json, "[449]");
        });
    }

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
            Box::new(m20261116_000005_floor_rover_switch::Migration),
            Box::new(m20261117_000005_export_current::Migration),
            Box::new(m20261117_000006_teacher_exception_integrity::Migration),
        ]
    }
}

mod m20261117_000006_teacher_exception_integrity {
    use sea_orm_migration::prelude::*;

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20261117_000006_teacher_exception_integrity"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            // 历史失效 ID 已无法还原教师身份，只保留仍存在的教师例外。
            crate::db::repos::teacher::remove_stale_middle_manager_exceptions(
                manager.get_connection(),
            )
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))
        }

        async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }
}

mod m20261116_000005_floor_rover_switch {
    use sea_orm_migration::prelude::*;
    pub struct Migration;
    impl MigrationName for Migration {
        fn name(&self) -> &str { "m20261116_000005_floor_rover_switch" }
    }
    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager.alter_table(Table::alter().table(Alias::new("invigilation_config_settings"))
                .add_column(ColumnDef::new(Alias::new("floor_rover_enabled")).integer().not_null().default(1)).to_owned()).await
        }
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager.alter_table(Table::alter().table(Alias::new("invigilation_config_settings"))
                .drop_column(Alias::new("floor_rover_enabled")).to_owned()).await
        }
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
