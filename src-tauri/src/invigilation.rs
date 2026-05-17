use tauri::AppHandle;

use crate::exam_staff;
use crate::export_invigilation;
use crate::score::ListResult;

pub use crate::exam_allocation::SuccessResponse;
pub use crate::exam_staff::{
    ExamSessionTime, ExamSessionTimeUpsert, ExamStaffPlanOverview, ExamStaffTask,
    GenerateExamStaffPlanPayload, GenerateLatestExamStaffPlanResult,
    InvigilationExclusionSessionOption, InvigilationRuleOptions, ListExamSessionTimesParams,
    ListExamStaffTasksParams, ListTeacherDutyStatsParams, MonitorDrawImportResult,
    PersistedInvigilationConfig, PersistedInvigilationCustomRule, PersistedInvigilationState,
    PersistedSelfStudyClassSubject, TeacherDutyStat,
};
pub use crate::export_invigilation::ExportLatestInvigilationScheduleResult;

#[tauri::command]
pub async fn list_exam_session_time_grade_options(app: AppHandle) -> Result<Vec<String>, String> {
    exam_staff::list_exam_session_time_grade_options(app).await
}

#[tauri::command]
pub async fn list_exam_session_times(
    app: AppHandle,
    params: Option<ListExamSessionTimesParams>,
) -> Result<Vec<ExamSessionTime>, String> {
    exam_staff::list_exam_session_times(app, params).await
}

#[tauri::command]
pub async fn list_invigilation_exclusion_session_options(
    app: AppHandle,
) -> Result<Vec<InvigilationExclusionSessionOption>, String> {
    exam_staff::list_invigilation_exclusion_session_options(app).await
}

#[tauri::command]
pub async fn upsert_exam_session_times(
    app: AppHandle,
    items: Vec<ExamSessionTimeUpsert>,
) -> Result<SuccessResponse, String> {
    exam_staff::upsert_exam_session_times(app, items).await
}

#[tauri::command]
pub async fn delete_exam_session_time(
    app: AppHandle,
    grade_name: String,
    subject: crate::score::Subject,
) -> Result<SuccessResponse, String> {
    exam_staff::delete_exam_session_time(app, grade_name, subject).await
}

#[tauri::command]
pub async fn get_persisted_invigilation_state(
    app: AppHandle,
) -> Result<PersistedInvigilationState, String> {
    exam_staff::get_persisted_invigilation_state(app).await
}

#[tauri::command]
pub async fn list_invigilation_custom_rule_options(
    app: AppHandle,
) -> Result<InvigilationRuleOptions, String> {
    exam_staff::list_invigilation_custom_rule_options(app).await
}

#[tauri::command]
pub async fn save_persisted_invigilation_config(
    app: AppHandle,
    payload: PersistedInvigilationConfig,
) -> Result<SuccessResponse, String> {
    exam_staff::save_persisted_invigilation_config(app, payload).await
}

#[tauri::command]
pub async fn replace_persisted_invigilation_custom_rules(
    app: AppHandle,
    items: Vec<PersistedInvigilationCustomRule>,
) -> Result<SuccessResponse, String> {
    exam_staff::replace_persisted_invigilation_custom_rules(app, items).await
}

#[tauri::command]
pub async fn save_persisted_self_study_class_subjects(
    app: AppHandle,
    items: Vec<PersistedSelfStudyClassSubject>,
) -> Result<SuccessResponse, String> {
    exam_staff::save_persisted_self_study_class_subjects(app, items).await
}

#[tauri::command]
pub fn import_monitor_draw_pairs_from_excel(
    app: AppHandle,
    file_path: String,
) -> Result<MonitorDrawImportResult, String> {
    exam_staff::import_monitor_draw_pairs_from_excel(app, file_path)
}

#[tauri::command]
pub async fn generate_latest_exam_staff_plan(
    app: AppHandle,
    payload: GenerateExamStaffPlanPayload,
) -> Result<GenerateLatestExamStaffPlanResult, String> {
    exam_staff::generate_latest_exam_staff_plan(app, payload).await
}

#[tauri::command]
pub async fn get_latest_exam_staff_plan_overview(
    app: AppHandle,
) -> Result<ExamStaffPlanOverview, String> {
    exam_staff::get_latest_exam_staff_plan_overview(app).await
}

#[tauri::command]
pub async fn list_latest_exam_staff_tasks(
    app: AppHandle,
    params: ListExamStaffTasksParams,
) -> Result<ListResult<ExamStaffTask>, String> {
    exam_staff::list_latest_exam_staff_tasks(app, params).await
}

#[tauri::command]
pub async fn list_latest_teacher_duty_stats(
    app: AppHandle,
    params: ListTeacherDutyStatsParams,
) -> Result<ListResult<TeacherDutyStat>, String> {
    exam_staff::list_latest_teacher_duty_stats(app, params).await
}

#[tauri::command]
pub async fn export_latest_invigilation_schedule(
    app: AppHandle,
) -> Result<ExportLatestInvigilationScheduleResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        export_invigilation::export_latest_invigilation_schedule(app)
    })
    .await
    .map_err(|error| format!("监考表导出任务执行失败: {error}"))?
}
