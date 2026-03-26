use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::exam_allocation::{self, SuccessResponse};
use crate::score::{self, AppError, ListResult, Subject};

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
    recommended_subject: Option<Subject>,
    priority_subject_chain: Vec<Subject>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn same_period_count(&self, half_day: HalfDay) -> i64 {
        match half_day {
            HalfDay::Morning => self.morning_tasks,
            HalfDay::Afternoon => self.afternoon_tasks,
        }
    }

    fn other_period_count(&self, half_day: HalfDay) -> i64 {
        match half_day {
            HalfDay::Morning => self.afternoon_tasks,
            HalfDay::Afternoon => self.morning_tasks,
        }
    }

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
    weighted_minutes: i64,
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
    recommended_subject: Option<Subject>,
    priority_subject_chain: Vec<Subject>,
    day_key: String,
    half_day: HalfDay,
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

fn role_effort_weight(role: StaffRole) -> i64 {
    match role {
        StaffRole::ExamRoomInvigilator => 3,
        StaffRole::FloorRover => 2,
        StaffRole::SelfStudySupervisor => 1,
    }
}

fn subject_chain_to_text(chain: &[Subject]) -> String {
    chain
        .iter()
        .map(|subject| subject.as_key())
        .collect::<Vec<_>>()
        .join(",")
}

