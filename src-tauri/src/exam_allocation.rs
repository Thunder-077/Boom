use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use chrono::Utc;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app_log;
use crate::class_config;
use crate::export_bundle;
use crate::score::{self, AppError, ListResult, Subject};
use crate::teacher;

const DEFAULT_CAPACITY: i64 = 40;
const DEFAULT_MAX_CAPACITY: i64 = 41;
const GENERATION_STAGE_PAUSE_MS: u64 = 650;
const DEFAULT_EXAM_TITLE: &str = "2026年3月月考";
const DEFAULT_EXAM_NOTICES: [&str; 5] = [
    "1. 考生进入考场，准备好2B铅笔、书写用0.5mm黑色签字笔、橡皮等考试必需用品。",
    "2. 每科开考前20分钟考生进入考场，不允许提前，也不允许退后。考生入场需在考场门口自觉排队等待监考教师安检入场，不可未经查验直接进入考场。进入考场后考生需对号入座，并将准考证放在课桌座号标签处。",
    "3. 考生不得提前交卷出场。",
    "4. 严禁携带手机等各种通讯工具、手表、电子存储记忆录放设备、发送接收设备、书包、学习资料、涂改液、修正带、计算器、计算尺等规定以外的物品进入考场。请考生将自己的物品妥善放置，以防丢失。",
    "5. 所有考场均启用视频监控，实时抓拍违规行为，请考生诚信应考。",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExamPlanSpaceType {
    ExamRoom,
    SelfStudyRoom,
}

impl ExamPlanSpaceType {
    fn as_key(self) -> &'static str {
        match self {
            ExamPlanSpaceType::ExamRoom => "exam_room",
            ExamPlanSpaceType::SelfStudyRoom => "self_study_room",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "exam_room" => Some(ExamPlanSpaceType::ExamRoom),
            "self_study_room" => Some(ExamPlanSpaceType::SelfStudyRoom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExamPlanSpaceSource {
    TeachingClass,
    ExamRoom,
    VirtualBackup,
}

impl ExamPlanSpaceSource {
    fn as_key(self) -> &'static str {
        match self {
            ExamPlanSpaceSource::TeachingClass => "teaching_class",
            ExamPlanSpaceSource::ExamRoom => "exam_room",
            ExamPlanSpaceSource::VirtualBackup => "virtual_backup",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "teaching_class" => Some(ExamPlanSpaceSource::TeachingClass),
            "exam_room" => Some(ExamPlanSpaceSource::ExamRoom),
            "virtual_backup" => Some(ExamPlanSpaceSource::VirtualBackup),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExamAllocationType {
    Exam,
    SelfStudy,
}

impl ExamAllocationType {
    fn as_key(self) -> &'static str {
        match self {
            ExamAllocationType::Exam => "exam",
            ExamAllocationType::SelfStudy => "self_study",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "exam" => Some(ExamAllocationType::Exam),
            "self_study" => Some(ExamAllocationType::SelfStudy),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamAllocationSettings {
    default_capacity: i64,
    max_capacity: i64,
    exam_title: String,
    exam_notices: Vec<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExamAllocationSettingsPayload {
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub exam_title: String,
    pub exam_notices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    success: bool,
}

impl SuccessResponse {
    pub(crate) fn ok() -> Self {
        Self { success: true }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateLatestExamPlanPayload {
    pub default_capacity: Option<i64>,
    pub max_capacity: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateLatestExamPlanResult {
    generated_at: String,
    grade_count: i64,
    session_count: i64,
    warning_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanOverview {
    generated_at: Option<String>,
    default_capacity: i64,
    max_capacity: i64,
    grade_count: i64,
    session_count: i64,
    exam_room_count: i64,
    self_study_room_count: i64,
    student_allocation_count: i64,
    warning_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamGenerationProgress {
    status: String,
    stage: String,
    stage_label: String,
    percent: i64,
    message: String,
    current_grade: Option<String>,
    total_grades: i64,
    completed_grades: i64,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanSession {
    id: i64,
    grade_name: String,
    subject: Subject,
    is_foreign_group: bool,
    foreign_order: Option<i64>,
    participant_count: i64,
    exam_room_count: i64,
    self_study_room_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExamPlanSessionsParams {
    pub grade_name: Option<String>,
    pub subject: Option<Subject>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanSpace {
    id: i64,
    session_id: i64,
    space_type: ExamPlanSpaceType,
    space_source: ExamPlanSpaceSource,
    grade_name: String,
    subject: Subject,
    space_name: String,
    original_class_name: Option<String>,
    building: String,
    floor: String,
    capacity: Option<i64>,
    sort_index: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanStudentAllocation {
    id: i64,
    session_id: i64,
    admission_no: String,
    student_name: String,
    class_name: String,
    allocation_type: ExamAllocationType,
    space_id: Option<i64>,
    seat_no: Option<i64>,
    subject_score: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanStaffAssignment {
    id: i64,
    session_id: i64,
    space_id: i64,
    teacher_name: String,
    assignment_type: String,
    note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamPlanSessionDetail {
    session: ExamPlanSession,
    spaces: Vec<ExamPlanSpace>,
    student_allocations: Vec<ExamPlanStudentAllocation>,
    staff_assignments: Vec<ExamPlanStaffAssignment>,
}

fn default_exam_notices_json() -> Result<String, AppError> {
    serde_json::to_string(&DEFAULT_EXAM_NOTICES)
        .map_err(|e| AppError::new(format!("默认考试须知序列化失败: {e}")))
}

fn should_replace_exam_notices(current_json: &str) -> bool {
    let trimmed = current_json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return true;
    }
    match serde_json::from_str::<Vec<String>>(trimmed) {
        Ok(items) => items.iter().any(|item| item.contains("考试科目及时间")),
        Err(_) => true,
    }
}

fn default_session_time_for_subject(subject: Subject) -> Option<(&'static str, &'static str)> {
    match subject {
        Subject::Chinese => Some(("2026-03-25T07:30", "2026-03-25T10:00")),
        Subject::Geography => Some(("2026-03-25T10:30", "2026-03-25T12:00")),
        Subject::Math => Some(("2026-03-25T14:00", "2026-03-25T16:00")),
        Subject::Biology => Some(("2026-03-25T16:30", "2026-03-25T18:00")),
        Subject::Physics => Some(("2026-03-25T18:50", "2026-03-25T20:20")),
        Subject::English => Some(("2026-03-26T08:00", "2026-03-26T10:00")),
        Subject::History => Some(("2026-03-26T10:30", "2026-03-26T12:00")),
        Subject::Chemistry => Some(("2026-03-26T14:10", "2026-03-26T15:40")),
        Subject::Politics => Some(("2026-03-26T16:10", "2026-03-26T17:40")),
        Subject::Russian | Subject::Japanese => None,
    }
}

fn seed_default_subject_time_templates(conn: &Connection) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    for subject in [
        Subject::Chinese,
        Subject::Geography,
        Subject::Math,
        Subject::Biology,
        Subject::Physics,
        Subject::English,
        Subject::History,
        Subject::Chemistry,
        Subject::Politics,
    ] {
        let Some((start_at, end_at)) = default_session_time_for_subject(subject) else {
            continue;
        };
        conn.execute(
            r#"
            INSERT INTO exam_subject_time_templates (subject, start_at, end_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(subject) DO NOTHING
            "#,
            params![subject.as_key(), start_at, end_at, now],
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Classroom {
    class_name: String,
    building: String,
    floor: String,
}

#[derive(Debug, Clone)]
struct ExamRoomResource {
    room_name: String,
    building: String,
    floor: String,
}

#[derive(Debug, Default)]
struct GradeContext {
    teaching_classes: Vec<Classroom>,
    class_subjects: HashMap<String, HashSet<Subject>>,
    exam_rooms: Vec<ExamRoomResource>,
}

#[derive(Debug, Clone)]
struct Participant {
    admission_no: String,
    student_name: String,
    class_name: String,
    score: Option<f64>,
}

#[derive(Debug)]
struct SessionBuildResult {
    warning_count: i64,
}

#[derive(Debug, Clone)]
struct SpaceCandidate {
    space_type: ExamPlanSpaceType,
    space_source: ExamPlanSpaceSource,
    space_name: String,
    original_class_name: Option<String>,
    building: String,
    floor: String,
    capacity: Option<i64>,
    sort_index: i64,
}

fn has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, AppError> {
    let sql = format!("PRAGMA table_info({table_name})");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_column(
    conn: &Connection,
    table_name: &str,
    column_sql: &str,
    column_name: &str,
) -> Result<(), AppError> {
    if has_column(conn, table_name, column_name)? {
        return Ok(());
    }
    let sql = format!("ALTER TABLE {table_name} ADD COLUMN {column_sql}");
    conn.execute(&sql, [])?;
    Ok(())
}

pub(crate) fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    score::init_schema(conn)?;
    class_config::ensure_schema(conn)?;
    teacher::ensure_schema(conn)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS exam_allocation_settings (
            id INTEGER PRIMARY KEY,
            default_capacity INTEGER NOT NULL,
            max_capacity INTEGER NOT NULL,
            exam_title TEXT NOT NULL DEFAULT '',
            exam_notices_json TEXT NOT NULL DEFAULT '[]',
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_plan_meta (
            id INTEGER PRIMARY KEY,
            generated_at TEXT NOT NULL,
            default_capacity INTEGER NOT NULL,
            max_capacity INTEGER NOT NULL,
            grade_count INTEGER NOT NULL,
            session_count INTEGER NOT NULL,
            warning_count INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS exam_generation_progress (
            id INTEGER PRIMARY KEY,
            status TEXT NOT NULL,
            stage TEXT NOT NULL,
            stage_label TEXT NOT NULL,
            percent INTEGER NOT NULL,
            message TEXT NOT NULL,
            current_grade TEXT,
            total_grades INTEGER NOT NULL DEFAULT 0,
            completed_grades INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_plan_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            grade_name TEXT NOT NULL,
            subject TEXT NOT NULL,
            is_foreign_group INTEGER NOT NULL,
            foreign_order INTEGER,
            participant_count INTEGER NOT NULL,
            exam_room_count INTEGER NOT NULL,
            self_study_room_count INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_plan_spaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            space_type TEXT NOT NULL,
            space_source TEXT NOT NULL,
            grade_name TEXT NOT NULL,
            subject TEXT NOT NULL,
            space_name TEXT NOT NULL,
            original_class_name TEXT,
            building TEXT NOT NULL,
            floor TEXT NOT NULL,
            capacity INTEGER,
            sort_index INTEGER NOT NULL,
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS latest_exam_plan_student_allocations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            admission_no TEXT NOT NULL,
            student_name TEXT NOT NULL,
            class_name TEXT NOT NULL,
            allocation_type TEXT NOT NULL,
            space_id INTEGER,
            seat_no INTEGER,
            subject_score REAL,
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
            FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_plan_staff_assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            space_id INTEGER NOT NULL,
            teacher_name TEXT NOT NULL,
            assignment_type TEXT NOT NULL,
            note TEXT,
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
            FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS exam_session_times (
            session_id INTEGER PRIMARY KEY,
            subject TEXT NOT NULL,
            start_at TEXT NOT NULL,
            end_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS exam_subject_time_templates (
            subject TEXT PRIMARY KEY,
            start_at TEXT NOT NULL,
            end_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS exam_space_staff_requirements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            space_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            required_count INTEGER NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(session_id, space_id, role),
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
            FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS latest_exam_staff_plan_meta (
            id INTEGER PRIMARY KEY,
            generated_at TEXT NOT NULL,
            session_count INTEGER NOT NULL,
            task_count INTEGER NOT NULL,
            assigned_count INTEGER NOT NULL,
            unassigned_count INTEGER NOT NULL,
            warning_count INTEGER NOT NULL,
            imbalance_minutes INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_staff_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            space_id INTEGER,
            role TEXT NOT NULL,
            grade_name TEXT NOT NULL,
            subject TEXT NOT NULL,
            space_name TEXT NOT NULL,
            floor TEXT NOT NULL,
            start_at TEXT NOT NULL,
            end_at TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL,
            recommended_subject TEXT,
            priority_subject_chain TEXT,
            status TEXT NOT NULL,
            reason TEXT,
            FOREIGN KEY(session_id) REFERENCES latest_exam_plan_sessions(id) ON DELETE CASCADE,
            FOREIGN KEY(space_id) REFERENCES latest_exam_plan_spaces(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS latest_exam_staff_assignments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            teacher_id INTEGER NOT NULL,
            teacher_name TEXT NOT NULL,
            assigned_at TEXT NOT NULL,
            FOREIGN KEY(task_id) REFERENCES latest_exam_staff_tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS latest_teacher_duty_stats (
            teacher_id INTEGER PRIMARY KEY,
            teacher_name TEXT NOT NULL,
            indoor_minutes INTEGER NOT NULL,
            outdoor_minutes INTEGER NOT NULL,
            total_minutes INTEGER NOT NULL,
            task_count INTEGER NOT NULL,
            exam_room_task_count INTEGER NOT NULL DEFAULT 0,
            self_study_task_count INTEGER NOT NULL DEFAULT 0,
            floor_rover_task_count INTEGER NOT NULL DEFAULT 0,
            is_middle_manager INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_exam_session_times_subject ON exam_session_times(subject);
        CREATE INDEX IF NOT EXISTS idx_exam_subject_time_templates_subject ON exam_subject_time_templates(subject);
        CREATE INDEX IF NOT EXISTS idx_exam_space_staff_req_session ON exam_space_staff_requirements(session_id);
        CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_tasks_session ON latest_exam_staff_tasks(session_id);
        CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_tasks_role_status ON latest_exam_staff_tasks(role, status);
        CREATE INDEX IF NOT EXISTS idx_latest_exam_staff_assignments_task ON latest_exam_staff_assignments(task_id);
        CREATE INDEX IF NOT EXISTS idx_latest_teacher_duty_stats_total ON latest_teacher_duty_stats(total_minutes);
        "#,
    )?;
    ensure_column(
        conn,
        "exam_allocation_settings",
        "exam_title TEXT NOT NULL DEFAULT ''",
        "exam_title",
    )?;
    ensure_column(
        conn,
        "exam_allocation_settings",
        "exam_notices_json TEXT NOT NULL DEFAULT '[]'",
        "exam_notices_json",
    )?;
    ensure_column(
        conn,
        "latest_teacher_duty_stats",
        "exam_room_task_count INTEGER NOT NULL DEFAULT 0",
        "exam_room_task_count",
    )?;
    ensure_column(
        conn,
        "latest_teacher_duty_stats",
        "self_study_task_count INTEGER NOT NULL DEFAULT 0",
        "self_study_task_count",
    )?;
    ensure_column(
        conn,
        "latest_teacher_duty_stats",
        "floor_rover_task_count INTEGER NOT NULL DEFAULT 0",
        "floor_rover_task_count",
    )?;
    ensure_column(
        conn,
        "latest_teacher_duty_stats",
        "is_middle_manager INTEGER NOT NULL DEFAULT 0",
        "is_middle_manager",
    )?;

    let now = Utc::now().to_rfc3339();
    let default_notices_json = default_exam_notices_json()?;
    conn.execute(
        "INSERT OR IGNORE INTO exam_allocation_settings (id, default_capacity, max_capacity, exam_title, exam_notices_json, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5)",
        params![DEFAULT_CAPACITY, DEFAULT_MAX_CAPACITY, DEFAULT_EXAM_TITLE, default_notices_json, now],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO exam_generation_progress (id, status, stage, stage_label, percent, message, current_grade, total_grades, completed_grades, updated_at) VALUES (1, 'idle', 'idle', '等待开始', 0, '等待开始分配考场', NULL, 0, 0, ?1)",
        params![now],
    )?;
    conn.execute(
        "UPDATE exam_allocation_settings SET exam_title = ?1 WHERE id = 1 AND TRIM(COALESCE(exam_title, '')) = ''",
        params![DEFAULT_EXAM_TITLE],
    )?;
    let current_notices_json: String = conn.query_row(
        "SELECT COALESCE(exam_notices_json, '') FROM exam_allocation_settings WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    if should_replace_exam_notices(&current_notices_json) {
        conn.execute(
            "UPDATE exam_allocation_settings SET exam_notices_json = ?1 WHERE id = 1",
            params![default_exam_notices_json()?],
        )?;
    }
    seed_default_subject_time_templates(conn)?;
    Ok(())
}

fn validate_capacity(default_capacity: i64, max_capacity: i64) -> Result<(), AppError> {
    if default_capacity <= 0 {
        return Err(AppError::new("默认容量必须大于 0"));
    }
    if max_capacity < default_capacity {
        return Err(AppError::new("最大容量不能小于默认容量"));
    }
    if max_capacity > 200 {
        return Err(AppError::new("最大容量超过合理范围"));
    }
    Ok(())
}

fn is_foreign_subject(subject: Subject) -> bool {
    matches!(
        subject,
        Subject::English | Subject::Russian | Subject::Japanese
    )
}

fn foreign_order(subject: Subject) -> Option<i64> {
    match subject {
        Subject::English => Some(1),
        Subject::Russian => Some(2),
        Subject::Japanese => Some(3),
        _ => None,
    }
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

fn class_number(name: &str, suffix: char) -> Option<i64> {
    let target = name.find(suffix)?;
    let mut chars = name[..target].chars().rev().peekable();
    let mut digits = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            digits.push(*ch);
            chars.next();
        } else {
            break;
        }
    }
    if digits.is_empty() {
        return None;
    }
    let parsed: String = digits.chars().rev().collect();
    parsed.parse::<i64>().ok()
}

fn sort_class_names(a: &str, b: &str) -> Ordering {
    class_number(a, '班')
        .cmp(&class_number(b, '班'))
        .then(a.cmp(b))
}

fn class_to_exam_room_name(class_name: &str) -> String {
    if let Some(stripped) = class_name.strip_suffix('班') {
        return format!("{stripped}场");
    }
    format!("{class_name}场")
}

fn grade_order_key(grade_name: &str) -> (i32, &str) {
    match grade_name {
        "高一" => (1, grade_name),
        "高二" => (2, grade_name),
        "高三" => (3, grade_name),
        _ => (4, grade_name),
    }
}

fn calculate_room_capacities(
    total_students: usize,
    default_capacity: i64,
    max_capacity: i64,
) -> Vec<i64> {
    if total_students == 0 {
        return Vec::new();
    }
    let default_capacity_usize = default_capacity as usize;
    let room_count = total_students.div_ceil(default_capacity_usize);
    let mut capacities = vec![default_capacity; room_count];
    let last_room_count = total_students - default_capacity_usize * (room_count - 1);
    capacities[room_count - 1] = last_room_count as i64;

    let extra = max_capacity - default_capacity;
    if room_count > 1 && extra > 0 {
        let new_room_count = room_count - 1;
        let max_total_after_reduce = max_capacity * new_room_count as i64;
        if total_students as i64 <= max_total_after_reduce {
            let mut reduced = vec![default_capacity; new_room_count];
            let mut remaining = total_students as i64 - default_capacity * new_room_count as i64;
            for cap in &mut reduced {
                if remaining <= 0 {
                    break;
                }
                let add = remaining.min(extra);
                *cap += add;
                remaining -= add;
            }
            return reduced;
        }
    }
    capacities
}

fn load_settings(conn: &Connection) -> Result<ExamAllocationSettings, AppError> {
    conn.query_row(
        "SELECT default_capacity, max_capacity, exam_title, exam_notices_json, updated_at FROM exam_allocation_settings WHERE id = 1",
        [],
        |r| {
            let notices_json: String = r.get(3)?;
            let exam_notices = serde_json::from_str::<Vec<String>>(&notices_json)
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>();
            Ok(ExamAllocationSettings {
                default_capacity: r.get(0)?,
                max_capacity: r.get(1)?,
                exam_title: r.get::<_, String>(2)?,
                exam_notices,
                updated_at: r.get::<_, String>(4).ok(),
            })
        },
    )
    .map_err(AppError::from)
}

fn load_grade_contexts(conn: &Connection) -> Result<HashMap<String, GradeContext>, AppError> {
    let mut ctx_map: HashMap<String, GradeContext> = HashMap::new();

    let mut teaching_stmt = conn.prepare(
        r#"
        SELECT c.grade_name, c.class_name, c.building, c.floor, s.subject
        FROM class_configs c
        LEFT JOIN class_config_subjects s ON s.config_id = c.id
        WHERE c.config_type = 'teaching_class'
        ORDER BY c.grade_name ASC, c.class_name ASC, c.id ASC, s.id ASC
        "#,
    )?;
    let rows = teaching_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })?;
    for row in rows {
        let (grade_name, class_name, building, floor, subject_key) = row?;
        let grade_ctx = ctx_map.entry(grade_name).or_default();
        if !grade_ctx
            .teaching_classes
            .iter()
            .any(|it| it.class_name == class_name)
        {
            grade_ctx.teaching_classes.push(Classroom {
                class_name: class_name.clone(),
                building: building.clone(),
                floor: floor.clone(),
            });
        }
        if let Some(subject_key) = subject_key {
            if let Some(subject) = Subject::from_key(&subject_key) {
                grade_ctx
                    .class_subjects
                    .entry(class_name)
                    .or_default()
                    .insert(subject);
            }
        }
    }

    let mut exam_stmt = conn.prepare(
        r#"
        SELECT grade_name, class_name, room_label, building, floor
        FROM class_configs
        WHERE config_type = 'exam_room'
        ORDER BY grade_name ASC, class_name ASC, id ASC
        "#,
    )?;
    let exam_rows = exam_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    for row in exam_rows {
        let (grade_name, class_name, room_label, building, floor) = row?;
        let grade_ctx = ctx_map.entry(grade_name).or_default();
        grade_ctx.exam_rooms.push(ExamRoomResource {
            room_name: room_label.unwrap_or(class_name),
            building,
            floor,
        });
    }

    for ctx in ctx_map.values_mut() {
        ctx.teaching_classes
            .sort_by(|a, b| sort_class_names(&a.class_name, &b.class_name));
        ctx.exam_rooms.sort_by(|a, b| a.room_name.cmp(&b.room_name));
    }

    Ok(ctx_map)
}

fn load_selected_participants(
    conn: &Connection,
    grade_name: &str,
    subject: Subject,
) -> Result<Vec<Participant>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT s.admission_no, s.student_name, s.class_name, ss.score
        FROM latest_student_scores s
        JOIN latest_subject_scores ss ON ss.admission_no = s.admission_no
        WHERE s.grade_name = ?1 AND ss.subject = ?2 AND ss.is_selected = 1
        "#,
    )?;
    let rows = stmt.query_map(params![grade_name, subject.as_key()], |row| {
        Ok(Participant {
            admission_no: row.get(0)?,
            student_name: row.get(1)?,
            class_name: row.get(2)?,
            score: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn load_not_selected_students(
    conn: &Connection,
    grade_name: &str,
    subject: Subject,
) -> Result<Vec<Participant>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT s.admission_no, s.student_name, s.class_name, ss.score
        FROM latest_student_scores s
        JOIN latest_subject_scores ss ON ss.admission_no = s.admission_no
        WHERE s.grade_name = ?1 AND ss.subject = ?2 AND ss.is_selected = 0
        "#,
    )?;
    let rows = stmt.query_map(params![grade_name, subject.as_key()], |row| {
        Ok(Participant {
            admission_no: row.get(0)?,
            student_name: row.get(1)?,
            class_name: row.get(2)?,
            score: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn build_round_robin_order(participants: &[Participant]) -> Vec<Participant> {
    let mut groups: HashMap<String, Vec<Participant>> = HashMap::new();
    for p in participants {
        groups
            .entry(p.class_name.clone())
            .or_default()
            .push(p.clone());
    }
    for list in groups.values_mut() {
        list.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(Ordering::Equal)
                .then(a.admission_no.cmp(&b.admission_no))
        });
    }

    let mut class_names: Vec<String> = groups.keys().cloned().collect();
    class_names.sort_by(|a, b| sort_class_names(a, b));

    let mut ordered = Vec::new();
    let mut index = 0usize;
    loop {
        let mut has_value = false;
        for class_name in &class_names {
            if let Some(list) = groups.get(class_name) {
                if index < list.len() {
                    ordered.push(list[index].clone());
                    has_value = true;
                }
            }
        }
        if !has_value {
            break;
        }
        index += 1;
    }
    ordered
}

fn clear_latest_plan(tx: &rusqlite::Transaction<'_>) -> Result<(), AppError> {
    tx.execute("DELETE FROM latest_exam_plan_staff_assignments", [])?;
    tx.execute("DELETE FROM latest_exam_plan_student_allocations", [])?;
    tx.execute("DELETE FROM latest_exam_plan_spaces", [])?;
    tx.execute("DELETE FROM latest_exam_plan_sessions", [])?;
    tx.execute("DELETE FROM latest_exam_plan_meta", [])?;
    Ok(())
}

fn build_session(
    tx: &rusqlite::Transaction<'_>,
    grade_name: &str,
    subject: Subject,
    grade_ctx: &GradeContext,
    default_capacity: i64,
    max_capacity: i64,
    foreign_occupied_classes: &mut HashSet<String>,
) -> Result<SessionBuildResult, AppError> {
    let mut warnings = 0_i64;
    let is_foreign = is_foreign_subject(subject);
    let foreign_seq = foreign_order(subject);
    let not_selected = load_not_selected_students(tx, grade_name, subject)?;
    let self_study_class_names: HashSet<String> = not_selected
        .iter()
        .map(|item| item.class_name.clone())
        .collect();

    let mut subject_classes = HashSet::new();
    if is_foreign {
        for (class_name, subjects) in &grade_ctx.class_subjects {
            if subjects.contains(&Subject::English)
                || subjects.contains(&Subject::Russian)
                || subjects.contains(&Subject::Japanese)
            {
                subject_classes.insert(class_name.clone());
            }
        }
    } else {
        for (class_name, subjects) in &grade_ctx.class_subjects {
            if subjects.contains(&subject) {
                subject_classes.insert(class_name.clone());
            }
        }
    }

    let mut participants = load_selected_participants(tx, grade_name, subject)?;
    for p in &participants {
        if !subject_classes.contains(&p.class_name) {
            warnings += 1;
        }
    }
    let capacities = calculate_room_capacities(participants.len(), default_capacity, max_capacity);
    let required_room_count = capacities.len();

    let mut chosen_spaces: Vec<SpaceCandidate> = Vec::new();
    let mut used_teaching_classes = HashSet::new();
    let mut teaching_candidates: Vec<Classroom> = grade_ctx
        .teaching_classes
        .iter()
        .filter(|c| {
            subject_classes.contains(&c.class_name)
                && !self_study_class_names.contains(&c.class_name)
        })
        .cloned()
        .collect();
    if is_foreign {
        teaching_candidates.retain(|c| !foreign_occupied_classes.contains(&c.class_name));
    }
    teaching_candidates.sort_by(|a, b| sort_class_names(&a.class_name, &b.class_name));

    for classroom in teaching_candidates {
        if chosen_spaces.len() >= required_room_count {
            break;
        }
        used_teaching_classes.insert(classroom.class_name.clone());
        chosen_spaces.push(SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::TeachingClass,
            space_name: class_to_exam_room_name(&classroom.class_name),
            original_class_name: Some(classroom.class_name),
            building: classroom.building,
            floor: classroom.floor,
            capacity: None,
            sort_index: chosen_spaces.len() as i64 + 1,
        });
    }
    for room in &grade_ctx.exam_rooms {
        if chosen_spaces.len() >= required_room_count {
            break;
        }
        chosen_spaces.push(SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::ExamRoom,
            space_name: room.room_name.clone(),
            original_class_name: None,
            building: room.building.clone(),
            floor: room.floor.clone(),
            capacity: None,
            sort_index: chosen_spaces.len() as i64 + 1,
        });
    }
    let mut virtual_index = grade_ctx.teaching_classes.len() as i64 + 1;
    while chosen_spaces.len() < required_room_count {
        chosen_spaces.push(SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::VirtualBackup,
            space_name: format!("{grade_name}{virtual_index}场"),
            original_class_name: None,
            building: "备用考场".to_string(),
            floor: "临时".to_string(),
            capacity: None,
            sort_index: chosen_spaces.len() as i64 + 1,
        });
        virtual_index += 1;
    }
    if is_foreign {
        for class_name in &used_teaching_classes {
            foreign_occupied_classes.insert(class_name.clone());
        }
    }

    let mut self_study_spaces: Vec<SpaceCandidate> = Vec::new();
    for classroom in &grade_ctx.teaching_classes {
        if !self_study_class_names.contains(&classroom.class_name) {
            continue;
        }
        self_study_spaces.push(SpaceCandidate {
            space_type: ExamPlanSpaceType::SelfStudyRoom,
            space_source: ExamPlanSpaceSource::TeachingClass,
            space_name: classroom.class_name.clone(),
            original_class_name: Some(classroom.class_name.clone()),
            building: classroom.building.clone(),
            floor: classroom.floor.clone(),
            capacity: None,
            sort_index: (chosen_spaces.len() + self_study_spaces.len()) as i64 + 1,
        });
    }

    tx.execute(
        r#"
        INSERT INTO latest_exam_plan_sessions
        (grade_name, subject, is_foreign_group, foreign_order, participant_count, exam_room_count, self_study_room_count)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            grade_name,
            subject.as_key(),
            if is_foreign { 1_i64 } else { 0_i64 },
            foreign_seq,
            participants.len() as i64,
            chosen_spaces.len() as i64,
            self_study_spaces.len() as i64
        ],
    )?;
    let session_id = tx.last_insert_rowid();

    let mut exam_space_ids = Vec::new();
    for (index, space) in chosen_spaces.iter_mut().enumerate() {
        space.capacity = capacities.get(index).copied();
        tx.execute(
            r#"
            INSERT INTO latest_exam_plan_spaces
            (session_id, space_type, space_source, grade_name, subject, space_name, original_class_name, building, floor, capacity, sort_index)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                session_id,
                space.space_type.as_key(),
                space.space_source.as_key(),
                grade_name,
                subject.as_key(),
                space.space_name,
                space.original_class_name,
                space.building,
                space.floor,
                space.capacity,
                space.sort_index
            ],
        )?;
        exam_space_ids.push(tx.last_insert_rowid());
    }

    let mut self_study_space_by_class = HashMap::new();
    let mut self_study_ids = Vec::new();
    for space in &self_study_spaces {
        tx.execute(
            r#"
            INSERT INTO latest_exam_plan_spaces
            (session_id, space_type, space_source, grade_name, subject, space_name, original_class_name, building, floor, capacity, sort_index)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                session_id,
                space.space_type.as_key(),
                space.space_source.as_key(),
                grade_name,
                subject.as_key(),
                space.space_name,
                space.original_class_name,
                space.building,
                space.floor,
                Option::<i64>::None,
                space.sort_index
            ],
        )?;
        let id = tx.last_insert_rowid();
        self_study_ids.push(id);
        if let Some(class_name) = &space.original_class_name {
            self_study_space_by_class.insert(class_name.clone(), id);
        }
    }

    participants.sort_by(|a, b| {
        sort_class_names(&a.class_name, &b.class_name).then(a.admission_no.cmp(&b.admission_no))
    });
    let ordered = build_round_robin_order(&participants);

    let mut start = 0usize;
    for (space_index, cap) in capacities.iter().enumerate() {
        let cap_u = (*cap).max(0) as usize;
        let end = (start + cap_u).min(ordered.len());
        let room_students = &ordered[start..end];
        for (seat_idx, student) in room_students.iter().enumerate() {
            tx.execute(
                r#"
                INSERT INTO latest_exam_plan_student_allocations
                (session_id, admission_no, student_name, class_name, allocation_type, space_id, seat_no, subject_score)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    session_id,
                    student.admission_no,
                    student.student_name,
                    student.class_name,
                    ExamAllocationType::Exam.as_key(),
                    exam_space_ids.get(space_index),
                    seat_idx as i64 + 1,
                    student.score
                ],
            )?;
        }
        start = end;
        if start >= ordered.len() {
            break;
        }
    }
    for student in not_selected {
        let mapped_id = self_study_space_by_class
            .get(&student.class_name)
            .copied()
            .ok_or_else(|| {
                AppError::new(format!(
                    "{} 未找到本班自习教室，无法完成自习安排",
                    student.class_name
                ))
            })?;
        tx.execute(
            r#"
            INSERT INTO latest_exam_plan_student_allocations
            (session_id, admission_no, student_name, class_name, allocation_type, space_id, seat_no, subject_score)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL)
            "#,
            params![
                session_id,
                student.admission_no,
                student.student_name,
                student.class_name,
                ExamAllocationType::SelfStudy.as_key(),
                mapped_id
            ],
        )?;
    }

    Ok(SessionBuildResult {
        warning_count: warnings,
    })
}

fn update_exam_generation_progress(
    conn: &Connection,
    status: &str,
    stage: &str,
    stage_label: &str,
    percent: i64,
    message: &str,
    current_grade: Option<&str>,
    total_grades: i64,
    completed_grades: i64,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        UPDATE exam_generation_progress
        SET status = ?1,
            stage = ?2,
            stage_label = ?3,
            percent = ?4,
            message = ?5,
            current_grade = ?6,
            total_grades = ?7,
            completed_grades = ?8,
            updated_at = ?9
        WHERE id = 1
        "#,
        params![
            status,
            stage,
            stage_label,
            percent.clamp(0, 100),
            message,
            current_grade,
            total_grades.max(0),
            completed_grades.max(0),
            now
        ],
    )?;
    Ok(())
}

fn pause_after_generation_stage() {
    thread::sleep(Duration::from_millis(GENERATION_STAGE_PAUSE_MS));
}

pub fn get_exam_allocation_settings(app: AppHandle) -> Result<ExamAllocationSettings, String> {
    let result = (|| -> Result<ExamAllocationSettings, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        load_settings(&conn)
    })();
    result.map_err(|e| e.to_string())
}

pub fn update_exam_allocation_settings(
    app: AppHandle,
    payload: UpdateExamAllocationSettingsPayload,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        validate_capacity(payload.default_capacity, payload.max_capacity)?;
        let exam_title = payload.exam_title.trim().to_string();
        let exam_notices = payload
            .exam_notices
            .iter()
            .map(|it| it.trim().to_string())
            .filter(|it| !it.is_empty())
            .collect::<Vec<_>>();
        let exam_notices_json = serde_json::to_string(&exam_notices)
            .map_err(|e| AppError::new(format!("考试须知序列化失败: {e}")))?;
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE exam_allocation_settings SET default_capacity = ?1, max_capacity = ?2, exam_title = ?3, exam_notices_json = ?4, updated_at = ?5 WHERE id = 1",
            params![payload.default_capacity, payload.max_capacity, exam_title, exam_notices_json, now],
        )?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|e| e.to_string())
}

fn generate_latest_exam_plan_internal(
    app: &AppHandle,
    payload: Option<GenerateLatestExamPlanPayload>,
) -> Result<GenerateLatestExamPlanResult, AppError> {
    let mut conn = score::open_connection(&app)?;
    ensure_schema(&conn)?;
    update_exam_generation_progress(
        &conn,
        "running",
        "loading_config",
        "读取配置",
        5,
        "正在读取考试配置、班级配置与考试时间设置",
        None,
        0,
        0,
    )?;
    pause_after_generation_stage();
    let settings = load_settings(&conn)?;
    let default_capacity = payload
        .as_ref()
        .and_then(|p| p.default_capacity)
        .unwrap_or(settings.default_capacity);
    let max_capacity = payload
        .as_ref()
        .and_then(|p| p.max_capacity)
        .unwrap_or(settings.max_capacity);
    validate_capacity(default_capacity, max_capacity)?;

    let grade_contexts = load_grade_contexts(&conn)?;
    let mut grades: Vec<String> = grade_contexts.keys().cloned().collect();
    grades.sort_by(|a, b| grade_order_key(a).cmp(&grade_order_key(b)).then(a.cmp(b)));
    let total_grades = grades.len() as i64;
    update_exam_generation_progress(
        &conn,
        "running",
        "clearing_snapshot",
        "清理旧结果",
        12,
        "正在清理上一轮考场分配结果",
        None,
        total_grades,
        0,
    )?;
    pause_after_generation_stage();

    let generated_at = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;
    clear_latest_plan(&tx)?;
    update_exam_generation_progress(
        &tx,
        "running",
        "building_sessions",
        "生成场次",
        20,
        "正在按年级和科目生成考试场次",
        None,
        total_grades,
        0,
    )?;
    pause_after_generation_stage();

    let mut session_count = 0_i64;
    let mut warning_count = 0_i64;

    for (grade_index, grade_name) in grades.iter().enumerate() {
        let alloc_percent = 28 + (((grade_index as i64) * 44) / total_grades.max(1));
        update_exam_generation_progress(
            &tx,
            "running",
            "allocating_rooms",
            "分配考场",
            alloc_percent,
            &format!("正在为 {grade_name} 生成考场与座位安排"),
            Some(grade_name),
            total_grades,
            grade_index as i64,
        )?;
        pause_after_generation_stage();
        let Some(grade_ctx) = grade_contexts.get(grade_name) else {
            continue;
        };
        let mut subject_set: HashSet<Subject> = HashSet::new();
        for subjects in grade_ctx.class_subjects.values() {
            for subject in subjects {
                subject_set.insert(*subject);
            }
        }
        let mut subjects: Vec<Subject> = subject_set.into_iter().collect();
        subjects.sort_by_key(|s| subject_order(*s));

        let mut foreign_occupied = HashSet::new();
        for subject in subjects {
            let built = build_session(
                &tx,
                grade_name,
                subject,
                grade_ctx,
                default_capacity,
                max_capacity,
                &mut foreign_occupied,
            )?;
            session_count += 1;
            warning_count += built.warning_count;
        }
    }

    tx.execute(
            "INSERT INTO latest_exam_plan_meta (id, generated_at, default_capacity, max_capacity, grade_count, session_count, warning_count) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
            params![generated_at, default_capacity, max_capacity, grades.len() as i64, session_count, warning_count],
        )?;
    tx.commit()?;
    update_exam_generation_progress(
        &conn,
        "running",
        "finalizing_results",
        "整理结果",
        76,
        "正在整理场次时间与分配结果摘要",
        None,
        total_grades,
        total_grades,
    )?;
    pause_after_generation_stage();
    crate::exam_staff::seed_default_session_times(&conn)?;
    update_exam_generation_progress(
        &conn,
        "running",
        "exporting_files",
        "生成文件",
        82,
        "考场分配已完成，正在生成各年级导出文件",
        None,
        total_grades,
        0,
    )?;
    pause_after_generation_stage();
    export_bundle::generate_export_files(&app, &conn, |grade_name, done, total| {
        let percent = 82 + (((done as i64) * 16) / (total as i64).max(1));
        let _ = update_exam_generation_progress(
            &conn,
            "running",
            "exporting_files",
            "生成文件",
            percent,
            &format!("已生成 {grade_name} 的导出文件"),
            Some(grade_name),
            total as i64,
            done as i64,
        );
        pause_after_generation_stage();
    })?;
    update_exam_generation_progress(
        &conn,
        "completed",
        "completed",
        "已完成",
        100,
        "考场分配与导出文件生成已完成，可按需打包 ZIP",
        None,
        total_grades,
        total_grades,
    )?;
    pause_after_generation_stage();

    Ok(GenerateLatestExamPlanResult {
        generated_at,
        grade_count: grades.len() as i64,
        session_count,
        warning_count,
    })
}

pub fn start_generate_latest_exam_plan(
    app: AppHandle,
    payload: Option<GenerateLatestExamPlanPayload>,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let running: String = conn.query_row(
            "SELECT status FROM exam_generation_progress WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        if running == "running" {
            return Err(AppError::new("考场分配正在执行中，请稍候"));
        }
        update_exam_generation_progress(
            &conn,
            "running",
            "queued",
            "准备开始",
            1,
            "已接收任务，准备开始分配考场",
            None,
            0,
            0,
        )?;
        let app_handle = app.clone();
        thread::spawn(move || {
            if let Err(error) = generate_latest_exam_plan_internal(&app_handle, payload) {
                if let Ok(conn) = score::open_connection(&app_handle) {
                    let _ = ensure_schema(&conn);
                    let _ = update_exam_generation_progress(
                        &conn,
                        "error",
                        "error",
                        "执行失败",
                        0,
                        &error.to_string(),
                        None,
                        0,
                        0,
                    );
                }
                app_log::log_error(
                    &app_handle,
                    "exam_allocation.start_generate_latest_exam_plan",
                    &error.to_string(),
                );
            }
        });
        Ok(SuccessResponse { success: true })
    })();
    result.map_err(|e| e.to_string())
}

pub fn get_latest_exam_plan_overview(app: AppHandle) -> Result<ExamPlanOverview, String> {
    let result = (|| -> Result<ExamPlanOverview, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let settings = load_settings(&conn)?;
        let meta_row: Option<(String, i64, i64, i64)> = conn
            .query_row(
                "SELECT generated_at, grade_count, session_count, warning_count FROM latest_exam_plan_meta WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .ok();
        let exam_room_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM latest_exam_plan_spaces WHERE space_type = 'exam_room'",
            [],
            |row| row.get(0),
        )?;
        let self_study_room_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM latest_exam_plan_spaces WHERE space_type = 'self_study_room'",
            [],
            |row| row.get(0),
        )?;
        let student_allocation_count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT admission_no) FROM latest_exam_plan_student_allocations WHERE allocation_type = 'exam'",
            [],
            |row| row.get(0),
        )?;
        Ok(ExamPlanOverview {
            generated_at: meta_row.as_ref().map(|v| v.0.clone()),
            default_capacity: settings.default_capacity,
            max_capacity: settings.max_capacity,
            grade_count: meta_row.as_ref().map(|v| v.1).unwrap_or(0),
            session_count: meta_row.as_ref().map(|v| v.2).unwrap_or(0),
            warning_count: meta_row.as_ref().map(|v| v.3).unwrap_or(0),
            exam_room_count,
            self_study_room_count,
            student_allocation_count,
        })
    })();
    result.map_err(|e| e.to_string())
}

pub fn get_exam_generation_progress(app: AppHandle) -> Result<ExamGenerationProgress, String> {
    let result = (|| -> Result<ExamGenerationProgress, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        conn.query_row(
            "SELECT status, stage, stage_label, percent, message, current_grade, total_grades, completed_grades, updated_at FROM exam_generation_progress WHERE id = 1",
            [],
            |row| {
                Ok(ExamGenerationProgress {
                    status: row.get(0)?,
                    stage: row.get(1)?,
                    stage_label: row.get(2)?,
                    percent: row.get(3)?,
                    message: row.get(4)?,
                    current_grade: row.get(5)?,
                    total_grades: row.get(6)?,
                    completed_grades: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(AppError::from)
    })();
    result.map_err(|e| e.to_string())
}

pub fn list_latest_exam_plan_sessions(
    app: AppHandle,
    params: ListExamPlanSessionsParams,
) -> Result<ListResult<ExamPlanSession>, String> {
    let result = (|| -> Result<ListResult<ExamPlanSession>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;

        let mut where_parts = Vec::new();
        let mut values = Vec::<Value>::new();
        if let Some(grade_name) = params
            .grade_name
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            where_parts.push("grade_name = ?".to_string());
            values.push(Value::Text(grade_name.to_string()));
        }
        if let Some(subject) = params.subject {
            where_parts.push("subject = ?".to_string());
            values.push(Value::Text(subject.as_key().to_string()));
        }
        let where_sql = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };
        let total_sql = format!("SELECT COUNT(*) FROM latest_exam_plan_sessions{where_sql}");
        let total: i64 = conn.query_row(&total_sql, params_from_iter(values.iter()), |row| {
            row.get(0)
        })?;

        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(100).clamp(1, 500);
        let offset = (page - 1) * page_size;
        let mut query_values = values;
        query_values.push(Value::Integer(page_size));
        query_values.push(Value::Integer(offset));

        let list_sql = format!(
            r#"
            SELECT id, grade_name, subject, is_foreign_group, foreign_order, participant_count, exam_room_count, self_study_room_count
            FROM latest_exam_plan_sessions
            {where_sql}
            ORDER BY grade_name ASC, is_foreign_group DESC, COALESCE(foreign_order, 99) ASC, subject ASC, id ASC
            LIMIT ? OFFSET ?
            "#
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(query_values.iter()), |row| {
            let subject_key: String = row.get(2)?;
            let subject = Subject::from_key(&subject_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "subject".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(ExamPlanSession {
                id: row.get(0)?,
                grade_name: row.get(1)?,
                subject,
                is_foreign_group: row.get::<_, i64>(3)? == 1,
                foreign_order: row.get(4)?,
                participant_count: row.get(5)?,
                exam_room_count: row.get(6)?,
                self_study_room_count: row.get(7)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(ListResult { items, total })
    })();
    result.map_err(|e| e.to_string())
}

fn get_session_by_id(conn: &Connection, session_id: i64) -> Result<ExamPlanSession, AppError> {
    conn.query_row(
        "SELECT id, grade_name, subject, is_foreign_group, foreign_order, participant_count, exam_room_count, self_study_room_count FROM latest_exam_plan_sessions WHERE id = ?1",
        params![session_id],
        |row| {
            let subject_key: String = row.get(2)?;
            let subject = Subject::from_key(&subject_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(2, "subject".to_string(), rusqlite::types::Type::Text)
            })?;
            Ok(ExamPlanSession {
                id: row.get(0)?,
                grade_name: row.get(1)?,
                subject,
                is_foreign_group: row.get::<_, i64>(3)? == 1,
                foreign_order: row.get(4)?,
                participant_count: row.get(5)?,
                exam_room_count: row.get(6)?,
                self_study_room_count: row.get(7)?,
            })
        },
    )
    .map_err(AppError::from)
}

pub fn get_latest_exam_plan_session_detail(
    app: AppHandle,
    session_id: i64,
) -> Result<ExamPlanSessionDetail, String> {
    let result = (|| -> Result<ExamPlanSessionDetail, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let session = get_session_by_id(&conn, session_id)?;

        let mut spaces_stmt = conn.prepare(
            "SELECT id, session_id, space_type, space_source, grade_name, subject, space_name, original_class_name, building, floor, capacity, sort_index FROM latest_exam_plan_spaces WHERE session_id = ?1 ORDER BY sort_index ASC, id ASC",
        )?;
        let space_rows = spaces_stmt.query_map(params![session_id], |row| {
            let space_type_key: String = row.get(2)?;
            let space_source_key: String = row.get(3)?;
            let subject_key: String = row.get(5)?;
            let space_type = ExamPlanSpaceType::from_key(&space_type_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    2,
                    "space_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            let space_source =
                ExamPlanSpaceSource::from_key(&space_source_key).ok_or_else(|| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "space_source".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
            let subject = Subject::from_key(&subject_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    5,
                    "subject".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(ExamPlanSpace {
                id: row.get(0)?,
                session_id: row.get(1)?,
                space_type,
                space_source,
                grade_name: row.get(4)?,
                subject,
                space_name: row.get(6)?,
                original_class_name: row.get(7)?,
                building: row.get(8)?,
                floor: row.get(9)?,
                capacity: row.get(10)?,
                sort_index: row.get(11)?,
            })
        })?;
        let mut spaces = Vec::new();
        for row in space_rows {
            spaces.push(row?);
        }

        let mut allocation_stmt = conn.prepare(
            "SELECT id, session_id, admission_no, student_name, class_name, allocation_type, space_id, seat_no, subject_score FROM latest_exam_plan_student_allocations WHERE session_id = ?1 ORDER BY allocation_type ASC, COALESCE(space_id, 0) ASC, COALESCE(seat_no, 9999) ASC, admission_no ASC",
        )?;
        let allocation_rows = allocation_stmt.query_map(params![session_id], |row| {
            let allocation_key: String = row.get(5)?;
            let allocation_type =
                ExamAllocationType::from_key(&allocation_key).ok_or_else(|| {
                    rusqlite::Error::InvalidColumnType(
                        5,
                        "allocation_type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;
            Ok(ExamPlanStudentAllocation {
                id: row.get(0)?,
                session_id: row.get(1)?,
                admission_no: row.get(2)?,
                student_name: row.get(3)?,
                class_name: row.get(4)?,
                allocation_type,
                space_id: row.get(6)?,
                seat_no: row.get(7)?,
                subject_score: row.get(8)?,
            })
        })?;
        let mut student_allocations = Vec::new();
        for row in allocation_rows {
            student_allocations.push(row?);
        }

        let mut staff_stmt = conn.prepare(
            "SELECT id, session_id, space_id, teacher_name, assignment_type, note FROM latest_exam_plan_staff_assignments WHERE session_id = ?1 ORDER BY id ASC",
        )?;
        let staff_rows = staff_stmt.query_map(params![session_id], |row| {
            Ok(ExamPlanStaffAssignment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                space_id: row.get(2)?,
                teacher_name: row.get(3)?,
                assignment_type: row.get(4)?,
                note: row.get(5)?,
            })
        })?;
        let mut staff_assignments = Vec::new();
        for row in staff_rows {
            staff_assignments.push(row?);
        }

        Ok(ExamPlanSessionDetail {
            session,
            spaces,
            student_allocations,
            staff_assignments,
        })
    })();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_rebalance() {
        let rooms = calculate_room_capacities(122, 40, 41);
        assert_eq!(rooms, vec![41, 41, 40]);
    }

    #[test]
    fn test_capacity_keep_last_room() {
        let rooms = calculate_room_capacities(124, 40, 41);
        assert_eq!(rooms, vec![40, 40, 40, 4]);
    }

    #[test]
    fn test_round_robin_order() {
        let ordered = build_round_robin_order(&[
            Participant {
                admission_no: "1".to_string(),
                student_name: "A".to_string(),
                class_name: "高一1班".to_string(),
                score: Some(95.0),
            },
            Participant {
                admission_no: "2".to_string(),
                student_name: "B".to_string(),
                class_name: "高一1班".to_string(),
                score: Some(90.0),
            },
            Participant {
                admission_no: "3".to_string(),
                student_name: "C".to_string(),
                class_name: "高一2班".to_string(),
                score: Some(92.0),
            },
        ]);
        let ids: Vec<String> = ordered.into_iter().map(|p| p.admission_no).collect();
        assert_eq!(ids, vec!["1", "3", "2"]);
    }
}
