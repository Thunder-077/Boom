mod app_log;
mod class_config;
mod course_management;
mod db;
mod entity;
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

use sea_orm::TransactionTrait;
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
            course_management::import_course_schedule_from_excel,
            course_management::get_course_schedule_summary,
            course_management::list_course_schedule_imports,
            course_management::update_course_schedule_import_settings,
            course_management::delete_course_schedule_import,
            course_management::list_course_schedule_classes,
            course_management::list_course_schedule_teachers,
            course_management::list_course_schedule_periods,
            course_management::list_course_substitution_candidates,
            course_management::save_course_substitutions,
            course_management::list_course_schedule_changes,
            course_management::revoke_course_schedule_change,
            course_management::get_course_workload_report,
            course_management::export_course_workload_report,
            course_management::get_course_schedule_view,
            exam_plan::get_exam_allocation_settings,
            exam_plan::update_exam_allocation_settings,
            exam_plan::start_generate_latest_exam_plan,
            exam_plan::get_latest_exam_plan_overview,
            exam_plan::get_exam_generation_progress,
            exam_plan::list_latest_exam_plan_sessions,
            exam_plan::get_latest_exam_plan_session_detail,
            export_bundle::export_latest_exam_allocation_bundle,
            invigilation::list_exam_session_time_grade_options,
            invigilation::list_exam_session_times,
            invigilation::list_invigilation_exclusion_session_options,
            invigilation::upsert_exam_session_times,
            invigilation::delete_exam_session_time,
            invigilation::get_persisted_invigilation_state,
            invigilation::list_invigilation_custom_rule_options,
            invigilation::save_persisted_invigilation_config,
            invigilation::replace_persisted_invigilation_custom_rules,
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
    tauri::async_runtime::block_on(async {
        let db = crate::db::connect(app).await?;
        crate::db::repos::exam_staff::clear_latest_staff_plan_snapshot(&db).await?;
        let tx = db.begin().await?;
        crate::db::repos::exam_allocation::clear_latest_plan_snapshot(&tx).await?;
        tx.commit().await?;
        crate::db::repos::exam_allocation::update_progress(
            &db,
            crate::db::repos::exam_allocation::ProgressRow {
                status: "idle".to_string(),
                stage: "idle".to_string(),
                stage_label: "等待开始".to_string(),
                percent: 0,
                message: "等待开始分配考场".to_string(),
                current_grade: None,
                total_grades: 0,
                completed_grades: 0,
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        )
        .await
    })?;
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
