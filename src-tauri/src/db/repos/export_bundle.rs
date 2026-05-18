use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::entity::{
    exam_allocation_settings, exam_session_times, latest_exam_plan_sessions,
    latest_exam_plan_spaces, latest_exam_plan_student_allocations, latest_student_scores,
};
use crate::score::{AppError, Subject};

#[derive(Debug, Clone)]
pub struct BundleSettings {
    pub exam_title: String,
    pub exam_notices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub subject_label: &'static str,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub start_ts: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ExamRow {
    pub admission_no: String,
    pub student_name: String,
    pub class_name: String,
    pub subject: Subject,
    pub subject_label: &'static str,
    pub space_name: String,
    pub seat_no: i64,
}

#[derive(Debug, Clone)]
pub struct StudentBase {
    pub admission_no: String,
    pub student_name: String,
    pub class_name: String,
    pub class_rank: i64,
}

pub async fn settings(db: &DatabaseConnection) -> Result<BundleSettings, AppError> {
    let row = exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("读取月考配置失败: 未找到配置"))?;
    let exam_notices = serde_json::from_str::<Vec<String>>(&row.exam_notices_json)
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    Ok(BundleSettings {
        exam_title: row.exam_title,
        exam_notices,
    })
}

pub async fn grades(db: &DatabaseConnection) -> Result<Vec<String>, AppError> {
    let rows = latest_exam_plan_sessions::Entity::find().all(db).await?;
    let mut grades = rows
        .into_iter()
        .map(|row| row.grade_name)
        .collect::<Vec<_>>();
    grades.sort_by(|a, b| grade_order_key(a).cmp(&grade_order_key(b)));
    grades.dedup();
    Ok(grades)
}

pub async fn sessions_for_grade(
    db: &DatabaseConnection,
    grade_name: &str,
) -> Result<HashMap<Subject, SessionInfo>, AppError> {
    let sessions = latest_exam_plan_sessions::Entity::find()
        .filter(latest_exam_plan_sessions::Column::GradeName.eq(grade_name))
        .all(db)
        .await?;
    let times = exam_session_times::Entity::find().all(db).await?;
    let times_by_session = times
        .into_iter()
        .map(|row| (row.session_id, row))
        .collect::<HashMap<_, _>>();

    let mut out = HashMap::new();
    for session in sessions {
        let Some(subject) = Subject::from_key(&session.subject) else {
            continue;
        };
        if !SUBJECT_EXPORT_ORDER.iter().any(|item| *item == subject) {
            continue;
        }
        let time = times_by_session.get(&session.id);
        let start_at = time.map(|row| row.start_at.clone());
        let end_at = time.map(|row| row.end_at.clone());
        let start_ts = start_at
            .as_deref()
            .and_then(parse_datetime)
            .map(|dt| dt.and_utc().timestamp_millis());
        out.insert(
            subject,
            SessionInfo {
                subject_label: subject_label(subject),
                start_at,
                end_at,
                start_ts,
            },
        );
    }
    Ok(out)
}

pub async fn exam_rows_for_grade(
    db: &DatabaseConnection,
    grade_name: &str,
) -> Result<Vec<ExamRow>, AppError> {
    let sessions = latest_exam_plan_sessions::Entity::find()
        .filter(latest_exam_plan_sessions::Column::GradeName.eq(grade_name))
        .all(db)
        .await?;
    let session_by_id = sessions
        .into_iter()
        .map(|row| (row.id, row))
        .collect::<HashMap<_, _>>();
    let session_ids = session_by_id.keys().copied().collect::<Vec<_>>();
    let spaces = latest_exam_plan_spaces::Entity::find().all(db).await?;
    let space_by_id = spaces
        .into_iter()
        .map(|row| (row.id, row.space_name))
        .collect::<HashMap<_, _>>();

    let mut out = Vec::new();
    for session_id in session_ids {
        let Some(session) = session_by_id.get(&session_id) else {
            continue;
        };
        let Some(subject) = Subject::from_key(&session.subject) else {
            continue;
        };
        let allocations = latest_exam_plan_student_allocations::Entity::find()
            .filter(latest_exam_plan_student_allocations::Column::SessionId.eq(session_id))
            .filter(latest_exam_plan_student_allocations::Column::AllocationType.eq("exam"))
            .all(db)
            .await?;
        for allocation in allocations {
            out.push(ExamRow {
                admission_no: allocation.admission_no,
                student_name: allocation.student_name,
                class_name: allocation.class_name,
                subject,
                subject_label: subject_label(subject),
                space_name: allocation
                    .space_id
                    .and_then(|id| space_by_id.get(&id).cloned())
                    .unwrap_or_default(),
                seat_no: allocation.seat_no.unwrap_or_default(),
            });
        }
    }
    Ok(out)
}

pub async fn students_for_grade(
    db: &DatabaseConnection,
    grade_name: &str,
) -> Result<Vec<StudentBase>, AppError> {
    let rows = latest_student_scores::Entity::find()
        .filter(latest_student_scores::Column::GradeName.eq(grade_name))
        .all(db)
        .await?;
    let mut out = rows
        .into_iter()
        .map(|row| StudentBase {
            admission_no: row.admission_no,
            student_name: row.student_name,
            class_name: row.class_name,
            class_rank: row.class_rank,
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
        sort_class_like(&a.class_name, &b.class_name)
            .then(a.class_rank.cmp(&b.class_rank))
            .then(a.admission_no.cmp(&b.admission_no))
    });
    Ok(out)
}

const SUBJECT_EXPORT_ORDER: [Subject; 11] = [
    Subject::Chinese,
    Subject::Math,
    Subject::English,
    Subject::Russian,
    Subject::Japanese,
    Subject::History,
    Subject::Geography,
    Subject::Biology,
    Subject::Politics,
    Subject::Physics,
    Subject::Chemistry,
];

fn subject_label(subject: Subject) -> &'static str {
    match subject {
        Subject::Chinese => "语文",
        Subject::Math => "数学",
        Subject::English => "英语",
        Subject::Russian => "俄语",
        Subject::Japanese => "日语",
        Subject::History => "历史",
        Subject::Geography => "地理",
        Subject::Biology => "生物",
        Subject::Politics => "政治",
        Subject::Physics => "物理",
        Subject::Chemistry => "化学",
    }
}

fn grade_order_key(grade_name: &str) -> (i32, &str) {
    match grade_name {
        "高一" => (1, grade_name),
        "高二" => (2, grade_name),
        "高三" => (3, grade_name),
        _ => (4, grade_name),
    }
}

fn class_number(name: &str) -> Option<i64> {
    let target = name.find('班').or_else(|| name.find('场'))?;
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
    digits.chars().rev().collect::<String>().parse::<i64>().ok()
}

fn sort_class_like(a: &str, b: &str) -> std::cmp::Ordering {
    class_number(a).cmp(&class_number(b)).then(a.cmp(b))
}

fn parse_datetime(value: &str) -> Option<chrono::NaiveDateTime> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value) {
        return Some(dt.naive_local());
    }
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .ok()
        .or_else(|| chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
}
