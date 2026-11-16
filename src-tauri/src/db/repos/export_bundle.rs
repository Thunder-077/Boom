use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entity::{exam_allocation_settings, latest_student_scores};
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
    pub exam_group: String,
    pub admission_no: String,
    pub student_name: String,
    pub class_name: String,
    pub class_rank: i64,
}

pub async fn settings<C: ConnectionTrait>(db: &C) -> Result<BundleSettings, AppError> {
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

pub async fn students_for_grade<C: ConnectionTrait>(
    db: &C,
    grade_name: &str,
) -> Result<Vec<StudentBase>, AppError> {
    let rows = latest_student_scores::Entity::find()
        .filter(latest_student_scores::Column::GradeName.eq(grade_name))
        .all(db)
        .await?;
    let mut out = rows
        .into_iter()
        .map(|row| StudentBase {
            exam_group: row.exam_group,
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

// 捕获源导入版本，发布前在写事务内复核，拒绝用旧学生/教师数据覆盖新结果。
pub async fn source_revision<C: ConnectionTrait>(
    db: &C,
) -> Result<(Option<String>, Option<String>), AppError> {
    let scores = crate::entity::latest_import_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.imported_at);
    let teachers = crate::entity::latest_teacher_import_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.imported_at);
    Ok((scores, teachers))
}

pub async fn plan_revision<C: ConnectionTrait>(db: &C) -> Result<String, AppError> {
    crate::entity::latest_exam_plan_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.generated_at)
        .ok_or_else(|| AppError::new("请先生成考场安排"))
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
