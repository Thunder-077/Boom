mod app_log;
mod class_config;
mod exam_allocation;
mod exam_plan;
mod exam_staff;
mod export_bundle;
mod export_invigilation;
mod invigilation;
mod schema;
mod score;
mod teacher;

use std::path::PathBuf;

use tauri::{AppHandle, Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            configure_cp_sat_runtime(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_log::append_app_log,
            app_log::get_app_log_path,
            app_log::reveal_in_explorer,
            score::import_scores_from_excel,
            score::list_latest_score_rows,
            score::get_score_detail,
            score::update_score_row,
            score::get_latest_summary,
            class_config::list_class_configs,
            class_config::get_class_config_detail,
            class_config::create_class_config,
            class_config::update_class_config,
            class_config::delete_class_config,
            class_config::list_grade_options,
            exam_plan::get_exam_allocation_settings,
            exam_plan::update_exam_allocation_settings,
            exam_plan::start_generate_latest_exam_plan,
            exam_plan::get_latest_exam_plan_overview,
            exam_plan::get_exam_generation_progress,
            exam_plan::list_latest_exam_plan_sessions,
            exam_plan::get_latest_exam_plan_session_detail,
            export_bundle::export_latest_exam_allocation_bundle,
            invigilation::list_exam_session_times,
            invigilation::list_invigilation_exclusion_session_options,
            invigilation::upsert_exam_session_times,
            invigilation::delete_exam_session_time,
            invigilation::get_persisted_invigilation_state,
            invigilation::save_persisted_invigilation_config,
            invigilation::replace_persisted_invigilation_exclusions,
            invigilation::save_persisted_self_study_class_subjects,
            invigilation::import_monitor_draw_pairs_from_excel,
            invigilation::generate_latest_exam_staff_plan,
            invigilation::get_latest_exam_staff_plan_overview,
            invigilation::list_latest_exam_staff_tasks,
            invigilation::list_latest_teacher_duty_stats,
            invigilation::export_latest_invigilation_schedule,
            teacher::import_teachers_from_excel,
            teacher::list_latest_teachers,
            teacher::get_latest_teacher_summary
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            if let Err(error) = clear_runtime_result_snapshots(app_handle) {
                app_log::log_error(
                    app_handle,
                    "app.shutdown.clear_runtime_result_snapshots",
                    &error.to_string(),
                );
            }
        }
    });
}

fn clear_runtime_result_snapshots(app: &AppHandle) -> Result<(), score::AppError> {
    let mut conn = score::open_connection(app)?;
    exam_allocation::ensure_schema(&conn)?;
    clear_runtime_result_snapshots_in_conn(&mut conn)
}

fn clear_runtime_result_snapshots_in_conn(
    conn: &mut rusqlite::Connection,
) -> Result<(), score::AppError> {
    let tx = conn.transaction()?;
    exam_staff::clear_latest_staff_plan_snapshot(&tx)?;
    exam_allocation::clear_latest_plan_snapshot(&tx)?;
    exam_allocation::reset_exam_generation_progress(&tx)?;
    tx.commit()?;
    Ok(())
}

