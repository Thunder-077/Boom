use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use cp_sat::builder::{BoolVar, CpModelBuilder, IntVar, LinearExpr};
use cp_sat::proto::{CpSolverResponse, CpSolverStatus, SatParameters};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::app_log;
use crate::exam_allocation::{self, SuccessResponse};
use crate::score::{self, AppError, ListResult, Subject};

const CP_SAT_MAX_SOLVE_MS: i64 = 30 * 60 * 1000;
const CP_SAT_MAX_SOLVE_LABEL: &str = "30 分钟";
const CP_SAT_FAST_STAGE_BUDGET_MS: i64 = 30 * 1000;
const CP_SAT_BALANCE_STAGE_BUDGET_MS: i64 = 90 * 1000;
const STAFF_ASSIGNMENT_PROGRESS_EVENT: &str = "invigilation_staff_assignment_progress";
const STAFF_ASSIGNMENT_TOTAL_STEPS: usize = 13;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffAssignmentProgressPayload {
    status: &'static str,
    stage: String,
    stage_label: String,
    percent: i64,
    message: String,
    completed_steps: i64,
    total_steps: i64,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct StaffAssignmentProgressReporter {
    app: AppHandle,
}

impl StaffAssignmentProgressReporter {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn emit_running(&self, step: usize, stage: &str, stage_label: &str, message: String) {
        let safe_step = step.clamp(1, STAFF_ASSIGNMENT_TOTAL_STEPS);
        self.emit_payload(StaffAssignmentProgressPayload {
            status: "running",
            stage: stage.to_string(),
            stage_label: stage_label.to_string(),
            percent: ((safe_step.saturating_sub(1) * 100) / STAFF_ASSIGNMENT_TOTAL_STEPS) as i64,
            message,
            completed_steps: safe_step.saturating_sub(1) as i64,
            total_steps: STAFF_ASSIGNMENT_TOTAL_STEPS as i64,
            updated_at: Utc::now().to_rfc3339(),
        });
    }

    fn emit_completed(&self, message: String) {
        self.emit_payload(StaffAssignmentProgressPayload {
            status: "completed",
            stage: "completed".to_string(),
            stage_label: "分配完成".to_string(),
            percent: 100,
            message,
            completed_steps: STAFF_ASSIGNMENT_TOTAL_STEPS as i64,
            total_steps: STAFF_ASSIGNMENT_TOTAL_STEPS as i64,
            updated_at: Utc::now().to_rfc3339(),
        });
    }

    fn emit_error(&self, stage: &str, stage_label: &str, message: String) {
        self.emit_payload(StaffAssignmentProgressPayload {
            status: "error",
            stage: stage.to_string(),
            stage_label: stage_label.to_string(),
            percent: 0,
            message,
            completed_steps: 0,
            total_steps: STAFF_ASSIGNMENT_TOTAL_STEPS as i64,
            updated_at: Utc::now().to_rfc3339(),
        });
    }

    fn emit_payload(&self, payload: StaffAssignmentProgressPayload) {
        let _ = self.app.emit(STAFF_ASSIGNMENT_PROGRESS_EVENT, payload);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StaffRole {
    ExamRoomInvigilator,
    SelfStudySupervisor,
    FloorRover,
}

impl StaffRole {
    pub(crate) fn as_key(self) -> &'static str {
        match self {
            StaffRole::ExamRoomInvigilator => "exam_room_invigilator",
            StaffRole::SelfStudySupervisor => "self_study_supervisor",
            StaffRole::FloorRover => "floor_rover",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "exam_room_invigilator" => Some(StaffRole::ExamRoomInvigilator),
            "self_study_supervisor" => Some(StaffRole::SelfStudySupervisor),
            "floor_rover" => Some(StaffRole::FloorRover),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Assigned,
    Unassigned,
}

impl TaskStatus {
    fn as_key(self) -> &'static str {
        match self {
            TaskStatus::Assigned => "assigned",
            TaskStatus::Unassigned => "unassigned",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "assigned" => Some(TaskStatus::Assigned),
            "unassigned" => Some(TaskStatus::Unassigned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StaffTaskSource {
    Exam,
    ExamLinkedSelfStudy,
    FullSelfStudy,
}

impl StaffTaskSource {
    fn as_key(self) -> &'static str {
        match self {
            Self::Exam => "exam",
            Self::ExamLinkedSelfStudy => "exam_linked_self_study",
            Self::FullSelfStudy => "full_self_study",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "exam" => Some(Self::Exam),
            "exam_linked_self_study" => Some(Self::ExamLinkedSelfStudy),
            "full_self_study" => Some(Self::FullSelfStudy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentTier {
    Primary,
    Homeroom,
    FallbackPool,
}

impl AssignmentTier {
    fn as_key(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Homeroom => "homeroom",
            Self::FallbackPool => "fallback_pool",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "primary" => Some(Self::Primary),
            "homeroom" => Some(Self::Homeroom),
            "fallback_pool" => Some(Self::FallbackPool),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SolverEngine {
    CpSat,
}

impl SolverEngine {
    fn as_key(self) -> &'static str {
        match self {
            Self::CpSat => "cp_sat",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "cp_sat" => Some(Self::CpSat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OptimalityStatus {
    Optimal,
    Feasible,
    Infeasible,
    Error,
}

impl OptimalityStatus {
    fn as_key(self) -> &'static str {
        match self {
            Self::Optimal => "optimal",
            Self::Feasible => "feasible",
            Self::Infeasible => "infeasible",
            Self::Error => "error",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "optimal" => Some(Self::Optimal),
            "feasible" => Some(Self::Feasible),
            "infeasible" => Some(Self::Infeasible),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FallbackReason {
    Timeout,
    Unknown,
    Infeasible,
    Error,
}

impl FallbackReason {
    fn as_key(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Unknown => "unknown",
            Self::Infeasible => "infeasible",
            Self::Error => "error",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "timeout" => Some(Self::Timeout),
            "unknown" => Some(Self::Unknown),
            "infeasible" => Some(Self::Infeasible),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSessionTime {
    session_id: i64,
    grade_name: String,
    subject: Subject,
    start_at: Option<String>,
    end_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSessionTimeUpsert {
    pub session_id: i64,
    pub subject: Subject,
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateLatestExamStaffPlanResult {
    generated_at: String,
    task_count: i64,
    assigned_count: i64,
    unassigned_count: i64,
    imbalance_minutes: i64,
    warning_count: i64,
    solver_engine: SolverEngine,
    optimality_status: OptimalityStatus,
    solve_duration_ms: i64,
    fallback_reason: Option<FallbackReason>,
    fallback_pool_assignments: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamStaffPlanOverview {
    generated_at: Option<String>,
    session_count: i64,
    task_count: i64,
    assigned_count: i64,
    unassigned_count: i64,
    warning_count: i64,
    imbalance_minutes: i64,
    solver_engine: SolverEngine,
    optimality_status: OptimalityStatus,
    solve_duration_ms: i64,
    fallback_reason: Option<FallbackReason>,
    fallback_pool_assignments: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamStaffTask {
    id: i64,
    session_id: Option<i64>,
    space_id: Option<i64>,
    task_source: StaffTaskSource,
    role: StaffRole,
    grade_name: String,
    subject: Subject,
    space_name: String,
    floor: String,
    start_at: String,
    end_at: String,
    duration_minutes: i64,
    recommended_self_study_topic: Option<exam_allocation::SelfStudyTopic>,
    priority_self_study_chain: Vec<exam_allocation::SelfStudyTopic>,
    assignment_tier: Option<AssignmentTier>,
    status: TaskStatus,
    reason: Option<String>,
    allowance_amount: f64,
    teacher_id: Option<i64>,
    teacher_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExamStaffTasksParams {
    pub session_id: Option<i64>,
    pub role: Option<StaffRole>,
    pub status: Option<TaskStatus>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherDutyStat {
    teacher_id: i64,
    teacher_name: String,
    indoor_minutes: i64,
    outdoor_minutes: i64,
    total_minutes: i64,
    task_count: i64,
    exam_room_task_count: i64,
    self_study_task_count: i64,
    floor_rover_task_count: i64,
    allowance_total: f64,
    indoor_allowance_total: f64,
    outdoor_allowance_total: f64,
    is_middle_manager: bool,
}

#[derive(Debug, Clone)]
struct RuntimeInvigilationConfig {
    default_exam_room_required_count: i64,
    indoor_allowance_per_minute: f64,
    outdoor_allowance_per_minute: f64,
    middle_manager_default_enabled: bool,
    middle_manager_exception_teacher_ids: HashSet<i64>,
    self_study_date: String,
    self_study_start_time: String,
    self_study_end_time: String,
    self_study_class_subjects: HashMap<i64, Subject>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateExamStaffPlanPayload {
    pub default_exam_room_required_count: i64,
    pub indoor_allowance_per_minute: f64,
    pub outdoor_allowance_per_minute: f64,
    pub staff_exclusions: Vec<GenerateExamStaffPlanExclusion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateExamStaffPlanExclusion {
    pub teacher_id: i64,
    pub session_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedInvigilationConfig {
    default_exam_room_required_count: i64,
    indoor_allowance_per_minute: f64,
    outdoor_allowance_per_minute: f64,
    middle_manager_default_enabled: bool,
    middle_manager_exception_teacher_ids: Vec<i64>,
    self_study_date: String,
    self_study_start_time: String,
    self_study_end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedExamStaffExclusion {
    teacher_id: i64,
    teacher_name: String,
    session_id: i64,
    session_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedSelfStudyClassSubject {
    class_id: i64,
    subject: Option<Subject>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedInvigilationState {
    config: PersistedInvigilationConfig,
    exclusions: Vec<PersistedExamStaffExclusion>,
    self_study_class_subjects: Vec<PersistedSelfStudyClassSubject>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTeacherDutyStatsParams {
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
struct SessionTimeRuntime {
    session_id: i64,
    grade_name: String,
    subject: Subject,
    start_at: String,
    end_at: String,
    start_ts: i64,
    end_ts: i64,
}

#[derive(Debug, Clone)]
struct TeacherInfo {
    id: i64,
    name: String,
    subjects: HashSet<Subject>,
    class_names: HashSet<String>,
    homeroom_classes: HashSet<String>,
    is_middle_manager: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HalfDay {
    Morning,
    Afternoon,
}

impl HalfDay {
    fn from_hour(hour: u32) -> Self {
        if hour < 12 {
            Self::Morning
        } else {
            Self::Afternoon
        }
    }
}

#[derive(Debug, Default, Clone)]
struct DayHalfLoad {
    morning_tasks: i64,
    afternoon_tasks: i64,
}

impl DayHalfLoad {
    fn add_task(&mut self, half_day: HalfDay) {
        match half_day {
            HalfDay::Morning => self.morning_tasks += 1,
            HalfDay::Afternoon => self.afternoon_tasks += 1,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct TeacherRuntimeState {
    indoor_minutes: i64,
    outdoor_minutes: i64,
    total_minutes: i64,
    invigilation_minutes: i64,
    self_study_minutes: i64,
    task_count: i64,
    exam_room_task_count: i64,
    self_study_task_count: i64,
    floor_rover_task_count: i64,
    allowance_total: f64,
    indoor_allowance_total: f64,
    outdoor_allowance_total: f64,
    busy_ranges: Vec<(i64, i64)>,
    day_half_loads: HashMap<String, DayHalfLoad>,
}

#[derive(Debug, Clone)]
struct TaskBuild {
    session_id: Option<i64>,
    space_id: Option<i64>,
    task_source: StaffTaskSource,
    role: StaffRole,
    grade_name: String,
    subject: Subject,
    space_name: String,
    floor: String,
    start_at: String,
    end_at: String,
    start_ts: i64,
    end_ts: i64,
    duration_minutes: i64,
    recommended_self_study_topic: Option<exam_allocation::SelfStudyTopic>,
    priority_self_study_chain: Vec<exam_allocation::SelfStudyTopic>,
    day_key: String,
    half_day: HalfDay,
}

#[derive(Debug, Clone)]
struct TaskCandidate {
    teacher_id: i64,
    assignment_tier: Option<AssignmentTier>,
}

#[derive(Debug, Clone)]
struct TaskCandidateSummary {
    candidates: Vec<TaskCandidate>,
}

#[derive(Debug, Clone)]
struct SolvedTaskRecord {
    task: TaskBuild,
    teacher_id: Option<i64>,
    reason: Option<String>,
    assignment_tier: Option<AssignmentTier>,
    allowance_amount: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PlanMetrics {
    assigned_count: i64,
    unassigned_count: i64,
    fallback_pool_assignments: i64,
    homeroom_assignments: i64,
    invigilation_minutes_gap: i64,
    self_study_minutes_gap: i64,
    cross_half_day_penalty: i64,
    imbalance_minutes: i64,
    warning_count: i64,
}

#[derive(Debug, Clone)]
struct SolvedPlan {
    records: Vec<SolvedTaskRecord>,
    runtime: HashMap<i64, TeacherRuntimeState>,
    metrics: PlanMetrics,
    solver_engine: SolverEngine,
    optimality_status: OptimalityStatus,
    solve_duration_ms: i64,
    fallback_reason: Option<FallbackReason>,
}

#[derive(Debug, Clone)]
struct CpSatAttempt {
    plan: Option<SolvedPlan>,
    fallback_reason: Option<FallbackReason>,
    diagnostic_message: Option<String>,
    solve_duration_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpaceType {
    ExamRoom,
    SelfStudyRoom,
}

fn subject_order(subject: Subject) -> i32 {
    match subject {
        Subject::Chinese => 1,
        Subject::Math => 2,
        Subject::English => 3,
        Subject::Physics => 4,
        Subject::Chemistry => 5,
        Subject::Biology => 6,
        Subject::Politics => 7,
        Subject::History => 8,
        Subject::Geography => 9,
        Subject::Russian => 10,
        Subject::Japanese => 11,
    }
}

fn subject_label(subject: Subject) -> &'static str {
    match subject {
        Subject::Chinese => "语文",
        Subject::Math => "数学",
        Subject::English => "英语",
        Subject::Physics => "物理",
        Subject::Chemistry => "化学",
        Subject::Biology => "生物",
        Subject::Politics => "政治",
        Subject::History => "历史",
        Subject::Geography => "地理",
        Subject::Russian => "俄语",
        Subject::Japanese => "日语",
    }
}

fn template_session_id(subject: Subject) -> i64 {
    -(subject_order(subject) as i64)
}

fn parse_datetime_to_ts(value: &str) -> Result<i64, AppError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.timestamp_millis());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Ok(naive.and_utc().timestamp_millis());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(naive.and_utc().timestamp_millis());
    }
    Err(AppError::new(format!("时间格式不正确: {}", value)))
}

fn duration_minutes(start_ts: i64, end_ts: i64) -> Result<i64, AppError> {
    if end_ts <= start_ts {
        return Err(AppError::new("考试结束时间必须晚于开始时间"));
    }
    Ok((end_ts - start_ts) / 60_000)
}

fn parse_day_slot(value: &str) -> Result<(String, HalfDay), AppError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok((
            dt.format("%Y-%m-%d").to_string(),
            HalfDay::from_hour(dt.hour()),
        ));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Ok((
            naive.format("%Y-%m-%d").to_string(),
            HalfDay::from_hour(naive.hour()),
        ));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok((
            naive.format("%Y-%m-%d").to_string(),
            HalfDay::from_hour(naive.hour()),
        ));
    }
    Err(AppError::new(format!("时间格式不正确: {}", value)))
}

fn build_self_study_datetime(date: &str, time: &str) -> Result<String, AppError> {
    let date = date.trim();
    let time = time.trim();
    if date.is_empty() || time.is_empty() {
        return Err(AppError::new("全员自习日期与时间未配置完整"));
    }
    let value = format!("{date}T{time}");
    parse_datetime_to_ts(&value)?;
    Ok(value)
}

fn role_priority(role: StaffRole) -> i32 {
    match role {
        StaffRole::ExamRoomInvigilator => 1,
        StaffRole::SelfStudySupervisor => 2,
        StaffRole::FloorRover => 3,
    }
}

fn self_study_topic_chain_to_text(
    chain: &[exam_allocation::SelfStudyTopic],
) -> Result<String, AppError> {
    serde_json::to_string(chain).map_err(|e| AppError::new(format!("自习主题链序列化失败: {e}")))
}

fn self_study_topic_chain_from_text(
    value: &str,
) -> Result<Vec<exam_allocation::SelfStudyTopic>, AppError> {
    serde_json::from_str(value).map_err(|e| AppError::new(format!("自习主题链解析失败: {e}")))
}

fn self_study_topic_from_parts(
    kind_key: Option<String>,
    subjects_json: Option<String>,
    label: Option<String>,
) -> Result<Option<exam_allocation::SelfStudyTopic>, AppError> {
    let Some(kind_key) = kind_key else {
        return Ok(None);
    };
    let kind = exam_allocation::SelfStudyTopicKind::from_key(&kind_key)
        .ok_or_else(|| AppError::new(format!("无效的自习主题类型: {kind_key}")))?;
    let subjects = match subjects_json {
        Some(value) if !value.trim().is_empty() => serde_json::from_str::<Vec<Subject>>(&value)
            .map_err(|e| AppError::new(format!("自习主题科目解析失败: {e}")))?,
        _ => Vec::new(),
    };
    Ok(Some(exam_allocation::SelfStudyTopic {
        kind,
        subjects,
        label: label.unwrap_or_default(),
    }))
}

fn round_to_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn allowance_rate_for_role(config: &RuntimeInvigilationConfig, role: StaffRole) -> f64 {
    match role {
        StaffRole::ExamRoomInvigilator | StaffRole::SelfStudySupervisor => {
            config.indoor_allowance_per_minute
        }
        StaffRole::FloorRover => config.outdoor_allowance_per_minute,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvigilationExclusionSessionOption {
    session_id: i64,
    grade_name: String,
    subject: Subject,
    start_at: String,
    end_at: String,
    label: String,
}

fn build_config_from_payload(payload: &GenerateExamStaffPlanPayload) -> RuntimeInvigilationConfig {
    RuntimeInvigilationConfig {
        default_exam_room_required_count: payload.default_exam_room_required_count.max(1),
        indoor_allowance_per_minute: payload.indoor_allowance_per_minute.max(0.0),
        outdoor_allowance_per_minute: payload.outdoor_allowance_per_minute.max(0.0),
        middle_manager_default_enabled: false,
        middle_manager_exception_teacher_ids: HashSet::new(),
        self_study_date: String::new(),
        self_study_start_time: "12:10".to_string(),
        self_study_end_time: "13:40".to_string(),
        self_study_class_subjects: HashMap::new(),
    }
}

fn hydrate_runtime_middle_manager_config(
    conn: &Connection,
    config: &mut RuntimeInvigilationConfig,
) -> Result<(), AppError> {
    let persisted: Option<(i64, String, String, String, String)> = conn
        .query_row(
            "SELECT middle_manager_default_enabled, middle_manager_exception_teacher_ids_json, self_study_date, self_study_start_time, self_study_end_time FROM invigilation_config_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .ok();
    if let Some((
        default_enabled,
        exception_json,
        self_study_date,
        self_study_start_time,
        self_study_end_time,
    )) = persisted
    {
        config.middle_manager_default_enabled = default_enabled == 1;
        config.middle_manager_exception_teacher_ids =
            serde_json::from_str::<Vec<i64>>(&exception_json)
                .map(normalize_teacher_id_list)
                .unwrap_or_default()
                .into_iter()
                .collect();
        config.self_study_date = self_study_date.trim().to_string();
        config.self_study_start_time = self_study_start_time.trim().to_string();
        config.self_study_end_time = self_study_end_time.trim().to_string();
    }
    Ok(())
}

fn normalize_teacher_id_list(items: Vec<i64>) -> Vec<i64> {
    let mut values: Vec<i64> = items.into_iter().filter(|item| *item > 0).collect();
    values.sort_unstable();
    values.dedup();
    values
}

fn is_middle_manager_enabled(teacher: &TeacherInfo, config: &RuntimeInvigilationConfig) -> bool {
    if !teacher.is_middle_manager {
        return true;
    }
    let is_exception = config
        .middle_manager_exception_teacher_ids
        .contains(&teacher.id);
    if config.middle_manager_default_enabled {
        !is_exception
    } else {
        is_exception
    }
}

fn is_teacher_enabled_for_task_source(
    teacher: &TeacherInfo,
    task_source: StaffTaskSource,
    config: &RuntimeInvigilationConfig,
) -> bool {
    match task_source {
        StaffTaskSource::FullSelfStudy => !teacher.is_middle_manager,
        StaffTaskSource::Exam | StaffTaskSource::ExamLinkedSelfStudy => {
            is_middle_manager_enabled(teacher, config)
        }
    }
}

fn load_session_time_template_rows(conn: &Connection) -> Result<Vec<ExamSessionTime>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT subject, start_at, end_at
        FROM exam_subject_time_templates
        ORDER BY subject ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let subject_key: String = row.get(0)?;
        let subject = Subject::from_key(&subject_key).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                0,
                "subject".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;
        Ok(ExamSessionTime {
            session_id: template_session_id(subject),
            grade_name: "全局".to_string(),
            subject,
            start_at: row.get(1)?,
            end_at: row.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    out.sort_by(|a, b| subject_order(a.subject).cmp(&subject_order(b.subject)));
    Ok(out)
}

pub(crate) fn seed_default_session_times(conn: &Connection) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let mut stmt =
        conn.prepare("SELECT id, subject FROM latest_exam_plan_sessions ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (session_id, subject_key) = row?;
        let Some(subject) = Subject::from_key(&subject_key) else {
            continue;
        };
        let template_time: Option<(String, String)> = conn
            .query_row(
                "SELECT start_at, end_at FROM exam_subject_time_templates WHERE subject = ?1",
                params![subject.as_key()],
                |inner_row| Ok((inner_row.get(0)?, inner_row.get(1)?)),
            )
            .ok();
        let Some((start_at, end_at)) = template_time else {
            continue;
        };
        conn.execute(
            r#"
            INSERT INTO exam_session_times (session_id, subject, start_at, end_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(session_id) DO NOTHING
            "#,
            params![session_id, subject.as_key(), start_at, end_at, now],
        )?;
    }
    Ok(())
}

fn load_session_times_runtime(conn: &Connection) -> Result<Vec<SessionTimeRuntime>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            s.id,
            s.grade_name,
            s.subject,
            COALESCE(t.start_at, tpl.start_at) AS start_at,
            COALESCE(t.end_at, tpl.end_at) AS end_at
        FROM latest_exam_plan_sessions s
        LEFT JOIN exam_session_times t ON t.session_id = s.id
        LEFT JOIN exam_subject_time_templates tpl ON tpl.subject = s.subject
        ORDER BY s.grade_name ASC, s.id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let subject_key: String = row.get(2)?;
        let subject = Subject::from_key(&subject_key).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                2,
                "subject".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;
        Ok(ExamSessionTime {
            session_id: row.get(0)?,
            grade_name: row.get(1)?,
            subject,
            start_at: row.get(3)?,
            end_at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        let row = row?;
        let start_at = row
            .start_at
            .clone()
            .ok_or_else(|| AppError::new(format!("场次 {} 未配置开始时间", row.session_id)))?;
        let end_at = row
            .end_at
            .clone()
            .ok_or_else(|| AppError::new(format!("场次 {} 未配置结束时间", row.session_id)))?;
        let start_ts = parse_datetime_to_ts(&start_at)?;
        let end_ts = parse_datetime_to_ts(&end_at)?;
        duration_minutes(start_ts, end_ts)?;
        out.push(SessionTimeRuntime {
            session_id: row.session_id,
            grade_name: row.grade_name,
            subject: row.subject,
            start_at,
            end_at,
            start_ts,
            end_ts,
        });
    }
    out.sort_by(|a, b| {
        a.start_ts
            .cmp(&b.start_ts)
            .then(a.session_id.cmp(&b.session_id))
    });
    Ok(out)
}

fn load_teacher_pool(conn: &Connection) -> Result<Vec<TeacherInfo>, AppError> {
    let mut map: HashMap<i64, TeacherInfo> = HashMap::new();

    let mut teacher_stmt =
        conn.prepare("SELECT id, teacher_name, COALESCE(is_middle_manager, 0) FROM latest_teachers_v2 ORDER BY id ASC")?;
    let teacher_rows = teacher_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    for row in teacher_rows {
        let (id, name, is_middle_manager) = row?;
        map.insert(
            id,
            TeacherInfo {
                id,
                name,
                subjects: HashSet::new(),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: is_middle_manager == 1,
            },
        );
    }

    let mut assignment_stmt =
        conn.prepare("SELECT teacher_id, subject, class_name FROM latest_teacher_assignments_v2 ORDER BY teacher_id ASC, id ASC")?;
    let assignment_rows = assignment_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in assignment_rows {
        let (teacher_id, subject_key, class_name) = row?;
        if let Some(entry) = map.get_mut(&teacher_id) {
            if let Some(subject) = Subject::from_key(&subject_key) {
                entry.subjects.insert(subject);
            }
            entry.class_names.insert(class_name);
        }
    }

    let mut homeroom_stmt =
        conn.prepare("SELECT teacher_id, class_name FROM latest_teacher_homerooms_v2 ORDER BY teacher_id ASC, id ASC")?;
    let homeroom_rows = homeroom_stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in homeroom_rows {
        let (teacher_id, class_name) = row?;
        if let Some(entry) = map.get_mut(&teacher_id) {
            entry.homeroom_classes.insert(class_name);
        }
    }

    let mut teachers: Vec<TeacherInfo> = map.into_values().collect();
    teachers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(teachers)
}

fn load_class_subject_map(
    conn: &Connection,
) -> Result<HashMap<(String, String), HashSet<Subject>>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT c.grade_name, c.class_name, s.subject
        FROM class_configs c
        JOIN class_config_subjects s ON s.config_id = c.id
        WHERE c.config_type = 'teaching_class'
        ORDER BY c.grade_name ASC, c.class_name ASC, s.id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut map: HashMap<(String, String), HashSet<Subject>> = HashMap::new();
    for row in rows {
        let (grade_name, class_name, subject_key) = row?;
        if let Some(subject) = Subject::from_key(&subject_key) {
            map.entry((grade_name, class_name))
                .or_default()
                .insert(subject);
        }
    }
    Ok(map)
}

#[derive(Debug, Clone)]
struct TeachingClassRuntime {
    id: i64,
    grade_name: String,
    class_name: String,
    floor: String,
}

fn load_self_study_class_subjects(conn: &Connection) -> Result<HashMap<i64, Subject>, AppError> {
    let json_text: String = conn
        .query_row(
            "SELECT COALESCE(self_study_class_subjects_json, '[]') FROM invigilation_config_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "[]".to_string());
    let items =
        serde_json::from_str::<Vec<PersistedSelfStudyClassSubject>>(&json_text).unwrap_or_default();
    let mut map = HashMap::new();
    for item in items {
        if item.class_id > 0 {
            if let Some(subject) = item.subject {
                map.insert(item.class_id, subject);
            }
        }
    }
    Ok(map)
}

fn load_teaching_classes(conn: &Connection) -> Result<Vec<TeachingClassRuntime>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, grade_name, class_name, floor
        FROM class_configs
        WHERE config_type = 'teaching_class'
        ORDER BY grade_name ASC, class_name ASC, id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TeachingClassRuntime {
            id: row.get(0)?,
            grade_name: row.get(1)?,
            class_name: row.get(2)?,
            floor: row.get(3)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn load_exam_room_requirement(default_count: i64) -> Result<i64, AppError> {
    Ok(default_count.max(1))
}

fn build_task_candidate_summary(
    task: &TaskBuild,
    teachers: &[TeacherInfo],
    exclusion_pairs: &HashSet<(i64, i64)>,
    config: &RuntimeInvigilationConfig,
) -> TaskCandidateSummary {
    let active_teachers: Vec<&TeacherInfo> = teachers
        .iter()
        .filter(|teacher| {
            is_teacher_enabled_for_task_source(teacher, task.task_source, config)
                && match task.session_id {
                    Some(session_id) if task.task_source != StaffTaskSource::FullSelfStudy => {
                        !exclusion_pairs.contains(&(teacher.id, session_id))
                    }
                    _ => true,
                }
        })
        .collect();
    if active_teachers.is_empty() {
        return TaskCandidateSummary {
            candidates: Vec::new(),
        };
    }

    if task.role == StaffRole::ExamRoomInvigilator {
        let candidates: Vec<TaskCandidate> = active_teachers
            .iter()
            .filter(|teacher| !teacher.subjects.contains(&task.subject))
            .map(|teacher| TaskCandidate {
                teacher_id: teacher.id,
                assignment_tier: None,
            })
            .collect();
        return TaskCandidateSummary {
            candidates,
        };
    }

    if task.role == StaffRole::SelfStudySupervisor {
        let class_name = task.space_name.as_str();
        let mut seen = HashSet::<i64>::new();
        let mut candidates = Vec::<TaskCandidate>::new();

        if let Some(topic) = task
            .recommended_self_study_topic
            .as_ref()
            .or_else(|| task.priority_self_study_chain.first())
        {
            for teacher in &active_teachers {
                let matches_primary = match topic.kind {
                    exam_allocation::SelfStudyTopicKind::Subject => topic
                        .subjects
                        .first()
                        .is_some_and(|subject| {
                            teacher.class_names.contains(class_name)
                                && teacher.subjects.contains(subject)
                        }),
                    exam_allocation::SelfStudyTopicKind::ForeignGroup => {
                        teacher.class_names.contains(class_name)
                            && topic
                                .subjects
                                .iter()
                                .any(|subject| teacher.subjects.contains(subject))
                    }
                    exam_allocation::SelfStudyTopicKind::FreeStudy => {
                        teacher.class_names.contains(class_name)
                    }
                };
                if matches_primary && seen.insert(teacher.id) {
                    candidates.push(TaskCandidate {
                        teacher_id: teacher.id,
                        assignment_tier: Some(AssignmentTier::Primary),
                    });
                }
            }
        }

        for teacher in &active_teachers {
            if teacher.homeroom_classes.contains(class_name) && seen.insert(teacher.id) {
                candidates.push(TaskCandidate {
                    teacher_id: teacher.id,
                    assignment_tier: Some(AssignmentTier::Homeroom),
                });
            }
        }

        for teacher in &active_teachers {
            if seen.insert(teacher.id) {
                candidates.push(TaskCandidate {
                    teacher_id: teacher.id,
                    assignment_tier: Some(AssignmentTier::FallbackPool),
                });
            }
        }
        return TaskCandidateSummary {
            candidates,
        };
    }

    TaskCandidateSummary {
        candidates: active_teachers
            .iter()
            .map(|teacher| TaskCandidate {
                teacher_id: teacher.id,
                assignment_tier: None,
            })
            .collect(),
    }
}

fn build_teacher_symmetry_groups(
    teachers: &[TeacherInfo],
    candidate_summaries: &[TaskCandidateSummary],
) -> Vec<Vec<i64>> {
    let mut signatures = HashMap::<i64, Vec<(usize, Option<AssignmentTier>)>>::new();
    for teacher in teachers {
        signatures.insert(teacher.id, Vec::new());
    }
    for (task_index, summary) in candidate_summaries.iter().enumerate() {
        for candidate in &summary.candidates {
            signatures
                .entry(candidate.teacher_id)
                .or_default()
                .push((task_index, candidate.assignment_tier));
        }
    }

    let mut grouped = HashMap::<Vec<(usize, Option<AssignmentTier>)>, Vec<i64>>::new();
    for teacher in teachers {
        let mut signature = signatures.remove(&teacher.id).unwrap_or_default();
        signature.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.as_ref().map(|tier| tier.as_key()).cmp(
                    &right.1.as_ref().map(|tier| tier.as_key()),
                ))
        });
        grouped.entry(signature).or_default().push(teacher.id);
    }

    let mut groups: Vec<Vec<i64>> = grouped
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();
    for group in &mut groups {
        group.sort_unstable();
    }
    groups.sort_by_key(|group| group[0]);
    groups
}

fn clear_latest_staff_plan(tx: &rusqlite::Transaction<'_>) -> Result<(), AppError> {
    tx.execute("DELETE FROM latest_exam_staff_assignments", [])?;
    tx.execute("DELETE FROM latest_exam_staff_tasks", [])?;
    tx.execute("DELETE FROM latest_teacher_duty_stats", [])?;
    tx.execute("DELETE FROM latest_exam_staff_plan_meta", [])?;
    Ok(())
}

fn load_spaces_for_session(
    conn: &Connection,
    session_id: i64,
) -> Result<
    Vec<(
        i64,
        SpaceType,
        String,
        Option<String>,
        Option<exam_allocation::SelfStudyTopic>,
        String,
    )>,
    AppError,
> {
    let mut stmt = conn.prepare(
        "SELECT id, space_type, space_name, original_class_name, self_study_topic_kind, self_study_topic_subjects_json, self_study_topic_label, floor FROM latest_exam_plan_spaces WHERE session_id = ?1 ORDER BY sort_index ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        let space_type_key: String = row.get(1)?;
        let space_type = match space_type_key.as_str() {
            "exam_room" => SpaceType::ExamRoom,
            "self_study_room" => SpaceType::SelfStudyRoom,
            _ => {
                return Err(rusqlite::Error::InvalidColumnType(
                    1,
                    "space_type".to_string(),
                    rusqlite::types::Type::Text,
                ))
            }
        };
        let self_study_topic = self_study_topic_from_parts(
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
        )
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;
        Ok((
            row.get::<_, i64>(0)?,
            space_type,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            self_study_topic,
            row.get::<_, String>(7)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn apply_assignment_to_runtime(state: &mut TeacherRuntimeState, task: &TaskBuild) {
    match task.role {
        StaffRole::ExamRoomInvigilator => {
            state.indoor_minutes += task.duration_minutes;
            state.invigilation_minutes += task.duration_minutes;
            state.exam_room_task_count += 1;
        }
        StaffRole::SelfStudySupervisor => {
            state.indoor_minutes += task.duration_minutes;
            state.self_study_minutes += task.duration_minutes;
            state.self_study_task_count += 1;
        }
        StaffRole::FloorRover => {
            state.outdoor_minutes += task.duration_minutes;
            state.invigilation_minutes += task.duration_minutes;
            state.floor_rover_task_count += 1;
        }
    }
    state.total_minutes += task.duration_minutes;
    state.task_count += 1;
    state.busy_ranges.push((task.start_ts, task.end_ts));
    state
        .day_half_loads
        .entry(task.day_key.clone())
        .or_default()
        .add_task(task.half_day);
}

fn cross_half_day_penalty(runtime: &HashMap<i64, TeacherRuntimeState>) -> i64 {
    runtime
        .values()
        .map(|state| {
            state
                .day_half_loads
                .values()
                .filter(|load| load.morning_tasks > 0 && load.afternoon_tasks > 0)
                .count() as i64
        })
        .sum()
}

fn compute_plan_metrics(
    teachers: &[TeacherInfo],
    runtime: &HashMap<i64, TeacherRuntimeState>,
    records: &[SolvedTaskRecord],
) -> PlanMetrics {
    let assigned_count = records
        .iter()
        .filter(|record| record.teacher_id.is_some())
        .count() as i64;
    let unassigned_count = records.len() as i64 - assigned_count;
    let fallback_pool_assignments = records
        .iter()
        .filter(|record| record.assignment_tier == Some(AssignmentTier::FallbackPool))
        .count() as i64;
    let homeroom_assignments = records
        .iter()
        .filter(|record| record.assignment_tier == Some(AssignmentTier::Homeroom))
        .count() as i64;

    let mut max_total = 0_i64;
    let mut min_total = i64::MAX;
    let mut max_invigilation = 0_i64;
    let mut min_invigilation = i64::MAX;
    let mut max_self_study = 0_i64;
    let mut min_self_study = i64::MAX;
    for teacher in teachers {
        let state = runtime.get(&teacher.id).cloned().unwrap_or_default();
        max_total = max_total.max(state.total_minutes);
        min_total = min_total.min(state.total_minutes);
        max_invigilation = max_invigilation.max(state.invigilation_minutes);
        min_invigilation = min_invigilation.min(state.invigilation_minutes);
        max_self_study = max_self_study.max(state.self_study_minutes);
        min_self_study = min_self_study.min(state.self_study_minutes);
    }
    let imbalance_minutes = if teachers.is_empty() {
        0
    } else {
        max_total.saturating_sub(min_total)
    };
    let invigilation_minutes_gap = if teachers.is_empty() {
        0
    } else {
        max_invigilation.saturating_sub(min_invigilation)
    };
    let self_study_minutes_gap = if teachers.is_empty() {
        0
    } else {
        max_self_study.saturating_sub(min_self_study)
    };
    let warning_count = unassigned_count + if imbalance_minutes > 90 { 1 } else { 0 };

    PlanMetrics {
        assigned_count,
        unassigned_count,
        fallback_pool_assignments,
        homeroom_assignments,
        invigilation_minutes_gap,
        self_study_minutes_gap,
        cross_half_day_penalty: cross_half_day_penalty(runtime),
        imbalance_minutes,
        warning_count,
    }
}

fn build_staff_tasks(
    conn: &Connection,
    session_times: &[SessionTimeRuntime],
    invigilation_config: &RuntimeInvigilationConfig,
    class_subject_map: &HashMap<(String, String), HashSet<Subject>>,
    teaching_classes: &[TeachingClassRuntime],
) -> Result<Vec<TaskBuild>, AppError> {
    let mut sessions_by_grade: HashMap<String, Vec<exam_allocation::SelfStudyScheduleSession>> =
        HashMap::new();
    for session in session_times {
        sessions_by_grade
            .entry(session.grade_name.clone())
            .or_default()
            .push(exam_allocation::SelfStudyScheduleSession {
                subject: session.subject,
                start_ts: session.start_ts,
                order_key: session.session_id,
                is_foreign_group: exam_allocation::is_foreign_subject(session.subject),
            });
    }
    for session_list in sessions_by_grade.values_mut() {
        session_list.sort_by(|a, b| {
            a.start_ts
                .cmp(&b.start_ts)
                .then(a.order_key.cmp(&b.order_key))
        });
    }
    let mut class_subjects_by_grade = HashMap::<String, HashMap<String, HashSet<Subject>>>::new();
    for ((grade_name, class_name), subjects) in class_subject_map {
        class_subjects_by_grade
            .entry(grade_name.clone())
            .or_default()
            .insert(class_name.clone(), subjects.clone());
    }

    let mut tasks = Vec::<TaskBuild>::new();
    for session in session_times {
        let spaces = load_spaces_for_session(conn, session.session_id)?;
        if spaces.is_empty() {
            return Err(AppError::new(format!(
                "场次 {} 无可用空间",
                session.session_id
            )));
        }

        let mut floors = HashSet::<String>::new();
        let (day_key, half_day) = parse_day_slot(&session.start_at)?;
        let grade_sessions = sessions_by_grade
            .get(&session.grade_name)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let grade_class_subjects = class_subjects_by_grade
            .get(&session.grade_name)
            .cloned()
            .unwrap_or_default();
        for (space_id, space_type, space_name, original_class_name, self_study_topic, floor) in &spaces
        {
            if floor.trim().is_empty() {
                return Err(AppError::new(format!(
                    "场次 {} 存在空楼层，无法分配流动监考",
                    session.session_id
                )));
            }
            floors.insert(floor.clone());
            match space_type {
                SpaceType::ExamRoom => {
                    let required = load_exam_room_requirement(
                        invigilation_config.default_exam_room_required_count,
                    )?;
                    for _ in 0..required {
                        tasks.push(TaskBuild {
                            session_id: Some(session.session_id),
                            space_id: Some(*space_id),
                            task_source: StaffTaskSource::Exam,
                            role: StaffRole::ExamRoomInvigilator,
                            grade_name: session.grade_name.clone(),
                            subject: session.subject,
                            space_name: space_name.clone(),
                            floor: floor.clone(),
                            start_at: session.start_at.clone(),
                            end_at: session.end_at.clone(),
                            start_ts: session.start_ts,
                            end_ts: session.end_ts,
                            duration_minutes: duration_minutes(session.start_ts, session.end_ts)?,
                            recommended_self_study_topic: None,
                            priority_self_study_chain: Vec::new(),
                            day_key: day_key.clone(),
                            half_day,
                        });
                    }
                }
                SpaceType::SelfStudyRoom => {
                    let class_name = original_class_name
                        .clone()
                        .unwrap_or_else(|| space_name.clone());
                    let computed_chain = exam_allocation::build_self_study_topic_chain(
                        session.start_ts,
                        &class_name,
                        grade_sessions,
                        &grade_class_subjects,
                    );
                    let recommended_self_study_topic = self_study_topic
                        .clone()
                        .or_else(|| computed_chain.first().cloned());
                    let priority_self_study_chain =
                        if let Some(saved_topic) = recommended_self_study_topic.clone() {
                            let mut chain = Vec::with_capacity(computed_chain.len().max(1));
                            chain.push(saved_topic.clone());
                            for topic in computed_chain {
                                if topic != saved_topic {
                                    chain.push(topic);
                                }
                            }
                            chain
                        } else {
                            computed_chain
                        };
                    tasks.push(TaskBuild {
                        session_id: Some(session.session_id),
                        space_id: Some(*space_id),
                        task_source: StaffTaskSource::ExamLinkedSelfStudy,
                        role: StaffRole::SelfStudySupervisor,
                        grade_name: session.grade_name.clone(),
                        subject: session.subject,
                        space_name: class_name,
                        floor: floor.clone(),
                        start_at: session.start_at.clone(),
                        end_at: session.end_at.clone(),
                        start_ts: session.start_ts,
                        end_ts: session.end_ts,
                        duration_minutes: duration_minutes(session.start_ts, session.end_ts)?,
                        recommended_self_study_topic,
                        priority_self_study_chain,
                        day_key: day_key.clone(),
                        half_day,
                    });
                }
            }
        }

        let mut sorted_floors: Vec<String> = floors.into_iter().collect();
        sorted_floors.sort();
        for floor in sorted_floors {
            tasks.push(TaskBuild {
                session_id: Some(session.session_id),
                space_id: None,
                task_source: StaffTaskSource::Exam,
                role: StaffRole::FloorRover,
                grade_name: session.grade_name.clone(),
                subject: session.subject,
                space_name: format!("{} 楼层流动", floor),
                floor,
                start_at: session.start_at.clone(),
                end_at: session.end_at.clone(),
                start_ts: session.start_ts,
                end_ts: session.end_ts,
                duration_minutes: duration_minutes(session.start_ts, session.end_ts)?,
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: day_key.clone(),
                half_day,
            });
        }
    }

    if !teaching_classes.is_empty() {
        let start_at = build_self_study_datetime(
            &invigilation_config.self_study_date,
            &invigilation_config.self_study_start_time,
        )?;
        let end_at = build_self_study_datetime(
            &invigilation_config.self_study_date,
            &invigilation_config.self_study_end_time,
        )?;
        let start_ts = parse_datetime_to_ts(&start_at)?;
        let end_ts = parse_datetime_to_ts(&end_at)?;
        let duration = duration_minutes(start_ts, end_ts)?;
        let (day_key, half_day) = parse_day_slot(&start_at)?;

        for teaching_class in teaching_classes {
            let Some(subject) = invigilation_config
                .self_study_class_subjects
                .get(&teaching_class.id)
                .copied()
            else {
                return Err(AppError::new(format!(
                    "班级 {} 未配置全员自习科目，无法分配全员自习老师",
                    teaching_class.class_name
                )));
            };
            tasks.push(TaskBuild {
                session_id: None,
                space_id: None,
                task_source: StaffTaskSource::FullSelfStudy,
                role: StaffRole::SelfStudySupervisor,
                grade_name: teaching_class.grade_name.clone(),
                subject,
                space_name: teaching_class.class_name.clone(),
                floor: teaching_class.floor.clone(),
                start_at: start_at.clone(),
                end_at: end_at.clone(),
                start_ts,
                end_ts,
                duration_minutes: duration,
                recommended_self_study_topic: Some(
                    exam_allocation::build_subject_self_study_topic(subject),
                ),
                priority_self_study_chain: vec![
                    exam_allocation::build_subject_self_study_topic(subject),
                ],
                day_key: day_key.clone(),
                half_day,
            });
        }
    }

    tasks.sort_by(|a, b| {
        a.start_ts
            .cmp(&b.start_ts)
            .then(role_priority(a.role).cmp(&role_priority(b.role)))
            .then(a.session_id.cmp(&b.session_id))
            .then(a.space_name.cmp(&b.space_name))
    });

    Ok(tasks)
}

fn initial_runtime_by_teacher(teachers: &[TeacherInfo]) -> HashMap<i64, TeacherRuntimeState> {
    let mut runtime = HashMap::new();
    for teacher in teachers {
        runtime.insert(teacher.id, TeacherRuntimeState::default());
    }
    runtime
}

fn apply_allowance_totals(
    state: &mut TeacherRuntimeState,
    task: &TaskBuild,
    allowance_amount: f64,
) {
    state.allowance_total = round_to_two(state.allowance_total + allowance_amount);
    match task.role {
        StaffRole::ExamRoomInvigilator | StaffRole::SelfStudySupervisor => {
            state.indoor_allowance_total =
                round_to_two(state.indoor_allowance_total + allowance_amount);
        }
        StaffRole::FloorRover => {
            state.outdoor_allowance_total =
                round_to_two(state.outdoor_allowance_total + allowance_amount);
        }
    }
}

fn cp_sat_time_limit_params(remaining_ms: i64) -> SatParameters {
    let mut params = SatParameters::default();
    params.max_time_in_seconds = Some((remaining_ms.max(1) as f64) / 1000.0);
    params.num_search_workers = Some(8);
    params.log_search_progress = Some(false);
    params.repair_hint = Some(true);
    params.hint_conflict_limit = Some(1_000);
    params
}

fn cp_sat_stage_time_budget_ms(stage_name: &str, remaining_ms: i64) -> i64 {
    let preferred_budget_ms = match stage_name {
        "unassigned_count" | "fallback_pool_assignments" | "homeroom_assignments" => {
            CP_SAT_FAST_STAGE_BUDGET_MS
        }
        "total_minutes_gap" | "invigilation_minutes_gap" | "self_study_minutes_gap" => {
            CP_SAT_BALANCE_STAGE_BUDGET_MS
        }
        "cross_half_day_penalty" => CP_SAT_FAST_STAGE_BUDGET_MS,
        _ => remaining_ms,
    };
    remaining_ms.min(preferred_budget_ms).max(1)
}

fn add_load_gap_var(
    model: &mut CpModelBuilder,
    load_vars: &[IntVar],
    capacity: i64,
    prefix: &str,
) -> IntVar {
    let gap_var = model.new_int_var_with_name([(0, capacity)], format!("{prefix}_gap"));

    if load_vars.len() <= 1 {
        model.add_eq(gap_var, 0);
        return gap_var;
    }

    for (left_index, left_var) in load_vars.iter().enumerate() {
        for right_var in load_vars.iter().skip(left_index + 1) {
            model.add_le(LinearExpr::from(*left_var) - *right_var, gap_var);
            model.add_le(LinearExpr::from(*right_var) - *left_var, gap_var);
        }
    }

    gap_var
}

fn add_teacher_timepoint_non_overlap_constraints(
    model: &mut CpModelBuilder,
    tasks: &[TaskBuild],
    teacher_assignment_vars: &HashMap<i64, Vec<(usize, BoolVar)>>,
) {
    for teacher_vars in teacher_assignment_vars.values() {
        if teacher_vars.len() <= 1 {
            continue;
        }
        let mut time_points: Vec<i64> = teacher_vars
            .iter()
            .map(|(task_index, _)| tasks[*task_index].start_ts)
            .collect();
        time_points.sort_unstable();
        time_points.dedup();

        let mut seen_groups = HashSet::<Vec<usize>>::new();
        for time_point in time_points {
            let mut active_group = Vec::<(usize, BoolVar)>::new();
            for (task_index, assignment_var) in teacher_vars {
                let task = &tasks[*task_index];
                if task.start_ts <= time_point && time_point < task.end_ts {
                    active_group.push((*task_index, *assignment_var));
                }
            }
            if active_group.len() <= 1 {
                continue;
            }

            let mut group_key: Vec<usize> =
                active_group.iter().map(|(task_index, _)| *task_index).collect();
            group_key.sort_unstable();
            if !seen_groups.insert(group_key) {
                continue;
            }

            model.add_at_most_one(
                active_group
                    .into_iter()
                    .map(|(_, assignment_var)| assignment_var),
            );
        }
    }
}

fn add_solution_hints(
    model: &mut CpModelBuilder,
    response: &CpSolverResponse,
    bool_vars: &[BoolVar],
    int_vars: &[IntVar],
) {
    if response.solution.is_empty() {
        return;
    }

    for bool_var in bool_vars {
        model.add_hint(*bool_var, if bool_var.solution_value(response) { 1 } else { 0 });
    }
    for int_var in int_vars {
        model.add_hint(*int_var, int_var.solution_value(response));
    }
}

fn add_manual_bool_hints(model: &mut CpModelBuilder, bool_hints: &[(BoolVar, i64)]) {
    for (bool_var, value) in bool_hints {
        model.add_hint(*bool_var, *value);
    }
}

fn build_teacher_load_rank_expr(
    total_load_var: IntVar,
    invigilation_load_var: IntVar,
    self_study_load_var: IntVar,
    invigilation_minutes_capacity: i64,
    self_study_minutes_capacity: i64,
) -> Option<LinearExpr> {
    let invigilation_multiplier = self_study_minutes_capacity.checked_add(1)?;
    let total_multiplier = invigilation_minutes_capacity
        .checked_add(1)?
        .checked_mul(invigilation_multiplier)?;
    let mut expr = LinearExpr::default();
    expr += (total_multiplier, total_load_var);
    expr += (invigilation_multiplier, invigilation_load_var);
    expr += self_study_load_var;
    Some(expr)
}

fn add_teacher_symmetry_breaking_constraints(
    model: &mut CpModelBuilder,
    symmetry_groups: &[Vec<i64>],
    teacher_load_vars: &HashMap<i64, (IntVar, IntVar, IntVar)>,
    invigilation_minutes_capacity: i64,
    self_study_minutes_capacity: i64,
) {
    for group in symmetry_groups {
        for teacher_window in group.windows(2) {
            let Some(left_vars) = teacher_load_vars.get(&teacher_window[0]).copied() else {
                continue;
            };
            let Some(right_vars) = teacher_load_vars.get(&teacher_window[1]).copied() else {
                continue;
            };
            let Some(left_rank_expr) = build_teacher_load_rank_expr(
                left_vars.0,
                left_vars.1,
                left_vars.2,
                invigilation_minutes_capacity,
                self_study_minutes_capacity,
            ) else {
                continue;
            };
            let Some(right_rank_expr) = build_teacher_load_rank_expr(
                right_vars.0,
                right_vars.1,
                right_vars.2,
                invigilation_minutes_capacity,
                self_study_minutes_capacity,
            ) else {
                continue;
            };
            model.add_ge(left_rank_expr, right_rank_expr);
        }
    }
}

fn solve_cp_sat_stage(
    base_proto: &cp_sat::proto::CpModelProto,
    fixed_objectives: &[(LinearExpr, i64)],
    objective: LinearExpr,
    remaining_ms: i64,
    hint_response: Option<&CpSolverResponse>,
    hint_bool_vars: &[BoolVar],
    hint_int_vars: &[IntVar],
    manual_bool_hints: &[(BoolVar, i64)],
) -> CpSolverResponse {
    let mut builder = CpModelBuilder::from_proto(base_proto.clone());
    for (expr, value) in fixed_objectives {
        builder.add_eq(expr.clone(), *value);
    }
    if let Some(response) = hint_response {
        add_solution_hints(&mut builder, response, hint_bool_vars, hint_int_vars);
    }
    add_manual_bool_hints(&mut builder, manual_bool_hints);
    builder.minimize(objective);
    builder.solve_with_parameters(&cp_sat_time_limit_params(remaining_ms))
}

fn cp_sat_response_kind(
    response: &CpSolverResponse,
    elapsed_ms: i64,
) -> Result<OptimalityStatus, FallbackReason> {
    match response.status() {
        CpSolverStatus::Optimal => Ok(OptimalityStatus::Optimal),
        CpSolverStatus::Feasible => Ok(OptimalityStatus::Feasible),
        CpSolverStatus::Infeasible => Err(FallbackReason::Infeasible),
        CpSolverStatus::Unknown => {
            if elapsed_ms >= CP_SAT_MAX_SOLVE_MS {
                Err(FallbackReason::Timeout)
            } else if !response.solution_info.trim().is_empty() {
                Err(FallbackReason::Error)
            } else {
                Err(FallbackReason::Unknown)
            }
        }
        CpSolverStatus::ModelInvalid => Err(FallbackReason::Error),
    }
}

fn cp_sat_diagnostic_message(response: &CpSolverResponse) -> Option<String> {
    let info = response.solution_info.trim();
    if !info.is_empty() {
        return Some(info.to_string());
    }
    let log = response.solve_log.trim();
    if !log.is_empty() {
        return Some(log.to_string());
    }
    None
}

fn build_cp_sat_plan_from_response(
    tasks: &[TaskBuild],
    teachers: &[TeacherInfo],
    invigilation_config: &RuntimeInvigilationConfig,
    candidate_bindings: &[Vec<(BoolVar, TaskCandidate)>],
    unassigned_vars: &[BoolVar],
    response: &CpSolverResponse,
    optimality_status: OptimalityStatus,
    solve_duration_ms: i64,
) -> SolvedPlan {
    let mut runtime = initial_runtime_by_teacher(teachers);
    let mut records = Vec::<SolvedTaskRecord>::new();

    for (task_index, task) in tasks.iter().enumerate() {
        let selected_candidate = candidate_bindings[task_index]
            .iter()
            .find(|(var, _)| var.solution_value(response));
        let selected_teacher_id = selected_candidate.map(|(_, candidate)| candidate.teacher_id);
        let selected_tier = selected_candidate.and_then(|(_, candidate)| candidate.assignment_tier);
        let is_unassigned = unassigned_vars
            .get(task_index)
            .map(|var| var.solution_value(response))
            .unwrap_or(false);
        let reason = if selected_teacher_id.is_none() {
            if is_unassigned {
                Some("unassigned_by_solver".to_string())
            } else {
                Some("solver_no_selection".to_string())
            }
        } else {
            None
        };
        let allowance_amount = if selected_teacher_id.is_some() {
            round_to_two(
                (task.duration_minutes as f64)
                    * allowance_rate_for_role(invigilation_config, task.role),
            )
        } else {
            0.0
        };
        if let Some(teacher_id) = selected_teacher_id {
            if let Some(state) = runtime.get_mut(&teacher_id) {
                apply_assignment_to_runtime(state, task);
                apply_allowance_totals(state, task, allowance_amount);
            }
        }
        records.push(SolvedTaskRecord {
            task: task.clone(),
            teacher_id: selected_teacher_id,
            reason,
            assignment_tier: selected_tier,
            allowance_amount,
        });
    }

    SolvedPlan {
        metrics: compute_plan_metrics(teachers, &runtime, &records),
        records,
        runtime,
        solver_engine: SolverEngine::CpSat,
        optimality_status,
        solve_duration_ms,
        fallback_reason: None,
    }
}

fn solve_with_cp_sat(
    tasks: &[TaskBuild],
    teachers: &[TeacherInfo],
    exclusion_pairs: &HashSet<(i64, i64)>,
    invigilation_config: &RuntimeInvigilationConfig,
    progress: Option<&StaffAssignmentProgressReporter>,
) -> CpSatAttempt {
    let started_at = Instant::now();
    let candidate_summaries: Vec<TaskCandidateSummary> = tasks
        .iter()
        .map(|task| {
            build_task_candidate_summary(task, teachers, exclusion_pairs, invigilation_config)
        })
        .collect();
    let teacher_symmetry_groups = build_teacher_symmetry_groups(teachers, &candidate_summaries);

    let mut model = CpModelBuilder::default();
    let mut candidate_bindings = Vec::<Vec<(BoolVar, TaskCandidate)>>::new();
    let mut unassigned_vars = Vec::<BoolVar>::new();
    let mut teacher_assignment_vars = HashMap::<i64, Vec<(usize, BoolVar)>>::new();
    let mut teacher_day_half_vars = HashMap::<(i64, String, HalfDay), Vec<BoolVar>>::new();
    let mut teacher_load_vars = HashMap::<i64, (IntVar, IntVar, IntVar)>::new();

    let total_minutes_capacity = tasks.iter().map(|task| task.duration_minutes).sum::<i64>().max(1);
    let invigilation_minutes_capacity = tasks
        .iter()
        .filter(|task| task.role != StaffRole::SelfStudySupervisor)
        .map(|task| task.duration_minutes)
        .sum::<i64>()
        .max(1);
    let self_study_minutes_capacity = tasks
        .iter()
        .filter(|task| task.role == StaffRole::SelfStudySupervisor)
        .map(|task| task.duration_minutes)
        .sum::<i64>()
        .max(1);

    let mut unassigned_expr = LinearExpr::default();
    let mut fallback_expr = LinearExpr::default();
    let mut homeroom_expr = LinearExpr::default();
    let self_study_task_capacity = tasks
        .iter()
        .filter(|task| task.role == StaffRole::SelfStudySupervisor)
        .count() as i64;

    for (task_index, task) in tasks.iter().enumerate() {
        let summary = &candidate_summaries[task_index];
        let mut exact_one_vars = Vec::<BoolVar>::new();
        let mut bindings_for_task = Vec::<(BoolVar, TaskCandidate)>::new();

        for candidate in &summary.candidates {
            let var = model.new_bool_var_with_name(format!(
                "assign_t{}_teacher_{}",
                task_index, candidate.teacher_id
            ));
            exact_one_vars.push(var);
            teacher_assignment_vars
                .entry(candidate.teacher_id)
                .or_default()
                .push((task_index, var));
            teacher_day_half_vars
                .entry((candidate.teacher_id, task.day_key.clone(), task.half_day))
                .or_default()
                .push(var);
            if candidate.assignment_tier == Some(AssignmentTier::FallbackPool) {
                fallback_expr += var;
            }
            if candidate.assignment_tier == Some(AssignmentTier::Homeroom) {
                homeroom_expr += var;
            }
            bindings_for_task.push((var, candidate.clone()));
        }

        let unassigned = model.new_bool_var_with_name(format!("unassigned_t{task_index}"));
        exact_one_vars.push(unassigned);
        model.add_exactly_one(exact_one_vars);
        unassigned_expr += unassigned;
        unassigned_vars.push(unassigned);
        candidate_bindings.push(bindings_for_task);
    }

    add_teacher_timepoint_non_overlap_constraints(&mut model, tasks, &teacher_assignment_vars);

    let mut total_load_vars = Vec::<IntVar>::new();
    let mut invigilation_load_vars = Vec::<IntVar>::new();
    let mut self_study_load_vars = Vec::<IntVar>::new();
    for teacher in teachers {
        let total_load_var = model.new_int_var_with_name(
            [(0, total_minutes_capacity)],
            format!("total_minutes_{}", teacher.id),
        );
        let total_expr: LinearExpr = teacher_assignment_vars
            .get(&teacher.id)
            .into_iter()
            .flat_map(|items| items.iter())
            .map(|(task_index, var)| (tasks[*task_index].duration_minutes, *var))
            .collect();
        model.add_eq(total_load_var, total_expr);
        total_load_vars.push(total_load_var);

        let invigilation_load_var = model.new_int_var_with_name(
            [(0, invigilation_minutes_capacity)],
            format!("invigilation_minutes_{}", teacher.id),
        );
        let invigilation_expr: LinearExpr = teacher_assignment_vars
            .get(&teacher.id)
            .into_iter()
            .flat_map(|items| items.iter())
            .filter(|(task_index, _)| tasks[*task_index].role != StaffRole::SelfStudySupervisor)
            .map(|(task_index, var)| (tasks[*task_index].duration_minutes, *var))
            .collect();
        model.add_eq(invigilation_load_var, invigilation_expr);
        invigilation_load_vars.push(invigilation_load_var);

        let self_study_load_var = model.new_int_var_with_name(
            [(0, self_study_minutes_capacity)],
            format!("self_study_minutes_{}", teacher.id),
        );
        let self_study_expr: LinearExpr = teacher_assignment_vars
            .get(&teacher.id)
            .into_iter()
            .flat_map(|items| items.iter())
            .filter(|(task_index, _)| tasks[*task_index].role == StaffRole::SelfStudySupervisor)
            .map(|(task_index, var)| (tasks[*task_index].duration_minutes, *var))
            .collect();
        model.add_eq(self_study_load_var, self_study_expr);
        model.add_eq(
            total_load_var,
            LinearExpr::from(invigilation_load_var) + self_study_load_var,
        );
        teacher_load_vars.insert(
            teacher.id,
            (total_load_var, invigilation_load_var, self_study_load_var),
        );
        self_study_load_vars.push(self_study_load_var);
    }

    add_teacher_symmetry_breaking_constraints(
        &mut model,
        &teacher_symmetry_groups,
        &teacher_load_vars,
        invigilation_minutes_capacity,
        self_study_minutes_capacity,
    );

    let total_minutes_gap_var = add_load_gap_var(
        &mut model,
        &total_load_vars,
        total_minutes_capacity,
        "total_minutes",
    );
    let invigilation_minutes_gap_var = add_load_gap_var(
        &mut model,
        &invigilation_load_vars,
        invigilation_minutes_capacity,
        "invigilation_minutes",
    );
    let self_study_minutes_gap_var = add_load_gap_var(
        &mut model,
        &self_study_load_vars,
        self_study_minutes_capacity,
        "self_study_minutes",
    );

    let mut total_assigned_minutes_expr = LinearExpr::default();
    for total_load_var in &total_load_vars {
        total_assigned_minutes_expr += *total_load_var;
    }
    let mut total_unassigned_minutes_expr = LinearExpr::default();
    for (task_index, unassigned_var) in unassigned_vars.iter().enumerate() {
        total_unassigned_minutes_expr += (tasks[task_index].duration_minutes, *unassigned_var);
    }
    model.add_eq(
        total_assigned_minutes_expr + total_unassigned_minutes_expr,
        tasks.iter().map(|task| task.duration_minutes).sum::<i64>(),
    );

    let mut invigilation_assigned_minutes_expr = LinearExpr::default();
    for invigilation_load_var in &invigilation_load_vars {
        invigilation_assigned_minutes_expr += *invigilation_load_var;
    }
    let mut invigilation_unassigned_minutes_expr = LinearExpr::default();
    for (task_index, unassigned_var) in unassigned_vars.iter().enumerate() {
        let task = &tasks[task_index];
        if task.role != StaffRole::SelfStudySupervisor {
            invigilation_unassigned_minutes_expr += (task.duration_minutes, *unassigned_var);
        }
    }
    model.add_eq(
        invigilation_assigned_minutes_expr + invigilation_unassigned_minutes_expr,
        tasks.iter()
            .filter(|task| task.role != StaffRole::SelfStudySupervisor)
            .map(|task| task.duration_minutes)
            .sum::<i64>(),
    );

    let mut self_study_assigned_minutes_expr = LinearExpr::default();
    for self_study_load_var in &self_study_load_vars {
        self_study_assigned_minutes_expr += *self_study_load_var;
    }
    let mut self_study_unassigned_minutes_expr = LinearExpr::default();
    for (task_index, unassigned_var) in unassigned_vars.iter().enumerate() {
        let task = &tasks[task_index];
        if task.role == StaffRole::SelfStudySupervisor {
            self_study_unassigned_minutes_expr += (task.duration_minutes, *unassigned_var);
        }
    }
    model.add_eq(
        self_study_assigned_minutes_expr + self_study_unassigned_minutes_expr,
        tasks.iter()
            .filter(|task| task.role == StaffRole::SelfStudySupervisor)
            .map(|task| task.duration_minutes)
            .sum::<i64>(),
    );

    let unassigned_count_var = model.new_int_var_with_name(
        [(0, tasks.len() as i64)],
        "unassigned_count",
    );
    model.add_eq(unassigned_count_var, unassigned_expr.clone());

    let fallback_count_var = model.new_int_var_with_name(
        [(0, self_study_task_capacity)],
        "fallback_pool_assignments",
    );
    model.add_eq(fallback_count_var, fallback_expr.clone());

    let homeroom_count_var = model.new_int_var_with_name(
        [(0, self_study_task_capacity)],
        "homeroom_assignments",
    );
    model.add_eq(homeroom_count_var, homeroom_expr.clone());

    let pre_cross_proto = model.into_proto();
    let mut fixed_objectives = Vec::<(LinearExpr, i64)>::new();
    let mut hint_bool_vars: Vec<BoolVar> = candidate_bindings
        .iter()
        .flat_map(|bindings| bindings.iter().map(|(assignment_var, _)| *assignment_var))
        .collect();
    hint_bool_vars.extend(unassigned_vars.iter().copied());
    let hint_int_vars: Vec<IntVar> = total_load_vars
        .iter()
        .copied()
        .chain([unassigned_count_var, fallback_count_var, homeroom_count_var])
        .chain(invigilation_load_vars.iter().copied())
        .chain(self_study_load_vars.iter().copied())
        .chain([
            total_minutes_gap_var,
            invigilation_minutes_gap_var,
            self_study_minutes_gap_var,
        ])
        .collect();
    let stage_objectives = vec![
        (
            "unassigned_count",
            "最小化未分配任务",
            LinearExpr::from(unassigned_count_var),
        ),
        (
            "fallback_pool_assignments",
            "减少其他老师兜底",
            LinearExpr::from(fallback_count_var),
        ),
        (
            "homeroom_assignments",
            "减少班主任兜底",
            LinearExpr::from(homeroom_count_var),
        ),
        (
            "total_minutes_gap",
            "平衡总工作量",
            LinearExpr::from(total_minutes_gap_var),
        ),
        (
            "invigilation_minutes_gap",
            "平衡监考工作量",
            LinearExpr::from(invigilation_minutes_gap_var),
        ),
        (
            "self_study_minutes_gap",
            "平衡看班工作量",
            LinearExpr::from(self_study_minutes_gap_var),
        ),
    ];

    let mut last_successful: Option<(CpSolverResponse, OptimalityStatus)> = None;
    for (stage_index, (stage_name, stage_label, objective)) in stage_objectives.iter().enumerate()
    {
        if let Some(progress) = progress {
            let step = 6 + stage_index;
            progress.emit_running(
                step,
                stage_name,
                stage_label,
                format!(
                    "正在执行第 {}/{} 步：{}。",
                    step, STAFF_ASSIGNMENT_TOTAL_STEPS, stage_label
                ),
            );
        }
        let elapsed_ms = started_at.elapsed().as_millis() as i64;
        if elapsed_ms >= CP_SAT_MAX_SOLVE_MS {
            return CpSatAttempt {
                plan: None,
                fallback_reason: Some(FallbackReason::Timeout),
                diagnostic_message: Some(format!(
                    "CP-SAT 在第 {} 阶段（{}）达到 {}时限",
                    stage_index + 1,
                    stage_label,
                    CP_SAT_MAX_SOLVE_LABEL
                )),
                solve_duration_ms: elapsed_ms,
            };
        }
        let response = solve_cp_sat_stage(
            &pre_cross_proto,
            &fixed_objectives,
            objective.clone(),
            cp_sat_stage_time_budget_ms(stage_name, CP_SAT_MAX_SOLVE_MS - elapsed_ms),
            last_successful.as_ref().map(|(response, _)| response),
            &hint_bool_vars,
            &hint_int_vars,
            &[],
        );
        let stage_elapsed_ms = started_at.elapsed().as_millis() as i64;
        let Ok(optimality_status) = cp_sat_response_kind(&response, stage_elapsed_ms) else {
            let fallback_reason = cp_sat_response_kind(&response, stage_elapsed_ms).err();
            let diagnostic_message = cp_sat_diagnostic_message(&response).map(|detail| {
                format!(
                    "CP-SAT 第 {} 阶段（{}）失败：{}",
                    stage_index + 1,
                    stage_label,
                    detail
                )
            });
            if let Some((best_response, best_status)) = last_successful {
                let plan = build_cp_sat_plan_from_response(
                    tasks,
                    teachers,
                    invigilation_config,
                    &candidate_bindings,
                    &unassigned_vars,
                    &best_response,
                    best_status,
                    stage_elapsed_ms,
                );
                return CpSatAttempt {
                    plan: Some(plan),
                    fallback_reason,
                    diagnostic_message,
                    solve_duration_ms: stage_elapsed_ms,
                };
            }
            return CpSatAttempt {
                plan: None,
                fallback_reason,
                diagnostic_message,
                solve_duration_ms: stage_elapsed_ms,
            };
        };
        let objective_value = response.objective_value.round() as i64;
        fixed_objectives.push((objective.clone(), objective_value));
        last_successful = Some((response, optimality_status));
    }

    let Some((pre_cross_response, pre_cross_status)) = last_successful else {
        let elapsed_ms = started_at.elapsed().as_millis() as i64;
        return CpSatAttempt {
            plan: None,
            fallback_reason: Some(FallbackReason::Unknown),
            diagnostic_message: Some("CP-SAT 未返回可用结果".to_string()),
            solve_duration_ms: elapsed_ms,
        };
    };

    let final_stage_number = stage_objectives.len() + 1;
    let final_step = 6 + stage_objectives.len();
    if let Some(progress) = progress {
        progress.emit_running(
            final_step,
            "cross_half_day_penalty",
            "尽量集中到同一晌",
            format!(
                "正在执行第 {}/{} 步：尽量集中到同一晌。",
                final_step, STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }

    let elapsed_ms = started_at.elapsed().as_millis() as i64;
    if elapsed_ms >= CP_SAT_MAX_SOLVE_MS {
        let plan = build_cp_sat_plan_from_response(
            tasks,
            teachers,
            invigilation_config,
            &candidate_bindings,
            &unassigned_vars,
            &pre_cross_response,
            pre_cross_status,
            elapsed_ms,
        );
        return CpSatAttempt {
            plan: Some(plan),
            fallback_reason: Some(FallbackReason::Timeout),
            diagnostic_message: Some(format!(
                "CP-SAT 在第 {} 阶段（尽量集中到同一晌）达到 {}时限",
                final_stage_number, CP_SAT_MAX_SOLVE_LABEL
            )),
            solve_duration_ms: elapsed_ms,
        };
    }

    let mut cross_builder = CpModelBuilder::from_proto(pre_cross_proto.clone());
    let mut cross_penalty_expr = LinearExpr::default();
    let mut teacher_days = HashSet::<(i64, String)>::new();
    for (teacher_id, day_key, _) in teacher_day_half_vars.keys() {
        teacher_days.insert((*teacher_id, day_key.clone()));
    }
    let mut cross_stage_hint_bools = Vec::<(BoolVar, i64)>::new();
    for (teacher_id, day_key) in teacher_days {
        let morning_vars = teacher_day_half_vars
            .get(&(teacher_id, day_key.clone(), HalfDay::Morning))
            .cloned()
            .unwrap_or_default();
        let afternoon_vars = teacher_day_half_vars
            .get(&(teacher_id, day_key.clone(), HalfDay::Afternoon))
            .cloned()
            .unwrap_or_default();

        let morning_present = cross_builder
            .new_bool_var_with_name(format!("dayhalf_morning_{}_{}", teacher_id, day_key));
        let afternoon_present = cross_builder
            .new_bool_var_with_name(format!("dayhalf_afternoon_{}_{}", teacher_id, day_key));

        let morning_value = if morning_vars.is_empty() {
            cross_builder.add_eq(morning_present, 0);
            0
        } else {
            let morning_expr: LinearExpr = morning_vars.iter().copied().collect();
            cross_builder.add_ge(morning_expr.clone(), morning_present);
            cross_builder.add_le(
                morning_expr,
                ((morning_vars.len() as i64), morning_present),
            );
            if morning_vars
                .iter()
                .any(|var| var.solution_value(&pre_cross_response))
            {
                1
            } else {
                0
            }
        };
        cross_stage_hint_bools.push((morning_present, morning_value));

        let afternoon_value = if afternoon_vars.is_empty() {
            cross_builder.add_eq(afternoon_present, 0);
            0
        } else {
            let afternoon_expr: LinearExpr = afternoon_vars.iter().copied().collect();
            cross_builder.add_ge(afternoon_expr.clone(), afternoon_present);
            cross_builder.add_le(
                afternoon_expr,
                ((afternoon_vars.len() as i64), afternoon_present),
            );
            if afternoon_vars
                .iter()
                .any(|var| var.solution_value(&pre_cross_response))
            {
                1
            } else {
                0
            }
        };
        cross_stage_hint_bools.push((afternoon_present, afternoon_value));

        let cross_var =
            cross_builder.new_bool_var_with_name(format!("cross_{}_{}", teacher_id, day_key));
        cross_builder.add_le(cross_var, morning_present);
        cross_builder.add_le(cross_var, afternoon_present);
        cross_builder.add_ge(
            cross_var,
            LinearExpr::from(morning_present) + afternoon_present - 1,
        );
        cross_penalty_expr += cross_var;
        cross_stage_hint_bools.push((
            cross_var,
            if morning_value == 1 && afternoon_value == 1 {
                1
            } else {
                0
            },
        ));
    }

    let cross_proto = cross_builder.into_proto();
    let cross_response = solve_cp_sat_stage(
        &cross_proto,
        &fixed_objectives,
        cross_penalty_expr.clone(),
        cp_sat_stage_time_budget_ms("cross_half_day_penalty", CP_SAT_MAX_SOLVE_MS - elapsed_ms),
        Some(&pre_cross_response),
        &hint_bool_vars,
        &hint_int_vars,
        &cross_stage_hint_bools,
    );
    let final_elapsed_ms = started_at.elapsed().as_millis() as i64;
    let Ok(final_status) = cp_sat_response_kind(&cross_response, final_elapsed_ms) else {
        let fallback_reason = cp_sat_response_kind(&cross_response, final_elapsed_ms).err();
        let diagnostic_message = cp_sat_diagnostic_message(&cross_response).map(|detail| {
            format!(
                "CP-SAT 第 {} 阶段（尽量集中到同一晌）失败：{}",
                final_stage_number, detail
            )
        });
        let plan = build_cp_sat_plan_from_response(
            tasks,
            teachers,
            invigilation_config,
            &candidate_bindings,
            &unassigned_vars,
            &pre_cross_response,
            pre_cross_status,
            final_elapsed_ms,
        );
        return CpSatAttempt {
            plan: Some(plan),
            fallback_reason,
            diagnostic_message,
            solve_duration_ms: final_elapsed_ms,
        };
    };

    let plan = build_cp_sat_plan_from_response(
        tasks,
        teachers,
        invigilation_config,
        &candidate_bindings,
        &unassigned_vars,
        &cross_response,
        final_status,
        final_elapsed_ms,
    );
    CpSatAttempt {
        plan: Some(plan),
        fallback_reason: None,
        diagnostic_message: None,
        solve_duration_ms: final_elapsed_ms,
    }
}

fn persist_solved_plan(
    conn: &mut Connection,
    session_count: i64,
    teachers: &[TeacherInfo],
    plan: &SolvedPlan,
) -> Result<GenerateLatestExamStaffPlanResult, AppError> {
    let tx = conn.transaction()?;
    clear_latest_staff_plan(&tx)?;

    let teacher_by_id: HashMap<i64, &TeacherInfo> = teachers
        .iter()
        .map(|teacher| (teacher.id, teacher))
        .collect();
    let generated_at = Utc::now().to_rfc3339();

    for record in &plan.records {
        let status = if record.teacher_id.is_some() {
            TaskStatus::Assigned
        } else {
            TaskStatus::Unassigned
        };
        tx.execute(
            r#"
            INSERT INTO latest_exam_staff_tasks
            (session_id, space_id, task_source, role, grade_name, subject, space_name, floor, start_at, end_at, duration_minutes, recommended_self_study_topic_kind, recommended_self_study_topic_subjects_json, recommended_self_study_topic_label, priority_self_study_chain_json, assignment_tier, status, reason, allowance_amount)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            params![
                record.task.session_id,
                record.task.space_id,
                record.task.task_source.as_key(),
                record.task.role.as_key(),
                record.task.grade_name,
                record.task.subject.as_key(),
                record.task.space_name,
                record.task.floor,
                record.task.start_at,
                record.task.end_at,
                record.task.duration_minutes,
                record
                    .task
                    .recommended_self_study_topic
                    .as_ref()
                    .map(|topic| topic.kind.as_key().to_string()),
                record
                    .task
                    .recommended_self_study_topic
                    .as_ref()
                    .map(|topic| serde_json::to_string(&topic.subjects))
                    .transpose()
                    .map_err(|e| AppError::new(format!("推荐自习主题科目序列化失败: {e}")))?,
                record
                    .task
                    .recommended_self_study_topic
                    .as_ref()
                    .map(|topic| topic.label.clone()),
                self_study_topic_chain_to_text(&record.task.priority_self_study_chain)?,
                record.assignment_tier.map(|tier| tier.as_key().to_string()),
                status.as_key(),
                record.reason,
                record.allowance_amount
            ],
        )?;
        let task_id = tx.last_insert_rowid();
        if let Some(teacher_id) = record.teacher_id {
            if let Some(teacher) = teacher_by_id.get(&teacher_id) {
                tx.execute(
                    "INSERT INTO latest_exam_staff_assignments (task_id, teacher_id, teacher_name, assigned_at) VALUES (?1, ?2, ?3, ?4)",
                    params![task_id, teacher_id, teacher.name, generated_at],
                )?;
            }
        }
    }

    for teacher in teachers {
        let state = plan.runtime.get(&teacher.id).cloned().unwrap_or_default();
        tx.execute(
            "INSERT INTO latest_teacher_duty_stats (teacher_id, teacher_name, indoor_minutes, outdoor_minutes, total_minutes, task_count, exam_room_task_count, self_study_task_count, floor_rover_task_count, allowance_total, indoor_allowance_total, outdoor_allowance_total, is_middle_manager) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                teacher.id,
                teacher.name,
                state.indoor_minutes,
                state.outdoor_minutes,
                state.total_minutes,
                state.task_count,
                state.exam_room_task_count,
                state.self_study_task_count,
                state.floor_rover_task_count,
                round_to_two(state.allowance_total),
                round_to_two(state.indoor_allowance_total),
                round_to_two(state.outdoor_allowance_total),
                if teacher.is_middle_manager { 1_i64 } else { 0_i64 }
            ],
        )?;
    }

    tx.execute(
        "INSERT INTO latest_exam_staff_plan_meta (id, generated_at, session_count, task_count, assigned_count, unassigned_count, warning_count, imbalance_minutes, solver_engine, optimality_status, solve_duration_ms, fallback_reason, fallback_pool_assignments) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            generated_at,
            session_count,
            plan.records.len() as i64,
            plan.metrics.assigned_count,
            plan.metrics.unassigned_count,
            plan.metrics.warning_count,
            plan.metrics.imbalance_minutes,
            plan.solver_engine.as_key(),
            plan.optimality_status.as_key(),
            plan.solve_duration_ms,
            plan.fallback_reason.map(|reason| reason.as_key().to_string()),
            plan.metrics.fallback_pool_assignments
        ],
    )?;
    tx.commit()?;

    Ok(GenerateLatestExamStaffPlanResult {
        generated_at,
        task_count: plan.records.len() as i64,
        assigned_count: plan.metrics.assigned_count,
        unassigned_count: plan.metrics.unassigned_count,
        imbalance_minutes: plan.metrics.imbalance_minutes,
        warning_count: plan.metrics.warning_count,
        solver_engine: plan.solver_engine,
        optimality_status: plan.optimality_status,
        solve_duration_ms: plan.solve_duration_ms,
        fallback_reason: plan.fallback_reason,
        fallback_pool_assignments: plan.metrics.fallback_pool_assignments,
    })
}

fn format_metrics_for_log(metrics: &PlanMetrics) -> String {
    format!(
        "assigned={}, unassigned={}, fallback_pool={}, homeroom={}, total_gap={}, invigilation_gap={}, self_study_gap={}, cross_half_day={}",
        metrics.assigned_count,
        metrics.unassigned_count,
        metrics.fallback_pool_assignments,
        metrics.homeroom_assignments,
        metrics.imbalance_minutes,
        metrics.invigilation_minutes_gap,
        metrics.self_study_minutes_gap,
        metrics.cross_half_day_penalty,
    )
}

fn log_solver_outcome(
    log_path: Option<&Path>,
    cp_sat_attempt: &CpSatAttempt,
    final_plan: Option<&SolvedPlan>,
) {
    let Some(log_path) = log_path else {
        return;
    };

    let scope = "exam_staff.solve";
    let diagnostic = cp_sat_attempt
        .diagnostic_message
        .as_deref()
        .unwrap_or("无额外诊断信息");

    if let Some(final_plan) = final_plan {
        let level = match final_plan.fallback_reason {
            Some(FallbackReason::Error) => "warn",
            Some(FallbackReason::Timeout)
            | Some(FallbackReason::Unknown)
            | Some(FallbackReason::Infeasible) => "warn",
            _ => "info",
        };
        let reason = final_plan
            .fallback_reason
            .map(|item| item.as_key().to_string())
            .unwrap_or_else(|| "completed".to_string());
        let message = format!(
            "采用 CP-SAT 结果。reason={}, solve_duration_ms={}, final={}, detail={}",
            reason,
            final_plan.solve_duration_ms,
            format_metrics_for_log(&final_plan.metrics),
            diagnostic
        );
        let _ = app_log::append_log_to_path(log_path, level, scope, &message);
        return;
    }

    let level = match cp_sat_attempt.fallback_reason {
        Some(FallbackReason::Error) => "error",
        Some(FallbackReason::Timeout)
        | Some(FallbackReason::Unknown)
        | Some(FallbackReason::Infeasible) => "warn",
        _ => "error",
    };
    let reason = cp_sat_attempt
        .fallback_reason
        .map(|item| item.as_key().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let message = format!(
        "CP-SAT 未生成可用结果。reason={}, solve_duration_ms={}, detail={}",
        reason, cp_sat_attempt.solve_duration_ms, diagnostic
    );
    let _ = app_log::append_log_to_path(log_path, level, scope, &message);
}

fn generate_latest_exam_staff_plan_internal(
    conn: &mut Connection,
    invigilation_config: RuntimeInvigilationConfig,
    exclusion_pairs: HashSet<(i64, i64)>,
    log_path: Option<&Path>,
    progress: Option<&StaffAssignmentProgressReporter>,
) -> Result<GenerateLatestExamStaffPlanResult, AppError> {
    if let Some(progress) = progress {
        progress.emit_running(
            1,
            "load_session_times",
            "读取考试时间",
            format!(
                "正在执行第 1/{} 步：读取考试场次与时间模板。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let session_times = load_session_times_runtime(conn)?;
    if let Some(progress) = progress {
        progress.emit_running(
            2,
            "load_teacher_pool",
            "读取教师池",
            format!(
                "正在执行第 2/{} 步：读取教师信息与任教关系。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let teachers = load_teacher_pool(conn)?;
    if let Some(progress) = progress {
        progress.emit_running(
            3,
            "load_class_subject_map",
            "读取班级科目配置",
            format!(
                "正在执行第 3/{} 步：读取班级选科和自习科目配置。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let class_subject_map = load_class_subject_map(conn)?;
    if let Some(progress) = progress {
        progress.emit_running(
            4,
            "load_teaching_classes",
            "读取教学班",
            format!(
                "正在执行第 4/{} 步：读取教学班与场地信息。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let teaching_classes = load_teaching_classes(conn)?;
    if let Some(progress) = progress {
        progress.emit_running(
            5,
            "build_staff_tasks",
            "构建监考任务",
            format!(
                "正在执行第 5/{} 步：生成任务和候选老师池。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let tasks = build_staff_tasks(
        conn,
        &session_times,
        &invigilation_config,
        &class_subject_map,
        &teaching_classes,
    )?;

    let cp_sat_attempt = solve_with_cp_sat(
        &tasks,
        &teachers,
        &exclusion_pairs,
        &invigilation_config,
        progress,
    );

    let Some(mut final_plan) = cp_sat_attempt.plan.clone() else {
        log_solver_outcome(log_path, &cp_sat_attempt, None);
        return Err(AppError::new(
            cp_sat_attempt
                .diagnostic_message
                .clone()
                .unwrap_or_else(|| "CP-SAT 未生成可用结果".to_string()),
        ));
    };

    final_plan.solve_duration_ms = cp_sat_attempt.solve_duration_ms;
    final_plan.solver_engine = SolverEngine::CpSat;
    final_plan.fallback_reason = cp_sat_attempt.fallback_reason;
    if final_plan.fallback_reason.is_some() {
        final_plan.optimality_status = OptimalityStatus::Feasible;
    }

    log_solver_outcome(log_path, &cp_sat_attempt, Some(&final_plan));

    if let Some(progress) = progress {
        progress.emit_running(
            13,
            "persist_result",
            "写入分配结果",
            format!(
                "正在执行第 13/{} 步：保存分配结果。",
                STAFF_ASSIGNMENT_TOTAL_STEPS
            ),
        );
    }
    let result = persist_solved_plan(conn, session_times.len() as i64, &teachers, &final_plan)?;
    if let Some(progress) = progress {
        progress.emit_completed(format!(
            "监考分配完成：已分配 {} 项，未分配 {} 项。",
            result.assigned_count, result.unassigned_count
        ));
    }
    Ok(result)
}

pub fn list_exam_session_times(app: AppHandle) -> Result<Vec<ExamSessionTime>, String> {
    let result = (|| -> Result<Vec<ExamSessionTime>, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        load_session_time_template_rows(&conn)
    })();
    result.map_err(|error| error.to_string())
}

pub fn upsert_exam_session_times(
    app: AppHandle,
    items: Vec<ExamSessionTimeUpsert>,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        for item in items {
            let start_at = item.start_at.clone();
            let end_at = item.end_at.clone();
            let start_ts = parse_datetime_to_ts(&start_at)?;
            let end_ts = parse_datetime_to_ts(&end_at)?;
            duration_minutes(start_ts, end_ts)?;
            tx.execute(
                r#"
                INSERT INTO exam_subject_time_templates (subject, start_at, end_at, updated_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(subject) DO UPDATE SET
                    start_at = excluded.start_at,
                    end_at = excluded.end_at,
                    updated_at = excluded.updated_at
                "#,
                params![item.subject.as_key(), &start_at, &end_at, &now],
            )?;
            let session_exists = item.session_id > 0
                && tx
                    .query_row(
                        "SELECT 1 FROM latest_exam_plan_sessions WHERE id = ?1",
                        params![item.session_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .ok()
                    .is_some();
            if session_exists {
                tx.execute(
                    r#"
                    INSERT INTO exam_session_times (session_id, subject, start_at, end_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    ON CONFLICT(session_id) DO UPDATE SET
                        subject = excluded.subject,
                        start_at = excluded.start_at,
                        end_at = excluded.end_at,
                        updated_at = excluded.updated_at
                    "#,
                    params![item.session_id, item.subject.as_key(), &start_at, &end_at, &now],
                )?;
            }
            tx.execute(
                r#"
                UPDATE exam_session_times
                SET start_at = ?1, end_at = ?2, updated_at = ?3
                WHERE subject = ?4
                "#,
                params![&start_at, &end_at, &now, item.subject.as_key()],
            )?;
        }
        tx.commit()?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|error| error.to_string())
}

pub fn delete_exam_session_time(
    app: AppHandle,
    subject: Subject,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM exam_subject_time_templates WHERE subject = ?1",
            params![subject.as_key()],
        )?;
        tx.execute(
            "DELETE FROM exam_session_times WHERE subject = ?1",
            params![subject.as_key()],
        )?;
        tx.commit()?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|error| error.to_string())
}

pub fn generate_latest_exam_staff_plan(
    app: AppHandle,
    payload: GenerateExamStaffPlanPayload,
) -> Result<GenerateLatestExamStaffPlanResult, String> {
    let progress = StaffAssignmentProgressReporter::new(app.clone());
    let result = (|| -> Result<GenerateLatestExamStaffPlanResult, AppError> {
        let mut conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let log_path = app_log::log_path(&app).ok();
        let mut config = build_config_from_payload(&payload);
        hydrate_runtime_middle_manager_config(&conn, &mut config)?;
        config.self_study_class_subjects = load_self_study_class_subjects(&conn)?;
        let exclusion_pairs = payload
            .staff_exclusions
            .iter()
            .filter(|item| item.teacher_id > 0 && item.session_id > 0)
            .map(|item| (item.teacher_id, item.session_id))
            .collect::<HashSet<_>>();
        generate_latest_exam_staff_plan_internal(
            &mut conn,
            config,
            exclusion_pairs,
            log_path.as_deref(),
            Some(&progress),
        )
    })();
    match result {
        Ok(result) => Ok(result),
        Err(error) => {
            progress.emit_error("error", "分配失败", error.to_string());
            Err(error.to_string())
        }
    }
}

pub fn list_invigilation_exclusion_session_options(
    app: AppHandle,
) -> Result<Vec<InvigilationExclusionSessionOption>, String> {
    let result = (|| -> Result<Vec<InvigilationExclusionSessionOption>, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let rows = load_session_time_template_rows(&conn)?;
        let mut items = Vec::new();
        for row in rows {
            let start_at = row.start_at.clone().unwrap_or_default();
            let end_at = row.end_at.clone().unwrap_or_default();
            items.push(InvigilationExclusionSessionOption {
                session_id: row.session_id,
                grade_name: row.grade_name.clone(),
                subject: row.subject,
                start_at: start_at.clone(),
                end_at: end_at.clone(),
                label: format!(
                    "{} {} {}-{}",
                    subject_label(row.subject),
                    if start_at.len() >= 10 {
                        &start_at[5..10]
                    } else {
                        "--"
                    },
                    if start_at.len() >= 16 {
                        &start_at[11..16]
                    } else {
                        "--:--"
                    },
                    if end_at.len() >= 16 {
                        &end_at[11..16]
                    } else {
                        "--:--"
                    },
                ),
            });
        }
        Ok(items)
    })();
    result.map_err(|error| error.to_string())
}

fn default_persisted_invigilation_config() -> PersistedInvigilationConfig {
    PersistedInvigilationConfig {
        default_exam_room_required_count: 1,
        indoor_allowance_per_minute: 0.5,
        outdoor_allowance_per_minute: 0.3,
        middle_manager_default_enabled: false,
        middle_manager_exception_teacher_ids: Vec::new(),
        self_study_date: Utc::now().format("%Y-%m-%d").to_string(),
        self_study_start_time: "12:10".to_string(),
        self_study_end_time: "13:40".to_string(),
    }
}

pub fn get_persisted_invigilation_state(
    app: AppHandle,
) -> Result<PersistedInvigilationState, String> {
    let result = (|| -> Result<PersistedInvigilationState, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;

        let config = conn
            .query_row(
                r#"
                SELECT
                    default_exam_room_required_count,
                    indoor_allowance_per_minute,
                    outdoor_allowance_per_minute,
                    middle_manager_default_enabled,
                    middle_manager_exception_teacher_ids_json,
                    self_study_date,
                    self_study_start_time,
                    self_study_end_time
                FROM invigilation_config_settings
                WHERE id = 1
                "#,
                [],
                |row| {
                    let self_study_date = row
                        .get::<_, String>(5)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    let middle_manager_exception_teacher_ids = row
                        .get::<_, String>(4)
                        .ok()
                        .and_then(|text| serde_json::from_str::<Vec<i64>>(&text).ok())
                        .map(normalize_teacher_id_list)
                        .unwrap_or_default();
                    Ok(PersistedInvigilationConfig {
                        default_exam_room_required_count: row.get::<_, i64>(0)?.max(1),
                        indoor_allowance_per_minute: row.get::<_, f64>(1)?.max(0.0),
                        outdoor_allowance_per_minute: row.get::<_, f64>(2)?.max(0.0),
                        middle_manager_default_enabled: row.get::<_, i64>(3)? == 1,
                        middle_manager_exception_teacher_ids,
                        self_study_date: if self_study_date.is_empty() {
                            Utc::now().format("%Y-%m-%d").to_string()
                        } else {
                            self_study_date
                        },
                        self_study_start_time: row.get(6)?,
                        self_study_end_time: row.get(7)?,
                    })
                },
            )
            .unwrap_or_else(|_| default_persisted_invigilation_config());

        let self_study_class_subjects = conn
            .query_row(
                "SELECT self_study_class_subjects_json FROM invigilation_config_settings WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|text| serde_json::from_str::<Vec<PersistedSelfStudyClassSubject>>(&text).ok())
            .unwrap_or_default();

        let mut stmt = conn.prepare(
            r#"
            SELECT teacher_id, teacher_name, session_id, session_label
            FROM invigilation_staff_exclusions
            ORDER BY id DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PersistedExamStaffExclusion {
                teacher_id: row.get(0)?,
                teacher_name: row.get(1)?,
                session_id: row.get(2)?,
                session_label: row.get(3)?,
            })
        })?;
        let mut exclusions = Vec::new();
        for row in rows {
            exclusions.push(row?);
        }

        Ok(PersistedInvigilationState {
            config,
            exclusions,
            self_study_class_subjects,
        })
    })();
    result.map_err(|e| e.to_string())
}

pub fn save_persisted_invigilation_config(
    app: AppHandle,
    payload: PersistedInvigilationConfig,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let now = Utc::now().to_rfc3339();
        let middle_manager_exception_teacher_ids_json = serde_json::to_string(
            &normalize_teacher_id_list(payload.middle_manager_exception_teacher_ids.clone()),
        )
        .map_err(|e| AppError::new(format!("中层监考例外序列化失败: {e}")))?;
        conn.execute(
            r#"
            INSERT INTO invigilation_config_settings
            (id, default_exam_room_required_count, indoor_allowance_per_minute, outdoor_allowance_per_minute, middle_manager_default_enabled, middle_manager_exception_teacher_ids_json, self_study_date, self_study_start_time, self_study_end_time, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                default_exam_room_required_count = excluded.default_exam_room_required_count,
                indoor_allowance_per_minute = excluded.indoor_allowance_per_minute,
                outdoor_allowance_per_minute = excluded.outdoor_allowance_per_minute,
                middle_manager_default_enabled = excluded.middle_manager_default_enabled,
                middle_manager_exception_teacher_ids_json = excluded.middle_manager_exception_teacher_ids_json,
                self_study_date = excluded.self_study_date,
                self_study_start_time = excluded.self_study_start_time,
                self_study_end_time = excluded.self_study_end_time,
                updated_at = excluded.updated_at
            "#,
            params![
                1_i64,
                payload.default_exam_room_required_count.max(1),
                payload.indoor_allowance_per_minute.max(0.0),
                payload.outdoor_allowance_per_minute.max(0.0),
                if payload.middle_manager_default_enabled { 1_i64 } else { 0_i64 },
                middle_manager_exception_teacher_ids_json,
                payload.self_study_date.trim(),
                payload.self_study_start_time.trim(),
                payload.self_study_end_time.trim(),
                now
            ],
        )?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|e| e.to_string())
}

pub fn save_persisted_self_study_class_subjects(
    app: AppHandle,
    items: Vec<PersistedSelfStudyClassSubject>,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let now = Utc::now().to_rfc3339();
        let json_text = serde_json::to_string(&items)
            .map_err(|e| AppError::new(format!("自习科目配置序列化失败: {e}")))?;
        conn.execute(
            r#"
            INSERT INTO invigilation_config_settings
            (id, default_exam_room_required_count, indoor_allowance_per_minute, outdoor_allowance_per_minute, middle_manager_default_enabled, middle_manager_exception_teacher_ids_json, self_study_date, self_study_start_time, self_study_end_time, self_study_class_subjects_json, updated_at)
            VALUES (
                1,
                COALESCE((SELECT default_exam_room_required_count FROM invigilation_config_settings WHERE id = 1), 1),
                COALESCE((SELECT indoor_allowance_per_minute FROM invigilation_config_settings WHERE id = 1), 0.5),
                COALESCE((SELECT outdoor_allowance_per_minute FROM invigilation_config_settings WHERE id = 1), 0.3),
                COALESCE((SELECT middle_manager_default_enabled FROM invigilation_config_settings WHERE id = 1), 0),
                COALESCE((SELECT middle_manager_exception_teacher_ids_json FROM invigilation_config_settings WHERE id = 1), '[]'),
                COALESCE((SELECT self_study_date FROM invigilation_config_settings WHERE id = 1), ''),
                COALESCE((SELECT self_study_start_time FROM invigilation_config_settings WHERE id = 1), '12:10'),
                COALESCE((SELECT self_study_end_time FROM invigilation_config_settings WHERE id = 1), '13:40'),
                ?1,
                ?2
            )
            ON CONFLICT(id) DO UPDATE SET
                self_study_class_subjects_json = excluded.self_study_class_subjects_json,
                updated_at = excluded.updated_at
            "#,
            params![json_text, now],
        )?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|e| e.to_string())
}

pub fn replace_persisted_invigilation_exclusions(
    app: AppHandle,
    items: Vec<PersistedExamStaffExclusion>,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM invigilation_staff_exclusions", [])?;
        let now = Utc::now().to_rfc3339();
        for item in items {
            tx.execute(
                r#"
                INSERT INTO invigilation_staff_exclusions
                (teacher_id, teacher_name, session_id, session_label, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    item.teacher_id,
                    item.teacher_name.trim(),
                    item.session_id,
                    item.session_label.trim(),
                    now
                ],
            )?;
        }
        tx.commit()?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|e| e.to_string())
}

pub fn get_latest_exam_staff_plan_overview(
    app: AppHandle,
) -> Result<ExamStaffPlanOverview, String> {
    let result = (|| -> Result<ExamStaffPlanOverview, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let meta: Option<(
            String,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            String,
            String,
            i64,
            Option<String>,
            i64,
        )> = conn
            .query_row(
                "SELECT generated_at, session_count, task_count, assigned_count, unassigned_count, warning_count, imbalance_minutes, solver_engine, optimality_status, solve_duration_ms, fallback_reason, fallback_pool_assignments FROM latest_exam_staff_plan_meta WHERE id = 1",
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
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                    ))
                },
            )
            .ok();
        Ok(ExamStaffPlanOverview {
            generated_at: meta.as_ref().map(|value| value.0.clone()),
            session_count: meta.as_ref().map(|value| value.1).unwrap_or(0),
            task_count: meta.as_ref().map(|value| value.2).unwrap_or(0),
            assigned_count: meta.as_ref().map(|value| value.3).unwrap_or(0),
            unassigned_count: meta.as_ref().map(|value| value.4).unwrap_or(0),
            warning_count: meta.as_ref().map(|value| value.5).unwrap_or(0),
            imbalance_minutes: meta.as_ref().map(|value| value.6).unwrap_or(0),
            solver_engine: meta
                .as_ref()
                .and_then(|value| SolverEngine::from_key(&value.7))
                .unwrap_or(SolverEngine::CpSat),
            optimality_status: meta
                .as_ref()
                .and_then(|value| OptimalityStatus::from_key(&value.8))
                .unwrap_or(OptimalityStatus::Feasible),
            solve_duration_ms: meta.as_ref().map(|value| value.9).unwrap_or(0),
            fallback_reason: meta
                .as_ref()
                .and_then(|value| value.10.as_deref().and_then(FallbackReason::from_key)),
            fallback_pool_assignments: meta.as_ref().map(|value| value.11).unwrap_or(0),
        })
    })();
    result.map_err(|error| error.to_string())
}

pub fn list_latest_exam_staff_tasks(
    app: AppHandle,
    params: ListExamStaffTasksParams,
) -> Result<ListResult<ExamStaffTask>, String> {
    let result = (|| -> Result<ListResult<ExamStaffTask>, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let mut where_parts = Vec::new();
        let mut bind_values = Vec::<Value>::new();
        if let Some(session_id) = params.session_id {
            where_parts.push("t.session_id = ?".to_string());
            bind_values.push(Value::Integer(session_id));
        }
        if let Some(role) = params.role {
            where_parts.push("t.role = ?".to_string());
            bind_values.push(Value::Text(role.as_key().to_string()));
        }
        if let Some(status) = params.status {
            where_parts.push("t.status = ?".to_string());
            bind_values.push(Value::Text(status.as_key().to_string()));
        }
        let where_sql = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };
        let total_sql = format!("SELECT COUNT(*) FROM latest_exam_staff_tasks t{where_sql}");
        let total: i64 =
            conn.query_row(&total_sql, params_from_iter(bind_values.iter()), |row| {
                row.get(0)
            })?;

        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(200).clamp(1, 1000);
        let offset = (page - 1) * page_size;
        let mut query_values = bind_values;
        query_values.push(Value::Integer(page_size));
        query_values.push(Value::Integer(offset));

        let list_sql = format!(
            r#"
            SELECT
              t.id, t.session_id, t.space_id, t.task_source, t.role, t.grade_name, t.subject, t.space_name, t.floor,
              t.start_at, t.end_at, t.duration_minutes, t.recommended_self_study_topic_kind, t.recommended_self_study_topic_subjects_json, t.recommended_self_study_topic_label, t.priority_self_study_chain_json, t.assignment_tier, t.status, t.reason, t.allowance_amount,
              a.teacher_id, a.teacher_name
            FROM latest_exam_staff_tasks t
            LEFT JOIN latest_exam_staff_assignments a ON a.task_id = t.id
            {where_sql}
            ORDER BY t.start_at ASC, CASE WHEN t.session_id IS NULL THEN 1 ELSE 0 END ASC, t.session_id ASC, t.id ASC
            LIMIT ? OFFSET ?
            "#
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(query_values.iter()), |row| {
            let task_source_key: String = row.get(3)?;
            let role_key: String = row.get(4)?;
            let subject_key: String = row.get(6)?;
            let status_key: String = row.get(17)?;
            let task_source = StaffTaskSource::from_key(&task_source_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    3,
                    "task_source".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let role = StaffRole::from_key(&role_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "role".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let subject = Subject::from_key(&subject_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    6,
                    "subject".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let status = TaskStatus::from_key(&status_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    17,
                    "status".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let recommended_self_study_topic = self_study_topic_from_parts(
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
            )
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    12,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.to_string(),
                    )),
                )
            })?;
            let chain_text: String = row.get(15)?;
            let assignment_tier = row
                .get::<_, Option<String>>(16)?
                .as_deref()
                .and_then(AssignmentTier::from_key);
            Ok(ExamStaffTask {
                id: row.get(0)?,
                session_id: row.get(1)?,
                space_id: row.get(2)?,
                task_source,
                role,
                grade_name: row.get(5)?,
                subject,
                space_name: row.get(7)?,
                floor: row.get(8)?,
                start_at: row.get(9)?,
                end_at: row.get(10)?,
                duration_minutes: row.get(11)?,
                recommended_self_study_topic,
                priority_self_study_chain: self_study_topic_chain_from_text(&chain_text).map_err(
                    |e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            15,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                e.to_string(),
                            )),
                        )
                    },
                )?,
                assignment_tier,
                status,
                reason: row.get(18)?,
                allowance_amount: row.get(19)?,
                teacher_id: row.get(20)?,
                teacher_name: row.get(21)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(ListResult { items, total })
    })();
    result.map_err(|error| error.to_string())
}

pub fn list_latest_teacher_duty_stats(
    app: AppHandle,
    params: ListTeacherDutyStatsParams,
) -> Result<ListResult<TeacherDutyStat>, String> {
    let result = (|| -> Result<ListResult<TeacherDutyStat>, AppError> {
        let conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let mut where_parts = Vec::new();
        let mut bind_values = Vec::<Value>::new();
        if let Some(keyword) = params
            .keyword
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            where_parts.push("teacher_name LIKE ?".to_string());
            bind_values.push(Value::Text(format!("%{}%", keyword)));
        }
        let where_sql = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };
        let total_sql = format!("SELECT COUNT(*) FROM latest_teacher_duty_stats{where_sql}");
        let total: i64 =
            conn.query_row(&total_sql, params_from_iter(bind_values.iter()), |row| {
                row.get(0)
            })?;
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(200).clamp(1, 1000);
        let offset = (page - 1) * page_size;
        let mut query_values = bind_values;
        query_values.push(Value::Integer(page_size));
        query_values.push(Value::Integer(offset));
        let list_sql = format!(
            "SELECT teacher_id, teacher_name, indoor_minutes, outdoor_minutes, total_minutes, task_count, exam_room_task_count, self_study_task_count, floor_rover_task_count, allowance_total, indoor_allowance_total, outdoor_allowance_total, is_middle_manager FROM latest_teacher_duty_stats{where_sql} ORDER BY total_minutes ASC, teacher_id ASC LIMIT ? OFFSET ?"
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(query_values.iter()), |row| {
            Ok(TeacherDutyStat {
                teacher_id: row.get(0)?,
                teacher_name: row.get(1)?,
                indoor_minutes: row.get(2)?,
                outdoor_minutes: row.get(3)?,
                total_minutes: row.get(4)?,
                task_count: row.get(5)?,
                exam_room_task_count: row.get(6)?,
                self_study_task_count: row.get(7)?,
                floor_rover_task_count: row.get(8)?,
                allowance_total: row.get(9)?,
                indoor_allowance_total: row.get(10)?,
                outdoor_allowance_total: row.get(11)?,
                is_middle_manager: row.get::<_, i64>(12)? == 1,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(ListResult { items, total })
    })();
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic_subject(subject: Subject) -> exam_allocation::SelfStudyTopic {
        exam_allocation::build_subject_self_study_topic(subject)
    }

    fn test_runtime_config() -> RuntimeInvigilationConfig {
        RuntimeInvigilationConfig {
            default_exam_room_required_count: 1,
            indoor_allowance_per_minute: 0.5,
            outdoor_allowance_per_minute: 0.3,
            middle_manager_default_enabled: false,
            middle_manager_exception_teacher_ids: HashSet::new(),
            self_study_date: "2026-03-24".to_string(),
            self_study_start_time: "12:10".to_string(),
            self_study_end_time: "13:40".to_string(),
            self_study_class_subjects: HashMap::new(),
        }
    }

    fn sample_exam_task(subject: Subject) -> TaskBuild {
        TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_self_study_topic: None,
            priority_self_study_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        }
    }

    fn sample_self_study_task(task_source: StaffTaskSource) -> TaskBuild {
        TaskBuild {
            session_id: if task_source == StaffTaskSource::FullSelfStudy {
                None
            } else {
                Some(1)
            },
            space_id: if task_source == StaffTaskSource::FullSelfStudy {
                None
            } else {
                Some(1)
            },
            task_source,
            role: StaffRole::SelfStudySupervisor,
            grade_name: "高二".to_string(),
            subject: Subject::Biology,
            space_name: "高二3班".to_string(),
            floor: "4层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_self_study_topic: Some(topic_subject(Subject::Physics)),
            priority_self_study_chain: vec![
                topic_subject(Subject::Physics),
                topic_subject(Subject::English),
            ],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        }
    }

    #[test]
    fn test_candidate_summary_exam_room_subject_conflict() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "语文老师".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let summary = build_task_candidate_summary(
            &sample_exam_task(Subject::Math),
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert_eq!(
            summary.candidates.iter().map(|item| item.teacher_id).collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn test_candidate_summary_self_study_tier_order() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "物理老师".to_string(),
                subjects: HashSet::from([Subject::Physics]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "班主任".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::from(["高二3班".to_string()]),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "通用老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let summary = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy),
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert_eq!(
            summary
                .candidates
                .iter()
                .map(|item| (item.teacher_id, item.assignment_tier))
                .collect::<Vec<_>>(),
            vec![
                (1, Some(AssignmentTier::Primary)),
                (2, Some(AssignmentTier::Homeroom)),
                (3, Some(AssignmentTier::FallbackPool)),
            ]
        );
    }

    #[test]
    fn test_candidate_summary_respects_middle_manager_and_full_self_study_rules() {
        let middle_manager = vec![TeacherInfo {
            id: 1,
            name: "中层老师".to_string(),
            subjects: HashSet::from([Subject::Physics]),
            class_names: HashSet::from(["高二3班".to_string()]),
            homeroom_classes: HashSet::from(["高二3班".to_string()]),
            is_middle_manager: true,
        }];
        let summary = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy),
            &middle_manager,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert!(summary.candidates.is_empty());

        let mut config = test_runtime_config();
        config.middle_manager_exception_teacher_ids = HashSet::from([1_i64]);
        let enabled = build_task_candidate_summary(
            &sample_exam_task(Subject::Math),
            &middle_manager,
            &HashSet::new(),
            &config,
        );
        assert_eq!(enabled.candidates.len(), 1);

        config.middle_manager_default_enabled = true;
        let disabled_again = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::FullSelfStudy),
            &middle_manager,
            &HashSet::new(),
            &config,
        );
        assert!(disabled_again.candidates.is_empty());
    }

    #[test]
    fn test_candidate_summary_uses_fallback_pool_and_full_self_study_ignores_exam_exclusion() {
        let teachers = vec![TeacherInfo {
            id: 9,
            name: "通用老师".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::new(),
            homeroom_classes: HashSet::new(),
            is_middle_manager: false,
        }];
        let exam_linked = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy),
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert_eq!(
            exam_linked.candidates[0].assignment_tier,
            Some(AssignmentTier::FallbackPool)
        );

        let full_self_study = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::FullSelfStudy),
            &teachers,
            &HashSet::from([(9_i64, 99_i64)]),
            &test_runtime_config(),
        );
        assert_eq!(full_self_study.candidates.len(), 1);
        assert_eq!(full_self_study.candidates[0].teacher_id, 9);
    }

    #[test]
    fn test_candidate_summary_supports_foreign_group_and_free_study_topics() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "俄语老师".to_string(),
                subjects: HashSet::from([Subject::Russian]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let mut foreign_task = sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy);
        foreign_task.recommended_self_study_topic = Some(
            exam_allocation::build_foreign_group_self_study_topic(vec![
                Subject::English,
                Subject::Russian,
            ]),
        );
        foreign_task.priority_self_study_chain = vec![
            exam_allocation::build_foreign_group_self_study_topic(vec![
                Subject::English,
                Subject::Russian,
            ]),
        ];
        let foreign_summary = build_task_candidate_summary(
            &foreign_task,
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert_eq!(
            foreign_summary
                .candidates
                .iter()
                .take(2)
                .map(|item| (item.teacher_id, item.assignment_tier))
                .collect::<Vec<_>>(),
            vec![
                (1, Some(AssignmentTier::Primary)),
                (2, Some(AssignmentTier::Primary)),
            ]
        );

        let mut free_task = sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy);
        free_task.recommended_self_study_topic = Some(exam_allocation::build_free_study_topic());
        free_task.priority_self_study_chain = vec![exam_allocation::build_free_study_topic()];
        let free_summary = build_task_candidate_summary(
            &free_task,
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
        );
        assert_eq!(free_summary.candidates[0].teacher_id, 1);
        assert_eq!(free_summary.candidates[1].teacher_id, 2);
        assert_eq!(free_summary.candidates[2].teacher_id, 3);
        assert!(free_summary
            .candidates
            .iter()
            .all(|item| item.assignment_tier == Some(AssignmentTier::Primary)));
    }

    #[test]
    fn test_allowance_rate_mapping() {
        let config = test_runtime_config();
        assert_eq!(
            allowance_rate_for_role(&config, StaffRole::ExamRoomInvigilator),
            0.5
        );
        assert_eq!(
            allowance_rate_for_role(&config, StaffRole::SelfStudySupervisor),
            0.5
        );
        assert_eq!(allowance_rate_for_role(&config, StaffRole::FloorRover), 0.3);
        assert_eq!(round_to_two(36.666), 36.67);
    }

    #[test]
    fn test_cp_sat_reduces_fallback_pool_in_direct_mode() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "通用老师".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let tasks = vec![
            TaskBuild {
                session_id: Some(1),
                space_id: Some(1),
                task_source: StaffTaskSource::Exam,
                role: StaffRole::ExamRoomInvigilator,
                grade_name: "高二".to_string(),
                subject: Subject::Math,
                space_name: "高二1场".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
                duration_minutes: 120,
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
            TaskBuild {
                session_id: Some(1),
                space_id: Some(2),
                task_source: StaffTaskSource::ExamLinkedSelfStudy,
                role: StaffRole::SelfStudySupervisor,
                grade_name: "高二".to_string(),
                subject: Subject::Biology,
                space_name: "高二1班".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
                duration_minutes: 120,
                recommended_self_study_topic: Some(topic_subject(Subject::English)),
                priority_self_study_chain: vec![topic_subject(Subject::English)],
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
        ];
        let cp_sat_attempt = solve_with_cp_sat(
            &tasks,
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
            None,
        );
        let cp_sat_plan = cp_sat_attempt.plan.expect("cp-sat should produce a plan");
        assert_eq!(cp_sat_plan.metrics.unassigned_count, 0);
        assert_eq!(cp_sat_plan.metrics.fallback_pool_assignments, 0);
        assert_eq!(cp_sat_plan.solver_engine, SolverEngine::CpSat);
    }

    #[test]
    fn test_cp_sat_balances_total_and_task_type_minutes() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "老师甲".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "老师乙".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let tasks = vec![
            TaskBuild {
                session_id: Some(1),
                space_id: Some(1),
                task_source: StaffTaskSource::Exam,
                role: StaffRole::ExamRoomInvigilator,
                grade_name: "高二".to_string(),
                subject: Subject::Math,
                space_name: "高二1场".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T09:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
                duration_minutes: 60,
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
            TaskBuild {
                session_id: Some(2),
                space_id: Some(2),
                task_source: StaffTaskSource::ExamLinkedSelfStudy,
                role: StaffRole::SelfStudySupervisor,
                grade_name: "高二".to_string(),
                subject: Subject::Biology,
                space_name: "高二1班".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T09:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 2_000,
                end_ts: 3_000,
                duration_minutes: 60,
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
            TaskBuild {
                session_id: Some(3),
                space_id: Some(3),
                task_source: StaffTaskSource::Exam,
                role: StaffRole::FloorRover,
                grade_name: "高二".to_string(),
                subject: Subject::Physics,
                space_name: "4层 楼层流动".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T10:00".to_string(),
                end_at: "2026-03-24T11:00".to_string(),
                start_ts: 3_000,
                end_ts: 4_000,
                duration_minutes: 60,
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
            TaskBuild {
                session_id: Some(4),
                space_id: Some(4),
                task_source: StaffTaskSource::FullSelfStudy,
                role: StaffRole::SelfStudySupervisor,
                grade_name: "高二".to_string(),
                subject: Subject::English,
                space_name: "高二2班".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T11:00".to_string(),
                end_at: "2026-03-24T12:00".to_string(),
                start_ts: 4_000,
                end_ts: 5_000,
                duration_minutes: 60,
                recommended_self_study_topic: Some(topic_subject(Subject::English)),
                priority_self_study_chain: vec![topic_subject(Subject::English)],
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
            },
        ];

        let cp_sat_attempt = solve_with_cp_sat(
            &tasks,
            &teachers,
            &HashSet::new(),
            &test_runtime_config(),
            None,
        );
        let cp_sat_plan = cp_sat_attempt.plan.expect("cp-sat should produce a plan");

        assert_eq!(cp_sat_plan.metrics.unassigned_count, 0);
        assert_eq!(cp_sat_plan.metrics.imbalance_minutes, 0);
        assert_eq!(cp_sat_plan.metrics.invigilation_minutes_gap, 0);
        assert_eq!(cp_sat_plan.metrics.self_study_minutes_gap, 0);
    }




    #[test]
    #[ignore = "manual integration test against the real sqlite database"]
    fn test_run_real_db_staff_plan_manual() {
        let db_path = std::env::var("ACADEMIC_REAL_DB_PATH")
            .expect("ACADEMIC_REAL_DB_PATH must point to scores.sqlite3");
        let db_path = std::path::PathBuf::from(db_path);
        let mut conn = Connection::open(&db_path).expect("open real sqlite db");
        crate::schema::ensure_schema(&conn).expect("ensure schema");

        let persisted_settings: (i64, f64, f64) = conn
            .query_row(
                "SELECT default_exam_room_required_count, indoor_allowance_per_minute, outdoor_allowance_per_minute FROM invigilation_config_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((1, 0.5, 0.3));
        let mut config = build_config_from_payload(&GenerateExamStaffPlanPayload {
            default_exam_room_required_count: persisted_settings.0,
            indoor_allowance_per_minute: persisted_settings.1,
            outdoor_allowance_per_minute: persisted_settings.2,
            staff_exclusions: Vec::new(),
        });
        hydrate_runtime_middle_manager_config(&conn, &mut config).expect("hydrate config");
        config.self_study_class_subjects =
            load_self_study_class_subjects(&conn).expect("load self study subjects");

        let mut exclusion_pairs = HashSet::new();
        let mut stmt = conn
            .prepare("SELECT teacher_id, session_id FROM invigilation_staff_exclusions")
            .expect("prepare exclusions");
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .expect("query exclusions");
        for row in rows {
            let (teacher_id, session_id) = row.expect("read exclusion row");
            if teacher_id > 0 && session_id > 0 {
                exclusion_pairs.insert((teacher_id, session_id));
            }
        }
        drop(stmt);

        let log_path = db_path
            .parent()
            .expect("db parent")
            .join("logs")
            .join("app.log");
        let result = generate_latest_exam_staff_plan_internal(
            &mut conn,
            config,
            exclusion_pairs,
            Some(log_path.as_path()),
            None,
        )
        .expect("generate staff plan on real db");

        let mut reason_stmt = conn
            .prepare(
                "SELECT COALESCE(reason, '<empty>') AS reason, COUNT(*) FROM latest_exam_staff_tasks WHERE status = 'unassigned' GROUP BY COALESCE(reason, '<empty>') ORDER BY COUNT(*) DESC, reason ASC",
            )
            .expect("prepare reason query");
        let reason_rows = reason_stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .expect("query reason rows");
        let mut reason_counts = Vec::<(String, i64)>::new();
        for row in reason_rows {
            reason_counts.push(row.expect("read reason row"));
        }

        println!(
            "REAL_DB_STAFF_PLAN {}",
            serde_json::to_string(&result).expect("serialize result")
        );
        println!(
            "REAL_DB_UNASSIGNED_REASONS {}",
            serde_json::to_string(&reason_counts).expect("serialize reason counts")
        );
        println!("REAL_DB_APP_LOG {}", log_path.display());
        assert!(result.task_count > 0);
    }
}
