use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use calamine::{open_workbook_auto, Data, Reader};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use cp_sat::builder::{BoolVar, CpModelBuilder, IntVar, LinearExpr};
use cp_sat::proto::{CpSolverResponse, CpSolverStatus, SatParameters};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::app_log;
use crate::db::repos::exam_staff as exam_staff_repo;
use crate::entity::invigilation_config_settings;
use crate::exam_allocation::{self, SuccessResponse};
use crate::score::{AppError, ListResult, Subject};

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
    source_grade_name: Option<String>,
    is_inherited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSessionTimeUpsert {
    pub session_id: i64,
    pub grade_name: String,
    pub subject: Subject,
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExamSessionTimesParams {
    pub grade_name: Option<String>,
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
    unassigned_details: Vec<String>,
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
    pub custom_rules: Vec<GenerateExamStaffPlanCustomRule>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateExamStaffPlanCustomRule {
    pub action_type: String,
    pub teacher_id: i64,
    pub teacher_name: Option<String>,
    pub time_scope_type: String,
    pub time_scope_ids: Vec<i64>,
    pub time_scope_labels: Vec<String>,
    pub task_scope_type: String,
    pub target_scope_type: String,
    pub target_ids: Vec<String>,
    pub target_labels: Vec<String>,
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
pub struct PersistedInvigilationCustomRule {
    pub action_type: String,
    pub teacher_id: i64,
    pub teacher_name: String,
    pub time_scope_type: String,
    pub time_scope_ids: Vec<i64>,
    pub time_scope_labels: Vec<String>,
    pub task_scope_type: String,
    pub target_scope_type: String,
    pub target_ids: Vec<String>,
    pub target_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvigilationRuleTimeScopeOption {
    id: i64,
    label: String,
    start_at: String,
    end_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvigilationRuleFullSelfStudyOption {
    label: String,
    start_at: String,
    end_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvigilationRuleTargetOption {
    id: String,
    label: String,
    subtitle: Option<String>,
    time_scope_type: String,
    time_scope_id: Option<i64>,
    task_scope_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvigilationRuleOptions {
    exam_session_options: Vec<InvigilationRuleTimeScopeOption>,
    full_self_study_option: Option<InvigilationRuleFullSelfStudyOption>,
    target_options: Vec<InvigilationRuleTargetOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedSelfStudyClassSubject {
    class_id: i64,
    subject: Option<Subject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDrawImportRow {
    group_no: String,
    invigilator_a_name: String,
    invigilator_b_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDrawImportResult {
    imported_at: String,
    row_count: i64,
    duration_ms: i64,
    rows: Vec<MonitorDrawImportRow>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedInvigilationState {
    config: PersistedInvigilationConfig,
    custom_rules: Vec<PersistedInvigilationCustomRule>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloorRoverSlotKey {
    grade_name: String,
    start_ts: i64,
    end_ts: i64,
    subject_group_key: String,
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
    // 楼层流动在外语分组场景下会把同一时间段的多门外语合并成一条任务。
    // 这里保留整组需要回避的科目，保证英语/日语/俄语老师都会被排除，
    // 不会因为只生成了一条楼层流动任务就漏掉科目回避。
    subject_avoidance_subjects: Vec<Subject>,
    recommended_self_study_topic: Option<exam_allocation::SelfStudyTopic>,
    priority_self_study_chain: Vec<exam_allocation::SelfStudyTopic>,
    day_key: String,
    half_day: HalfDay,
    rule_target_id: String,
}

const RULE_ACTION_EXCLUDE: &str = "exclude";
const RULE_ACTION_REQUIRE: &str = "require";
const RULE_TIME_SCOPE_EXAM_SESSION: &str = "exam_session";
const RULE_TIME_SCOPE_FULL_SELF_STUDY: &str = "full_self_study";
const RULE_TASK_SCOPE_EXAM_ROOM: &str = "exam_room";
const RULE_TASK_SCOPE_EXAM_LINKED_SELF_STUDY: &str = "exam_linked_self_study";
const RULE_TASK_SCOPE_FULL_SELF_STUDY: &str = "full_self_study";
const RULE_TASK_SCOPE_FLOOR_ROVER: &str = "floor_rover";
const RULE_TARGET_SCOPE_ALL: &str = "all";
const RULE_TARGET_SCOPE_SELECTED: &str = "selected_targets";

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

fn sorted_grade_names(mut grades: Vec<String>) -> Vec<String> {
    grades.sort_by(|a, b| {
        exam_allocation::grade_order_key(a)
            .cmp(&exam_allocation::grade_order_key(b))
            .then(a.cmp(b))
    });
    grades.dedup();
    grades
}

async fn load_effective_session_time_grade_options(
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<String>, AppError> {
    let mut grades = exam_staff_repo::load_teaching_class_rows(db)
        .await?
        .into_iter()
        .map(|row| row.grade_name)
        .collect::<Vec<_>>();
    for grade in exam_staff_repo::list_template_grades(db).await? {
        grades.push(grade);
    }
    Ok(sorted_grade_names(grades))
}

async fn load_grade_subject_template_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<HashMap<String, HashMap<Subject, (String, String)>>, AppError> {
    let mut out = HashMap::<String, HashMap<Subject, (String, String)>>::new();
    for row in exam_staff_repo::list_grade_subject_templates(db).await? {
        let Some(subject) = Subject::from_key(&row.subject) else {
            continue;
        };
        out.entry(row.grade_name)
            .or_default()
            .insert(subject, (row.start_at, row.end_at));
    }
    Ok(out)
}

fn resolve_effective_grade_subject_template(
    grade_name: &str,
    subject: Subject,
    grade_templates: &HashMap<String, HashMap<Subject, (String, String)>>,
) -> Option<(String, String, Option<String>, bool)> {
    if let Some((start_at, end_at)) = grade_templates
        .get(grade_name)
        .and_then(|map| map.get(&subject))
        .cloned()
    {
        return Some((start_at, end_at, Some(grade_name.to_string()), false));
    }
    if matches!(subject, Subject::Russian | Subject::Japanese) {
        if let Some((start_at, end_at)) = grade_templates
            .get(grade_name)
            .and_then(|map| map.get(&Subject::English))
            .cloned()
        {
            return Some((start_at, end_at, Some(grade_name.to_string()), true));
        }
    }

    None
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

fn role_label(role: StaffRole) -> &'static str {
    match role {
        StaffRole::ExamRoomInvigilator => "考场监考",
        StaffRole::SelfStudySupervisor => "自习看管",
        StaffRole::FloorRover => "楼层流动",
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

async fn hydrate_runtime_middle_manager_config(
    db: &sea_orm::DatabaseConnection,
    config: &mut RuntimeInvigilationConfig,
) -> Result<(), AppError> {
    if let Some(persisted) = exam_staff_repo::get_config(db).await? {
        config.middle_manager_default_enabled = persisted.middle_manager_default_enabled == 1;
        config.middle_manager_exception_teacher_ids =
            serde_json::from_str::<Vec<i64>>(&persisted.middle_manager_exception_teacher_ids_json)
                .map(normalize_teacher_id_list)
                .unwrap_or_default()
                .into_iter()
                .collect();
        config.self_study_date = persisted.self_study_date.trim().to_string();
        config.self_study_start_time = persisted.self_study_start_time.trim().to_string();
        config.self_study_end_time = persisted.self_study_end_time.trim().to_string();
    }
    Ok(())
}

async fn load_runtime_invigilation_config(
    db: &sea_orm::DatabaseConnection,
) -> Result<RuntimeInvigilationConfig, AppError> {
    let mut config = RuntimeInvigilationConfig {
        default_exam_room_required_count: 1,
        indoor_allowance_per_minute: 0.5,
        outdoor_allowance_per_minute: 0.3,
        middle_manager_default_enabled: false,
        middle_manager_exception_teacher_ids: HashSet::new(),
        self_study_date: String::new(),
        self_study_start_time: "12:10".to_string(),
        self_study_end_time: "13:40".to_string(),
        self_study_class_subjects: load_self_study_class_subjects(db).await?,
    };
    hydrate_runtime_middle_manager_config(db, &mut config).await?;
    Ok(config)
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

async fn load_session_time_template_rows(
    db: &sea_orm::DatabaseConnection,
    selected_grade_name: &str,
) -> Result<Vec<ExamSessionTime>, AppError> {
    let grade_templates = load_grade_subject_template_map(db).await?;
    let mut out = Vec::<ExamSessionTime>::new();
    for subject in [
        Subject::Chinese,
        Subject::Math,
        Subject::English,
        Subject::Physics,
        Subject::Chemistry,
        Subject::Biology,
        Subject::Politics,
        Subject::History,
        Subject::Geography,
        Subject::Russian,
        Subject::Japanese,
    ] {
        let resolved = resolve_effective_grade_subject_template(
            selected_grade_name,
            subject,
            &grade_templates,
        );
        let (start_at, end_at, source_grade_name, is_inherited) = match resolved {
            Some((start_at, end_at, source_grade_name, is_inherited)) => (
                Some(start_at),
                Some(end_at),
                source_grade_name,
                is_inherited,
            ),
            None => (None, None, None, false),
        };
        out.push(ExamSessionTime {
            session_id: template_session_id(subject),
            grade_name: selected_grade_name.to_string(),
            subject,
            start_at,
            end_at,
            source_grade_name,
            is_inherited,
        });
    }
    out.sort_by(|a, b| {
        let a_ts = a
            .start_at
            .as_ref()
            .and_then(|s| parse_datetime_to_ts(s).ok())
            .unwrap_or(i64::MAX);
        let b_ts = b
            .start_at
            .as_ref()
            .and_then(|s| parse_datetime_to_ts(s).ok())
            .unwrap_or(i64::MAX);
        a_ts.cmp(&b_ts)
            .then(subject_order(a.subject).cmp(&subject_order(b.subject)))
    });
    Ok(out)
}

async fn load_session_times_runtime(
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<SessionTimeRuntime>, AppError> {
    load_session_times_runtime_with_policy(db, false).await
}

async fn load_configured_session_times_runtime(
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<SessionTimeRuntime>, AppError> {
    load_session_times_runtime_with_policy(db, true).await
}

async fn load_session_times_runtime_with_policy(
    db: &sea_orm::DatabaseConnection,
    skip_missing_time: bool,
) -> Result<Vec<SessionTimeRuntime>, AppError> {
    let grade_templates = load_grade_subject_template_map(db).await?;
    let mut out = Vec::new();
    for row in exam_staff_repo::list_session_time_rows(db).await? {
        let subject = Subject::from_key(&row.subject)
            .ok_or_else(|| AppError::new(format!("无效的科目: {}", row.subject)))?;
        let resolved_time = match (row.start_at.clone(), row.end_at.clone()) {
            (Some(start_at), Some(end_at)) => (start_at, end_at),
            _ => {
                let Some((start_at, end_at, _, _)) = resolve_effective_grade_subject_template(
                    &row.grade_name,
                    subject,
                    &grade_templates,
                ) else {
                    if skip_missing_time {
                        continue;
                    }
                    return Err(AppError::new(format!(
                        "场次 {} 未配置开始或结束时间",
                        row.session_id
                    )));
                };
                (start_at, end_at)
            }
        };
        let start_at = resolved_time.0;
        let end_at = resolved_time.1;
        let start_ts = parse_datetime_to_ts(&start_at)?;
        let end_ts = parse_datetime_to_ts(&end_at)?;
        duration_minutes(start_ts, end_ts)?;
        out.push(SessionTimeRuntime {
            session_id: row.session_id,
            grade_name: row.grade_name,
            subject,
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

async fn load_teacher_pool(db: &sea_orm::DatabaseConnection) -> Result<Vec<TeacherInfo>, AppError> {
    let mut map: HashMap<i64, TeacherInfo> = HashMap::new();

    for row in exam_staff_repo::load_teacher_rows(db).await? {
        map.insert(
            row.id,
            TeacherInfo {
                id: row.id,
                name: row.name,
                subjects: HashSet::new(),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: row.is_middle_manager,
            },
        );
    }

    for row in exam_staff_repo::load_teacher_assignment_rows(db).await? {
        if let Some(entry) = map.get_mut(&row.teacher_id) {
            if let Some(subject) = Subject::from_key(&row.subject) {
                entry.subjects.insert(subject);
            }
            entry.class_names.insert(row.class_name);
        }
    }

    for row in exam_staff_repo::load_teacher_homeroom_rows(db).await? {
        if let Some(entry) = map.get_mut(&row.teacher_id) {
            entry.homeroom_classes.insert(row.class_name);
        }
    }

    let mut teachers: Vec<TeacherInfo> = map.into_values().collect();
    teachers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(teachers)
}

fn infer_grade_name_from_class_name(class_name: &str) -> Option<String> {
    let trimmed = class_name.trim();
    if trimmed.starts_with("高一") {
        return Some("高一".to_string());
    }
    if trimmed.starts_with("高二") {
        return Some("高二".to_string());
    }
    if trimmed.starts_with("高三") {
        return Some("高三".to_string());
    }
    None
}

async fn load_teacher_grade_subject_pairs(
    db: &sea_orm::DatabaseConnection,
) -> Result<HashMap<i64, HashSet<(String, Subject)>>, AppError> {
    let mut out = HashMap::<i64, HashSet<(String, Subject)>>::new();
    for row in exam_staff_repo::load_teacher_grade_subject_rows(db).await? {
        let Some(subject) = Subject::from_key(&row.subject) else {
            continue;
        };
        let Some(grade_name) = row
            .grade_name
            .and_then(|value| {
                let trimmed = value.trim().to_string();
                (!trimmed.is_empty()).then_some(trimmed)
            })
            .or_else(|| infer_grade_name_from_class_name(&row.class_name))
        else {
            continue;
        };
        out.entry(row.teacher_id)
            .or_default()
            .insert((grade_name, subject));
    }
    Ok(out)
}

async fn load_class_subject_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<HashMap<(String, String), HashSet<Subject>>, AppError> {
    let mut map: HashMap<(String, String), HashSet<Subject>> = HashMap::new();
    for row in exam_staff_repo::load_class_subject_rows(db).await? {
        if let Some(subject) = Subject::from_key(&row.subject) {
            map.entry((row.grade_name, row.class_name))
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

async fn load_self_study_class_subjects(
    db: &sea_orm::DatabaseConnection,
) -> Result<HashMap<i64, Subject>, AppError> {
    let json_text = exam_staff_repo::get_config(db)
        .await?
        .map(|row| row.self_study_class_subjects_json)
        .unwrap_or_else(|| "[]".to_string());
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

async fn load_teaching_classes(
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<TeachingClassRuntime>, AppError> {
    Ok(exam_staff_repo::load_teaching_class_rows(db)
        .await?
        .into_iter()
        .map(|row| TeachingClassRuntime {
            id: row.id,
            grade_name: row.grade_name,
            class_name: row.class_name,
            floor: row.floor,
        })
        .collect())
}

fn teaching_classes_for_sessions<'a>(
    teaching_classes: &'a [TeachingClassRuntime],
    session_times: &[SessionTimeRuntime],
) -> Vec<&'a TeachingClassRuntime> {
    let active_grades = session_times
        .iter()
        .map(|session| session.grade_name.as_str())
        .collect::<HashSet<_>>();
    teaching_classes
        .iter()
        .filter(|teaching_class| active_grades.contains(teaching_class.grade_name.as_str()))
        .collect()
}

fn load_exam_room_requirement(default_count: i64) -> Result<i64, AppError> {
    Ok(default_count.max(1))
}

fn rule_task_scope_for_task(task: &TaskBuild) -> &'static str {
    match (task.task_source, task.role) {
        (StaffTaskSource::Exam, StaffRole::ExamRoomInvigilator) => RULE_TASK_SCOPE_EXAM_ROOM,
        (StaffTaskSource::ExamLinkedSelfStudy, StaffRole::SelfStudySupervisor) => {
            RULE_TASK_SCOPE_EXAM_LINKED_SELF_STUDY
        }
        (StaffTaskSource::FullSelfStudy, StaffRole::SelfStudySupervisor) => {
            RULE_TASK_SCOPE_FULL_SELF_STUDY
        }
        (_, StaffRole::FloorRover) => RULE_TASK_SCOPE_FLOOR_ROVER,
        _ => RULE_TASK_SCOPE_EXAM_ROOM,
    }
}

fn task_matches_custom_rule(task: &TaskBuild, rule: &GenerateExamStaffPlanCustomRule) -> bool {
    if rule_task_scope_for_task(task) != rule.task_scope_type {
        return false;
    }
    match rule.time_scope_type.as_str() {
        RULE_TIME_SCOPE_EXAM_SESSION => {
            let Some(session_id) = task.session_id else {
                return false;
            };
            if !rule.time_scope_ids.iter().any(|id| *id == session_id) {
                return false;
            }
        }
        RULE_TIME_SCOPE_FULL_SELF_STUDY => {
            if task.task_source != StaffTaskSource::FullSelfStudy {
                return false;
            }
        }
        _ => return false,
    }
    match rule.target_scope_type.as_str() {
        RULE_TARGET_SCOPE_ALL => true,
        RULE_TARGET_SCOPE_SELECTED => rule.target_ids.iter().any(|id| id == &task.rule_target_id),
        _ => false,
    }
}

fn parse_json_i64_list(text: &str) -> Vec<i64> {
    serde_json::from_str::<Vec<i64>>(text).unwrap_or_default()
}

fn parse_json_string_list(text: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(text).unwrap_or_default()
}

fn to_json_i64_list(values: &[i64]) -> Result<String, AppError> {
    serde_json::to_string(values)
        .map_err(|error| AppError::new(format!("排班规则时段序列化失败: {error}")))
}

fn to_json_string_list(values: &[String], label: &str) -> Result<String, AppError> {
    serde_json::to_string(values)
        .map_err(|error| AppError::new(format!("排班规则{label}序列化失败: {error}")))
}

fn build_task_candidate_summary(
    task: &TaskBuild,
    teachers: &[TeacherInfo],
    custom_rules: &[GenerateExamStaffPlanCustomRule],
    config: &RuntimeInvigilationConfig,
    slot_forbidden_grade_subjects: &HashSet<(String, Subject)>,
    teacher_grade_subject_pairs: &HashMap<i64, HashSet<(String, Subject)>>,
) -> TaskCandidateSummary {
    let required_teacher_ids: HashSet<i64> = teachers
        .iter()
        .filter(|teacher| {
            custom_rules.iter().any(|rule| {
                rule.action_type == RULE_ACTION_REQUIRE
                    && rule.teacher_id == teacher.id
                    && task_matches_custom_rule(task, rule)
            })
        })
        .map(|t| t.id)
        .collect();

    let active_teachers: Vec<&TeacherInfo> = teachers
        .iter()
        .filter(|teacher| {
            if required_teacher_ids.contains(&teacher.id) {
                return true;
            }
            if !required_teacher_ids.is_empty() {
                return false;
            }
            if !is_teacher_enabled_for_task_source(teacher, task.task_source, config) {
                return false;
            }
            let teacher_pairs = teacher_grade_subject_pairs.get(&teacher.id);
            let hit_forbidden_slot_pair = slot_forbidden_grade_subjects.iter().any(|pair| {
                teacher_pairs
                    .map(|pairs| pairs.contains(pair))
                    .unwrap_or(false)
            });
            if hit_forbidden_slot_pair {
                return false;
            }

            let is_excluded = custom_rules.iter().any(|rule| {
                rule.action_type == RULE_ACTION_EXCLUDE
                    && rule.teacher_id == teacher.id
                    && task_matches_custom_rule(task, rule)
            });

            !is_excluded
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
            .filter(|teacher| {
                required_teacher_ids.contains(&teacher.id)
                    || !teacher.subjects.contains(&task.subject)
            })
            .map(|teacher| TaskCandidate {
                teacher_id: teacher.id,
                assignment_tier: None,
            })
            .collect();
        return TaskCandidateSummary { candidates };
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
                    exam_allocation::SelfStudyTopicKind::Subject => {
                        topic.subjects.first().is_some_and(|subject| {
                            teacher.class_names.contains(class_name)
                                && teacher.subjects.contains(subject)
                        })
                    }
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
        return TaskCandidateSummary { candidates };
    }

    // FloorRover subject avoidance is applied against the whole merged subject set.
    // For example, an external-language rover for one floor must avoid all
    // English/Japanese/Russian teachers in that slot, not just task.subject.
    TaskCandidateSummary {
        candidates: active_teachers
            .iter()
            .filter(|teacher| {
                if task.role == StaffRole::FloorRover {
                    !task
                        .subject_avoidance_subjects
                        .iter()
                        .any(|subject| teacher.subjects.contains(subject))
                } else {
                    true
                }
            })
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
            left.0.cmp(&right.0).then(
                left.1
                    .as_ref()
                    .map(|tier| tier.as_key())
                    .cmp(&right.1.as_ref().map(|tier| tier.as_key())),
            )
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

async fn load_spaces_for_session(
    db: &sea_orm::DatabaseConnection,
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
    let mut out = Vec::new();
    for row in exam_staff_repo::load_spaces_for_session(db, session_id).await? {
        let space_type = match row.space_type.as_str() {
            "exam_room" => SpaceType::ExamRoom,
            "self_study_room" => SpaceType::SelfStudyRoom,
            _ => return Err(AppError::new(format!("无效的空间类型: {}", row.space_type))),
        };
        let self_study_topic = self_study_topic_from_parts(
            row.self_study_topic_kind,
            row.self_study_topic_subjects_json,
            row.self_study_topic_label,
        )?;
        out.push((
            row.id,
            space_type,
            row.space_name,
            row.original_class_name,
            self_study_topic,
            row.floor,
        ));
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

fn floor_rover_subject_group_key(subject: Subject) -> String {
    // 外语同一时间考试时，共用同一组楼层流动监考，所以这里统一折叠成一个分组键。
    if exam_allocation::is_foreign_subject(subject) {
        "foreign_group".to_string()
    } else {
        subject.as_key().to_string()
    }
}

fn build_floor_rover_subjects_by_slot(
    session_times: &[SessionTimeRuntime],
) -> HashMap<FloorRoverSlotKey, Vec<Subject>> {
    let mut subjects_by_slot = HashMap::<FloorRoverSlotKey, Vec<Subject>>::new();
    for session in session_times {
        let key = FloorRoverSlotKey {
            grade_name: session.grade_name.clone(),
            start_ts: session.start_ts,
            end_ts: session.end_ts,
            subject_group_key: floor_rover_subject_group_key(session.subject),
        };
        subjects_by_slot
            .entry(key)
            .or_default()
            .push(session.subject);
    }

    for subjects in subjects_by_slot.values_mut() {
        subjects.sort_by_key(|subject| subject_order(*subject));
        subjects.dedup();
    }

    subjects_by_slot
}

async fn build_staff_tasks(
    db: &sea_orm::DatabaseConnection,
    session_times: &[SessionTimeRuntime],
    invigilation_config: &RuntimeInvigilationConfig,
    class_subject_map: &HashMap<(String, String), HashSet<Subject>>,
    teaching_classes: &[TeachingClassRuntime],
) -> Result<Vec<TaskBuild>, AppError> {
    let active_teaching_classes =
        teaching_classes_for_sessions(teaching_classes, session_times);
    let floor_rover_subjects_by_slot = build_floor_rover_subjects_by_slot(session_times);
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
    // 同一年级、同一时间段、同一科目组、同一楼层只生成一条楼层流动任务。
    // 这样外语场次即使拆成英语/日语/俄语多个 session，也仍然是一层一个老师。
    let mut generated_floor_rovers = HashSet::<(FloorRoverSlotKey, String)>::new();
    for session in session_times {
        let spaces = load_spaces_for_session(db, session.session_id).await?;
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
        for (space_id, space_type, space_name, original_class_name, self_study_topic, floor) in
            &spaces
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
                            subject_avoidance_subjects: vec![session.subject],
                            recommended_self_study_topic: None,
                            priority_self_study_chain: Vec::new(),
                            day_key: day_key.clone(),
                            half_day,
                            rule_target_id: format!("space:{space_id}"),
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
                        subject_avoidance_subjects: vec![session.subject],
                        recommended_self_study_topic,
                        priority_self_study_chain,
                        day_key: day_key.clone(),
                        half_day,
                        rule_target_id: format!("space:{space_id}"),
                    });
                }
            }
        }

        let floor_rover_slot_key = FloorRoverSlotKey {
            grade_name: session.grade_name.clone(),
            start_ts: session.start_ts,
            end_ts: session.end_ts,
            subject_group_key: floor_rover_subject_group_key(session.subject),
        };
        let subject_avoidance_subjects = floor_rover_subjects_by_slot
            .get(&floor_rover_slot_key)
            .cloned()
            .unwrap_or_else(|| vec![session.subject]);
        let mut sorted_floors: Vec<String> = floors.into_iter().collect();
        sorted_floors.sort();
        for floor in sorted_floors {
            if !generated_floor_rovers.insert((floor_rover_slot_key.clone(), floor.clone())) {
                continue;
            }
            tasks.push(TaskBuild {
                session_id: Some(session.session_id),
                space_id: None,
                task_source: StaffTaskSource::Exam,
                role: StaffRole::FloorRover,
                grade_name: session.grade_name.clone(),
                subject: session.subject,
                space_name: format!("{} 楼层流动", floor),
                floor: floor.clone(),
                start_at: session.start_at.clone(),
                end_at: session.end_at.clone(),
                start_ts: session.start_ts,
                end_ts: session.end_ts,
                duration_minutes: duration_minutes(session.start_ts, session.end_ts)?,
                subject_avoidance_subjects: subject_avoidance_subjects.clone(),
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: day_key.clone(),
                half_day,
                rule_target_id: format!("floor:{}:{}", session.session_id, floor),
            });
        }
    }

    if !active_teaching_classes.is_empty() {
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

        for teaching_class in active_teaching_classes {
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
                subject_avoidance_subjects: vec![subject],
                recommended_self_study_topic: Some(
                    exam_allocation::build_subject_self_study_topic(subject),
                ),
                priority_self_study_chain: vec![exam_allocation::build_subject_self_study_topic(
                    subject,
                )],
                day_key: day_key.clone(),
                half_day,
                rule_target_id: format!("class:{}", teaching_class.id),
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

fn build_custom_rule_summary(rule: &PersistedInvigilationCustomRule) -> String {
    let action_label = if rule.action_type == RULE_ACTION_REQUIRE {
        "指定安排"
    } else {
        "禁排"
    };
    let time_label = match rule.time_scope_type.as_str() {
        RULE_TIME_SCOPE_FULL_SELF_STUDY => rule
            .time_scope_labels
            .first()
            .cloned()
            .unwrap_or_else(|| "全员自习时段".to_string()),
        _ => {
            if rule.time_scope_labels.is_empty() {
                "未选择考试时段".to_string()
            } else {
                rule.time_scope_labels.join("、")
            }
        }
    };
    let task_label = match rule.task_scope_type.as_str() {
        RULE_TASK_SCOPE_EXAM_ROOM => "考试任务",
        RULE_TASK_SCOPE_EXAM_LINKED_SELF_STUDY => "考试期间自习看班",
        RULE_TASK_SCOPE_FULL_SELF_STUDY => "全员自习看班",
        RULE_TASK_SCOPE_FLOOR_ROVER => "流动监考",
        _ => "未知任务",
    };
    let target_label = if rule.target_scope_type == RULE_TARGET_SCOPE_ALL {
        "全部对象".to_string()
    } else if rule.target_labels.is_empty() {
        "未选择对象".to_string()
    } else {
        rule.target_labels.join("、")
    };
    format!(
        "{} {} 在 {} 的 {}（{}）",
        action_label, rule.teacher_name, time_label, task_label, target_label
    )
}

fn to_persisted_rule(
    rule: &GenerateExamStaffPlanCustomRule,
    teacher_name: String,
    time_scope_labels: Vec<String>,
    target_labels: Vec<String>,
) -> PersistedInvigilationCustomRule {
    PersistedInvigilationCustomRule {
        action_type: rule.action_type.clone(),
        teacher_id: rule.teacher_id,
        teacher_name,
        time_scope_type: rule.time_scope_type.clone(),
        time_scope_ids: rule.time_scope_ids.clone(),
        time_scope_labels,
        task_scope_type: rule.task_scope_type.clone(),
        target_scope_type: rule.target_scope_type.clone(),
        target_ids: rule.target_ids.clone(),
        target_labels,
    }
}

fn persisted_rules_from_payload(
    rules: &[GenerateExamStaffPlanCustomRule],
) -> Vec<PersistedInvigilationCustomRule> {
    rules
        .iter()
        .map(|rule| {
            to_persisted_rule(
                rule,
                rule.teacher_name
                    .clone()
                    .unwrap_or_else(|| format!("教师{}", rule.teacher_id)),
                rule.time_scope_labels.clone(),
                rule.target_labels.clone(),
            )
        })
        .collect()
}

fn validate_custom_rule_shapes(rules: &[PersistedInvigilationCustomRule]) -> Result<(), AppError> {
    for rule in rules {
        if rule.teacher_id <= 0 || rule.teacher_name.trim().is_empty() {
            return Err(AppError::new("排班规则缺少教师信息"));
        }
        if rule.action_type != RULE_ACTION_EXCLUDE && rule.action_type != RULE_ACTION_REQUIRE {
            return Err(AppError::new(format!(
                "排班规则动作无效：{}",
                rule.action_type
            )));
        }
        if rule.time_scope_type != RULE_TIME_SCOPE_EXAM_SESSION
            && rule.time_scope_type != RULE_TIME_SCOPE_FULL_SELF_STUDY
        {
            return Err(AppError::new(format!(
                "排班规则时间范围无效：{}",
                rule.time_scope_type
            )));
        }
        if rule.time_scope_type == RULE_TIME_SCOPE_EXAM_SESSION && rule.time_scope_ids.is_empty() {
            return Err(AppError::new("考试时段规则至少需要选择一个考试时段"));
        }
        if !matches!(
            rule.task_scope_type.as_str(),
            RULE_TASK_SCOPE_EXAM_ROOM
                | RULE_TASK_SCOPE_EXAM_LINKED_SELF_STUDY
                | RULE_TASK_SCOPE_FULL_SELF_STUDY
                | RULE_TASK_SCOPE_FLOOR_ROVER
        ) {
            return Err(AppError::new(format!(
                "排班规则任务类型无效：{}",
                rule.task_scope_type
            )));
        }
        if rule.target_scope_type != RULE_TARGET_SCOPE_ALL
            && rule.target_scope_type != RULE_TARGET_SCOPE_SELECTED
        {
            return Err(AppError::new(format!(
                "排班规则对象范围无效：{}",
                rule.target_scope_type
            )));
        }
        if rule.target_scope_type == RULE_TARGET_SCOPE_SELECTED && rule.target_ids.is_empty() {
            return Err(AppError::new("指定对象规则至少需要选择一个对象"));
        }
        if rule.time_scope_type == RULE_TIME_SCOPE_FULL_SELF_STUDY
            && rule.task_scope_type != RULE_TASK_SCOPE_FULL_SELF_STUDY
        {
            return Err(AppError::new("全员自习时段只能配置全员自习看班规则"));
        }
        if rule.time_scope_type == RULE_TIME_SCOPE_EXAM_SESSION
            && rule.task_scope_type == RULE_TASK_SCOPE_FULL_SELF_STUDY
        {
            return Err(AppError::new("考试时段不能配置全员自习看班规则"));
        }
    }
    Ok(())
}

fn validate_custom_rules_against_tasks(
    rules: &[PersistedInvigilationCustomRule],
    tasks: &[TaskBuild],
) -> Result<(), AppError> {
    validate_custom_rule_shapes(rules)?;
    let generated_rules = rules
        .iter()
        .map(|rule| GenerateExamStaffPlanCustomRule {
            action_type: rule.action_type.clone(),
            teacher_id: rule.teacher_id,
            teacher_name: Some(rule.teacher_name.clone()),
            time_scope_type: rule.time_scope_type.clone(),
            time_scope_ids: rule.time_scope_ids.clone(),
            time_scope_labels: rule.time_scope_labels.clone(),
            task_scope_type: rule.task_scope_type.clone(),
            target_scope_type: rule.target_scope_type.clone(),
            target_ids: rule.target_ids.clone(),
            target_labels: rule.target_labels.clone(),
        })
        .collect::<Vec<_>>();
    let matched_task_indexes = generated_rules
        .iter()
        .map(|rule| {
            tasks
                .iter()
                .enumerate()
                .filter_map(|(index, task)| task_matches_custom_rule(task, rule).then_some(index))
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>();

    for (index, rule) in rules.iter().enumerate() {
        if rule.target_scope_type == RULE_TARGET_SCOPE_SELECTED
            && matched_task_indexes[index].is_empty()
        {
            return Err(AppError::new(format!(
                "排班规则未命中任何可用对象：{}。如需指定考场或班级，请先完成一次考场/任务生成。",
                build_custom_rule_summary(rule)
            )));
        }
    }

    for left in 0..rules.len() {
        for right in (left + 1)..rules.len() {
            if rules[left].teacher_id != rules[right].teacher_id {
                continue;
            }
            if rules[left].action_type == rules[right].action_type {
                continue;
            }
            if rules[left].task_scope_type != rules[right].task_scope_type {
                continue;
            }
            if !matched_task_indexes[left]
                .intersection(&matched_task_indexes[right])
                .next()
                .is_some()
            {
                continue;
            }
            return Err(AppError::new(format!(
                "排班规则冲突：{} 与 {} 命中了同一批任务，请先调整后再保存。",
                build_custom_rule_summary(&rules[left]),
                build_custom_rule_summary(&rules[right])
            )));
        }
    }

    let mut required_task_records = Vec::<(i64, usize, String)>::new();
    for (rule_index, rule) in rules.iter().enumerate() {
        if rule.action_type != RULE_ACTION_REQUIRE {
            continue;
        }
        for task_index in &matched_task_indexes[rule_index] {
            required_task_records.push((
                rule.teacher_id,
                *task_index,
                build_custom_rule_summary(rule),
            ));
        }
    }
    for left in 0..required_task_records.len() {
        for right in (left + 1)..required_task_records.len() {
            if required_task_records[left].0 != required_task_records[right].0 {
                continue;
            }
            if required_task_records[left].1 == required_task_records[right].1 {
                continue;
            }
            let left_task = &tasks[required_task_records[left].1];
            let right_task = &tasks[required_task_records[right].1];
            if left_task.start_ts < right_task.end_ts && right_task.start_ts < left_task.end_ts {
                return Err(AppError::new(format!(
                    "指定安排冲突：老师 {} 同时被要求承担重叠时段任务（{} / {}）。",
                    rules
                        .iter()
                        .find(|rule| rule.teacher_id == required_task_records[left].0)
                        .map(|rule| rule.teacher_name.as_str())
                        .unwrap_or("未知老师"),
                    required_task_records[left].2,
                    required_task_records[right].2
                )));
            }
        }
    }

    Ok(())
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

            let mut group_key: Vec<usize> = active_group
                .iter()
                .map(|(task_index, _)| *task_index)
                .collect();
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
        model.add_hint(
            *bool_var,
            if bool_var.solution_value(response) {
                1
            } else {
                0
            },
        );
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
    custom_rules: &[GenerateExamStaffPlanCustomRule],
    invigilation_config: &RuntimeInvigilationConfig,
    teacher_grade_subject_pairs: &HashMap<i64, HashSet<(String, Subject)>>,
    progress: Option<&StaffAssignmentProgressReporter>,
) -> CpSatAttempt {
    let started_at = Instant::now();
    let mut forbidden_grade_subjects_by_slot =
        HashMap::<(i64, i64), HashSet<(String, Subject)>>::new();
    for task in tasks {
        // “师生同考”禁排只对考试时段生效：同一时段内出现的全部(年级,科目)都应回避。
        if task.session_id.is_some() {
            forbidden_grade_subjects_by_slot
                .entry((task.start_ts, task.end_ts))
                .or_default()
                .insert((task.grade_name.clone(), task.subject));
        }
    }

    let empty_forbidden_pairs = HashSet::<(String, Subject)>::new();
    let candidate_summaries: Vec<TaskCandidateSummary> = tasks
        .iter()
        .map(|task| {
            let slot_forbidden_grade_subjects = forbidden_grade_subjects_by_slot
                .get(&(task.start_ts, task.end_ts))
                .unwrap_or(&empty_forbidden_pairs);
            build_task_candidate_summary(
                task,
                teachers,
                custom_rules,
                invigilation_config,
                slot_forbidden_grade_subjects,
                teacher_grade_subject_pairs,
            )
        })
        .collect();
    let teacher_symmetry_groups = build_teacher_symmetry_groups(teachers, &candidate_summaries);

    let mut model = CpModelBuilder::default();
    let mut candidate_bindings = Vec::<Vec<(BoolVar, TaskCandidate)>>::new();
    let mut unassigned_vars = Vec::<BoolVar>::new();
    let mut teacher_assignment_vars = HashMap::<i64, Vec<(usize, BoolVar)>>::new();
    let mut teacher_day_half_vars = HashMap::<(i64, String, HalfDay), Vec<BoolVar>>::new();
    let mut teacher_load_vars = HashMap::<i64, (IntVar, IntVar, IntVar)>::new();

    let total_minutes_capacity = tasks
        .iter()
        .map(|task| task.duration_minutes)
        .sum::<i64>()
        .max(1);
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
    let mut unassigned_penalty_expr = LinearExpr::default();
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
        let penalty_weight = if task.role == StaffRole::FloorRover {
            1_i64
        } else {
            10000_i64
        };
        unassigned_penalty_expr += (penalty_weight, unassigned);
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
        tasks
            .iter()
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
        tasks
            .iter()
            .filter(|task| task.role == StaffRole::SelfStudySupervisor)
            .map(|task| task.duration_minutes)
            .sum::<i64>(),
    );

    let unassigned_count_var =
        model.new_int_var_with_name([(0, tasks.len() as i64)], "unassigned_count");
    model.add_eq(unassigned_count_var, unassigned_expr.clone());

    let unassigned_penalty_var =
        model.new_int_var_with_name([(0, tasks.len() as i64 * 10000)], "unassigned_penalty");
    model.add_eq(unassigned_penalty_var, unassigned_penalty_expr);

    let fallback_count_var =
        model.new_int_var_with_name([(0, self_study_task_capacity)], "fallback_pool_assignments");
    model.add_eq(fallback_count_var, fallback_expr.clone());

    let homeroom_count_var =
        model.new_int_var_with_name([(0, self_study_task_capacity)], "homeroom_assignments");
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
        .chain([
            unassigned_count_var,
            unassigned_penalty_var,
            fallback_count_var,
            homeroom_count_var,
        ])
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
            "unassigned_penalty",
            "优先分配考场监考",
            LinearExpr::from(unassigned_penalty_var),
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
    for (stage_index, (stage_name, stage_label, objective)) in stage_objectives.iter().enumerate() {
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
            cross_builder.add_le(morning_expr, ((morning_vars.len() as i64), morning_present));
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

async fn persist_solved_plan(
    db: &sea_orm::DatabaseConnection,
    session_count: i64,
    teachers: &[TeacherInfo],
    plan: &SolvedPlan,
) -> Result<GenerateLatestExamStaffPlanResult, AppError> {
    let teacher_by_id: HashMap<i64, &TeacherInfo> = teachers
        .iter()
        .map(|teacher| (teacher.id, teacher))
        .collect();
    let generated_at = Utc::now().to_rfc3339();
    let mut task_rows = Vec::new();

    for record in &plan.records {
        let status = if record.teacher_id.is_some() {
            TaskStatus::Assigned
        } else {
            TaskStatus::Unassigned
        };
        let teacher = record
            .teacher_id
            .and_then(|teacher_id| teacher_by_id.get(&teacher_id).copied());
        task_rows.push(exam_staff_repo::PersistedTaskRow {
            session_id: record.task.session_id,
            space_id: record.task.space_id,
            task_source: record.task.task_source.as_key().to_string(),
            role: record.task.role.as_key().to_string(),
            grade_name: record.task.grade_name.clone(),
            subject: record.task.subject.as_key().to_string(),
            space_name: record.task.space_name.clone(),
            floor: record.task.floor.clone(),
            start_at: record.task.start_at.clone(),
            end_at: record.task.end_at.clone(),
            duration_minutes: record.task.duration_minutes,
            recommended_self_study_topic_kind: record
                .task
                .recommended_self_study_topic
                .as_ref()
                .map(|topic| topic.kind.as_key().to_string()),
            recommended_self_study_topic_subjects_json: record
                .task
                .recommended_self_study_topic
                .as_ref()
                .map(|topic| serde_json::to_string(&topic.subjects))
                .transpose()
                .map_err(|e| AppError::new(format!("推荐自习主题科目序列化失败: {e}")))?,
            recommended_self_study_topic_label: record
                .task
                .recommended_self_study_topic
                .as_ref()
                .map(|topic| topic.label.clone()),
            priority_self_study_chain_json: self_study_topic_chain_to_text(
                &record.task.priority_self_study_chain,
            )?,
            assignment_tier: record.assignment_tier.map(|tier| tier.as_key().to_string()),
            status: status.as_key().to_string(),
            reason: record.reason.clone(),
            allowance_amount: record.allowance_amount,
            teacher_id: teacher.map(|item| item.id),
            teacher_name: teacher.map(|item| item.name.clone()),
        });
    }

    let mut duty_rows = Vec::new();
    for teacher in teachers {
        let state = plan.runtime.get(&teacher.id).cloned().unwrap_or_default();
        duty_rows.push(exam_staff_repo::PersistedDutyStatRow {
            teacher_id: teacher.id,
            teacher_name: teacher.name.clone(),
            indoor_minutes: state.indoor_minutes,
            outdoor_minutes: state.outdoor_minutes,
            total_minutes: state.total_minutes,
            task_count: state.task_count,
            exam_room_task_count: state.exam_room_task_count,
            self_study_task_count: state.self_study_task_count,
            floor_rover_task_count: state.floor_rover_task_count,
            allowance_total: round_to_two(state.allowance_total),
            indoor_allowance_total: round_to_two(state.indoor_allowance_total),
            outdoor_allowance_total: round_to_two(state.outdoor_allowance_total),
            is_middle_manager: teacher.is_middle_manager,
        });
    }

    exam_staff_repo::persist_plan_snapshot(
        db,
        exam_staff_repo::PersistedPlanMetaRow {
            generated_at: generated_at.clone(),
            session_count,
            task_count: plan.records.len() as i64,
            assigned_count: plan.metrics.assigned_count,
            unassigned_count: plan.metrics.unassigned_count,
            warning_count: plan.metrics.warning_count,
            imbalance_minutes: plan.metrics.imbalance_minutes,
            solver_engine: plan.solver_engine.as_key().to_string(),
            optimality_status: plan.optimality_status.as_key().to_string(),
            solve_duration_ms: plan.solve_duration_ms,
            fallback_reason: plan
                .fallback_reason
                .map(|reason| reason.as_key().to_string()),
            fallback_pool_assignments: plan.metrics.fallback_pool_assignments,
        },
        task_rows,
        duty_rows,
    )
    .await?;

    let unassigned_details: Vec<String> = plan
        .records
        .iter()
        .filter(|record| record.teacher_id.is_none())
        .map(|record| {
            let task_detail = if record.task.role == StaffRole::FloorRover {
                record.task.space_name.clone()
            } else {
                format!(
                    "{} {}",
                    record.task.space_name,
                    role_label(record.task.role)
                )
            };
            format!(
                "{}{} {}",
                record.task.grade_name,
                subject_label(record.task.subject),
                task_detail,
            )
        })
        .collect();

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
        unassigned_details,
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

async fn generate_latest_exam_staff_plan_internal(
    db: &sea_orm::DatabaseConnection,
    invigilation_config: RuntimeInvigilationConfig,
    custom_rules: Vec<GenerateExamStaffPlanCustomRule>,
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
    let session_times = load_session_times_runtime(db).await?;
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
    let teachers = load_teacher_pool(db).await?;
    let teacher_grade_subject_pairs = load_teacher_grade_subject_pairs(db).await?;
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
    let class_subject_map = load_class_subject_map(db).await?;
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
    let teaching_classes = load_teaching_classes(db).await?;
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
        db,
        &session_times,
        &invigilation_config,
        &class_subject_map,
        &teaching_classes,
    )
    .await?;
    let persisted_rules = persisted_rules_from_payload(&custom_rules);
    validate_custom_rules_against_tasks(&persisted_rules, &tasks)?;

    let cp_sat_attempt = solve_with_cp_sat(
        &tasks,
        &teachers,
        &custom_rules,
        &invigilation_config,
        &teacher_grade_subject_pairs,
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
    let result =
        persist_solved_plan(db, session_times.len() as i64, &teachers, &final_plan).await?;
    if let Some(progress) = progress {
        progress.emit_completed(format!(
            "监考分配完成：已分配 {} 项，未分配 {} 项。",
            result.assigned_count, result.unassigned_count
        ));
    }
    Ok(result)
}

pub async fn list_exam_session_time_grade_options(app: AppHandle) -> Result<Vec<String>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        load_effective_session_time_grade_options(&db).await
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn list_exam_session_times(
    app: AppHandle,
    params: Option<ListExamSessionTimesParams>,
) -> Result<Vec<ExamSessionTime>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let grade_options = load_effective_session_time_grade_options(&db).await?;
        let selected_grade = params
            .as_ref()
            .and_then(|value| value.grade_name.as_ref())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .or_else(|| grade_options.first().cloned());
        let Some(selected_grade) = selected_grade else {
            return Ok(Vec::new());
        };
        load_session_time_template_rows(&db, &selected_grade).await
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn upsert_exam_session_times(
    app: AppHandle,
    items: Vec<ExamSessionTimeUpsert>,
) -> Result<SuccessResponse, String> {
    let result = async {
        let now = Utc::now().to_rfc3339();
        let mut rows = Vec::new();
        for item in items {
            let grade_name = item.grade_name.trim().to_string();
            if grade_name.is_empty() {
                return Err(AppError::new("年级不能为空"));
            }
            let start_at = item.start_at.clone();
            let end_at = item.end_at.clone();
            let start_ts = parse_datetime_to_ts(&start_at)?;
            let end_ts = parse_datetime_to_ts(&end_at)?;
            duration_minutes(start_ts, end_ts)?;
            rows.push(exam_staff_repo::UpsertSessionTimeRow {
                session_id: item.session_id,
                grade_name,
                subject: item.subject.as_key().to_string(),
                start_at,
                end_at,
            });
        }
        let db = crate::db::connect(&app).await?;
        exam_staff_repo::upsert_session_times(&db, &rows, &now).await?;
        Ok(SuccessResponse::ok())
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn delete_exam_session_time(
    app: AppHandle,
    grade_name: String,
    subject: Subject,
) -> Result<SuccessResponse, String> {
    let result = async {
        let trimmed_grade_name = grade_name.trim();
        if trimmed_grade_name.is_empty() {
            return Err(AppError::new("年级不能为空"));
        }
        let db = crate::db::connect(&app).await?;
        let subjects = if matches!(
            subject,
            Subject::English | Subject::Russian | Subject::Japanese
        ) {
            vec![Subject::English, Subject::Russian, Subject::Japanese]
        } else {
            vec![subject]
        };
        for subject in subjects {
            exam_staff_repo::delete_session_time_template(
                &db,
                trimmed_grade_name,
                subject.as_key(),
            )
            .await?;
        }
        Ok(SuccessResponse::ok())
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn generate_latest_exam_staff_plan(
    app: AppHandle,
    payload: GenerateExamStaffPlanPayload,
) -> Result<GenerateLatestExamStaffPlanResult, String> {
    let progress = StaffAssignmentProgressReporter::new(app.clone());
    let result = async {
        let db = crate::db::connect(&app).await?;
        let log_path = app_log::log_path(&app).ok();
        let mut config = build_config_from_payload(&payload);
        hydrate_runtime_middle_manager_config(&db, &mut config).await?;
        config.self_study_class_subjects = load_self_study_class_subjects(&db).await?;
        let custom_rules = payload.custom_rules;
        generate_latest_exam_staff_plan_internal(
            &db,
            config,
            custom_rules,
            log_path.as_deref(),
            Some(&progress),
        )
        .await
    }
    .await;
    match result {
        Ok(result) => Ok(result),
        Err(error) => {
            progress.emit_error("error", "分配失败", error.to_string());
            Err(error.to_string())
        }
    }
}

pub async fn list_invigilation_exclusion_session_options(
    app: AppHandle,
) -> Result<Vec<InvigilationExclusionSessionOption>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let rows = load_configured_session_times_runtime(&db).await?;
        let mut items = Vec::new();
        for row in rows {
            let start_at = row.start_at.clone();
            let end_at = row.end_at.clone();
            items.push(InvigilationExclusionSessionOption {
                session_id: row.session_id,
                grade_name: row.grade_name.clone(),
                subject: row.subject,
                start_at: start_at.clone(),
                end_at: end_at.clone(),
                label: format!(
                    "{} {} {} {}-{}",
                    row.grade_name,
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
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn list_invigilation_custom_rule_options(
    app: AppHandle,
) -> Result<InvigilationRuleOptions, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let session_times = load_configured_session_times_runtime(&db).await?;
        let exam_session_options = session_times
            .iter()
            .map(|session| InvigilationRuleTimeScopeOption {
                id: session.session_id,
                label: build_session_label(
                    &session.grade_name,
                    session.subject,
                    &session.start_at,
                    &session.end_at,
                ),
                start_at: session.start_at.clone(),
                end_at: session.end_at.clone(),
            })
            .collect::<Vec<_>>();

        let config = load_runtime_invigilation_config(&db).await?;
        let teaching_classes = load_teaching_classes(&db).await?;

        let full_self_study_option = if !config.self_study_date.trim().is_empty()
            && !config.self_study_start_time.trim().is_empty()
            && !config.self_study_end_time.trim().is_empty()
        {
            let start_at =
                build_self_study_datetime(&config.self_study_date, &config.self_study_start_time)?;
            let end_at =
                build_self_study_datetime(&config.self_study_date, &config.self_study_end_time)?;
            Some(InvigilationRuleFullSelfStudyOption {
                label: format!(
                    "全员自习 {} {}-{}",
                    &config.self_study_date[5..],
                    config.self_study_start_time,
                    config.self_study_end_time
                ),
                start_at,
                end_at,
            })
        } else {
            None
        };

        let target_options =
            build_rule_target_options_from_spaces(&db, &session_times, &config, &teaching_classes)
                .await?;

        Ok(InvigilationRuleOptions {
            exam_session_options,
            full_self_study_option,
            target_options,
        })
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
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

fn monitor_draw_cell_to_string(cell: Option<&Data>) -> String {
    match cell {
        Some(Data::String(s)) => s.trim().to_string(),
        Some(Data::Float(v)) => {
            if v.fract().abs() < 1e-9 {
                format!("{v:.0}")
            } else {
                v.to_string()
            }
        }
        Some(Data::Int(v)) => v.to_string(),
        Some(Data::Bool(v)) => v.to_string(),
        Some(Data::DateTimeIso(s)) => s.trim().to_string(),
        Some(Data::DurationIso(s)) => s.trim().to_string(),
        Some(Data::Empty) | None => String::new(),
        Some(other) => other.to_string().trim().to_string(),
    }
}

fn normalize_monitor_draw_header(text: &str) -> String {
    text.trim().replace(' ', "").replace('\n', "")
}

fn monitor_draw_header_has(header: &str, aliases: &[&str]) -> bool {
    aliases.iter().any(|alias| header == *alias)
}

pub fn import_monitor_draw_pairs_from_excel(
    app: AppHandle,
    file_path: String,
) -> Result<MonitorDrawImportResult, String> {
    let started = Instant::now();
    let result = (|| -> Result<MonitorDrawImportResult, AppError> {
        let path_text = file_path.trim();
        if path_text.is_empty() {
            return Err(AppError::new("未提供可导入的 Excel 文件路径"));
        }
        let path = Path::new(path_text);
        if !path.exists() {
            return Err(AppError::new(format!("文件不存在：{}", path.display())));
        }

        let mut workbook = open_workbook_auto(path)
            .map_err(|error| AppError::new(format!("无法打开 Excel 文件: {error}")))?;
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| AppError::new("Excel 文件没有可读取的工作表"))?;
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|error| AppError::new(format!("读取工作表失败: {error}")))?;

        let mut rows_iter = range.rows();
        let header_row = rows_iter
            .next()
            .ok_or_else(|| AppError::new("Excel 内容为空，至少需要一行表头"))?;
        let headers = header_row
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                (
                    index,
                    normalize_monitor_draw_header(&monitor_draw_cell_to_string(Some(cell))),
                )
            })
            .collect::<Vec<_>>();

        let group_col = headers
            .iter()
            .find(|(_, text)| monitor_draw_header_has(text, &["组号", "普通序号", "序号"]))
            .map(|(index, _)| *index)
            .ok_or_else(|| AppError::new("导入失败：缺少“组号”列"))?;
        let a_col = headers
            .iter()
            .find(|(_, text)| monitor_draw_header_has(text, &["监考员甲", "甲姓名", "甲"]))
            .map(|(index, _)| *index)
            .ok_or_else(|| AppError::new("导入失败：缺少“监考员甲”列"))?;
        let b_col = headers
            .iter()
            .find(|(_, text)| monitor_draw_header_has(text, &["监考员乙", "乙姓名", "乙"]))
            .map(|(index, _)| *index)
            .ok_or_else(|| AppError::new("导入失败：缺少“监考员乙”列"))?;

        let mut seen_groups = HashSet::<String>::new();
        let mut seen_a_names = HashSet::<String>::new();
        let mut seen_b_names = HashSet::<String>::new();
        let mut imported_rows = Vec::<MonitorDrawImportRow>::new();

        for (offset, row) in rows_iter.enumerate() {
            let row_no = offset + 2;
            let group_no = monitor_draw_cell_to_string(row.get(group_col));
            let invigilator_a_name = monitor_draw_cell_to_string(row.get(a_col));
            let invigilator_b_name = monitor_draw_cell_to_string(row.get(b_col));

            if group_no.is_empty() && invigilator_a_name.is_empty() && invigilator_b_name.is_empty()
            {
                continue;
            }
            if group_no.is_empty() || invigilator_a_name.is_empty() || invigilator_b_name.is_empty()
            {
                return Err(AppError::new(format!(
                    "第 {row_no} 行存在空值：组号、监考员甲、监考员乙均为必填"
                )));
            }
            if invigilator_a_name == invigilator_b_name {
                return Err(AppError::new(format!(
                    "第 {row_no} 行数据非法：监考员甲与监考员乙不能为同一人"
                )));
            }
            if !seen_groups.insert(group_no.clone()) {
                return Err(AppError::new(format!("导入失败：组号“{group_no}”重复")));
            }
            if !seen_a_names.insert(invigilator_a_name.clone()) {
                return Err(AppError::new(format!(
                    "导入失败：监考员甲“{invigilator_a_name}”重复"
                )));
            }
            if !seen_b_names.insert(invigilator_b_name.clone()) {
                return Err(AppError::new(format!(
                    "导入失败：监考员乙“{invigilator_b_name}”重复"
                )));
            }

            imported_rows.push(MonitorDrawImportRow {
                group_no,
                invigilator_a_name,
                invigilator_b_name,
            });
        }

        if imported_rows.is_empty() {
            return Err(AppError::new("未识别到可用数据，请检查表格内容"));
        }

        Ok(MonitorDrawImportResult {
            imported_at: Utc::now().to_rfc3339(),
            row_count: imported_rows.len() as i64,
            duration_ms: started.elapsed().as_millis() as i64,
            rows: imported_rows,
        })
    })();
    if let Err(error) = &result {
        let _ = app_log::append_log(
            &app,
            "error",
            "invigilation.import_monitor_draw_pairs_from_excel",
            &format!("失败: {error}"),
        );
    }
    result.map_err(|error| error.to_string())
}

fn build_session_label(grade_name: &str, subject: Subject, start_at: &str, end_at: &str) -> String {
    format!(
        "{} {} {} {}-{}",
        grade_name,
        subject_label(subject),
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
    )
}

async fn build_rule_target_options_from_spaces(
    db: &sea_orm::DatabaseConnection,
    session_times: &[SessionTimeRuntime],
    config: &RuntimeInvigilationConfig,
    teaching_classes: &[TeachingClassRuntime],
) -> Result<Vec<InvigilationRuleTargetOption>, AppError> {
    let active_teaching_classes =
        teaching_classes_for_sessions(teaching_classes, session_times);
    let mut seen = HashSet::<(String, String, Option<i64>, String)>::new();
    let mut target_options = Vec::<InvigilationRuleTargetOption>::new();

    for session in session_times {
        let subtitle = build_session_label(
            &session.grade_name,
            session.subject,
            &session.start_at,
            &session.end_at,
        );
        let mut floors = HashSet::<String>::new();
        for (space_id, space_type, space_name, original_class_name, self_study_topic, floor) in
            load_spaces_for_session(db, session.session_id).await?
        {
            if !floor.trim().is_empty() {
                floors.insert(floor);
            }
            let task_scope_type = match space_type {
                SpaceType::ExamRoom => RULE_TASK_SCOPE_EXAM_ROOM,
                SpaceType::SelfStudyRoom => RULE_TASK_SCOPE_EXAM_LINKED_SELF_STUDY,
            }
            .to_string();
            let label = match space_type {
                SpaceType::ExamRoom => space_name,
                SpaceType::SelfStudyRoom => original_class_name.unwrap_or(space_name),
            };
            let item_subtitle = match space_type {
                SpaceType::ExamRoom => Some(subtitle.clone()),
                SpaceType::SelfStudyRoom => self_study_topic
                    .as_ref()
                    .map(|topic| topic.label.clone())
                    .or_else(|| Some(subtitle.clone())),
            };
            let id = format!("space:{space_id}");
            let key = (
                task_scope_type.clone(),
                RULE_TIME_SCOPE_EXAM_SESSION.to_string(),
                Some(session.session_id),
                id.clone(),
            );
            if seen.insert(key) {
                target_options.push(InvigilationRuleTargetOption {
                    id,
                    label,
                    subtitle: item_subtitle,
                    time_scope_type: RULE_TIME_SCOPE_EXAM_SESSION.to_string(),
                    time_scope_id: Some(session.session_id),
                    task_scope_type,
                });
            }
        }

        for floor in floors {
            let id = format!("floor:{}:{}", session.session_id, floor);
            let key = (
                RULE_TASK_SCOPE_FLOOR_ROVER.to_string(),
                RULE_TIME_SCOPE_EXAM_SESSION.to_string(),
                Some(session.session_id),
                id.clone(),
            );
            if seen.insert(key) {
                target_options.push(InvigilationRuleTargetOption {
                    id,
                    label: format!("{} 楼层流动", floor),
                    subtitle: Some(subtitle.clone()),
                    time_scope_type: RULE_TIME_SCOPE_EXAM_SESSION.to_string(),
                    time_scope_id: Some(session.session_id),
                    task_scope_type: RULE_TASK_SCOPE_FLOOR_ROVER.to_string(),
                });
            }
        }
    }

    if !config.self_study_date.trim().is_empty()
        && !config.self_study_start_time.trim().is_empty()
        && !config.self_study_end_time.trim().is_empty()
    {
        let start_at =
            build_self_study_datetime(&config.self_study_date, &config.self_study_start_time)?;
        let end_at =
            build_self_study_datetime(&config.self_study_date, &config.self_study_end_time)?;
        let subtitle = Some(format!("{} {}", start_at, end_at));
        for teaching_class in active_teaching_classes {
            if !config
                .self_study_class_subjects
                .contains_key(&teaching_class.id)
            {
                continue;
            }
            let id = format!("class:{}", teaching_class.id);
            let key = (
                RULE_TASK_SCOPE_FULL_SELF_STUDY.to_string(),
                RULE_TIME_SCOPE_FULL_SELF_STUDY.to_string(),
                None,
                id.clone(),
            );
            if seen.insert(key) {
                target_options.push(InvigilationRuleTargetOption {
                    id,
                    label: teaching_class.class_name.clone(),
                    subtitle: subtitle.clone(),
                    time_scope_type: RULE_TIME_SCOPE_FULL_SELF_STUDY.to_string(),
                    time_scope_id: None,
                    task_scope_type: RULE_TASK_SCOPE_FULL_SELF_STUDY.to_string(),
                });
            }
        }
    }

    target_options.sort_by(|a, b| {
        a.task_scope_type
            .cmp(&b.task_scope_type)
            .then(a.time_scope_type.cmp(&b.time_scope_type))
            .then(a.time_scope_id.cmp(&b.time_scope_id))
            .then(a.label.cmp(&b.label))
    });
    Ok(target_options)
}

pub async fn get_persisted_invigilation_state(
    app: AppHandle,
) -> Result<PersistedInvigilationState, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let config_row = exam_staff_repo::get_config(&db).await?;
        let config = config_row
            .as_ref()
            .map(|row| {
                let self_study_date = row.self_study_date.trim().to_string();
                let middle_manager_exception_teacher_ids = serde_json::from_str::<Vec<i64>>(
                    &row.middle_manager_exception_teacher_ids_json,
                )
                .map(normalize_teacher_id_list)
                .unwrap_or_default();
                PersistedInvigilationConfig {
                    default_exam_room_required_count: row.default_exam_room_required_count.max(1),
                    indoor_allowance_per_minute: row.indoor_allowance_per_minute.max(0.0),
                    outdoor_allowance_per_minute: row.outdoor_allowance_per_minute.max(0.0),
                    middle_manager_default_enabled: row.middle_manager_default_enabled == 1,
                    middle_manager_exception_teacher_ids,
                    self_study_date: if self_study_date.is_empty() {
                        Utc::now().format("%Y-%m-%d").to_string()
                    } else {
                        self_study_date
                    },
                    self_study_start_time: row.self_study_start_time.clone(),
                    self_study_end_time: row.self_study_end_time.clone(),
                }
            })
            .unwrap_or_else(default_persisted_invigilation_config);

        let self_study_class_subjects = config_row
            .map(|row| row.self_study_class_subjects_json)
            .and_then(|text| {
                serde_json::from_str::<Vec<PersistedSelfStudyClassSubject>>(&text).ok()
            })
            .unwrap_or_default();

        let custom_rules = exam_staff_repo::list_custom_rules(&db)
            .await?
            .into_iter()
            .map(|row| PersistedInvigilationCustomRule {
                action_type: row.action_type,
                teacher_id: row.teacher_id,
                teacher_name: row.teacher_name,
                time_scope_type: row.time_scope_type,
                time_scope_ids: parse_json_i64_list(&row.time_scope_ids_json),
                time_scope_labels: parse_json_string_list(&row.time_scope_labels_json),
                task_scope_type: row.task_scope_type,
                target_scope_type: row.target_scope_type,
                target_ids: parse_json_string_list(&row.target_ids_json),
                target_labels: parse_json_string_list(&row.target_labels_json),
            })
            .collect();

        Ok(PersistedInvigilationState {
            config,
            custom_rules,
            self_study_class_subjects,
        })
    }
    .await;
    result.map_err(|e: AppError| e.to_string())
}

pub async fn save_persisted_invigilation_config(
    app: AppHandle,
    payload: PersistedInvigilationConfig,
) -> Result<SuccessResponse, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let now = Utc::now().to_rfc3339();
        let middle_manager_exception_teacher_ids_json = serde_json::to_string(
            &normalize_teacher_id_list(payload.middle_manager_exception_teacher_ids.clone()),
        )
        .map_err(|e| AppError::new(format!("中层监考例外序列化失败: {e}")))?;
        exam_staff_repo::upsert_config(
            &db,
            invigilation_config_settings::ActiveModel {
                id: sea_orm::ActiveValue::Set(1),
                default_exam_room_required_count: sea_orm::ActiveValue::Set(
                    payload.default_exam_room_required_count.max(1),
                ),
                indoor_allowance_per_minute: sea_orm::ActiveValue::Set(
                    payload.indoor_allowance_per_minute.max(0.0),
                ),
                outdoor_allowance_per_minute: sea_orm::ActiveValue::Set(
                    payload.outdoor_allowance_per_minute.max(0.0),
                ),
                middle_manager_default_enabled: sea_orm::ActiveValue::Set(
                    if payload.middle_manager_default_enabled {
                        1
                    } else {
                        0
                    },
                ),
                middle_manager_exception_teacher_ids_json: sea_orm::ActiveValue::Set(
                    middle_manager_exception_teacher_ids_json,
                ),
                self_study_date: sea_orm::ActiveValue::Set(
                    payload.self_study_date.trim().to_string(),
                ),
                self_study_start_time: sea_orm::ActiveValue::Set(
                    payload.self_study_start_time.trim().to_string(),
                ),
                self_study_end_time: sea_orm::ActiveValue::Set(
                    payload.self_study_end_time.trim().to_string(),
                ),
                self_study_class_subjects_json: sea_orm::ActiveValue::Set("[]".to_string()),
                updated_at: sea_orm::ActiveValue::Set(now),
            },
        )
        .await?;
        Ok(SuccessResponse::ok())
    }
    .await;
    result.map_err(|e: AppError| e.to_string())
}

pub async fn save_persisted_self_study_class_subjects(
    app: AppHandle,
    items: Vec<PersistedSelfStudyClassSubject>,
) -> Result<SuccessResponse, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let now = Utc::now().to_rfc3339();
        let json_text = serde_json::to_string(&items)
            .map_err(|e| AppError::new(format!("自习科目配置序列化失败: {e}")))?;
        exam_staff_repo::update_self_study_class_subjects_json(&db, &json_text, &now).await?;
        Ok(SuccessResponse::ok())
    }
    .await;
    result.map_err(|e: AppError| e.to_string())
}

pub async fn replace_persisted_invigilation_custom_rules(
    app: AppHandle,
    items: Vec<PersistedInvigilationCustomRule>,
) -> Result<SuccessResponse, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let session_times = load_session_times_runtime(&db).await?;
        let config = load_runtime_invigilation_config(&db).await?;
        let class_subject_map = load_class_subject_map(&db).await?;
        let teaching_classes = load_teaching_classes(&db).await?;
        let tasks = build_staff_tasks(
            &db,
            &session_times,
            &config,
            &class_subject_map,
            &teaching_classes,
        )
        .await?;
        validate_custom_rules_against_tasks(&items, &tasks)?;
        let now = Utc::now().to_rfc3339();
        let mut rows = Vec::new();
        for item in items {
            let time_scope_ids_json = to_json_i64_list(&item.time_scope_ids)?;
            let time_scope_labels_json = to_json_string_list(&item.time_scope_labels, "时段标签")?;
            let target_ids_json = to_json_string_list(&item.target_ids, "对象 ID")?;
            let target_labels_json = to_json_string_list(&item.target_labels, "对象标签")?;
            rows.push(crate::entity::invigilation_custom_rules::ActiveModel {
                action_type: sea_orm::ActiveValue::Set(item.action_type),
                teacher_id: sea_orm::ActiveValue::Set(item.teacher_id),
                teacher_name: sea_orm::ActiveValue::Set(item.teacher_name.trim().to_string()),
                time_scope_type: sea_orm::ActiveValue::Set(item.time_scope_type),
                time_scope_ids_json: sea_orm::ActiveValue::Set(time_scope_ids_json),
                time_scope_labels_json: sea_orm::ActiveValue::Set(time_scope_labels_json),
                task_scope_type: sea_orm::ActiveValue::Set(item.task_scope_type),
                target_scope_type: sea_orm::ActiveValue::Set(item.target_scope_type),
                target_ids_json: sea_orm::ActiveValue::Set(target_ids_json),
                target_labels_json: sea_orm::ActiveValue::Set(target_labels_json),
                created_at: sea_orm::ActiveValue::Set(now.clone()),
                ..Default::default()
            });
        }
        exam_staff_repo::replace_custom_rules(&db, rows).await?;
        Ok(SuccessResponse::ok())
    }
    .await;
    result.map_err(|e: AppError| e.to_string())
}

pub async fn get_latest_exam_staff_plan_overview(
    app: AppHandle,
) -> Result<ExamStaffPlanOverview, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let meta = exam_staff_repo::latest_plan_meta(&db).await?;
        Ok(ExamStaffPlanOverview {
            generated_at: meta.as_ref().map(|value| value.generated_at.clone()),
            session_count: meta.as_ref().map(|value| value.session_count).unwrap_or(0),
            task_count: meta.as_ref().map(|value| value.task_count).unwrap_or(0),
            assigned_count: meta.as_ref().map(|value| value.assigned_count).unwrap_or(0),
            unassigned_count: meta
                .as_ref()
                .map(|value| value.unassigned_count)
                .unwrap_or(0),
            warning_count: meta.as_ref().map(|value| value.warning_count).unwrap_or(0),
            imbalance_minutes: meta
                .as_ref()
                .map(|value| value.imbalance_minutes)
                .unwrap_or(0),
            solver_engine: meta
                .as_ref()
                .and_then(|value| SolverEngine::from_key(&value.solver_engine))
                .unwrap_or(SolverEngine::CpSat),
            optimality_status: meta
                .as_ref()
                .and_then(|value| OptimalityStatus::from_key(&value.optimality_status))
                .unwrap_or(OptimalityStatus::Feasible),
            solve_duration_ms: meta
                .as_ref()
                .map(|value| value.solve_duration_ms)
                .unwrap_or(0),
            fallback_reason: meta.as_ref().and_then(|value| {
                value
                    .fallback_reason
                    .as_deref()
                    .and_then(FallbackReason::from_key)
            }),
            fallback_pool_assignments: meta
                .as_ref()
                .map(|value| value.fallback_pool_assignments)
                .unwrap_or(0),
        })
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn list_latest_exam_staff_tasks(
    app: AppHandle,
    params: ListExamStaffTasksParams,
) -> Result<ListResult<ExamStaffTask>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(200).clamp(1, 1000);
        let rows = exam_staff_repo::list_tasks(
            &db,
            exam_staff_repo::TaskListFilters {
                session_id: params.session_id,
                role: params.role.map(|role| role.as_key().to_string()),
                status: params.status.map(|status| status.as_key().to_string()),
                page,
                page_size,
            },
        )
        .await?;
        let mut items = Vec::new();
        for row in rows.items {
            let task = row.task;
            let task_source = StaffTaskSource::from_key(&task.task_source)
                .ok_or_else(|| AppError::new(format!("无效的任务来源: {}", task.task_source)))?;
            let role = StaffRole::from_key(&task.role)
                .ok_or_else(|| AppError::new(format!("无效的岗位: {}", task.role)))?;
            let subject = Subject::from_key(&task.subject)
                .ok_or_else(|| AppError::new(format!("无效的科目: {}", task.subject)))?;
            let status = TaskStatus::from_key(&task.status)
                .ok_or_else(|| AppError::new(format!("无效的任务状态: {}", task.status)))?;
            let recommended_self_study_topic = self_study_topic_from_parts(
                task.recommended_self_study_topic_kind.clone(),
                task.recommended_self_study_topic_subjects_json.clone(),
                task.recommended_self_study_topic_label.clone(),
            )?;
            let assignment_tier = task
                .assignment_tier
                .as_deref()
                .and_then(AssignmentTier::from_key);
            items.push(ExamStaffTask {
                id: task.id,
                session_id: task.session_id,
                space_id: task.space_id,
                task_source,
                role,
                grade_name: task.grade_name,
                subject,
                space_name: task.space_name,
                floor: task.floor,
                start_at: task.start_at,
                end_at: task.end_at,
                duration_minutes: task.duration_minutes,
                recommended_self_study_topic,
                priority_self_study_chain: self_study_topic_chain_from_text(
                    &task.priority_self_study_chain_json,
                )?,
                assignment_tier,
                status,
                reason: task.reason,
                allowance_amount: task.allowance_amount,
                teacher_id: row.assignment.as_ref().map(|item| item.teacher_id),
                teacher_name: row.assignment.map(|item| item.teacher_name),
            });
        }
        Ok(ListResult {
            total: rows.total,
            items,
        })
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

pub async fn list_latest_teacher_duty_stats(
    app: AppHandle,
    params: ListTeacherDutyStatsParams,
) -> Result<ListResult<TeacherDutyStat>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        let keyword = params
            .keyword
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(200).clamp(1, 1000);
        let rows = exam_staff_repo::list_duty_stats(
            &db,
            exam_staff_repo::DutyStatFilters {
                keyword,
                page,
                page_size,
            },
        )
        .await?;
        Ok(ListResult {
            total: rows.total,
            items: rows
                .items
                .into_iter()
                .map(|row| TeacherDutyStat {
                    teacher_id: row.teacher_id,
                    teacher_name: row.teacher_name,
                    indoor_minutes: row.indoor_minutes,
                    outdoor_minutes: row.outdoor_minutes,
                    total_minutes: row.total_minutes,
                    task_count: row.task_count,
                    exam_room_task_count: row.exam_room_task_count,
                    self_study_task_count: row.self_study_task_count,
                    floor_rover_task_count: row.floor_rover_task_count,
                    allowance_total: row.allowance_total,
                    indoor_allowance_total: row.indoor_allowance_total,
                    outdoor_allowance_total: row.outdoor_allowance_total,
                    is_middle_manager: row.is_middle_manager == 1,
                })
                .collect(),
        })
    }
    .await;
    result.map_err(|error: AppError| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::Migrator;
    use crate::entity::{latest_exam_plan_sessions, latest_exam_plan_spaces};
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database};
    use sea_orm_migration::MigratorTrait;

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

    fn build_test_teacher_grade_subject_pairs(
        teachers: &[TeacherInfo],
    ) -> HashMap<i64, HashSet<(String, Subject)>> {
        let mut result = HashMap::new();
        for teacher in teachers {
            let mut pairs = HashSet::new();
            for class_name in &teacher.class_names {
                let grade_name = if class_name.starts_with("高一") {
                    "高一"
                } else if class_name.starts_with("高二") {
                    "高二"
                } else if class_name.starts_with("高三") {
                    "高三"
                } else {
                    continue;
                };
                for subject in &teacher.subjects {
                    pairs.insert((grade_name.to_string(), *subject));
                }
            }
            result.insert(teacher.id, pairs);
        }
        result
    }

    fn setup_build_staff_tasks_test_db() -> sea_orm::DatabaseConnection {
        tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:")
                .await
                .expect("in-memory sqlite should open");
            Migrator::up(&db, None)
                .await
                .expect("test schema should be migrated");
            db
        })
    }

    fn insert_test_plan_space(
        db: &sea_orm::DatabaseConnection,
        id: i64,
        session_id: i64,
        space_name: &str,
        floor: &str,
        sort_index: i64,
    ) {
        tauri::async_runtime::block_on(async {
            latest_exam_plan_spaces::ActiveModel {
                id: Set(id),
                session_id: Set(session_id),
                space_type: Set("exam_room".to_string()),
                space_source: Set("test".to_string()),
                grade_name: Set("高二".to_string()),
                subject: Set("英语".to_string()),
                space_name: Set(space_name.to_string()),
                original_class_name: Set(None),
                self_study_topic_kind: Set(None),
                self_study_topic_subjects_json: Set(None),
                self_study_topic_label: Set(None),
                building: Set(String::new()),
                floor: Set(floor.to_string()),
                capacity: Set(None),
                sort_index: Set(sort_index),
            }
            .insert(db)
            .await
            .expect("test space should be inserted");
        });
    }

    fn insert_test_plan_session(db: &sea_orm::DatabaseConnection, id: i64, subject: Subject) {
        tauri::async_runtime::block_on(async {
            latest_exam_plan_sessions::ActiveModel {
                id: Set(id),
                grade_name: Set("高二".to_string()),
                subject: Set(subject_label(subject).to_string()),
                is_foreign_group: Set(if exam_allocation::is_foreign_subject(subject) {
                    1
                } else {
                    0
                }),
                foreign_order: Set(None),
                participant_count: Set(0),
                exam_room_count: Set(0),
                self_study_room_count: Set(0),
            }
            .insert(db)
            .await
            .expect("test session should be inserted");
        });
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
            subject_avoidance_subjects: vec![subject],
            recommended_self_study_topic: None,
            priority_self_study_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
            rule_target_id: String::new(),
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
            subject_avoidance_subjects: vec![Subject::Biology],
            recommended_self_study_topic: Some(topic_subject(Subject::Physics)),
            priority_self_study_chain: vec![
                topic_subject(Subject::Physics),
                topic_subject(Subject::English),
            ],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
            rule_target_id: String::new(),
        }
    }

    #[test]
    fn test_teaching_classes_for_sessions_excludes_inactive_grades() {
        let teaching_classes = vec![
            TeachingClassRuntime {
                id: 1,
                grade_name: "高一".to_string(),
                class_name: "高一1班".to_string(),
                floor: "3层".to_string(),
            },
            TeachingClassRuntime {
                id: 2,
                grade_name: "高三".to_string(),
                class_name: "高三1班".to_string(),
                floor: "5层".to_string(),
            },
        ];
        let session_times = vec![SessionTimeRuntime {
            session_id: 1,
            grade_name: "高三".to_string(),
            subject: Subject::Chemistry,
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
        }];

        let active = teaching_classes_for_sessions(&teaching_classes, &session_times);

        assert_eq!(active.len(), 1);
        assert_eq!(active[0].grade_name, "高三");
    }

    #[test]
    fn test_build_staff_tasks_deduplicates_foreign_group_floor_rovers_per_floor() {
        let db = setup_build_staff_tasks_test_db();
        insert_test_plan_session(&db, 101, Subject::English);
        insert_test_plan_session(&db, 102, Subject::Russian);
        insert_test_plan_space(&db, 1, 101, "高二1考场", "3层", 1);
        insert_test_plan_space(&db, 2, 101, "高二2考场", "4层", 2);
        insert_test_plan_space(&db, 3, 102, "高二3考场", "3层", 1);
        insert_test_plan_space(&db, 4, 102, "高二4考场", "4层", 2);

        let session_times = vec![
            SessionTimeRuntime {
                session_id: 101,
                grade_name: "高二".to_string(),
                subject: Subject::English,
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
            },
            SessionTimeRuntime {
                session_id: 102,
                grade_name: "高二".to_string(),
                subject: Subject::Russian,
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
            },
        ];

        let tasks = tauri::async_runtime::block_on(build_staff_tasks(
            &db,
            &session_times,
            &test_runtime_config(),
            &HashMap::new(),
            &[],
        ))
        .expect("foreign-group tasks should build");

        let floor_rovers = tasks
            .iter()
            .filter(|task| task.role == StaffRole::FloorRover)
            .collect::<Vec<_>>();
        assert_eq!(floor_rovers.len(), 2, "三楼和四楼各只保留一个流动监考");
        assert!(floor_rovers.iter().any(|task| task.floor == "3层"));
        assert!(floor_rovers.iter().any(|task| task.floor == "4层"));
        for task in floor_rovers {
            assert_eq!(
                task.subject_avoidance_subjects,
                vec![Subject::English, Subject::Russian]
            );
        }
    }

    #[test]
    fn test_floor_rover_candidates_avoid_all_subjects_in_foreign_group() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "俄语老师".to_string(),
                subjects: HashSet::from([Subject::Russian]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let task = TaskBuild {
            session_id: Some(101),
            space_id: None,
            task_source: StaffTaskSource::Exam,
            role: StaffRole::FloorRover,
            grade_name: "高二".to_string(),
            subject: Subject::English,
            space_name: "3层 楼层流动".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            subject_avoidance_subjects: vec![Subject::English, Subject::Russian],
            recommended_self_study_topic: None,
            priority_self_study_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
            rule_target_id: String::new(),
        };

        let summary = build_task_candidate_summary(
            &task,
            &teachers,
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );

        assert_eq!(summary.candidates.len(), 1);
        assert_eq!(summary.candidates[0].teacher_id, 3);
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
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(
            summary
                .candidates
                .iter()
                .map(|item| item.teacher_id)
                .collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn test_candidate_summary_blocks_teachers_for_all_grade_subjects_in_same_slot() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高二数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高一英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高一2班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高二历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::English),
            ("高二".to_string(), Subject::Math),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let summary = super::build_task_candidate_summary(
            &sample_exam_task(Subject::English),
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert_eq!(
            summary
                .candidates
                .iter()
                .map(|item| item.teacher_id)
                .collect::<Vec<_>>(),
            vec![3]
        );
    }

    #[test]
    fn test_candidate_summary_blocks_teachers_for_cross_grade_slot_english_math() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高一英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高一1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高二数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高一历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高一3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::English),
            ("高二".to_string(), Subject::Math),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let summary_english_exam = super::build_task_candidate_summary(
            &sample_exam_task(Subject::English),
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert!(
            !summary_english_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 1),
            "高一英语老师不应出现在高一英语考试监考候选中"
        );
        assert!(
            !summary_english_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 2),
            "高二数学老师不应出现在高一英语考试监考候选中（同场有高二数学考试）"
        );
        assert!(
            summary_english_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 3),
            "高一历史老师应该可以监考高一英语考试"
        );
    }

    #[test]
    fn test_candidate_summary_blocks_teachers_for_cross_grade_slot_math_english() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高一数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::from(["高一1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高二英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高二物理老师".to_string(),
                subjects: HashSet::from([Subject::Physics]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::Math),
            ("高二".to_string(), Subject::English),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let mut math_task = sample_exam_task(Subject::Math);
        math_task.grade_name = "高一".to_string();
        let summary_math_exam = super::build_task_candidate_summary(
            &math_task,
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert!(
            !summary_math_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 1),
            "高一数学老师不应出现在高一数学考试监考候选中"
        );
        assert!(
            !summary_math_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 2),
            "高二英语老师不应出现在高一数学考试监考候选中（同场有高二英语考试）"
        );
        assert!(
            summary_math_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 3),
            "高二物理老师应该可以监考高一数学考试"
        );
    }

    #[test]
    fn test_candidate_summary_blocks_teachers_for_same_slot_geography() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高一地理老师".to_string(),
                subjects: HashSet::from([Subject::Geography]),
                class_names: HashSet::from(["高一1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高二地理老师".to_string(),
                subjects: HashSet::from([Subject::Geography]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高一历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高一3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::Geography),
            ("高二".to_string(), Subject::Geography),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let mut geo_task = sample_exam_task(Subject::Geography);
        geo_task.grade_name = "高一".to_string();
        let summary_geo_exam = super::build_task_candidate_summary(
            &geo_task,
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert!(
            !summary_geo_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 1),
            "高一地理老师不应出现在高一地理考试监考候选中"
        );
        assert!(
            !summary_geo_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 2),
            "高二地理老师不应出现在高一地理考试监考候选中（同场有高二地理考试）"
        );
        assert!(
            summary_geo_exam
                .candidates
                .iter()
                .any(|c| c.teacher_id == 3),
            "高一历史老师应该可以监考高一地理考试"
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
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
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
    fn test_self_study_blocks_teachers_for_slot_forbidden_grade_subjects() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高一英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高一1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高二数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高一历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高一3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::English),
            ("高二".to_string(), Subject::Math),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let self_study_task = sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy);
        let summary = super::build_task_candidate_summary(
            &self_study_task,
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert!(
            !summary.candidates.iter().any(|c| c.teacher_id == 1),
            "高一英语老师不应出现在自习监考候选中（同场有高一英语考试）"
        );
        assert!(
            !summary.candidates.iter().any(|c| c.teacher_id == 2),
            "高二数学老师不应出现在自习监考候选中（同场有高二数学考试）"
        );
        assert!(
            summary.candidates.iter().any(|c| c.teacher_id == 3),
            "高一历史老师应该可以参与自习监考"
        );
    }

    #[test]
    fn test_floor_rover_blocks_teachers_for_slot_forbidden_grade_subjects() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "高一英语老师".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::from(["高一1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "高二数学老师".to_string(),
                subjects: HashSet::from([Subject::Math]),
                class_names: HashSet::from(["高二1班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 3,
                name: "高一历史老师".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::from(["高一3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let slot_forbidden_grade_subjects = HashSet::from([
            ("高一".to_string(), Subject::English),
            ("高二".to_string(), Subject::Math),
        ]);
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();

        let floor_rover_task = TaskBuild {
            session_id: Some(101),
            space_id: None,
            task_source: StaffTaskSource::Exam,
            role: StaffRole::FloorRover,
            grade_name: "高一".to_string(),
            subject: Subject::English,
            space_name: "3层 楼层流动".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            subject_avoidance_subjects: vec![Subject::English],
            recommended_self_study_topic: None,
            priority_self_study_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
            rule_target_id: String::new(),
        };
        let summary = super::build_task_candidate_summary(
            &floor_rover_task,
            &teachers,
            &custom_rules,
            &test_runtime_config(),
            &slot_forbidden_grade_subjects,
            &teacher_grade_subject_pairs,
        );
        assert!(
            !summary.candidates.iter().any(|c| c.teacher_id == 1),
            "高一英语老师不应出现在楼层流动监考候选中（同场有高一英语考试）"
        );
        assert!(
            !summary.candidates.iter().any(|c| c.teacher_id == 2),
            "高二数学老师不应出现在楼层流动监考候选中（同场有高二数学考试）"
        );
        assert!(
            summary.candidates.iter().any(|c| c.teacher_id == 3),
            "高一历史老师应该可以参与楼层流动监考"
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
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert!(summary.candidates.is_empty());

        let mut config = test_runtime_config();
        config.middle_manager_exception_teacher_ids = HashSet::from([1_i64]);
        let enabled = build_task_candidate_summary(
            &sample_exam_task(Subject::Math),
            &middle_manager,
            &[],
            &config,
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(enabled.candidates.len(), 1);

        config.middle_manager_default_enabled = true;
        let disabled_again = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::FullSelfStudy),
            &middle_manager,
            &[],
            &config,
            &HashSet::new(),
            &HashMap::new(),
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
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(
            exam_linked.candidates[0].assignment_tier,
            Some(AssignmentTier::FallbackPool)
        );

        let full_self_study = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::FullSelfStudy),
            &teachers,
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(full_self_study.candidates.len(), 1);
        assert_eq!(full_self_study.candidates[0].teacher_id, 9);
    }

    #[test]
    fn test_exam_exclusion_only_blocks_exam_room_not_self_study_or_floor_rover() {
        let teachers = vec![TeacherInfo {
            id: 11,
            name: "可排老师".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::new(),
            homeroom_classes: HashSet::new(),
            is_middle_manager: false,
        }];
        let exclusion_rules = vec![GenerateExamStaffPlanCustomRule {
            action_type: RULE_ACTION_EXCLUDE.to_string(),
            teacher_id: 11,
            teacher_name: None,
            time_scope_type: RULE_TIME_SCOPE_EXAM_SESSION.to_string(),
            time_scope_ids: vec![1],
            time_scope_labels: Vec::new(),
            task_scope_type: RULE_TASK_SCOPE_EXAM_ROOM.to_string(),
            target_scope_type: RULE_TARGET_SCOPE_ALL.to_string(),
            target_ids: Vec::new(),
            target_labels: Vec::new(),
        }];

        let exam_room_summary = build_task_candidate_summary(
            &sample_exam_task(Subject::Math),
            &teachers,
            &exclusion_rules,
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert!(exam_room_summary.candidates.is_empty());

        let self_study_summary = build_task_candidate_summary(
            &sample_self_study_task(StaffTaskSource::ExamLinkedSelfStudy),
            &teachers,
            &exclusion_rules,
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(self_study_summary.candidates.len(), 1);
        assert_eq!(self_study_summary.candidates[0].teacher_id, 11);

        let mut floor_rover_task = sample_exam_task(Subject::Math);
        floor_rover_task.role = StaffRole::FloorRover;
        floor_rover_task.space_id = None;
        floor_rover_task.space_name = "4层 楼层流动".to_string();
        floor_rover_task.subject_avoidance_subjects = vec![Subject::Math];
        let floor_rover_summary = build_task_candidate_summary(
            &floor_rover_task,
            &teachers,
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
        );
        assert_eq!(floor_rover_summary.candidates.len(), 1);
        assert_eq!(floor_rover_summary.candidates[0].teacher_id, 11);
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
        foreign_task.recommended_self_study_topic =
            Some(exam_allocation::build_foreign_group_self_study_topic(vec![
                Subject::English,
                Subject::Russian,
            ]));
        foreign_task.priority_self_study_chain =
            vec![exam_allocation::build_foreign_group_self_study_topic(vec![
                Subject::English,
                Subject::Russian,
            ])];
        let foreign_summary = build_task_candidate_summary(
            &foreign_task,
            &teachers,
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
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
            &[],
            &test_runtime_config(),
            &HashSet::new(),
            &HashMap::new(),
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
                subject_avoidance_subjects: vec![Subject::Math],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
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
                subject_avoidance_subjects: vec![Subject::Biology],
                recommended_self_study_topic: Some(topic_subject(Subject::English)),
                priority_self_study_chain: vec![topic_subject(Subject::English)],
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
            },
        ];
        let empty_custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let cp_sat_attempt = solve_with_cp_sat(
            &tasks,
            &teachers,
            &empty_custom_rules,
            &test_runtime_config(),
            &teacher_grade_subject_pairs,
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
                subject_avoidance_subjects: vec![Subject::Math],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
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
                subject_avoidance_subjects: vec![Subject::Biology],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
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
                subject_avoidance_subjects: vec![Subject::Physics],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
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
                subject_avoidance_subjects: vec![Subject::English],
                recommended_self_study_topic: Some(topic_subject(Subject::English)),
                priority_self_study_chain: vec![topic_subject(Subject::English)],
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
            },
        ];

        let empty_custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let cp_sat_attempt = solve_with_cp_sat(
            &tasks,
            &teachers,
            &empty_custom_rules,
            &test_runtime_config(),
            &teacher_grade_subject_pairs,
            None,
        );
        let cp_sat_plan = cp_sat_attempt.plan.expect("cp-sat should produce a plan");

        assert_eq!(cp_sat_plan.metrics.unassigned_count, 0);
        assert_eq!(cp_sat_plan.metrics.imbalance_minutes, 0);
        assert_eq!(cp_sat_plan.metrics.invigilation_minutes_gap, 0);
        assert_eq!(cp_sat_plan.metrics.self_study_minutes_gap, 0);
    }

    #[test]
    fn test_cp_sat_prioritizes_exam_room_over_floor_rover_when_teachers_are_insufficient() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "老师甲".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::new(),
            homeroom_classes: HashSet::new(),
            is_middle_manager: false,
        }];
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
                subject_avoidance_subjects: vec![Subject::Math],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
            },
            TaskBuild {
                session_id: Some(1),
                space_id: None,
                task_source: StaffTaskSource::Exam,
                role: StaffRole::FloorRover,
                grade_name: "高二".to_string(),
                subject: Subject::Math,
                space_name: "4层 楼层流动".to_string(),
                floor: "4层".to_string(),
                start_at: "2026-03-24T08:00".to_string(),
                end_at: "2026-03-24T10:00".to_string(),
                start_ts: 1_000,
                end_ts: 2_000,
                duration_minutes: 120,
                subject_avoidance_subjects: vec![Subject::Math],
                recommended_self_study_topic: None,
                priority_self_study_chain: Vec::new(),
                day_key: "2026-03-24".to_string(),
                half_day: HalfDay::Morning,
                rule_target_id: String::new(),
            },
        ];

        let empty_custom_rules = Vec::<GenerateExamStaffPlanCustomRule>::new();
        let teacher_grade_subject_pairs = build_test_teacher_grade_subject_pairs(&teachers);
        let cp_sat_attempt = solve_with_cp_sat(
            &tasks,
            &teachers,
            &empty_custom_rules,
            &test_runtime_config(),
            &teacher_grade_subject_pairs,
            None,
        );
        let cp_sat_plan = cp_sat_attempt.plan.expect("cp-sat should produce a plan");

        assert_eq!(cp_sat_plan.metrics.unassigned_count, 1);
        assert!(
            cp_sat_plan.records.iter().any(|record| {
                record.task.role == StaffRole::ExamRoomInvigilator && record.teacher_id.is_some()
            }),
            "考场监考应优先被分配"
        );
        assert!(
            cp_sat_plan.records.iter().any(|record| {
                record.task.role == StaffRole::FloorRover && record.teacher_id.is_none()
            }),
            "楼层流动在老师不足时允许不分配"
        );
    }

    #[test]
    #[ignore = "manual integration test against the real sqlite database"]
    fn test_run_real_db_staff_plan_manual() {
        let db_path = std::env::var("ACADEMIC_REAL_DB_PATH")
            .expect("ACADEMIC_REAL_DB_PATH must point to scores.sqlite3");
        let db_path = std::path::PathBuf::from(db_path);
        let db_url = format!(
            "sqlite://{}?mode=rwc",
            db_path.to_string_lossy().replace('\\', "/")
        );
        let db = tauri::async_runtime::block_on(async {
            let db = Database::connect(db_url)
                .await
                .expect("open real sqlite db");
            Migrator::up(&db, None).await.expect("ensure schema");
            db
        });

        let config_row =
            tauri::async_runtime::block_on(exam_staff_repo::get_config(&db)).expect("load config");
        let mut config = build_config_from_payload(&GenerateExamStaffPlanPayload {
            default_exam_room_required_count: config_row
                .as_ref()
                .map(|row| row.default_exam_room_required_count)
                .unwrap_or(1),
            indoor_allowance_per_minute: config_row
                .as_ref()
                .map(|row| row.indoor_allowance_per_minute)
                .unwrap_or(0.5),
            outdoor_allowance_per_minute: config_row
                .as_ref()
                .map(|row| row.outdoor_allowance_per_minute)
                .unwrap_or(0.3),
            custom_rules: Vec::new(),
        });
        tauri::async_runtime::block_on(hydrate_runtime_middle_manager_config(&db, &mut config))
            .expect("hydrate config");
        config.self_study_class_subjects =
            tauri::async_runtime::block_on(load_self_study_class_subjects(&db))
                .expect("load self study subjects");

        let custom_rules = tauri::async_runtime::block_on(exam_staff_repo::list_custom_rules(&db))
            .expect("load custom rules")
            .into_iter()
            .map(|row| GenerateExamStaffPlanCustomRule {
                action_type: row.action_type,
                teacher_id: row.teacher_id,
                teacher_name: Some(row.teacher_name),
                time_scope_type: row.time_scope_type,
                time_scope_ids: parse_json_i64_list(&row.time_scope_ids_json),
                time_scope_labels: parse_json_string_list(&row.time_scope_labels_json),
                task_scope_type: row.task_scope_type,
                target_scope_type: row.target_scope_type,
                target_ids: parse_json_string_list(&row.target_ids_json),
                target_labels: parse_json_string_list(&row.target_labels_json),
            })
            .collect::<Vec<_>>();

        let log_path = db_path
            .parent()
            .expect("db parent")
            .join("logs")
            .join("app.log");
        let result = tauri::async_runtime::block_on(generate_latest_exam_staff_plan_internal(
            &db,
            config,
            custom_rules,
            Some(log_path.as_path()),
            None,
        ))
        .expect("generate staff plan on real db");

        println!(
            "REAL_DB_STAFF_PLAN {}",
            serde_json::to_string(&result).expect("serialize result")
        );
        println!("REAL_DB_APP_LOG {}", log_path.display());
        assert!(result.task_count > 0);
    }
}
