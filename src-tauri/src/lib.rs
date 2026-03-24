mod app_log;
mod class_config;
mod exam_allocation;
mod exam_plan;
mod export_bundle;
mod invigilation;
mod score;
mod teacher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            invigilation::upsert_exam_session_times,
            invigilation::delete_exam_session_time,
            invigilation::list_exam_space_staff_requirements,
            invigilation::upsert_exam_space_staff_requirements,
            invigilation::generate_latest_exam_staff_plan,
            invigilation::get_latest_exam_staff_plan_overview,
            invigilation::list_latest_exam_staff_tasks,
            invigilation::list_latest_teacher_duty_stats,
            teacher::import_teachers_from_excel,
            teacher::list_latest_teachers,
            teacher::get_latest_teacher_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
