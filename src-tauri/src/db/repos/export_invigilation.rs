use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDateTime};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::entity::{
    exam_allocation_settings, exam_session_times, invigilation_config_settings,
    latest_exam_plan_sessions, latest_exam_plan_student_allocations, latest_exam_staff_assignments,
    latest_exam_staff_tasks, latest_teacher_assignments_v2, latest_teacher_duty_stats,
};
use crate::export_invigilation::{AccountingConfig, AccountingTeacherRow, TaskExportRow};
use crate::score::{AppError, Subject};

pub async fn exam_title(db: &DatabaseConnection) -> Result<String, AppError> {
    Ok(exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.exam_title)
        .unwrap_or_default())
}

pub async fn task_export_rows(db: &DatabaseConnection) -> Result<Vec<TaskExportRow>, AppError> {
    let tasks = latest_exam_staff_tasks::Entity::find()
        .order_by_asc(latest_exam_staff_tasks::Column::StartAt)
        .order_by_asc(latest_exam_staff_tasks::Column::Id)
        .all(db)
        .await?;
    let assignments = latest_exam_staff_assignments::Entity::find()
        .all(db)
        .await?;
    let mut assignments_by_task = HashMap::<i64, Vec<String>>::new();
    for assignment in assignments {
        assignments_by_task
            .entry(assignment.task_id)
            .or_default()
            .push(assignment.teacher_name);
    }

    let mut out = Vec::new();
    for task in tasks {
        let subject = Subject::from_key(&task.subject)
            .ok_or_else(|| AppError::new(format!("无法识别监考任务科目: {}", task.subject)))?;
        let start_ts = parse_datetime(&task.start_at)
            .map(|dt| dt.and_utc().timestamp())
            .ok_or_else(|| AppError::new(format!("无法解析任务开始时间: {}", task.start_at)))?;
        let teacher_names = assignments_by_task.remove(&task.id).unwrap_or_default();
        if teacher_names.is_empty() {
            out.push(task_to_export_row(&task, subject, start_ts, None));
        } else {
            for teacher_name in teacher_names {
                out.push(task_to_export_row(
                    &task,
                    subject,
                    start_ts,
                    Some(teacher_name),
                ));
            }
        }
    }
    if out.is_empty() {
        return Err(AppError::new("暂无监考分配结果，请先执行监考分配"));
    }
    Ok(out)
}

pub async fn exam_counts(db: &DatabaseConnection) -> Result<HashMap<(i64, i64), i64>, AppError> {
    let rows = latest_exam_plan_student_allocations::Entity::find()
        .filter(latest_exam_plan_student_allocations::Column::AllocationType.eq("exam"))
        .all(db)
        .await?;
    let mut out = HashMap::new();
    for row in rows {
        let Some(space_id) = row.space_id else {
            continue;
        };
        *out.entry((row.session_id, space_id)).or_insert(0) += 1;
    }
    Ok(out)
}

pub async fn accounting_teacher_rows(
    db: &DatabaseConnection,
) -> Result<Vec<AccountingTeacherRow>, AppError> {
    let group_subjects = teacher_group_subjects(db).await?;
    let rows = latest_teacher_duty_stats::Entity::find()
        .order_by_asc(latest_teacher_duty_stats::Column::TeacherId)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| AccountingTeacherRow {
            teacher_id: row.teacher_id,
            teacher_name: row.teacher_name,
            group_subject: group_subjects
                .get(&row.teacher_id)
                .cloned()
                .unwrap_or_else(|| "艺体".to_string()),
            is_middle_manager: row.is_middle_manager == 1,
        })
        .collect())
}

pub async fn accounting_config(db: &DatabaseConnection) -> Result<AccountingConfig, AppError> {
    let row = invigilation_config_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("读取监考配置失败: 未找到配置"))?;
    let exception_ids =
        serde_json::from_str::<Vec<i64>>(&row.middle_manager_exception_teacher_ids_json)
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<_>>();
    let total_exam_minutes = total_exam_minutes(db).await?;
    let total_self_study_minutes = if !row.self_study_date.trim().is_empty()
        && !row.self_study_start_time.trim().is_empty()
        && !row.self_study_end_time.trim().is_empty()
    {
        let start = format!(
            "{}T{}",
            row.self_study_date.trim(),
            row.self_study_start_time.trim()
        );
        let end = format!(
            "{}T{}",
            row.self_study_date.trim(),
            row.self_study_end_time.trim()
        );
        match (parse_datetime(&start), parse_datetime(&end)) {
            (Some(start_dt), Some(end_dt)) if end_dt > start_dt => {
                (end_dt - start_dt).num_minutes()
            }
            _ => 0,
        }
    } else {
        0
    };

    Ok(AccountingConfig {
        indoor_allowance_per_minute: row.indoor_allowance_per_minute,
        outdoor_allowance_per_minute: row.outdoor_allowance_per_minute,
        middle_manager_default_enabled: row.middle_manager_default_enabled == 1,
        middle_manager_exception_teacher_ids: exception_ids,
        total_exam_and_self_study_minutes: total_exam_minutes + total_self_study_minutes,
    })
}

