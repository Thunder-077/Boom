use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app_log;
use crate::db::repos::exam_allocation as exam_allocation_repo;
use crate::export_bundle;
use crate::score::{AppError, ListResult, Subject};

const DEFAULT_CAPACITY: i64 = 40;
const DEFAULT_MAX_CAPACITY: i64 = 41;
const GENERATION_STAGE_PAUSE_MS: u64 = 30;
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
    self_study_topic: Option<SelfStudyTopic>,
    building: String,
    floor: String,
    capacity: Option<i64>,
    sort_index: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelfStudyTopicKind {
    Subject,
    ForeignGroup,
    FreeStudy,
}

impl SelfStudyTopicKind {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Subject => "subject",
            Self::ForeignGroup => "foreign_group",
            Self::FreeStudy => "free_study",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "subject" => Some(Self::Subject),
            "foreign_group" => Some(Self::ForeignGroup),
            "free_study" => Some(Self::FreeStudy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SelfStudyTopic {
    pub kind: SelfStudyTopicKind,
    pub subjects: Vec<Subject>,
    pub label: String,
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
    total_score: f64,
    score: Option<f64>,
}

#[derive(Debug)]
struct SessionBuildResult {
    warning_count: i64,
}

#[derive(Debug, Clone)]
pub struct SelfStudyScheduleSession {
    pub subject: Subject,
    pub start_ts: i64,
    pub order_key: i64,
    pub is_foreign_group: bool,
}

#[derive(Debug, Clone)]
struct SpaceCandidate {
    space_type: ExamPlanSpaceType,
    space_source: ExamPlanSpaceSource,
    space_name: String,
    original_class_name: Option<String>,
    self_study_topic: Option<SelfStudyTopic>,
    building: String,
    floor: String,
    capacity: Option<i64>,
    sort_index: i64,
}

fn open_exam_allocation_db(app: &AppHandle) -> Result<DatabaseConnection, AppError> {
    tauri::async_runtime::block_on(crate::db::connect(app))
}

fn ensure_exam_allocation_defaults(db: &DatabaseConnection) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let default_notices_json = default_exam_notices_json()?;
    tauri::async_runtime::block_on(exam_allocation_repo::ensure_defaults(
        db,
        DEFAULT_CAPACITY,
        DEFAULT_MAX_CAPACITY,
        DEFAULT_EXAM_TITLE,
        &default_notices_json,
        &now,
    ))?;
    let settings = tauri::async_runtime::block_on(exam_allocation_repo::get_settings(db))?;
    if should_replace_exam_notices(&settings.exam_notices_json) {
        tauri::async_runtime::block_on(exam_allocation_repo::replace_default_notices_if_needed(
            db,
            &default_exam_notices_json()?,
        ))?;
    }
    seed_preset_grade_subject_time_templates(db)?;
    Ok(())
}

fn parse_schedule_timestamp(value: &str) -> Option<i64> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.timestamp_millis());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Some(naive.and_utc().timestamp_millis());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc().timestamp_millis());
    }
    None
}

fn load_grade_subject_schedule_order(
    db: &DatabaseConnection,
) -> Result<HashMap<String, HashMap<Subject, i64>>, AppError> {
    let mut out = HashMap::<String, HashMap<Subject, i64>>::new();
    for row in
        tauri::async_runtime::block_on(exam_allocation_repo::list_grade_subject_templates(db))?
    {
        let Some(subject) = Subject::from_key(&row.subject) else {
            continue;
        };
        let Some(ts) = parse_schedule_timestamp(&row.start_at) else {
            continue;
        };
        out.entry(row.grade_name).or_default().insert(subject, ts);
    }
    Ok(out)
}

pub(crate) fn grade_order_key(grade_name: &str) -> (i32, &str) {
    match grade_name {
        "高一" => (1, grade_name),
        "高二" => (2, grade_name),
        "高三" => (3, grade_name),
        _ => (4, grade_name),
    }
}

fn seed_preset_grade_subject_time_templates(db: &DatabaseConnection) -> Result<(), AppError> {
    let existing_templates =
        tauri::async_runtime::block_on(exam_allocation_repo::list_grade_subject_templates(db))?;
    if !existing_templates.is_empty() {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    // 根据用户提供的考试安排图片，预置高一/高二科目时间。
    let presets: [(&str, Subject, &str, &str); 18] = [
        (
            "高一",
            Subject::Chinese,
            "2026-04-28T08:00",
            "2026-04-28T10:30",
        ),
        (
            "高一",
            Subject::Physics,
            "2026-04-28T14:10",
            "2026-04-28T15:40",
        ),
        (
            "高一",
            Subject::Geography,
            "2026-04-28T16:10",
            "2026-04-28T17:40",
        ),
        (
            "高一",
            Subject::Math,
            "2026-04-29T08:00",
            "2026-04-29T10:00",
        ),
        (
            "高一",
            Subject::Biology,
            "2026-04-29T10:30",
            "2026-04-29T12:00",
        ),
        (
            "高一",
            Subject::Chemistry,
            "2026-04-29T14:10",
            "2026-04-29T15:40",
        ),
        (
            "高一",
            Subject::Politics,
            "2026-04-29T16:10",
            "2026-04-29T17:40",
        ),
        (
            "高一",
            Subject::History,
            "2026-04-30T08:00",
            "2026-04-30T09:30",
        ),
        (
            "高一",
            Subject::English,
            "2026-04-30T10:00",
            "2026-04-30T12:00",
        ),
        (
            "高二",
            Subject::Chinese,
            "2026-04-28T08:00",
            "2026-04-28T10:30",
        ),
        (
            "高二",
            Subject::Physics,
            "2026-04-28T14:10",
            "2026-04-28T15:40",
        ),
        (
            "高二",
            Subject::Geography,
            "2026-04-28T16:10",
            "2026-04-28T17:40",
        ),
        (
            "高二",
            Subject::English,
            "2026-04-29T08:00",
            "2026-04-29T10:00",
        ),
        (
            "高二",
            Subject::Biology,
            "2026-04-29T10:30",
            "2026-04-29T12:00",
        ),
        (
            "高二",
            Subject::Chemistry,
            "2026-04-29T14:10",
            "2026-04-29T15:40",
        ),
        (
            "高二",
            Subject::Politics,
            "2026-04-29T16:10",
            "2026-04-29T17:40",
        ),
        (
            "高二",
            Subject::History,
            "2026-04-30T08:00",
            "2026-04-30T09:30",
        ),
        (
            "高二",
            Subject::Math,
            "2026-04-30T10:00",
            "2026-04-30T12:00",
        ),
    ];
    let rows = presets
        .into_iter()
        .flat_map(|(grade_name, subject, start_at, end_at)| {
            let subjects = if subject == Subject::English {
                vec![Subject::English, Subject::Russian, Subject::Japanese]
            } else {
                vec![subject]
            };
            subjects.into_iter().map(move |subject| {
                exam_allocation_repo::GradeSubjectTemplateSeedRow {
                    grade_name: grade_name.to_string(),
                    subject: subject.as_key().to_string(),
                    start_at: start_at.to_string(),
                    end_at: end_at.to_string(),
                }
            })
        })
        .collect::<Vec<_>>();
    tauri::async_runtime::block_on(exam_allocation_repo::seed_grade_subject_templates(
        db, &rows, &now,
    ))?;
    Ok(())
}

fn resolve_grade_subject_schedule_order(
    grade_name: &str,
    subject: Subject,
    grade_order_map: &HashMap<String, HashMap<Subject, i64>>,
) -> i64 {
    if let Some(map) = grade_order_map.get(grade_name) {
        if let Some(ts) = map.get(&subject).copied() {
            return ts;
        }
        if matches!(subject, Subject::Russian | Subject::Japanese) {
            if let Some(ts) = map.get(&Subject::English).copied() {
                return ts;
            }
        }
    }
    // Keep deterministic ordering when no time template is configured yet.
    subject_order(subject) as i64
}

pub fn subject_label(subject: Subject) -> &'static str {
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

pub fn build_subject_self_study_topic(subject: Subject) -> SelfStudyTopic {
    SelfStudyTopic {
        kind: SelfStudyTopicKind::Subject,
        subjects: vec![subject],
        label: format!("{}自习", subject_label(subject)),
    }
}

pub fn build_free_study_topic() -> SelfStudyTopic {
    SelfStudyTopic {
        kind: SelfStudyTopicKind::FreeStudy,
        subjects: Vec::new(),
        label: "自由自习".to_string(),
    }
}

fn foreign_self_study_short_label(subject: Subject) -> &'static str {
    match subject {
        Subject::English => "英",
        Subject::Russian => "俄",
        Subject::Japanese => "日",
        _ => subject_label(subject),
    }
}

