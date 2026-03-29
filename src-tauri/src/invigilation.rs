use tauri::AppHandle;

use crate::exam_staff;
use crate::export_invigilation;
use crate::score::ListResult;

pub use crate::exam_allocation::SuccessResponse;
pub use crate::exam_staff::{
    ExamSessionTime, ExamSessionTimeUpsert, ExamStaffPlanOverview, ExamStaffTask,
    GenerateExamStaffPlanPayload, GenerateLatestExamStaffPlanResult,
    InvigilationExclusionSessionOption, ListExamStaffTasksParams, ListTeacherDutyStatsParams,
    PersistedExamStaffExclusion, PersistedInvigilationConfig, PersistedInvigilationState,
    MonitorDrawImportResult,
    PersistedSelfStudyClassSubject, TeacherDutyStat,
};
pub use crate::export_invigilation::ExportLatestInvigilationScheduleResult;

#[tauri::command]
pub fn list_exam_session_times(app: AppHandle) -> Result<Vec<ExamSessionTime>, String> {
    exam_staff::list_exam_session_times(app)
}

#[tauri::command]
pub fn list_invigilation_exclusion_session_options(
    app: AppHandle,
) -> Result<Vec<InvigilationExclusionSessionOption>, String> {
    exam_staff::list_invigilation_exclusion_session_options(app)
}

#[tauri::command]
pub fn upsert_exam_session_times(
    app: AppHandle,
    items: Vec<ExamSessionTimeUpsert>,
) -> Result<SuccessResponse, String> {
    exam_staff::upsert_exam_session_times(app, items)
}

#[tauri::command]
pub fn delete_exam_session_time(
    app: AppHandle,
    subject: crate::score::Subject,
) -> Result<SuccessResponse, String> {
    exam_staff::delete_exam_session_time(app, subject)
}

#[tauri::command]
pub fn get_persisted_invigilation_state(
    app: AppHandle,
) -> Result<PersistedInvigilationState, String> {
    exam_staff::get_persisted_invigilation_state(app)
}

#[tauri::command]
pub fn save_persisted_invigilation_config(
    app: AppHandle,
    payload: PersistedInvigilationConfig,
) -> Result<SuccessResponse, String> {
    exam_staff::save_persisted_invigilation_config(app, payload)
}

#[tauri::command]
pub fn replace_persisted_invigilation_exclusions(
    app: AppHandle,
    items: Vec<PersistedExamStaffExclusion>,
) -> Result<SuccessResponse, String> {
    exam_staff::replace_persisted_invigilation_exclusions(app, items)
}

#[tauri::command]
pub fn save_persisted_self_study_class_subjects(
    app: AppHandle,
    items: Vec<PersistedSelfStudyClassSubject>,
) -> Result<SuccessResponse, String> {
    exam_staff::save_persisted_self_study_class_subjects(app, items)
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
    tauri::async_runtime::spawn_blocking(move || {
        exam_staff::generate_latest_exam_staff_plan(app, payload)
    })
    .await
    .map_err(|error| format!("监考分配任务执行失败: {error}"))?
}

#[tauri::command]
pub fn get_latest_exam_staff_plan_overview(
    app: AppHandle,
) -> Result<ExamStaffPlanOverview, String> {
    exam_staff::get_latest_exam_staff_plan_overview(app)
}

#[tauri::command]
pub fn list_latest_exam_staff_tasks(
    app: AppHandle,
    params: ListExamStaffTasksParams,
) -> Result<ListResult<ExamStaffTask>, String> {
    exam_staff::list_latest_exam_staff_tasks(app, params)
}

#[tauri::command]
pub fn list_latest_teacher_duty_stats(
    app: AppHandle,
    params: ListTeacherDutyStatsParams,
) -> Result<ListResult<TeacherDutyStat>, String> {
    exam_staff::list_latest_teacher_duty_stats(app, params)
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