fn configure_cp_sat_runtime(app: &mut tauri::App) {
    let mut candidates = Vec::<PathBuf>::new();
    if let Some(path) = option_env!("ACADEMIC_ORTOOLS_DEV_DIR") {
        candidates.push(PathBuf::from(path).join("sat_runner.exe"));
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("ortools").join("sat_runner.exe"));
    }
    for candidate in candidates {
        if candidate.is_file() {
            std::env::set_var("CP_SAT_SAT_RUNNER", candidate);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clear_runtime_result_snapshots_in_conn;
    use crate::exam_allocation;
    use rusqlite::Connection;

    #[test]
    fn test_clear_runtime_result_snapshots_only_removes_generated_results() {
        let mut conn = Connection::open_in_memory().unwrap();
        exam_allocation::ensure_schema(&conn).unwrap();

        conn.execute(
            "INSERT INTO latest_exam_plan_sessions (id, grade_name, subject, is_foreign_group, foreign_order, participant_count, exam_room_count, self_study_room_count) VALUES (1, '高一', 'math', 0, NULL, 30, 1, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_plan_spaces (id, session_id, space_type, space_source, grade_name, subject, space_name, original_class_name, self_study_topic_kind, self_study_topic_subjects_json, self_study_topic_label, building, floor, capacity, sort_index) VALUES (1, 1, 'exam_room', 'teaching_class', '高一', 'math', '高一1场', '高一1班', NULL, NULL, NULL, '向远楼', '3层', 30, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_plan_student_allocations (session_id, admission_no, student_name, class_name, allocation_type, space_id, seat_no, subject_score) VALUES (1, '1001', '张三', '高一1班', 'exam', 1, 1, 120.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_plan_staff_assignments (session_id, space_id, teacher_name, assignment_type, note) VALUES (1, 1, '李老师', 'invigilator', NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_plan_meta (id, generated_at, default_capacity, max_capacity, grade_count, session_count, warning_count) VALUES (1, '2026-04-08T08:00:00', 30, 40, 1, 1, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO exam_session_times (session_id, subject, start_at, end_at, updated_at) VALUES (1, 'math', '2026-04-09T08:00', '2026-04-09T10:00', '2026-04-08T08:00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_staff_tasks (id, session_id, space_id, task_source, role, grade_name, subject, space_name, floor, start_at, end_at, duration_minutes, recommended_self_study_topic_kind, recommended_self_study_topic_subjects_json, recommended_self_study_topic_label, priority_self_study_chain_json, assignment_tier, status, reason, allowance_amount) VALUES (1, 1, 1, 'exam', 'exam_room_invigilator', '高一', 'math', '高一1场', '3层', '2026-04-09T08:00', '2026-04-09T10:00', 120, NULL, NULL, NULL, '[]', 'primary', 'assigned', NULL, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_staff_assignments (task_id, teacher_id, teacher_name, assigned_at) VALUES (1, 11, '李老师', '2026-04-08T08:10:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_teacher_duty_stats (teacher_id, teacher_name, indoor_minutes, outdoor_minutes, total_minutes, task_count, exam_room_task_count, self_study_task_count, floor_rover_task_count, is_middle_manager, allowance_total, indoor_allowance_total, outdoor_allowance_total) VALUES (11, '李老师', 120, 0, 120, 1, 1, 0, 0, 0, 60.0, 60.0, 0.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO latest_exam_staff_plan_meta (id, generated_at, session_count, task_count, assigned_count, unassigned_count, warning_count, imbalance_minutes, solver_engine, optimality_status, solve_duration_ms, fallback_reason, fallback_pool_assignments) VALUES (1, '2026-04-08T08:10:00', 1, 1, 1, 0, 0, 0, 'cp_sat', 'feasible', 100, NULL, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO invigilation_staff_exclusions (teacher_id, teacher_name, session_id, session_label, created_at) VALUES (11, '李老师', 1, '高一数学', '2026-04-08T08:00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE exam_generation_progress SET status = 'running', stage = 'allocating', stage_label = '分配考场', percent = 66, message = '处理中', current_grade = '高一', total_grades = 1, completed_grades = 0 WHERE id = 1",
            [],
        )
        .unwrap();

        clear_runtime_result_snapshots_in_conn(&mut conn).unwrap();

        for table in [
            "latest_exam_plan_meta",
            "latest_exam_plan_sessions",
            "latest_exam_plan_spaces",
            "latest_exam_plan_student_allocations",
            "latest_exam_plan_staff_assignments",
            "latest_exam_staff_plan_meta",
            "latest_exam_staff_tasks",
            "latest_exam_staff_assignments",
            "latest_teacher_duty_stats",
        ] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0, "{table} 应该在关闭时被清空");
        }

        let progress: (String, String, String, i64, String, i64, i64) = conn
            .query_row(
                "SELECT status, stage, stage_label, percent, message, total_grades, completed_grades FROM exam_generation_progress WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(
            progress,
            (
                "idle".to_string(),
                "idle".to_string(),
                "等待开始".to_string(),
                0,
                "等待开始分配考场".to_string(),
                0,
                0,
            )
        );

        let template_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM exam_subject_time_templates", [], |row| row.get(0))
            .unwrap();
        assert!(template_count > 0, "科目时间模板应当保留");

        let config_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM invigilation_config_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(config_count, 1, "监考配置应当保留");

        let exclusion_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM invigilation_staff_exclusions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(exclusion_count, 1, "监考排除设置应当保留");

        let session_time_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM exam_session_times", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            session_time_count, 0,
            "session 级考试时间依赖考场会话，应随快照一起清空"
        );
    }
}
