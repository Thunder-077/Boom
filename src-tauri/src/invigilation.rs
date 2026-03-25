use tauri::AppHandle;

use crate::exam_staff;
use crate::score::ListResult;

pub use crate::exam_allocation::SuccessResponse;
pub use crate::exam_staff::{
    ExamSessionTime, ExamSessionTimeUpsert, ExamStaffPlanOverview, ExamStaffTask,
    GenerateExamStaffPlanPayload, GenerateLatestExamStaffPlanResult,
    InvigilationExclusionSessionOption, ListExamStaffTasksParams, ListTeacherDutyStatsParams,
    TeacherDutyStat,
};

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
pub fn generate_latest_exam_staff_plan(
    app: AppHandle,
    payload: GenerateExamStaffPlanPayload,
) -> Result<GenerateLatestExamStaffPlanResult, String> {
    exam_staff::generate_latest_exam_staff_plan(app, payload)
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