pub fn is_foreign_subject(subject: Subject) -> bool {
    matches!(
        subject,
        Subject::English | Subject::Russian | Subject::Japanese
    )
}

fn sort_subjects_for_topic(subjects: &mut Vec<Subject>) {
    subjects.sort_by_key(|subject| {
        if let Some(order) = foreign_order(*subject) {
            (0_i32, order as i32)
        } else {
            (1_i32, subject_order(*subject))
        }
    });
    subjects.dedup();
}

pub fn build_foreign_group_self_study_topic(subjects: Vec<Subject>) -> SelfStudyTopic {
    let mut subjects = subjects;
    sort_subjects_for_topic(&mut subjects);
    let names = subjects
        .iter()
        .map(|subject| foreign_self_study_short_label(*subject))
        .collect::<Vec<_>>()
        .join("、");
    SelfStudyTopic {
        kind: SelfStudyTopicKind::ForeignGroup,
        subjects,
        label: format!("外语自习（{}）", names),
    }
}

#[derive(Debug, Clone)]
struct SelfStudyFutureSlot {
    start_ts: i64,
    order_key: i64,
    is_foreign_group: bool,
    subjects: Vec<Subject>,
}

fn resolve_class_topic_for_slot(
    slot: &SelfStudyFutureSlot,
    subjects_for_class: &HashSet<Subject>,
) -> Option<SelfStudyTopic> {
    if slot.is_foreign_group {
        let mut matched = slot
            .subjects
            .iter()
            .copied()
            .filter(|subject| subjects_for_class.contains(subject))
            .collect::<Vec<_>>();
        sort_subjects_for_topic(&mut matched);
        return match matched.len() {
            0 => None,
            1 => Some(build_subject_self_study_topic(matched[0])),
            _ => Some(build_foreign_group_self_study_topic(matched)),
        };
    }

    let subject = slot.subjects[0];
    subjects_for_class
        .contains(&subject)
        .then_some(build_subject_self_study_topic(subject))
}

pub fn build_self_study_topic_chain(
    current_start_ts: i64,
    class_name: &str,
    grade_sessions: &[SelfStudyScheduleSession],
    class_subjects: &HashMap<String, HashSet<Subject>>,
) -> Vec<SelfStudyTopic> {
    let Some(subjects_for_class) = class_subjects.get(class_name) else {
        return vec![build_free_study_topic()];
    };

    let mut ordered_sessions = grade_sessions.to_vec();
    ordered_sessions.sort_by(|a, b| {
        a.start_ts
            .cmp(&b.start_ts)
            .then(a.order_key.cmp(&b.order_key))
            .then(subject_order(a.subject).cmp(&subject_order(b.subject)))
    });

    let mut slots = Vec::<SelfStudyFutureSlot>::new();
    for session in ordered_sessions {
        if session.is_foreign_group {
            if let Some(last) = slots.last_mut() {
                if last.is_foreign_group && last.start_ts == session.start_ts {
                    last.subjects.push(session.subject);
                    continue;
                }
            }
        }
        slots.push(SelfStudyFutureSlot {
            start_ts: session.start_ts,
            order_key: session.order_key,
            is_foreign_group: session.is_foreign_group,
            subjects: vec![session.subject],
        });
    }
    slots.sort_by(|a, b| {
        a.start_ts
            .cmp(&b.start_ts)
            .then(a.order_key.cmp(&b.order_key))
    });

    // 自习主题始终围绕“这个班下一门还要考什么”来推荐。
    // 这里不再根据历史自习场次跳过未来科目，避免连续两场自习被推进到不同科目，
    // 也避免明明后续还有考试却被回退成“自由自习”。
    let mut chain = Vec::<SelfStudyTopic>::new();
    for slot in &slots {
        if slot.start_ts <= current_start_ts {
            continue;
        }
        if let Some(topic) = resolve_class_topic_for_slot(slot, subjects_for_class) {
            chain.push(topic);
        }
    }

    if chain.is_empty() {
        chain.push(build_free_study_topic());
    }
    chain
}

