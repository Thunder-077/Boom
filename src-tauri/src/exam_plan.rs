use tauri::AppHandle;

use crate::exam_allocation;

pub use crate::exam_allocation::{
    ExamAllocationSettings, ExamPlanOverview, ExamPlanSessionDetail, GenerateLatestExamPlanPayload,
    GenerateLatestExamPlanResult, ListExamPlanSessionsParams, SuccessResponse,
};
pub use crate::score::ListResult;

#[tauri::command]
pub fn get_exam_allocation_settings(app: AppHandle) -> Result<ExamAllocationSettings, String> {
    exam_allocation::get_exam_allocation_settings(app)
}

#[tauri::command]
pub fn update_exam_allocation_settings(
    app: AppHandle,
    payload: UpdateExamAllocationSettingsPayload,
) -> Result<SuccessResponse, String> {
    exam_allocation::update_exam_allocation_settings(app, payload)
}

pub use crate::exam_allocation::UpdateExamAllocationSettingsPayload;

#[tauri::command]
pub fn generate_latest_exam_plan(
    app: AppHandle,
    payload: Option<GenerateLatestExamPlanPayload>,
) -> Result<GenerateLatestExamPlanResult, String> {
    exam_allocation::generate_latest_exam_plan(app, payload)
}

#[tauri::command]
pub fn get_latest_exam_plan_overview(app: AppHandle) -> Result<ExamPlanOverview, String> {
    exam_allocation::get_latest_exam_plan_overview(app)
}

#[tauri::command]
pub fn list_latest_exam_plan_sessions(
    app: AppHandle,
    params: ListExamPlanSessionsParams,
) -> Result<ListResult<crate::exam_allocation::ExamPlanSession>, String> {
    exam_allocation::list_latest_exam_plan_sessions(app, params)
}

#[tauri::command]
pub fn get_latest_exam_plan_session_detail(
    app: AppHandle,
    session_id: i64,
) -> Result<ExamPlanSessionDetail, String> {
    exam_allocation::get_latest_exam_plan_session_detail(app, session_id)
}
