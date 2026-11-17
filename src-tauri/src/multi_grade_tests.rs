use crate::{entity::*, exam_plan_snapshot::ExamPlanDraft, score::Subject};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

fn teacher_fixture(root: &Path) -> PathBuf {
    let path = root.join("teachers.xlsx");
    let mut book = rust_xlsxwriter::Workbook::new();
    for grade in ["高一", "高二", "高三"] {
        let sheet = book.add_worksheet();
        sheet.set_name(grade).unwrap();
        for (col, label) in ["教师姓名", "任教学科", "任教班级", "备注"]
            .into_iter()
            .enumerate()
        {
            sheet.write_string(0, col as u16, label).unwrap();
        }
        for index in 0..12 {
            // 同一Sheet重复行应聚合；跨年级任课不能改变Sheet归属。
            let name = format!("{grade}教师{}", index / 2);
            let subject = if index < 8 { "数学" } else { "历史" };
            for (col, text) in [name.as_str(), subject, "101/204", ""]
                .into_iter()
                .enumerate()
            {
                sheet.write_string(index + 1, col as u16, text).unwrap();
            }
        }
    }
    book.save(&path).unwrap();
    path
}

async fn setup() -> (DatabaseConnection, PathBuf, ExamPlanDraft) {
    let root = std::env::temp_dir().join(format!(
        "boom-pipeline-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    fs::create_dir_all(&root).unwrap();
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::db::migration::Migrator::up(&db, None).await.unwrap();
    invigilation_config_settings::ActiveModel {
        id: Set(1),
        default_exam_room_required_count: Set(2),
        indoor_allowance_per_minute: Set(0.5),
        outdoor_allowance_per_minute: Set(0.3),
        floor_rover_enabled: Set(1),
        middle_manager_default_enabled: Set(0),
        middle_manager_exception_teacher_ids_json: Set("[]".into()),
        self_study_date: Set("2026-11-18".into()),
        self_study_start_time: Set("12:10".into()),
        self_study_end_time: Set("13:40".into()),
        self_study_class_subjects_json: Set("[]".into()),
        updated_at: Set("now".into()),
    }
    .insert(&db)
    .await
    .unwrap();
    exam_allocation_settings::ActiveModel {
        id: Set(1),
        default_capacity: Set(2),
        max_capacity: Set(2),
        exam_title: Set("测试".into()),
        exam_notices_json: Set("[]".into()),
        updated_at: Set("now".into()),
    }
    .insert(&db)
    .await
    .unwrap();
    let scores = crate::score::multi_sheet_tests::fixture("pipeline");
    assert_eq!(
        crate::score::import_scores_into_db(&db, scores.to_str().unwrap(), "scores-v1")
            .await
            .unwrap(),
        9
    );
    fs::remove_file(scores).unwrap();
    let teachers = teacher_fixture(&root);
    assert_eq!(
        crate::teacher::import_teachers_into_db(&db, teachers.to_str().unwrap(), "teachers-v1")
            .await
            .unwrap(),
        18
    );
    let mut class_ids = Vec::new();
    for grade in ["高一", "高二", "高三"] {
        for number in [1, 11, 13, 14, 15, 16] {
            if number == 11 && grade != "高三" {
                continue;
            }
            let teaching = number <= 11;
            let row = class_configs::ActiveModel {
                config_type: Set(if teaching {
                    "teaching_class"
                } else {
                    "exam_room"
                }
                .into()),
                grade_name: Set(grade.into()),
                class_name: Set(format!(
                    "{grade}{number}{}",
                    if teaching { "班" } else { "场" }
                )),
                building: Set("测试楼".into()),
                floor: Set("一楼".into()),
                created_at: Set("now".into()),
                updated_at: Set("now".into()),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
            if teaching {
                class_ids.push(row.id);
                class_config_subjects::ActiveModel {
                    config_id: Set(row.id),
                    subject: Set("math".into()),
                    ..Default::default()
                }
                .insert(&db)
                .await
                .unwrap();
            }
        }
    }
    let mut config = invigilation_config_settings::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap()
        .into_active_model();
    config.self_study_date = Set("2026-11-18".into());
    config.self_study_start_time = Set("12:10".into());
    config.self_study_end_time = Set("13:40".into());
    config.self_study_class_subjects_json = Set(serde_json::to_string(
        &class_ids
            .into_iter()
            .map(|id| serde_json::json!({"classId":id,"subject":"math"}))
            .collect::<Vec<_>>(),
    )
    .unwrap());
    config.update(&db).await.unwrap();
    let mut draft = ExamPlanDraft::default();
    for grade in ["高一", "高二", "高三"] {
        let connection = db.clone();
        draft = tauri::async_runtime::spawn_blocking(move || {
            crate::exam_allocation::build_subject_draft_for_test(
                &connection,
                grade,
                Subject::Math,
                &mut draft,
                2,
            )
            .unwrap();
            draft
        })
        .await
        .unwrap();
    }
    (db, root, draft)
}

async fn complete_exam_batch(db: &DatabaseConnection, root: &Path, draft: &ExamPlanDraft) {
    let inputs = crate::export_bundle::bundle_data_from_draft(db, draft)
        .await
        .unwrap();
    let batch = crate::export_batch::prepare_export_batch(root, "测试考场安排").unwrap();
    crate::export_bundle::write_exam_bundle_data(
        &batch.staging_dir,
        "测试",
        &[],
        &inputs,
        |_, _, _| {},
    )
    .unwrap();
    let revision = crate::db::repos::export_bundle::source_revision(db)
        .await
        .unwrap();
    crate::export_batch::write_batch_identity(&batch.staging_dir, "plan-v1", &revision).unwrap();
    let tx = db.begin().await.unwrap();
    crate::db::repos::exam_allocation::replace_plan_snapshot(&tx, draft)
        .await
        .unwrap();
    crate::db::repos::exam_allocation::insert_plan_meta(
        &tx,
        crate::db::repos::exam_allocation::PlanMetaInsertRow {
            generated_at: "plan-v1".into(),
            default_capacity: 2,
            max_capacity: 2,
            grade_count: 3,
            session_count: 3,
            warning_count: 0,
        },
    )
    .await
    .unwrap();
    let mut settings = exam_allocation_settings::Entity::find_by_id(1)
        .one(&tx)
        .await
        .unwrap()
        .unwrap()
        .into_active_model();
    settings.exam_title = Set("测试".into());
    settings.update(&tx).await.unwrap();
    let published = crate::export_batch::publish_prepared_batch(&batch).unwrap();
    tx.commit().await.unwrap();
    published.finish().unwrap();
}

async fn snapshot(db: &DatabaseConnection) -> String {
    format!(
        "{:?}",
        (
            latest_student_scores::Entity::find().all(db).await.unwrap(),
            latest_teachers_v2::Entity::find().all(db).await.unwrap(),
            latest_exam_plan_sessions::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_plan_spaces::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_plan_student_allocations::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_staff_tasks::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_staff_assignments::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_teacher_duty_stats::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_staff_plan_meta::Entity::find()
                .all(db)
                .await
                .unwrap(),
            latest_exam_plan_meta::Entity::find().all(db).await.unwrap(),
        )
    )
}

#[test]
fn test_export_rejects_replaced_staff_snapshot_with_same_sources() {
    tauri::async_runtime::block_on(async {
        let (db, root, draft) = setup().await;
        complete_exam_batch(&db, &root, &draft).await;
        crate::exam_staff::generate_staff_plan_for_test(&db, &root)
            .await
            .unwrap();
        let revision = crate::db::repos::export_bundle::source_revision(&db)
            .await
            .unwrap();
        let previous = latest_exam_staff_plan_meta::Entity::find_by_id(1)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let previous_version = previous.generated_at.clone();
        assert!(crate::export_invigilation::validate_export_revisions(
            &db,
            &revision,
            "plan-v1",
            &previous_version
        )
        .await
        .is_ok());
        let folder = crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap();
        assert!(
            crate::export_batch::verify_staff_source(&folder, &revision, &previous_version).is_ok()
        );
        assert!(
            crate::export_batch::verify_staff_source(&folder, &revision, "new-staff-version")
                .is_err()
        );
        let mut current = previous.into_active_model();
        current.generated_at = Set("new-staff-version".into());
        current.update(&db).await.unwrap();
        assert!(crate::export_invigilation::validate_export_revisions(
            &db,
            &revision,
            "plan-v1",
            &previous_version
        )
        .await
        .is_err());
        fs::remove_dir_all(root).unwrap();
    });
}

#[test]
fn test_multigrade_import_to_export_pipeline() {
    tauri::async_runtime::block_on(async {
        let (db, root, draft) = setup().await;
        assert_eq!(draft.sessions.len(), 3);
        assert_eq!(draft.allocations.len(), 9);
        let students = latest_student_scores::Entity::find()
            .all(&db)
            .await
            .unwrap();
        assert_eq!(
            students
                .iter()
                .filter(|student| student.exam_group == "comprehensive")
                .count(),
            2
        );
        assert_eq!(
            students
                .iter()
                .filter(|student| student.language == "俄语")
                .count(),
            1
        );
        for space in &draft.spaces {
            let local_id = draft
                .spaces
                .iter()
                .position(|item| std::ptr::eq(item, space))
                .unwrap() as i64
                + 1;
            let groups: std::collections::HashSet<_> = draft
                .allocations
                .iter()
                .filter(|allocation| allocation.space_id == Some(local_id))
                .map(|allocation| draft.class_groups[&allocation.class_name].as_str())
                .collect();
            assert_eq!(groups.len(), 1);
            assert!(space.capacity.unwrap() <= 2);
        }
        complete_exam_batch(&db, &root, &draft).await;
        use calamine::Reader;
        let mut original: calamine::Xlsx<_> = calamine::open_workbook(root.join("测试考场安排/高三/考场分配结果汇总.xlsx")).unwrap();
        let original = original.worksheets();
        let result = crate::exam_staff::generate_staff_plan_for_test(&db, &root)
            .await
            .unwrap();
        let result = serde_json::to_value(result).unwrap();
        assert_eq!(result["unassignedCount"], 0);
        let teachers: HashMap<_, _> = latest_teachers_v2::Entity::find()
            .all(&db)
            .await
            .unwrap()
            .into_iter()
            .map(|teacher| (i64::from(teacher.id), teacher))
            .collect();
        let tasks: HashMap<_, _> = latest_exam_staff_tasks::Entity::find()
            .all(&db)
            .await
            .unwrap()
            .into_iter()
            .map(|task| (task.id, task))
            .collect();
        for assignment in latest_exam_staff_assignments::Entity::find()
            .all(&db)
            .await
            .unwrap()
        {
            assert_eq!(
                teachers[&assignment.teacher_id]
                    .invigilation_grade_name
                    .as_deref(),
                Some(tasks[&assignment.task_id].grade_name.as_str())
            );
        }
        let folder = crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap();
        for grade in ["高一", "高二", "高三"] {
            assert!(folder
                .join(grade)
                .join("测试监考表.xlsx")
                .is_file());
        }
        let mut regenerated: calamine::Xlsx<_> = calamine::open_workbook(folder.join("高三/考场分配结果汇总.xlsx")).unwrap();
        assert_eq!(regenerated.worksheets(), original);
        fs::remove_dir_all(root).unwrap();
    });
}

#[cfg(windows)]
#[test]
fn test_regenerate_staff_with_exclusively_locked_old_workbooks() {
    use std::os::windows::fs::OpenOptionsExt;
    tauri::async_runtime::block_on(async {
        let (db, root, draft) = setup().await;
        complete_exam_batch(&db, &root, &draft).await;
        let legacy = root.join("测试考场安排/高三/考场分配结果汇总.xlsx");
        let locked = fs::OpenOptions::new().read(true).share_mode(0).open(&legacy).unwrap();
        crate::exam_staff::generate_staff_plan_for_test(&db, &root).await.unwrap();
        drop(locked);
        let first = crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap();
        let locked_student = fs::OpenOptions::new().read(true).share_mode(0)
            .open(first.join("高三/考场分配结果汇总.xlsx")).unwrap();
        let locked_staff = fs::OpenOptions::new().read(true).share_mode(0)
            .open(first.join("高三/测试监考表.xlsx")).unwrap();
        crate::exam_staff::generate_staff_plan_for_test(&db, &root).await.unwrap();
        let second = crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap();
        assert_ne!(first, second);
        assert!(second.join("高三/考场分配结果汇总.xlsx").is_file());
        assert!(second.join("高三/测试监考表.xlsx").is_file());
        drop((locked_student, locked_staff));
        fs::remove_dir_all(root).unwrap();
    });
}

#[test]
fn test_multigrade_failure_preserves_completed_batch() {
    tauri::async_runtime::block_on(async {
        let (db, root, draft) = setup().await;
        complete_exam_batch(&db, &root, &draft).await;
        crate::exam_staff::generate_staff_plan_for_test(&db, &root)
            .await
            .unwrap();
        let before = snapshot(&db).await;
        let path = crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap().join("高三/测试监考表.xlsx");
        let bytes = fs::read(&path).unwrap();
        let bad = crate::score::multi_sheet_tests::fixture("mixed");
        assert!(
            crate::score::import_scores_into_db(&db, bad.to_str().unwrap(), "bad")
                .await
                .is_err()
        );
        fs::remove_file(bad).unwrap();
        assert_eq!(snapshot(&db).await, before);
        let reserves = class_configs::Entity::find()
            .filter(class_configs::Column::ConfigType.eq("exam_room"))
            .all(&db)
            .await
            .unwrap();
        class_configs::Entity::delete_many()
            .filter(class_configs::Column::ConfigType.eq("exam_room"))
            .exec(&db)
            .await
            .unwrap();
        let connection = db.clone();
        assert!(tauri::async_runtime::spawn_blocking(move || {
            crate::exam_allocation::build_subject_draft_for_test(
                &connection,
                "高二",
                Subject::Math,
                &mut ExamPlanDraft::default(),
                1,
            )
        })
        .await
        .unwrap()
        .is_err());
        for reserve in reserves {
            reserve.into_active_model().insert(&db).await.unwrap();
        }
        assert_eq!(snapshot(&db).await, before);
        let tx = db.begin().await.unwrap();
        let teacher = latest_teachers_v2::Entity::find_by_id(1)
            .one(&tx)
            .await
            .unwrap()
            .unwrap();
        let mut teacher = teacher.into_active_model();
        teacher.invigilation_grade_name = Set(None);
        teacher.update(&tx).await.unwrap();
        tx.commit().await.unwrap();
        let missing_before = snapshot(&db).await;
        assert!(crate::exam_staff::generate_staff_plan_for_test(&db, &root)
            .await
            .is_err());
        assert_eq!(snapshot(&db).await, missing_before);
        let teacher = latest_teachers_v2::Entity::find_by_id(1)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut teacher = teacher.into_active_model();
        teacher.invigilation_grade_name = Set(Some("高一".into()));
        teacher.update(&db).await.unwrap();
        // 标识损坏代表输出不可用；不得先持久化新的监考数据再发现导出失败。
        fs::write(crate::export_batch::current_batch_dir(&db, &root, "测试").await.unwrap().join(".boom-batch.json"), b"bad").unwrap();
        assert!(crate::exam_staff::generate_staff_plan_for_test(&db, &root)
            .await
            .is_err());
        assert_eq!(snapshot(&db).await, before);
        assert_eq!(fs::read(path).unwrap(), bytes);
        fs::remove_dir_all(root).unwrap();
    });
}