async fn teacher_group_subjects(db: &DatabaseConnection) -> Result<HashMap<i64, String>, AppError> {
    let assignments = latest_teacher_assignments_v2::Entity::find()
        .order_by_asc(latest_teacher_assignments_v2::Column::TeacherId)
        .order_by_asc(latest_teacher_assignments_v2::Column::Id)
        .all(db)
        .await?;
    let mut grouped = HashMap::<i64, Vec<String>>::new();
    for assignment in assignments {
        if let Some(subject) = Subject::from_key(&assignment.subject) {
            let label = normalize_subject_group(subject).to_string();
            let entry = grouped.entry(i64::from(assignment.teacher_id)).or_default();
            if !entry.iter().any(|item| item == &label) {
                entry.push(label);
            }
        }
    }

    let mut out = HashMap::new();
    for (teacher_id, mut labels) in grouped {
        labels.sort_by(|a, b| {
            accounting_group_rank(a)
                .cmp(&accounting_group_rank(b))
                .then(a.cmp(b))
        });
        out.insert(
            teacher_id,
            if labels.is_empty() {
                "艺体".to_string()
            } else {
                labels.join("、")
            },
        );
    }
    Ok(out)
}

async fn total_exam_minutes(db: &DatabaseConnection) -> Result<i64, AppError> {
    let sessions = latest_exam_plan_sessions::Entity::find().all(db).await?;
    let grade_by_session = sessions
        .into_iter()
        .map(|row| (row.id, row.grade_name))
        .collect::<HashMap<_, _>>();
    let times = exam_session_times::Entity::find().all(db).await?;
    let mut seen = HashSet::<(String, String, String)>::new();
    let mut totals = HashMap::<String, i64>::new();
    for time in times {
        if time.start_at.trim().is_empty() || time.end_at.trim().is_empty() {
            continue;
        }
        let Some(grade_name) = grade_by_session.get(&time.session_id).cloned() else {
            continue;
        };
        if !seen.insert((
            grade_name.clone(),
            time.start_at.clone(),
            time.end_at.clone(),
        )) {
            continue;
        }
        let Some(start) = parse_datetime(&time.start_at) else {
            continue;
        };
        let Some(end) = parse_datetime(&time.end_at) else {
            continue;
        };
        *totals.entry(grade_name).or_default() += (end - start).num_minutes();
    }
    Ok(totals.into_values().max().unwrap_or_default())
}

fn task_to_export_row(
    task: &latest_exam_staff_tasks::Model,
    subject: Subject,
    start_ts: i64,
    teacher_name: Option<String>,
) -> TaskExportRow {
    TaskExportRow {
        session_id: task.session_id,
        space_id: task.space_id,
        task_source: task.task_source.clone(),
        role: task.role.clone(),
        grade_name: task.grade_name.clone(),
        subject,
        space_name: task.space_name.clone(),
        floor: task.floor.clone(),
        start_at: task.start_at.clone(),
        end_at: task.end_at.clone(),
        start_ts,
        duration_minutes: task.duration_minutes,
        recommended_self_study_topic_label: task.recommended_self_study_topic_label.clone(),
        teacher_name,
    }
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.naive_local());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
}

fn normalize_subject_group(subject: Subject) -> &'static str {
    match subject {
        Subject::Chinese => "语文",
        Subject::Math => "数学",
        Subject::English | Subject::Russian | Subject::Japanese => "外语",
        Subject::History => "历史",
        Subject::Geography => "地理",
        Subject::Biology => "生物",
        Subject::Politics => "思想政治",
        Subject::Physics => "物理",
        Subject::Chemistry => "化学",
    }
}

fn accounting_group_rank(label: &str) -> i32 {
    match label {
        "语文" => 1,
        "数学" => 2,
        "外语" => 3,
        "历史" => 4,
        "地理" => 5,
        "生物" => 6,
        "思想政治" => 7,
        "物理" => 8,
        "化学" => 9,
        _ => 99,
    }
}
