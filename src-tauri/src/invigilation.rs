use tauri::AppHandle;

use crate::exam_staff;
use crate::score::ListResult;

pub use crate::exam_allocation::SuccessResponse;
pub use crate::exam_staff::{
    CreateExamStaffExclusionPayload, ExamSessionTime, ExamSessionTimeUpsert, ExamStaffExclusion,
    ExamStaffPlanOverview, ExamStaffTask, GenerateLatestExamStaffPlanResult, InvigilationConfig,
    ListExamStaffTasksParams, ListTeacherDutyStatsParams, SpaceStaffRequirement,
    SpaceStaffRequirementUpsert, TeacherDutyStat, UpdateInvigilationConfigPayload,
};

#[tauri::command]
pub fn list_exam_session_times(app: AppHandle) -> Result<Vec<ExamSessionTime>, String> {
    exam_staff::list_exam_session_times(app)
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
pub fn list_exam_space_staff_requirements(
    app: AppHandle,
    session_id: i64,
) -> Result<Vec<SpaceStaffRequirement>, String> {
    exam_staff::list_exam_space_staff_requirements(app, session_id)
}

#[tauri::command]
pub fn upsert_exam_space_staff_requirements(
    app: AppHandle,
    session_id: i64,
    items: Vec<SpaceStaffRequirementUpsert>,
) -> Result<SuccessResponse, String> {
    exam_staff::upsert_exam_space_staff_requirements(app, session_id, items)
}

#[tauri::command]
pub fn generate_latest_exam_staff_plan(
    app: AppHandle,
) -> Result<GenerateLatestExamStaffPlanResult, String> {
    exam_staff::generate_latest_exam_staff_plan(app)
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
pub fn get_invigilation_config(app: AppHandle) -> Result<InvigilationConfig, String> {
    exam_staff::get_invigilation_config(app)
}

#[tauri::command]
pub fn update_invigilation_config(
    app: AppHandle,
    payload: UpdateInvigilationConfigPayload,
) -> Result<SuccessResponse, String> {
    exam_staff::update_invigilation_config(app, payload)
}

#[tauri::command]
pub fn list_exam_staff_exclusions(app: AppHandle) -> Result<Vec<ExamStaffExclusion>, String> {
    exam_staff::list_exam_staff_exclusions(app)
}

#[tauri::command]
pub fn create_exam_staff_exclusion(
    app: AppHandle,
    payload: CreateExamStaffExclusionPayload,
) -> Result<SuccessResponse, String> {
    exam_staff::create_exam_staff_exclusion(app, payload)
}

#[tauri::command]
pub fn delete_exam_staff_exclusion(
    app: AppHandle,
    id: i64,
) -> Result<SuccessResponse, String> {
    exam_staff::delete_exam_staff_exclusion(app, id)
}