fn subject_chain_from_text(value: &str) -> Vec<Subject> {
    value
        .split(',')
        .filter_map(|item| Subject::from_key(item.trim()))
        .collect()
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
    if let Some((default_enabled, exception_json, self_study_date, self_study_start_time, self_study_end_time)) = persisted {
        config.middle_manager_default_enabled = default_enabled == 1;
        config.middle_manager_exception_teacher_ids = serde_json::from_str::<Vec<i64>>(&exception_json)
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

fn is_middle_manager_enabled(
    teacher: &TeacherInfo,
    config: &RuntimeInvigilationConfig,
) -> bool {
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

fn load_self_study_class_subjects(
    conn: &Connection,
) -> Result<HashMap<i64, Subject>, AppError> {
    let json_text: String = conn
        .query_row(
            "SELECT COALESCE(self_study_class_subjects_json, '[]') FROM invigilation_config_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "[]".to_string());
    let items = serde_json::from_str::<Vec<PersistedSelfStudyClassSubject>>(&json_text)
        .unwrap_or_default();
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

fn load_teaching_classes(
    conn: &Connection,
) -> Result<Vec<TeachingClassRuntime>, AppError> {
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

fn load_exam_room_requirement(
    default_count: i64,
) -> Result<i64, AppError> {
    Ok(default_count.max(1))
}

fn overlap(a_start: i64, a_end: i64, b_start: i64, b_end: i64) -> bool {
    a_start < b_end && b_start < a_end
}

fn is_teacher_available(state: &TeacherRuntimeState, start_ts: i64, end_ts: i64) -> bool {
    !state
        .busy_ranges
        .iter()
        .any(|(busy_start, busy_end)| overlap(*busy_start, *busy_end, start_ts, end_ts))
}

fn build_priority_subject_chain(
    current: &SessionTimeRuntime,
    class_name: &str,
    sessions_by_grade: &HashMap<String, Vec<SessionTimeRuntime>>,
    class_subject_map: &HashMap<(String, String), HashSet<Subject>>,
) -> Vec<Subject> {
    let mut chain = Vec::new();
    let Some(class_subjects) =
        class_subject_map.get(&(current.grade_name.clone(), class_name.to_string()))
    else {
        return chain;
    };
    let Some(grade_sessions) = sessions_by_grade.get(&current.grade_name) else {
        return chain;
    };
    let mut seen = HashSet::new();
    for session in grade_sessions {
        if session.start_ts <= current.start_ts {
            continue;
        }
        if !class_subjects.contains(&session.subject) {
            continue;
        }
        if seen.insert(session.subject.as_key()) {
            chain.push(session.subject);
            break;
        }
    }
    chain
}

fn current_self_study_surplus(state: &TeacherRuntimeState) -> i64 {
    state.self_study_task_count - (state.exam_room_task_count + state.floor_rover_task_count)
}

fn projected_self_study_surplus(state: &TeacherRuntimeState, role: StaffRole) -> i64 {
    match role {
        StaffRole::SelfStudySupervisor => {
            state.self_study_task_count + 1
                - (state.exam_room_task_count + state.floor_rover_task_count)
        }
        StaffRole::ExamRoomInvigilator => {
            state.self_study_task_count
                - (state.exam_room_task_count + 1 + state.floor_rover_task_count)
        }
        StaffRole::FloorRover => {
            state.self_study_task_count
                - (state.exam_room_task_count + state.floor_rover_task_count + 1)
        }
    }
}

fn projected_weighted_minutes(state: &TeacherRuntimeState, task: &TaskBuild) -> i64 {
    state.weighted_minutes + task.duration_minutes * role_effort_weight(task.role)
}

fn projected_total_minutes(state: &TeacherRuntimeState, task: &TaskBuild) -> i64 {
    state.total_minutes + task.duration_minutes
}

fn period_spread_penalty(state: &TeacherRuntimeState, task: &TaskBuild) -> i32 {
    let Some(day_load) = state.day_half_loads.get(&task.day_key) else {
        return 0;
    };
    if day_load.same_period_count(task.half_day) == 0
        && day_load.other_period_count(task.half_day) > 0
    {
        1
    } else {
        0
    }
}

fn same_period_task_count(state: &TeacherRuntimeState, task: &TaskBuild) -> i64 {
    state
        .day_half_loads
        .get(&task.day_key)
        .map(|day_load| day_load.same_period_count(task.half_day))
        .unwrap_or(0)
}

fn choose_teacher_from_candidates(
    task: &TaskBuild,
    teachers: &[TeacherInfo],
    candidate_ids: &[i64],
    runtime: &HashMap<i64, TeacherRuntimeState>,
) -> Option<i64> {
    let teacher_by_id: HashMap<i64, &TeacherInfo> = teachers
        .iter()
        .map(|teacher| (teacher.id, teacher))
        .collect();
    let mut sorted = candidate_ids.to_vec();
    sorted.sort_by(|a, b| {
        let a_state = runtime.get(a).cloned().unwrap_or_default();
        let b_state = runtime.get(b).cloned().unwrap_or_default();
        match task.role {
            StaffRole::SelfStudySupervisor => (
                period_spread_penalty(&a_state, task),
                Reverse(same_period_task_count(&a_state, task)),
                projected_self_study_surplus(&a_state, task.role).max(0),
                projected_weighted_minutes(&a_state, task),
                projected_total_minutes(&a_state, task),
                a_state.self_study_task_count,
                a_state.task_count,
                *a,
            )
                .cmp(&(
                    period_spread_penalty(&b_state, task),
                    Reverse(same_period_task_count(&b_state, task)),
                    projected_self_study_surplus(&b_state, task.role).max(0),
                    projected_weighted_minutes(&b_state, task),
                    projected_total_minutes(&b_state, task),
                    b_state.self_study_task_count,
                    b_state.task_count,
                    *b,
                )),
            StaffRole::ExamRoomInvigilator | StaffRole::FloorRover => (
                period_spread_penalty(&a_state, task),
                if current_self_study_surplus(&a_state) > 0 {
                    0
                } else {
                    1
                },
                Reverse(current_self_study_surplus(&a_state).max(0)),
                Reverse(same_period_task_count(&a_state, task)),
                projected_weighted_minutes(&a_state, task),
                projected_total_minutes(&a_state, task),
                a_state.exam_room_task_count + a_state.floor_rover_task_count,
                a_state.task_count,
                *a,
            )
                .cmp(&(
                    period_spread_penalty(&b_state, task),
                    if current_self_study_surplus(&b_state) > 0 {
                        0
                    } else {
                        1
                    },
                    Reverse(current_self_study_surplus(&b_state).max(0)),
                    Reverse(same_period_task_count(&b_state, task)),
                    projected_weighted_minutes(&b_state, task),
                    projected_total_minutes(&b_state, task),
                    b_state.exam_room_task_count + b_state.floor_rover_task_count,
                    b_state.task_count,
                    *b,
                )),
        }
    });
    sorted
        .into_iter()
        .find(|teacher_id| teacher_by_id.contains_key(teacher_id))
}

fn choose_teacher_for_task(
    task: &TaskBuild,
    teachers: &[TeacherInfo],
    runtime: &HashMap<i64, TeacherRuntimeState>,
    self_study_class_name: Option<&str>,
    exclusion_pairs: &HashSet<(i64, i64)>,
    config: &RuntimeInvigilationConfig,
) -> (Option<i64>, Option<String>) {
    let active_teachers: Vec<&TeacherInfo> = teachers
        .iter()
        .filter(|teacher| is_teacher_enabled_for_task_source(teacher, task.task_source, config))
        .collect();
    if active_teachers.is_empty() {
        return (None, Some("no_available_teacher".to_string()));
    }

    let mut time_filtered = Vec::<&TeacherInfo>::new();
    for teacher in &active_teachers {
        if let Some(session_id) = task.session_id {
            if task.task_source != StaffTaskSource::FullSelfStudy
                && exclusion_pairs.contains(&(teacher.id, session_id))
            {
                continue;
            }
        }
        let state = runtime.get(&teacher.id).cloned().unwrap_or_default();
        if is_teacher_available(&state, task.start_ts, task.end_ts) {
            time_filtered.push(*teacher);
        }
    }
    if time_filtered.is_empty() {
        return (None, Some("time_conflict".to_string()));
    }

    let mut available = Vec::<&TeacherInfo>::new();
    for teacher in time_filtered {
        if task.role == StaffRole::ExamRoomInvigilator && teacher.subjects.contains(&task.subject) {
            continue;
        }
        available.push(teacher);
    }

    if available.is_empty() {
        if task.role == StaffRole::ExamRoomInvigilator {
            return (None, Some("subject_conflict".to_string()));
        }
        return (None, Some("no_available_teacher".to_string()));
    }

    if task.role == StaffRole::SelfStudySupervisor {
        if let Some(class_name) = self_study_class_name {
            let next_subject = task
                .recommended_subject
                .or_else(|| task.priority_subject_chain.first().copied());
            let level1_all: Vec<i64> = active_teachers
                .iter()
                .filter(|teacher| {
                    if let Some(subject) = next_subject {
                        teacher.class_names.contains(class_name)
                            && teacher.subjects.contains(&subject)
                    } else {
                        false
                    }
                })
                .map(|teacher| teacher.id)
                .collect();
            let level1_available: Vec<i64> = available
                .iter()
                .filter(|teacher| {
                    if let Some(subject) = next_subject {
                        teacher.class_names.contains(class_name)
                            && teacher.subjects.contains(&subject)
                    } else {
                        false
                    }
                })
                .map(|teacher| teacher.id)
                .collect();
            if !level1_available.is_empty() {
                let teacher_id =
                    choose_teacher_from_candidates(task, teachers, &level1_available, runtime);
                return (teacher_id, None);
            }

            let level2_all: Vec<i64> = active_teachers
                .iter()
                .filter(|teacher| teacher.homeroom_classes.contains(class_name))
                .map(|teacher| teacher.id)
                .collect();
            let level2_available: Vec<i64> = available
                .iter()
                .filter(|teacher| teacher.homeroom_classes.contains(class_name))
                .map(|teacher| teacher.id)
                .collect();
            if !level2_available.is_empty() {
                let teacher_id =
                    choose_teacher_from_candidates(task, teachers, &level2_available, runtime);
                return (teacher_id, None);
            }

            if !level1_all.is_empty() {
                if level2_all.is_empty() {
                    return (None, Some("no_homeroom_teacher".to_string()));
                }
                return (None, Some("time_conflict".to_string()));
            }
            if level2_all.is_empty() {
                return (
                    None,
                    Some(if task.task_source == StaffTaskSource::FullSelfStudy {
                        "no_self_study_subject_teacher".to_string()
                    } else {
                        "no_next_subject_teacher".to_string()
                    }),
                );
            }
            return (None, Some("time_conflict".to_string()));
        }
    }

    let available_ids: Vec<i64> = available.iter().map(|teacher| teacher.id).collect();
    let selected_teacher_id =
        choose_teacher_from_candidates(task, teachers, &available_ids, runtime);
    if selected_teacher_id.is_some() {
        (selected_teacher_id, None)
    } else {
        (None, Some("no_available_teacher".to_string()))
    }
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
) -> Result<Vec<(i64, SpaceType, String, Option<String>, Option<Subject>, String)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, space_type, space_name, original_class_name, self_study_subject, floor FROM latest_exam_plan_spaces WHERE session_id = ?1 ORDER BY sort_index ASC, id ASC",
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
        Ok((
            row.get::<_, i64>(0)?,
            space_type,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?
                .and_then(|value| Subject::from_key(&value)),
            row.get::<_, String>(5)?,
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
            state.exam_room_task_count += 1;
        }
        StaffRole::SelfStudySupervisor => {
            state.indoor_minutes += task.duration_minutes;
            state.self_study_task_count += 1;
        }
        StaffRole::FloorRover => {
            state.outdoor_minutes += task.duration_minutes;
            state.floor_rover_task_count += 1;
        }
    }
    state.total_minutes += task.duration_minutes;
    state.weighted_minutes += task.duration_minutes * role_effort_weight(task.role);
    state.task_count += 1;
    state.busy_ranges.push((task.start_ts, task.end_ts));
    state
        .day_half_loads
        .entry(task.day_key.clone())
        .or_default()
        .add_task(task.half_day);
}

fn generate_latest_exam_staff_plan_internal(
    conn: &mut Connection,
    invigilation_config: RuntimeInvigilationConfig,
    exclusion_pairs: HashSet<(i64, i64)>,
) -> Result<GenerateLatestExamStaffPlanResult, AppError> {
    let session_times = load_session_times_runtime(conn)?;
    let teachers = load_teacher_pool(conn)?;
    let class_subject_map = load_class_subject_map(conn)?;
    let teaching_classes = load_teaching_classes(conn)?;

    let mut sessions_by_grade: HashMap<String, Vec<SessionTimeRuntime>> = HashMap::new();
    for session in &session_times {
        sessions_by_grade
            .entry(session.grade_name.clone())
            .or_default()
            .push(session.clone());
    }
    for session_list in sessions_by_grade.values_mut() {
        session_list.sort_by(|a, b| {
            a.start_ts
                .cmp(&b.start_ts)
                .then(a.session_id.cmp(&b.session_id))
        });
    }

    let mut tasks = Vec::<TaskBuild>::new();
    for session in &session_times {
        let spaces = load_spaces_for_session(conn, session.session_id)?;
        if spaces.is_empty() {
            return Err(AppError::new(format!(
                "场次 {} 无可用空间",
                session.session_id
            )));
        }

        let mut floors = HashSet::<String>::new();
        let (day_key, half_day) = parse_day_slot(&session.start_at)?;
        for (space_id, space_type, space_name, original_class_name, self_study_subject, floor) in &spaces {
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
                            recommended_subject: None,
                            priority_subject_chain: Vec::new(),
                            day_key: day_key.clone(),
                            half_day,
                        });
                    }
                }
                SpaceType::SelfStudyRoom => {
                    let class_name = original_class_name
                        .clone()
                        .unwrap_or_else(|| space_name.clone());
                    let (recommended_subject, chain) = if let Some(saved_subject) = self_study_subject
                    {
                        (Some(*saved_subject), vec![*saved_subject])
                    } else {
                        let chain = build_priority_subject_chain(
                            session,
                            &class_name,
                            &sessions_by_grade,
                            &class_subject_map,
                        );
                        let recommended_subject = chain.first().copied();
                        (recommended_subject, chain)
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
                        recommended_subject,
                        priority_subject_chain: chain,
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
                recommended_subject: None,
                priority_subject_chain: Vec::new(),
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

        for teaching_class in &teaching_classes {
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
                recommended_subject: Some(subject),
                priority_subject_chain: vec![subject],
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

    let tx = conn.transaction()?;
    clear_latest_staff_plan(&tx)?;

    let mut runtime: HashMap<i64, TeacherRuntimeState> = HashMap::new();
    for teacher in &teachers {
        runtime.insert(teacher.id, TeacherRuntimeState::default());
    }

    let teacher_by_id: HashMap<i64, &TeacherInfo> = teachers
        .iter()
        .map(|teacher| (teacher.id, teacher))
        .collect();
    let generated_at = Utc::now().to_rfc3339();
    let mut assigned_count = 0_i64;
    let mut unassigned_count = 0_i64;

    for task in &tasks {
        let self_study_class_name = if task.role == StaffRole::SelfStudySupervisor {
            Some(task.space_name.as_str())
        } else {
            None
        };
        let (selected_teacher_id, reason) =
            choose_teacher_for_task(
                task,
                &teachers,
                &runtime,
                self_study_class_name,
                &exclusion_pairs,
                &invigilation_config,
            );
        let status = if selected_teacher_id.is_some() {
            TaskStatus::Assigned
        } else {
            TaskStatus::Unassigned
        };
        let allowance_amount = if selected_teacher_id.is_some() {
            round_to_two(
                (task.duration_minutes as f64)
                    * allowance_rate_for_role(&invigilation_config, task.role),
            )
        } else {
            0.0
        };

        tx.execute(
            r#"
            INSERT INTO latest_exam_staff_tasks
            (session_id, space_id, task_source, role, grade_name, subject, space_name, floor, start_at, end_at, duration_minutes, recommended_subject, priority_subject_chain, status, reason, allowance_amount)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            "#,
            params![
                task.session_id,
                task.space_id,
                task.task_source.as_key(),
                task.role.as_key(),
                task.grade_name,
                task.subject.as_key(),
                task.space_name,
                task.floor,
                task.start_at,
                task.end_at,
                task.duration_minutes,
                task.recommended_subject.map(|subject| subject.as_key().to_string()),
                subject_chain_to_text(&task.priority_subject_chain),
                status.as_key(),
                reason,
                allowance_amount
            ],
        )?;
        let task_id = tx.last_insert_rowid();

        if let Some(teacher_id) = selected_teacher_id {
            if let Some(teacher) = teacher_by_id.get(&teacher_id) {
                tx.execute(
                    "INSERT INTO latest_exam_staff_assignments (task_id, teacher_id, teacher_name, assigned_at) VALUES (?1, ?2, ?3, ?4)",
                    params![task_id, teacher_id, teacher.name, generated_at],
                )?;
                if let Some(state) = runtime.get_mut(&teacher_id) {
                    apply_assignment_to_runtime(state, task);
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
                assigned_count += 1;
            } else {
                unassigned_count += 1;
            }
        } else {
            unassigned_count += 1;
        }
    }

    let mut max_total = 0_i64;
    let mut min_total = i64::MAX;
    for teacher in &teachers {
        let state = runtime.get(&teacher.id).cloned().unwrap_or_default();
        max_total = max_total.max(state.total_minutes);
        min_total = min_total.min(state.total_minutes);
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
    let imbalance = if teachers.is_empty() {
        0
    } else {
        max_total.saturating_sub(min_total)
    };
    let warning_count = unassigned_count + if imbalance > 90 { 1 } else { 0 };

    tx.execute(
        "INSERT INTO latest_exam_staff_plan_meta (id, generated_at, session_count, task_count, assigned_count, unassigned_count, warning_count, imbalance_minutes) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            generated_at,
            session_times.len() as i64,
            tasks.len() as i64,
            assigned_count,
            unassigned_count,
            warning_count,
            imbalance
        ],
    )?;
    tx.commit()?;

    Ok(GenerateLatestExamStaffPlanResult {
        generated_at,
        task_count: tasks.len() as i64,
        assigned_count,
        unassigned_count,
        imbalance_minutes: imbalance,
        warning_count,
    })
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
    let result = (|| -> Result<GenerateLatestExamStaffPlanResult, AppError> {
        let mut conn = score::open_connection(&app)?;
        exam_allocation::ensure_schema(&conn)?;
        let mut config = build_config_from_payload(&payload);
        hydrate_runtime_middle_manager_config(&conn, &mut config)?;
        config.self_study_class_subjects = load_self_study_class_subjects(&conn)?;
        let exclusion_pairs = payload
            .staff_exclusions
            .iter()
            .filter(|item| item.teacher_id > 0 && item.session_id > 0)
            .map(|item| (item.teacher_id, item.session_id))
            .collect::<HashSet<_>>();
        generate_latest_exam_staff_plan_internal(&mut conn, config, exclusion_pairs)
    })();
    result.map_err(|error| error.to_string())
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
        let meta: Option<(String, i64, i64, i64, i64, i64, i64)> = conn
            .query_row(
                "SELECT generated_at, session_count, task_count, assigned_count, unassigned_count, warning_count, imbalance_minutes FROM latest_exam_staff_plan_meta WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
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
              t.start_at, t.end_at, t.duration_minutes, t.recommended_subject, t.priority_subject_chain, t.status, t.reason, t.allowance_amount,
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
            let status_key: String = row.get(14)?;
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
                    14,
                    "status".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let recommended_subject = row
                .get::<_, Option<String>>(12)?
                .as_deref()
                .and_then(Subject::from_key);
            let chain_text: Option<String> = row.get(13)?;
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
                recommended_subject,
                priority_subject_chain: chain_text
                    .as_deref()
                    .map(subject_chain_from_text)
                    .unwrap_or_default(),
                status,
                reason: row.get(15)?,
                allowance_amount: row.get(16)?,
                teacher_id: row.get(17)?,
                teacher_name: row.get(18)?,
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

    #[test]
    fn test_choose_teacher_exam_room_subject_conflict() {
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
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: None,
            priority_subject_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            None,
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(2));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_choose_teacher_self_study_priority_chain() {
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
                name: "物理老师".to_string(),
                subjects: HashSet::from([Subject::Physics]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::ExamLinkedSelfStudy,
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
            recommended_subject: Some(Subject::Physics),
            priority_subject_chain: vec![Subject::Physics, Subject::English],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高二3班"),
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(2));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_choose_teacher_self_study_homeroom_fallback() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "班主任".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::new(),
            homeroom_classes: HashSet::from(["高二3班".to_string()]),
            is_middle_manager: false,
        }];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::ExamLinkedSelfStudy,
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
            recommended_subject: Some(Subject::Physics),
            priority_subject_chain: vec![Subject::Physics],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高二3班"),
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(1));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_choose_teacher_middle_manager_excluded() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "中层".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::from(["高一1班".to_string()]),
            homeroom_classes: HashSet::from(["高一1班".to_string()]),
            is_middle_manager: true,
        }];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::ExamLinkedSelfStudy,
            role: StaffRole::SelfStudySupervisor,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1班".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: Some(Subject::Chinese),
            priority_subject_chain: vec![Subject::Chinese],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高一1班"),
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, None);
        assert_eq!(reason, Some("no_available_teacher".to_string()));
    }

    #[test]
    fn test_choose_teacher_middle_manager_exception_enabled() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "中层".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::new(),
            homeroom_classes: HashSet::new(),
            is_middle_manager: true,
        }];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T08:00".to_string(),
            end_at: "2026-03-24T10:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: None,
            priority_subject_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let mut config = test_runtime_config();
        config.middle_manager_exception_teacher_ids = HashSet::from([1_i64]);
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            None,
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(1));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_self_study_prefers_teacher_with_less_self_study_surplus() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "物理老师A".to_string(),
                subjects: HashSet::from([Subject::Physics]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "物理老师B".to_string(),
                subjects: HashSet::from([Subject::Physics]),
                class_names: HashSet::from(["高二3班".to_string()]),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let mut runtime = HashMap::<i64, TeacherRuntimeState>::new();
        runtime.insert(
            1,
            TeacherRuntimeState {
                self_study_task_count: 2,
                exam_room_task_count: 0,
                ..TeacherRuntimeState::default()
            },
        );
        runtime.insert(
            2,
            TeacherRuntimeState {
                self_study_task_count: 0,
                exam_room_task_count: 1,
                ..TeacherRuntimeState::default()
            },
        );
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::ExamLinkedSelfStudy,
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
            recommended_subject: Some(Subject::Physics),
            priority_subject_chain: vec![Subject::Physics],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Morning,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高二3班"),
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(2));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_hard_duty_prefers_teacher_needing_compensation() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "老师A".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "老师B".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let mut runtime = HashMap::<i64, TeacherRuntimeState>::new();
        runtime.insert(
            1,
            TeacherRuntimeState {
                self_study_task_count: 2,
                ..TeacherRuntimeState::default()
            },
        );
        runtime.insert(2, TeacherRuntimeState::default());
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T14:00".to_string(),
            end_at: "2026-03-24T16:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: None,
            priority_subject_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Afternoon,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            None,
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(1));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_same_half_day_is_preferred() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "老师A".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "老师B".to_string(),
                subjects: HashSet::from([Subject::History]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let mut runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let mut first_state = TeacherRuntimeState::default();
        first_state.day_half_loads.insert(
            "2026-03-24".to_string(),
            DayHalfLoad {
                morning_tasks: 1,
                afternoon_tasks: 0,
            },
        );
        runtime.insert(1, first_state);
        let mut second_state = TeacherRuntimeState::default();
        second_state.day_half_loads.insert(
            "2026-03-24".to_string(),
            DayHalfLoad {
                morning_tasks: 0,
                afternoon_tasks: 1,
            },
        );
        runtime.insert(2, second_state);
        let task = TaskBuild {
            session_id: Some(1),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T14:00".to_string(),
            end_at: "2026-03-24T16:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: None,
            priority_subject_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Afternoon,
        };
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            None,
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, Some(2));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_choose_teacher_respects_exclusion() {
        let teachers = vec![
            TeacherInfo {
                id: 1,
                name: "老师A".to_string(),
                subjects: HashSet::from([Subject::Chinese]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
            TeacherInfo {
                id: 2,
                name: "老师B".to_string(),
                subjects: HashSet::from([Subject::English]),
                class_names: HashSet::new(),
                homeroom_classes: HashSet::new(),
                is_middle_manager: false,
            },
        ];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: Some(99),
            space_id: Some(1),
            task_source: StaffTaskSource::Exam,
            role: StaffRole::ExamRoomInvigilator,
            grade_name: "高一".to_string(),
            subject: Subject::Math,
            space_name: "高一1场".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T14:00".to_string(),
            end_at: "2026-03-24T16:00".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 120,
            recommended_subject: None,
            priority_subject_chain: Vec::new(),
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Afternoon,
        };
        let exclusions = HashSet::from([(2_i64, 99_i64)]);
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            None,
            &exclusions,
            &config,
        );
        assert_eq!(teacher_id, Some(1));
        assert_eq!(reason, None);
    }

    #[test]
    fn test_full_self_study_ignores_middle_manager_exception() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "中层老师".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::from(["高一1班".to_string()]),
            homeroom_classes: HashSet::from(["高一1班".to_string()]),
            is_middle_manager: true,
        }];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: None,
            space_id: None,
            task_source: StaffTaskSource::FullSelfStudy,
            role: StaffRole::SelfStudySupervisor,
            grade_name: "高一".to_string(),
            subject: Subject::Chinese,
            space_name: "高一1班".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T12:10".to_string(),
            end_at: "2026-03-24T13:40".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 90,
            recommended_subject: Some(Subject::Chinese),
            priority_subject_chain: vec![Subject::Chinese],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Afternoon,
        };
        let mut config = test_runtime_config();
        config.middle_manager_default_enabled = true;
        config.middle_manager_exception_teacher_ids = HashSet::from([1_i64]);
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高一1班"),
            &HashSet::new(),
            &config,
        );
        assert_eq!(teacher_id, None);
        assert_eq!(reason, Some("no_available_teacher".to_string()));
    }

    #[test]
    fn test_full_self_study_ignores_exam_exclusion() {
        let teachers = vec![TeacherInfo {
            id: 1,
            name: "语文老师".to_string(),
            subjects: HashSet::from([Subject::Chinese]),
            class_names: HashSet::from(["高一1班".to_string()]),
            homeroom_classes: HashSet::new(),
            is_middle_manager: false,
        }];
        let runtime = HashMap::<i64, TeacherRuntimeState>::new();
        let task = TaskBuild {
            session_id: None,
            space_id: None,
            task_source: StaffTaskSource::FullSelfStudy,
            role: StaffRole::SelfStudySupervisor,
            grade_name: "高一".to_string(),
            subject: Subject::Chinese,
            space_name: "高一1班".to_string(),
            floor: "3层".to_string(),
            start_at: "2026-03-24T12:10".to_string(),
            end_at: "2026-03-24T13:40".to_string(),
            start_ts: 1_000,
            end_ts: 2_000,
            duration_minutes: 90,
            recommended_subject: Some(Subject::Chinese),
            priority_subject_chain: vec![Subject::Chinese],
            day_key: "2026-03-24".to_string(),
            half_day: HalfDay::Afternoon,
        };
        let exclusions = HashSet::from([(1_i64, 99_i64)]);
        let config = test_runtime_config();
        let (teacher_id, reason) = choose_teacher_for_task(
            &task,
            &teachers,
            &runtime,
            Some("高一1班"),
            &exclusions,
            &config,
        );
        assert_eq!(teacher_id, Some(1));
        assert_eq!(reason, None);
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
}