fn deserialize_self_study_topic(
    kind_key: Option<String>,
    subjects_json: Option<String>,
    label: Option<String>,
) -> Result<Option<SelfStudyTopic>, AppError> {
    let Some(kind_key) = kind_key else {
        return Ok(None);
    };
    let kind = SelfStudyTopicKind::from_key(&kind_key)
        .ok_or_else(|| AppError::new(format!("无效的考试期间自习主题类型: {kind_key}")))?;
    let subjects = match subjects_json {
        Some(value) if !value.trim().is_empty() => serde_json::from_str::<Vec<Subject>>(&value)
            .map_err(|e| AppError::new(format!("考试期间自习主题科目解析失败: {e}")))?,
        _ => Vec::new(),
    };
    Ok(Some(SelfStudyTopic {
        kind,
        subjects,
        label: label.unwrap_or_default(),
    }))
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

fn load_settings(db: &DatabaseConnection) -> Result<ExamAllocationSettings, AppError> {
    let row = tauri::async_runtime::block_on(exam_allocation_repo::get_settings(db))?;
    let exam_notices = serde_json::from_str::<Vec<String>>(&row.exam_notices_json)
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    Ok(ExamAllocationSettings {
        default_capacity: row.default_capacity,
        max_capacity: row.max_capacity,
        exam_title: row.exam_title,
        exam_notices,
        updated_at: Some(row.updated_at),
    })
}

fn load_grade_contexts(db: &DatabaseConnection) -> Result<HashMap<String, GradeContext>, AppError> {
    let mut ctx_map: HashMap<String, GradeContext> = HashMap::new();
    for row in tauri::async_runtime::block_on(exam_allocation_repo::list_class_config_rows(db))? {
        let grade_name = row.grade_name;
        let class_name = row.class_name;
        let building = row.building;
        let floor = row.floor;
        let grade_ctx = ctx_map.entry(grade_name).or_default();
        if row.config_type == "teaching_class" {
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
            if let Some(subject_key) = row.subject {
                if let Some(subject) = Subject::from_key(&subject_key) {
                    grade_ctx
                        .class_subjects
                        .entry(class_name)
                        .or_default()
                        .insert(subject);
                }
            }
        } else if row.config_type == "exam_room" {
            // Exam-room allocation now always uses the configured class_name as the exported room name.
            grade_ctx.exam_rooms.push(ExamRoomResource {
                room_name: class_name,
                building,
                floor,
            });
        }
    }

    for ctx in ctx_map.values_mut() {
        ctx.teaching_classes
            .sort_by(|a, b| sort_class_names(&a.class_name, &b.class_name));
        ctx.exam_rooms.sort_by(|a, b| a.room_name.cmp(&b.room_name));
    }

    Ok(ctx_map)
}

fn load_active_grade_subjects(
    db: &DatabaseConnection,
) -> Result<HashMap<String, HashSet<Subject>>, AppError> {
    let mut active = HashMap::<String, HashSet<Subject>>::new();
    for row in tauri::async_runtime::block_on(
        exam_allocation_repo::list_active_grade_subjects(db),
    )? {
        let Some(subject) = Subject::from_key(&row.subject) else {
            continue;
        };
        active.entry(row.grade_name).or_default().insert(subject);
    }
    Ok(active)
}

fn load_selected_participants(
    db: &DatabaseConnection,
    grade_name: &str,
    subject: Subject,
) -> Result<Vec<Participant>, AppError> {
    Ok(
        tauri::async_runtime::block_on(exam_allocation_repo::list_participants(
            db,
            grade_name,
            subject.as_key(),
            1,
        ))?
        .into_iter()
        .map(|row| Participant {
            admission_no: row.admission_no,
            student_name: row.student_name,
            class_name: row.class_name,
            total_score: row.total_score,
            score: row.score,
        })
        .collect(),
    )
}

fn load_not_selected_students(
    db: &DatabaseConnection,
    grade_name: &str,
    subject: Subject,
) -> Result<Vec<Participant>, AppError> {
    Ok(
        tauri::async_runtime::block_on(exam_allocation_repo::list_participants(
            db,
            grade_name,
            subject.as_key(),
            0,
        ))?
        .into_iter()
        .map(|row| Participant {
            admission_no: row.admission_no,
            student_name: row.student_name,
            class_name: row.class_name,
            total_score: row.total_score,
            score: row.score,
        })
        .collect(),
    )
}

fn load_self_study_students_for_session(
    db: &DatabaseConnection,
    grade_name: &str,
    subject: Subject,
    grade_sessions: &[SelfStudyScheduleSession],
    current_start_ts: i64,
) -> Result<Vec<Participant>, AppError> {
    if is_foreign_subject(subject) {
        return Ok(Vec::new());
    }

    let mut concurrent_subjects = grade_sessions
        .iter()
        .filter(|session| session.start_ts == current_start_ts)
        .map(|session| session.subject)
        .collect::<Vec<_>>();
    concurrent_subjects.sort_by_key(|item| subject_order(*item));
    concurrent_subjects.dedup();

    // A self-study room belongs to the time slot, not to every subject session in that slot.
    // Use the first non-foreign subject as the slot owner to avoid duplicate self-study tasks.
    let slot_owner = concurrent_subjects
        .iter()
        .copied()
        .find(|item| !is_foreign_subject(*item))
        .unwrap_or(subject);
    if subject != slot_owner {
        return Ok(Vec::new());
    }

    let mut students = load_not_selected_students(db, grade_name, subject)?;
    if concurrent_subjects.len() <= 1 {
        return Ok(students);
    }

    let mut concurrent_examinees = HashSet::<String>::new();
    for concurrent_subject in concurrent_subjects {
        if concurrent_subject == subject {
            continue;
        }
        for participant in load_selected_participants(db, grade_name, concurrent_subject)? {
            concurrent_examinees.insert(participant.admission_no);
        }
    }
    students.retain(|student| !concurrent_examinees.contains(&student.admission_no));
    Ok(students)
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
            // 班内按总分从高到低排序，保证座位编排以综合成绩为准。
            b.total_score
                .partial_cmp(&a.total_score)
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

pub(crate) fn clear_latest_plan_snapshot(tx: &DatabaseTransaction) -> Result<(), AppError> {
    tauri::async_runtime::block_on(exam_allocation_repo::clear_latest_plan_snapshot(tx))?;
    Ok(())
}

fn fill_with_configured_exam_rooms(
    grade_name: &str,
    subject: Subject,
    chosen_spaces: &mut Vec<SpaceCandidate>,
    required_room_count: usize,
    exam_rooms: &[ExamRoomResource],
) -> Result<(), AppError> {
    for room in exam_rooms {
        if chosen_spaces.len() >= required_room_count {
            break;
        }
        chosen_spaces.push(SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::ExamRoom,
            space_name: room.room_name.clone(),
            original_class_name: None,
            self_study_topic: None,
            building: room.building.clone(),
            floor: room.floor.clone(),
            capacity: None,
            sort_index: chosen_spaces.len() as i64 + 1,
        });
    }

    if chosen_spaces.len() < required_room_count {
        return Err(AppError::new(format!(
            "{} {} 考场不足：需要 {} 个考场，现有教学教室和 exam_room 共 {} 个。请在 class_configs 中补充 exam_room 配置。",
            grade_name,
            subject_label(subject),
            required_room_count,
            chosen_spaces.len()
        )));
    }
    Ok(())
}

fn build_session(
    db: &DatabaseConnection,
    tx: &DatabaseTransaction,
    grade_name: &str,
    subject: Subject,
    grade_ctx: &GradeContext,
    grade_schedule_sessions: &[SelfStudyScheduleSession],
    current_start_ts: i64,
    default_capacity: i64,
    max_capacity: i64,
    foreign_occupied_classes: &mut HashSet<String>,
) -> Result<SessionBuildResult, AppError> {
    let mut warnings = 0_i64;
    let is_foreign = is_foreign_subject(subject);
    let foreign_seq = foreign_order(subject);
    let not_selected = load_self_study_students_for_session(
        db,
        grade_name,
        subject,
        grade_schedule_sessions,
        current_start_ts,
    )?;
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

    let mut participants = load_selected_participants(db, grade_name, subject)?;
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
            self_study_topic: None,
            building: classroom.building,
            floor: classroom.floor,
            capacity: None,
            sort_index: chosen_spaces.len() as i64 + 1,
        });
    }
    fill_with_configured_exam_rooms(
        grade_name,
        subject,
        &mut chosen_spaces,
        required_room_count,
        &grade_ctx.exam_rooms,
    )?;
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
            self_study_topic: Some(
                build_self_study_topic_chain(
                    current_start_ts,
                    &classroom.class_name,
                    grade_schedule_sessions,
                    &grade_ctx.class_subjects,
                )
                .into_iter()
                .next()
                .unwrap_or_else(build_free_study_topic),
            ),
            building: classroom.building.clone(),
            floor: classroom.floor.clone(),
            capacity: None,
            sort_index: (chosen_spaces.len() + self_study_spaces.len()) as i64 + 1,
        });
    }

    let session_id = tauri::async_runtime::block_on(exam_allocation_repo::insert_session(
        tx,
        exam_allocation_repo::SessionInsertRow {
            grade_name: grade_name.to_string(),
            subject: subject.as_key().to_string(),
            is_foreign_group: if is_foreign { 1 } else { 0 },
            foreign_order: foreign_seq,
            participant_count: participants.len() as i64,
            exam_room_count: chosen_spaces.len() as i64,
            self_study_room_count: self_study_spaces.len() as i64,
        },
    ))?;

    let mut exam_space_ids = Vec::new();
    for (index, space) in chosen_spaces.iter_mut().enumerate() {
        space.capacity = capacities.get(index).copied();
        let space_id = tauri::async_runtime::block_on(exam_allocation_repo::insert_space(
            tx,
            exam_allocation_repo::SpaceInsertRow {
                session_id,
                space_type: space.space_type.as_key().to_string(),
                space_source: space.space_source.as_key().to_string(),
                grade_name: grade_name.to_string(),
                subject: subject.as_key().to_string(),
                space_name: space.space_name.clone(),
                original_class_name: space.original_class_name.clone(),
                self_study_topic_kind: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| value.kind.as_key().to_string()),
                self_study_topic_subjects_json: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| serde_json::to_string(&value.subjects))
                    .transpose()
                    .map_err(|e| AppError::new(format!("考试期间自习主题科目序列化失败: {e}")))?,
                self_study_topic_label: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| value.label.clone()),
                building: space.building.clone(),
                floor: space.floor.clone(),
                capacity: space.capacity,
                sort_index: space.sort_index,
            },
        ))?;
        exam_space_ids.push(space_id);
    }

    let mut self_study_space_by_class = HashMap::new();
    let mut self_study_ids = Vec::new();
    for space in &self_study_spaces {
        let id = tauri::async_runtime::block_on(exam_allocation_repo::insert_space(
            tx,
            exam_allocation_repo::SpaceInsertRow {
                session_id,
                space_type: space.space_type.as_key().to_string(),
                space_source: space.space_source.as_key().to_string(),
                grade_name: grade_name.to_string(),
                subject: subject.as_key().to_string(),
                space_name: space.space_name.clone(),
                original_class_name: space.original_class_name.clone(),
                self_study_topic_kind: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| value.kind.as_key().to_string()),
                self_study_topic_subjects_json: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| serde_json::to_string(&value.subjects))
                    .transpose()
                    .map_err(|e| AppError::new(format!("考试期间自习主题科目序列化失败: {e}")))?,
                self_study_topic_label: space
                    .self_study_topic
                    .as_ref()
                    .map(|value| value.label.clone()),
                building: space.building.clone(),
                floor: space.floor.clone(),
                capacity: None,
                sort_index: space.sort_index,
            },
        ))?;
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
            tauri::async_runtime::block_on(exam_allocation_repo::insert_student_allocation(
                tx,
                exam_allocation_repo::StudentAllocationInsertRow {
                    session_id,
                    admission_no: student.admission_no.clone(),
                    student_name: student.student_name.clone(),
                    class_name: student.class_name.clone(),
                    allocation_type: ExamAllocationType::Exam.as_key().to_string(),
                    space_id: exam_space_ids.get(space_index).copied(),
                    seat_no: Some(seat_idx as i64 + 1),
                    subject_score: student.score,
                },
            ))?;
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
        tauri::async_runtime::block_on(exam_allocation_repo::insert_student_allocation(
            tx,
            exam_allocation_repo::StudentAllocationInsertRow {
                session_id,
                admission_no: student.admission_no,
                student_name: student.student_name,
                class_name: student.class_name,
                allocation_type: ExamAllocationType::SelfStudy.as_key().to_string(),
                space_id: Some(mapped_id),
                seat_no: None,
                subject_score: None,
            },
        ))?;
    }

    Ok(SessionBuildResult {
        warning_count: warnings,
    })
}

fn update_exam_generation_progress(
    db: &DatabaseConnection,
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
    tauri::async_runtime::block_on(exam_allocation_repo::update_progress(
        db,
        exam_allocation_repo::ProgressRow {
            status: status.to_string(),
            stage: stage.to_string(),
            stage_label: stage_label.to_string(),
            percent,
            message: message.to_string(),
            current_grade: current_grade.map(ToString::to_string),
            total_grades,
            completed_grades,
            updated_at: now,
        },
    ))?;
    Ok(())
}

fn pause_after_generation_stage() {
    thread::sleep(Duration::from_millis(GENERATION_STAGE_PAUSE_MS));
}

pub fn get_exam_allocation_settings(app: AppHandle) -> Result<ExamAllocationSettings, String> {
    let result = (|| -> Result<ExamAllocationSettings, AppError> {
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        load_settings(&db)
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
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let now = Utc::now().to_rfc3339();
        tauri::async_runtime::block_on(exam_allocation_repo::update_settings(
            &db,
            payload.default_capacity,
            payload.max_capacity,
            &exam_title,
            &exam_notices_json,
            &now,
        ))?;
        Ok(SuccessResponse::ok())
    })();
    result.map_err(|e| e.to_string())
}

fn generate_latest_exam_plan_internal(
    app: &AppHandle,
    payload: Option<GenerateLatestExamPlanPayload>,
) -> Result<GenerateLatestExamPlanResult, AppError> {
    let db = open_exam_allocation_db(app)?;
    ensure_exam_allocation_defaults(&db)?;
    update_exam_generation_progress(
        &db,
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
    let settings = load_settings(&db)?;
    let default_capacity = payload
        .as_ref()
        .and_then(|p| p.default_capacity)
        .unwrap_or(settings.default_capacity);
    let max_capacity = payload
        .as_ref()
        .and_then(|p| p.max_capacity)
        .unwrap_or(settings.max_capacity);
    validate_capacity(default_capacity, max_capacity)?;

    let grade_contexts = load_grade_contexts(&db)?;
    let active_grade_subjects = load_active_grade_subjects(&db)?;
    let grade_subject_schedule_order = load_grade_subject_schedule_order(&db)?;
    let mut grades: Vec<String> = grade_contexts
        .keys()
        .filter(|grade_name| {
            active_grade_subjects
                .get(*grade_name)
                .is_some_and(|subjects| !subjects.is_empty())
        })
        .cloned()
        .collect();
    grades.sort_by(|a, b| grade_order_key(a).cmp(&grade_order_key(b)).then(a.cmp(b)));
    let total_grades = grades.len() as i64;
    let _ = app_log::append_log(
        app,
        "info",
        "exam_allocation.generate_latest_exam_plan",
        &format!(
            "loaded grade contexts: grades={} [{}]",
            total_grades,
            grades.join(", ")
        ),
    );
    if grades.is_empty() {
        return Err(AppError::new(
            "未读取到同时具备班级配置和实际考生的年级，请检查成绩导入与班级配置。",
        ));
    }
    update_exam_generation_progress(
        &db,
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
    {
        let clear_tx = tauri::async_runtime::block_on(db.begin())?;
        clear_latest_plan_snapshot(&clear_tx)?;
        tauri::async_runtime::block_on(clear_tx.commit())?;
    }
    update_exam_generation_progress(
        &db,
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
            &db,
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
        if let Some(active_subjects) = active_grade_subjects.get(grade_name) {
            subject_set.retain(|subject| active_subjects.contains(subject));
        } else {
            subject_set.clear();
        }
        let mut subjects: Vec<Subject> = subject_set.into_iter().collect();
        subjects.sort_by_key(|s| subject_order(*s));
        let _ = app_log::append_log(
            app,
            "info",
            "exam_allocation.generate_latest_exam_plan",
            &format!(
                "building grade={grade_name} subjects={} [{}]",
                subjects.len(),
                subjects
                    .iter()
                    .map(|subject| subject.as_key())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        if subjects.is_empty() {
            return Err(AppError::new(format!(
                "{grade_name} 未配置任何考试科目，请检查班级配置中的科目设置。"
            )));
        }
        let mut current_grade_schedule_order = HashMap::<Subject, i64>::new();
        for subject in &subjects {
            current_grade_schedule_order.insert(
                *subject,
                resolve_grade_subject_schedule_order(
                    grade_name,
                    *subject,
                    &grade_subject_schedule_order,
                ),
            );
        }
        let grade_schedule_sessions = subjects
            .iter()
            .enumerate()
            .map(|(index, subject)| SelfStudyScheduleSession {
                subject: *subject,
                start_ts: current_grade_schedule_order
                    .get(subject)
                    .copied()
                    .unwrap_or_else(|| subject_order(*subject) as i64),
                order_key: index as i64,
                is_foreign_group: is_foreign_subject(*subject),
            })
            .collect::<Vec<_>>();

        let grade_tx = tauri::async_runtime::block_on(db.begin())?;
        let mut foreign_occupied = HashSet::new();
        for subject in subjects {
            let current_start_ts = current_grade_schedule_order
                .get(&subject)
                .copied()
                .unwrap_or_else(|| subject_order(subject) as i64);
            let built = build_session(
                &db,
                &grade_tx,
                grade_name,
                subject,
                grade_ctx,
                &grade_schedule_sessions,
                current_start_ts,
                default_capacity,
                max_capacity,
                &mut foreign_occupied,
            )?;
            session_count += 1;
            warning_count += built.warning_count;
        }
        tauri::async_runtime::block_on(grade_tx.commit())?;
    }
    if session_count == 0 {
        return Err(AppError::new(
            "未生成任何考试场次，请检查班级配置中的考试科目和成绩数据是否匹配。",
        ));
    }

    {
        let meta_tx = tauri::async_runtime::block_on(db.begin())?;
        tauri::async_runtime::block_on(exam_allocation_repo::insert_plan_meta(
            &meta_tx,
            exam_allocation_repo::PlanMetaInsertRow {
                generated_at: generated_at.clone(),
                default_capacity,
                max_capacity,
                grade_count: grades.len() as i64,
                session_count,
                warning_count,
            },
        ))?;
        tauri::async_runtime::block_on(meta_tx.commit())?;
    }
    update_exam_generation_progress(
        &db,
        "running",
        "finalizing_results",
        "整理结果",
        76,
        "正在整理场次时间与分配结果路径",
        None,
        total_grades,
        total_grades,
    )?;
    pause_after_generation_stage();
    tauri::async_runtime::block_on(async {
        crate::db::repos::exam_staff::seed_default_session_times(&db, &Utc::now().to_rfc3339())
            .await
    })?;
    update_exam_generation_progress(
        &db,
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
    export_bundle::generate_export_files(&app, |grade_name, done, total| {
        let percent = 82 + (((done as i64) * 16) / (total as i64).max(1));
        let _ = update_exam_generation_progress(
            &db,
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
        &db,
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
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let progress = tauri::async_runtime::block_on(exam_allocation_repo::get_progress(&db))?;
        if progress.status == "running" {
            return Err(AppError::new("考场分配正在执行中，请稍候"));
        }
        update_exam_generation_progress(
            &db,
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
                if let Ok(db) = open_exam_allocation_db(&app_handle) {
                    let _ = ensure_exam_allocation_defaults(&db);
                    let _ = update_exam_generation_progress(
                        &db,
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
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let settings = load_settings(&db)?;
        let meta_row = tauri::async_runtime::block_on(exam_allocation_repo::latest_plan_meta(&db))?;
        let counts = tauri::async_runtime::block_on(exam_allocation_repo::overview_counts(&db))?;
        Ok(ExamPlanOverview {
            generated_at: meta_row.as_ref().map(|v| v.generated_at.clone()),
            default_capacity: settings.default_capacity,
            max_capacity: settings.max_capacity,
            grade_count: meta_row.as_ref().map(|v| v.grade_count).unwrap_or(0),
            session_count: meta_row.as_ref().map(|v| v.session_count).unwrap_or(0),
            warning_count: meta_row.as_ref().map(|v| v.warning_count).unwrap_or(0),
            exam_room_count: counts.exam_room_count,
            self_study_room_count: counts.self_study_room_count,
            student_allocation_count: counts.student_allocation_count,
        })
    })();
    result.map_err(|e| e.to_string())
}

pub fn get_exam_generation_progress(app: AppHandle) -> Result<ExamGenerationProgress, String> {
    let result = (|| -> Result<ExamGenerationProgress, AppError> {
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let row = tauri::async_runtime::block_on(exam_allocation_repo::get_progress(&db))?;
        Ok(ExamGenerationProgress {
            status: row.status,
            stage: row.stage,
            stage_label: row.stage_label,
            percent: row.percent,
            message: row.message,
            current_grade: row.current_grade,
            total_grades: row.total_grades,
            completed_grades: row.completed_grades,
            updated_at: row.updated_at,
        })
    })();
    result.map_err(|e| e.to_string())
}

pub fn list_latest_exam_plan_sessions(
    app: AppHandle,
    params: ListExamPlanSessionsParams,
) -> Result<ListResult<ExamPlanSession>, String> {
    let result = (|| -> Result<ListResult<ExamPlanSession>, AppError> {
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let grade_name = if let Some(grade_name) = params
            .grade_name
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            Some(grade_name.to_string())
        } else {
            None
        };
        let subject = params.subject.map(|subject| subject.as_key().to_string());
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(100).clamp(1, 500);
        let result = tauri::async_runtime::block_on(exam_allocation_repo::list_sessions(
            &db,
            exam_allocation_repo::SessionFilters {
                grade_name,
                subject,
                page,
                page_size,
            },
        ))?;
        let items = result
            .items
            .into_iter()
            .map(session_from_model)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ListResult {
            items,
            total: result.total,
        })
    })();
    result.map_err(|e| e.to_string())
}

fn session_from_model(
    row: crate::entity::latest_exam_plan_sessions::Model,
) -> Result<ExamPlanSession, AppError> {
    let subject = Subject::from_key(&row.subject)
        .ok_or_else(|| AppError::new(format!("无效的科目: {}", row.subject)))?;
    Ok(ExamPlanSession {
        id: row.id,
        grade_name: row.grade_name,
        subject,
        is_foreign_group: row.is_foreign_group == 1,
        foreign_order: row.foreign_order,
        participant_count: row.participant_count,
        exam_room_count: row.exam_room_count,
        self_study_room_count: row.self_study_room_count,
    })
}

fn space_from_model(
    row: crate::entity::latest_exam_plan_spaces::Model,
) -> Result<ExamPlanSpace, AppError> {
    let space_type = ExamPlanSpaceType::from_key(&row.space_type)
        .ok_or_else(|| AppError::new(format!("无效的空间类型: {}", row.space_type)))?;
    let space_source = ExamPlanSpaceSource::from_key(&row.space_source)
        .ok_or_else(|| AppError::new(format!("无效的空间来源: {}", row.space_source)))?;
    let subject = Subject::from_key(&row.subject)
        .ok_or_else(|| AppError::new(format!("无效的科目: {}", row.subject)))?;
    let self_study_topic = deserialize_self_study_topic(
        row.self_study_topic_kind,
        row.self_study_topic_subjects_json,
        row.self_study_topic_label,
    )?;
    Ok(ExamPlanSpace {
        id: row.id,
        session_id: row.session_id,
        space_type,
        space_source,
        grade_name: row.grade_name,
        subject,
        space_name: row.space_name,
        original_class_name: row.original_class_name,
        self_study_topic,
        building: row.building,
        floor: row.floor,
        capacity: row.capacity,
        sort_index: row.sort_index,
    })
}

fn student_allocation_from_model(
    row: crate::entity::latest_exam_plan_student_allocations::Model,
) -> Result<ExamPlanStudentAllocation, AppError> {
    let allocation_type = ExamAllocationType::from_key(&row.allocation_type)
        .ok_or_else(|| AppError::new(format!("无效的分配类型: {}", row.allocation_type)))?;
    Ok(ExamPlanStudentAllocation {
        id: row.id,
        session_id: row.session_id,
        admission_no: row.admission_no,
        student_name: row.student_name,
        class_name: row.class_name,
        allocation_type,
        space_id: row.space_id,
        seat_no: row.seat_no,
        subject_score: row.subject_score,
    })
}

pub fn get_latest_exam_plan_session_detail(
    app: AppHandle,
    session_id: i64,
) -> Result<ExamPlanSessionDetail, String> {
    let result = (|| -> Result<ExamPlanSessionDetail, AppError> {
        let db = open_exam_allocation_db(&app)?;
        ensure_exam_allocation_defaults(&db)?;
        let session =
            tauri::async_runtime::block_on(exam_allocation_repo::get_session(&db, session_id))?
                .ok_or_else(|| AppError::new("未找到考试场次"))?;
        let session = session_from_model(session)?;

        let spaces =
            tauri::async_runtime::block_on(exam_allocation_repo::list_spaces(&db, session_id))?
                .into_iter()
                .map(space_from_model)
                .collect::<Result<Vec<_>, _>>()?;

        let student_allocations = tauri::async_runtime::block_on(
            exam_allocation_repo::list_student_allocations(&db, session_id),
        )?
        .into_iter()
        .map(student_allocation_from_model)
        .collect::<Result<Vec<_>, _>>()?;

        let staff_assignments = tauri::async_runtime::block_on(
            exam_allocation_repo::list_staff_assignments(&db, session_id),
        )?
        .into_iter()
        .map(|row| ExamPlanStaffAssignment {
            id: row.id,
            session_id: row.session_id,
            space_id: row.space_id,
            teacher_name: row.teacher_name,
            assignment_type: row.assignment_type,
            note: row.note,
        })
        .collect();

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
    use crate::db::migration::Migrator;
    use crate::entity::{latest_student_scores, latest_subject_scores};
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database};
    use sea_orm_migration::MigratorTrait;

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
                total_score: 600.0,
                score: Some(95.0),
            },
            Participant {
                admission_no: "2".to_string(),
                student_name: "B".to_string(),
                class_name: "高一1班".to_string(),
                total_score: 590.0,
                score: Some(90.0),
            },
            Participant {
                admission_no: "3".to_string(),
                student_name: "C".to_string(),
                class_name: "高一2班".to_string(),
                total_score: 595.0,
                score: Some(92.0),
            },
        ]);
        let ids: Vec<String> = ordered.into_iter().map(|p| p.admission_no).collect();
        assert_eq!(ids, vec!["1", "3", "2"]);
    }

    #[test]
    fn test_round_robin_order_uses_total_score_within_class() {
        let ordered = build_round_robin_order(&[
            Participant {
                admission_no: "1".to_string(),
                student_name: "A".to_string(),
                class_name: "高一1班".to_string(),
                total_score: 580.0,
                score: Some(98.0),
            },
            Participant {
                admission_no: "2".to_string(),
                student_name: "B".to_string(),
                class_name: "高一1班".to_string(),
                total_score: 600.0,
                score: Some(90.0),
            },
            Participant {
                admission_no: "3".to_string(),
                student_name: "C".to_string(),
                class_name: "高一2班".to_string(),
                total_score: 590.0,
                score: Some(95.0),
            },
        ]);
        let ids: Vec<String> = ordered.into_iter().map(|p| p.admission_no).collect();
        assert_eq!(ids, vec!["2", "3", "1"]);
    }

    #[test]
    fn test_self_study_topic_chain_supports_single_foreign_subject() {
        let sessions = vec![
            SelfStudyScheduleSession {
                subject: Subject::English,
                start_ts: 2_000,
                order_key: 1,
                is_foreign_group: true,
            },
            SelfStudyScheduleSession {
                subject: Subject::Russian,
                start_ts: 2_000,
                order_key: 2,
                is_foreign_group: true,
            },
        ];
        let class_subjects =
            HashMap::from([("高二1班".to_string(), HashSet::from([Subject::English]))]);
        let chain = build_self_study_topic_chain(1_000, "高二1班", &sessions, &class_subjects);
        assert_eq!(
            chain,
            vec![build_subject_self_study_topic(Subject::English)]
        );
    }

    #[test]
    fn test_self_study_topic_chain_supports_foreign_group_topic() {
        let sessions = vec![
            SelfStudyScheduleSession {
                subject: Subject::English,
                start_ts: 2_000,
                order_key: 1,
                is_foreign_group: true,
            },
            SelfStudyScheduleSession {
                subject: Subject::Russian,
                start_ts: 2_000,
                order_key: 2,
                is_foreign_group: true,
            },
            SelfStudyScheduleSession {
                subject: Subject::Japanese,
                start_ts: 2_000,
                order_key: 3,
                is_foreign_group: true,
            },
        ];
        let class_subjects = HashMap::from([(
            "高二8班".to_string(),
            HashSet::from([Subject::English, Subject::Russian]),
        )]);
        let chain = build_self_study_topic_chain(1_000, "高二8班", &sessions, &class_subjects);
        assert_eq!(
            chain,
            vec![build_foreign_group_self_study_topic(vec![
                Subject::English,
                Subject::Russian,
            ])]
        );
    }

    #[test]
    fn test_foreign_group_self_study_topic_uses_short_label() {
        let topic = build_foreign_group_self_study_topic(vec![Subject::English, Subject::Russian]);
        assert_eq!(topic.label, "外语自习（英、俄）");
    }

    #[test]
    fn test_fill_with_configured_exam_rooms_uses_exam_rooms_after_teaching_classes() {
        let mut chosen_spaces = vec![SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::TeachingClass,
            space_name: "高一1场".to_string(),
            original_class_name: Some("高一1班".to_string()),
            self_study_topic: None,
            building: "向远楼".to_string(),
            floor: "3层".to_string(),
            capacity: None,
            sort_index: 1,
        }];
        let exam_rooms = vec![
            ExamRoomResource {
                room_name: "高一5场".to_string(),
                building: "向远楼".to_string(),
                floor: "5层".to_string(),
            },
            ExamRoomResource {
                room_name: "高一6场".to_string(),
                building: "向远楼".to_string(),
                floor: "5层".to_string(),
            },
        ];

        fill_with_configured_exam_rooms("高一", Subject::Math, &mut chosen_spaces, 3, &exam_rooms)
            .unwrap();

        assert_eq!(chosen_spaces.len(), 3);
        assert_eq!(
            chosen_spaces
                .iter()
                .map(|space| space.space_source)
                .collect::<Vec<_>>(),
            vec![
                ExamPlanSpaceSource::TeachingClass,
                ExamPlanSpaceSource::ExamRoom,
                ExamPlanSpaceSource::ExamRoom,
            ]
        );
        assert_eq!(chosen_spaces[1].space_name, "高一5场");
        assert_eq!(chosen_spaces[2].space_name, "高一6场");
    }

    #[test]
    fn test_fill_with_configured_exam_rooms_errors_when_rooms_are_insufficient() {
        let mut chosen_spaces = vec![SpaceCandidate {
            space_type: ExamPlanSpaceType::ExamRoom,
            space_source: ExamPlanSpaceSource::TeachingClass,
            space_name: "高一1场".to_string(),
            original_class_name: Some("高一1班".to_string()),
            self_study_topic: None,
            building: "向远楼".to_string(),
            floor: "3层".to_string(),
            capacity: None,
            sort_index: 1,
        }];
        let exam_rooms = vec![ExamRoomResource {
            room_name: "高一5场".to_string(),
            building: "向远楼".to_string(),
            floor: "5层".to_string(),
        }];

        let err = fill_with_configured_exam_rooms(
            "高一",
            Subject::Math,
            &mut chosen_spaces,
            3,
            &exam_rooms,
        )
        .expect_err("应在 teaching_class + exam_room 仍不足时直接报错");

        let message = err.to_string();
        assert!(message.contains("高一 数学 考场不足"));
        assert!(message.contains("请在 class_configs 中补充 exam_room 配置"));
        assert_eq!(chosen_spaces.len(), 2);
        assert_eq!(chosen_spaces[1].space_source, ExamPlanSpaceSource::ExamRoom);
    }

    #[test]
    fn test_foreign_sessions_do_not_create_subject_based_self_study_students() {
        let db = tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            Migrator::up(&db, None).await.unwrap();
            latest_student_scores::ActiveModel {
                admission_no: Set("s1".to_string()),
                student_name: Set("张三".to_string()),
                class_name: Set("高一1班".to_string()),
                grade_name: Set("高一".to_string()),
                subject_combination: Set(String::new()),
                language: Set(String::new()),
                total_score: Set(600.0),
                class_rank: Set(1),
                grade_rank: Set(1),
                selected_subject_count: Set(1),
            }
            .insert(&db)
            .await
            .unwrap();
            for (subject, is_selected) in [("english", 0), ("russian", 1), ("math", 0)] {
                latest_subject_scores::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    admission_no: Set("s1".to_string()),
                    subject: Set(subject.to_string()),
                    score: Set(None),
                    is_selected: Set(is_selected),
                    is_absent: Set(0),
                }
                .insert(&db)
                .await
                .unwrap();
            }
            db
        });

        let english_self_study = load_self_study_students_for_session(
            &db,
            "高一",
            Subject::English,
            &[],
            0,
        )
        .unwrap();
        let russian_self_study = load_self_study_students_for_session(
            &db,
            "高一",
            Subject::Russian,
            &[],
            0,
        )
        .unwrap();
        let math_self_study =
            load_self_study_students_for_session(&db, "高一", Subject::Math, &[], 0).unwrap();

        assert!(english_self_study.is_empty());
        assert!(russian_self_study.is_empty());
        assert_eq!(math_self_study.len(), 1);
        assert_eq!(math_self_study[0].class_name, "高一1班");
    }

    #[test]
    fn test_active_grade_subjects_only_include_selected_examinees() {
        let db = tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            Migrator::up(&db, None).await.unwrap();
            for (admission_no, grade_name, class_name) in [
                ("g1", "高一", "高一1班"),
                ("g3", "高三", "高三1班"),
            ] {
                latest_student_scores::ActiveModel {
                    admission_no: Set(admission_no.to_string()),
                    student_name: Set(admission_no.to_string()),
                    class_name: Set(class_name.to_string()),
                    grade_name: Set(grade_name.to_string()),
                    subject_combination: Set(String::new()),
                    language: Set(String::new()),
                    total_score: Set(600.0),
                    class_rank: Set(1),
                    grade_rank: Set(1),
                    selected_subject_count: Set(1),
                }
                .insert(&db)
                .await
                .unwrap();
            }
            for (admission_no, subject, is_selected) in
                [("g1", "math", 0), ("g3", "chemistry", 1)]
            {
                latest_subject_scores::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    admission_no: Set(admission_no.to_string()),
                    subject: Set(subject.to_string()),
                    score: Set(None),
                    is_selected: Set(is_selected),
                    is_absent: Set(0),
                }
                .insert(&db)
                .await
                .unwrap();
            }
            db
        });

        let active = load_active_grade_subjects(&db).unwrap();

        assert!(!active.contains_key("高一"));
        assert_eq!(
            active.get("高三"),
            Some(&HashSet::from([Subject::Chemistry]))
        );
    }

    #[test]
    fn test_concurrent_subject_examinees_are_not_assigned_to_self_study() {
        let db = tauri::async_runtime::block_on(async {
            let db = Database::connect("sqlite::memory:").await.unwrap();
            Migrator::up(&db, None).await.unwrap();
            for (admission_no, class_name) in [
                ("chem", "高三1班"),
                ("history", "高三5班"),
                ("later", "高三9班"),
            ] {
                latest_student_scores::ActiveModel {
                    admission_no: Set(admission_no.to_string()),
                    student_name: Set(admission_no.to_string()),
                    class_name: Set(class_name.to_string()),
                    grade_name: Set("高三".to_string()),
                    subject_combination: Set(String::new()),
                    language: Set(String::new()),
                    total_score: Set(600.0),
                    class_rank: Set(1),
                    grade_rank: Set(1),
                    selected_subject_count: Set(1),
                }
                .insert(&db)
                .await
                .unwrap();
            }
            for (admission_no, subject, is_selected) in [
                ("chem", "chemistry", 1),
                ("chem", "history", 0),
                ("history", "chemistry", 0),
                ("history", "history", 1),
                ("later", "chemistry", 0),
                ("later", "history", 0),
                ("later", "geography", 1),
            ] {
                latest_subject_scores::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    admission_no: Set(admission_no.to_string()),
                    subject: Set(subject.to_string()),
                    score: Set(None),
                    is_selected: Set(is_selected),
                    is_absent: Set(0),
                }
                .insert(&db)
                .await
                .unwrap();
            }
            db
        });
        let sessions = vec![
            SelfStudyScheduleSession {
                subject: Subject::Chemistry,
                start_ts: 1_000,
                order_key: 1,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::History,
                start_ts: 1_000,
                order_key: 2,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Geography,
                start_ts: 2_000,
                order_key: 3,
                is_foreign_group: false,
            },
        ];

        let chemistry_self_study = load_self_study_students_for_session(
            &db,
            "高三",
            Subject::Chemistry,
            &sessions,
            1_000,
        )
        .unwrap();
        let history_self_study = load_self_study_students_for_session(
            &db,
            "高三",
            Subject::History,
            &sessions,
            1_000,
        )
        .unwrap();

        assert_eq!(
            chemistry_self_study
                .iter()
                .map(|student| student.admission_no.as_str())
                .collect::<Vec<_>>(),
            vec!["later"]
        );
        assert!(history_self_study.is_empty());
    }

    #[test]
    fn test_self_study_topic_chain_keeps_next_exam_for_consecutive_self_study() {
        let sessions = vec![
            SelfStudyScheduleSession {
                subject: Subject::Math,
                start_ts: 1_000,
                order_key: 0,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Biology,
                start_ts: 2_000,
                order_key: 1,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Physics,
                start_ts: 3_000,
                order_key: 2,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Russian,
                start_ts: 4_000,
                order_key: 3,
                is_foreign_group: true,
            },
            SelfStudyScheduleSession {
                subject: Subject::History,
                start_ts: 5_000,
                order_key: 4,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Politics,
                start_ts: 6_000,
                order_key: 5,
                is_foreign_group: false,
            },
        ];
        let class_subjects = HashMap::from([(
            "高二7班".to_string(),
            HashSet::from([
                Subject::Math,
                Subject::Russian,
                Subject::History,
                Subject::Politics,
            ]),
        )]);
        let first_chain =
            build_self_study_topic_chain(2_000, "高二7班", &sessions, &class_subjects);
        let second_chain =
            build_self_study_topic_chain(3_000, "高二7班", &sessions, &class_subjects);
        assert_eq!(
            first_chain,
            vec![
                build_subject_self_study_topic(Subject::Russian),
                build_subject_self_study_topic(Subject::History),
                build_subject_self_study_topic(Subject::Politics),
            ]
        );
        assert_eq!(
            second_chain,
            vec![
                build_subject_self_study_topic(Subject::Russian),
                build_subject_self_study_topic(Subject::History),
                build_subject_self_study_topic(Subject::Politics),
            ]
        );
    }

    #[test]
    fn test_self_study_topic_chain_falls_back_to_free_study() {
        let sessions = vec![SelfStudyScheduleSession {
            subject: Subject::Physics,
            start_ts: 1_000,
            order_key: 1,
            is_foreign_group: false,
        }];
        let class_subjects =
            HashMap::from([("高二5班".to_string(), HashSet::from([Subject::Physics]))]);
        let chain = build_self_study_topic_chain(2_000, "高二5班", &sessions, &class_subjects);
        assert_eq!(chain, vec![build_free_study_topic()]);
    }

    #[test]
    fn test_self_study_topic_chain_prefers_later_exam_over_free_study() {
        let sessions = vec![
            SelfStudyScheduleSession {
                subject: Subject::Physics,
                start_ts: 1_000,
                order_key: 0,
                is_foreign_group: false,
            },
            SelfStudyScheduleSession {
                subject: Subject::Chemistry,
                start_ts: 4_000,
                order_key: 1,
                is_foreign_group: false,
            },
        ];
        let class_subjects = HashMap::from([(
            "高二9班".to_string(),
            HashSet::from([Subject::Physics, Subject::Chemistry]),
        )]);

        let chain = build_self_study_topic_chain(2_000, "高二9班", &sessions, &class_subjects);

        assert_eq!(
            chain,
            vec![build_subject_self_study_topic(Subject::Chemistry)]
        );
    }
}
