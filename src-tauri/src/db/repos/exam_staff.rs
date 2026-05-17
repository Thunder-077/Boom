use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

use crate::entity::{
    class_config_subjects, class_configs, exam_grade_subject_time_templates, exam_session_times,
    invigilation_config_settings, invigilation_custom_rules, latest_exam_plan_sessions,
    latest_exam_plan_spaces, latest_exam_staff_assignments, latest_exam_staff_plan_meta,
    latest_exam_staff_tasks, latest_teacher_assignments_v2, latest_teacher_duty_stats,
    latest_teacher_homerooms_v2, latest_teachers_v2,
};
use crate::score::{AppError, ListResult};

#[derive(Debug, Clone)]
pub struct SessionTemplateRow {
    pub grade_name: String,
    pub subject: String,
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone)]
pub struct SessionTimeRow {
    pub session_id: i64,
    pub grade_name: String,
    pub subject: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionSubjectRow {
    pub session_id: i64,
    pub grade_name: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct TeacherRow {
    pub id: i64,
    pub name: String,
    pub is_middle_manager: bool,
}

#[derive(Debug, Clone)]
pub struct TeacherAssignmentRow {
    pub teacher_id: i64,
    pub subject: String,
    pub class_name: String,
}

#[derive(Debug, Clone)]
pub struct TeacherHomeroomRow {
    pub teacher_id: i64,
    pub class_name: String,
}

#[derive(Debug, Clone)]
pub struct TeacherGradeSubjectRow {
    pub teacher_id: i64,
    pub subject: String,
    pub class_name: String,
    pub grade_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassSubjectRow {
    pub grade_name: String,
    pub class_name: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct TeachingClassRow {
    pub id: i64,
    pub grade_name: String,
    pub class_name: String,
    pub floor: String,
}

#[derive(Debug, Clone)]
pub struct SpaceRow {
    pub id: i64,
    pub space_type: String,
    pub space_name: String,
    pub original_class_name: Option<String>,
    pub self_study_topic_kind: Option<String>,
    pub self_study_topic_subjects_json: Option<String>,
    pub self_study_topic_label: Option<String>,
    pub floor: String,
}

#[derive(Debug, Clone)]
pub struct UpsertSessionTimeRow {
    pub session_id: i64,
    pub grade_name: String,
    pub subject: String,
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone)]
pub struct PersistedTaskRow {
    pub session_id: Option<i64>,
    pub space_id: Option<i64>,
    pub task_source: String,
    pub role: String,
    pub grade_name: String,
    pub subject: String,
    pub space_name: String,
    pub floor: String,
    pub start_at: String,
    pub end_at: String,
    pub duration_minutes: i64,
    pub recommended_self_study_topic_kind: Option<String>,
    pub recommended_self_study_topic_subjects_json: Option<String>,
    pub recommended_self_study_topic_label: Option<String>,
    pub priority_self_study_chain_json: String,
    pub assignment_tier: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub allowance_amount: f64,
    pub teacher_id: Option<i64>,
    pub teacher_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersistedDutyStatRow {
    pub teacher_id: i64,
    pub teacher_name: String,
    pub indoor_minutes: i64,
    pub outdoor_minutes: i64,
    pub total_minutes: i64,
    pub task_count: i64,
    pub exam_room_task_count: i64,
    pub self_study_task_count: i64,
    pub floor_rover_task_count: i64,
    pub allowance_total: f64,
    pub indoor_allowance_total: f64,
    pub outdoor_allowance_total: f64,
    pub is_middle_manager: bool,
}

#[derive(Debug, Clone)]
pub struct PersistedPlanMetaRow {
    pub generated_at: String,
    pub session_count: i64,
    pub task_count: i64,
    pub assigned_count: i64,
    pub unassigned_count: i64,
    pub warning_count: i64,
    pub imbalance_minutes: i64,
    pub solver_engine: String,
    pub optimality_status: String,
    pub solve_duration_ms: i64,
    pub fallback_reason: Option<String>,
    pub fallback_pool_assignments: i64,
}

#[derive(Debug, Clone)]
pub struct TaskListFilters {
    pub session_id: Option<i64>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct TaskListRow {
    pub task: latest_exam_staff_tasks::Model,
    pub assignment: Option<latest_exam_staff_assignments::Model>,
}

#[derive(Debug, Clone)]
pub struct DutyStatFilters {
    pub keyword: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub async fn list_template_grades(db: &DatabaseConnection) -> Result<Vec<String>, AppError> {
    let rows = exam_grade_subject_time_templates::Entity::find()
        .order_by_asc(exam_grade_subject_time_templates::Column::GradeName)
        .all(db)
        .await?;
    let mut grades = rows
        .into_iter()
        .map(|row| row.grade_name)
        .collect::<Vec<_>>();
    grades.sort();
    grades.dedup();
    Ok(grades)
}

pub async fn list_grade_subject_templates(
    db: &DatabaseConnection,
) -> Result<Vec<SessionTemplateRow>, AppError> {
    let rows = exam_grade_subject_time_templates::Entity::find()
        .order_by_asc(exam_grade_subject_time_templates::Column::GradeName)
        .order_by_asc(exam_grade_subject_time_templates::Column::Subject)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| SessionTemplateRow {
            grade_name: row.grade_name,
            subject: row.subject,
            start_at: row.start_at,
            end_at: row.end_at,
        })
        .collect())
}

pub async fn list_session_time_rows(
    db: &DatabaseConnection,
) -> Result<Vec<SessionTimeRow>, AppError> {
    let sessions = latest_exam_plan_sessions::Entity::find()
        .order_by_asc(latest_exam_plan_sessions::Column::GradeName)
        .order_by_asc(latest_exam_plan_sessions::Column::Id)
        .all(db)
        .await?;
    let times = exam_session_times::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.session_id, row))
        .collect::<std::collections::HashMap<_, _>>();
    Ok(sessions
        .into_iter()
        .map(|session| {
            let time = times.get(&session.id);
            SessionTimeRow {
                session_id: session.id,
                grade_name: session.grade_name,
                subject: session.subject,
                start_at: time.map(|row| row.start_at.clone()),
                end_at: time.map(|row| row.end_at.clone()),
            }
        })
        .collect())
}

pub async fn list_session_subjects(
    db: &DatabaseConnection,
) -> Result<Vec<SessionSubjectRow>, AppError> {
    let rows = latest_exam_plan_sessions::Entity::find()
        .order_by_asc(latest_exam_plan_sessions::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| SessionSubjectRow {
            session_id: row.id,
            grade_name: row.grade_name,
            subject: row.subject,
        })
        .collect())
}

pub async fn upsert_session_times(
    db: &DatabaseConnection,
    items: &[UpsertSessionTimeRow],
    now: &str,
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    for item in items {
        let existing_template = exam_grade_subject_time_templates::Entity::find_by_id((
            item.grade_name.clone(),
            item.subject.clone(),
        ))
        .one(&tx)
        .await?;
        if let Some(row) = existing_template {
            let mut active = row.into_active_model();
            active.start_at = Set(item.start_at.clone());
            active.end_at = Set(item.end_at.clone());
            active.updated_at = Set(now.to_string());
            active.update(&tx).await?;
        } else {
            exam_grade_subject_time_templates::ActiveModel {
                grade_name: Set(item.grade_name.clone()),
                subject: Set(item.subject.clone()),
                start_at: Set(item.start_at.clone()),
                end_at: Set(item.end_at.clone()),
                updated_at: Set(now.to_string()),
            }
            .insert(&tx)
            .await?;
        }

        let session_exists = item.session_id > 0
            && latest_exam_plan_sessions::Entity::find_by_id(item.session_id)
                .one(&tx)
                .await?
                .is_some();
        if session_exists {
            if let Some(row) = exam_session_times::Entity::find_by_id(item.session_id)
                .one(&tx)
                .await?
            {
                let mut active = row.into_active_model();
                active.subject = Set(item.subject.clone());
                active.start_at = Set(item.start_at.clone());
                active.end_at = Set(item.end_at.clone());
                active.updated_at = Set(now.to_string());
                active.update(&tx).await?;
            } else {
                exam_session_times::ActiveModel {
                    session_id: Set(item.session_id),
                    subject: Set(item.subject.clone()),
                    start_at: Set(item.start_at.clone()),
                    end_at: Set(item.end_at.clone()),
                    updated_at: Set(now.to_string()),
                }
                .insert(&tx)
                .await?;
            }
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn delete_session_time_template(
    db: &DatabaseConnection,
    grade_name: &str,
    subject: &str,
) -> Result<(), AppError> {
    exam_grade_subject_time_templates::Entity::delete_by_id((
        grade_name.to_string(),
        subject.to_string(),
    ))
    .exec(db)
    .await?;
    Ok(())
}

pub async fn seed_default_session_times(db: &DatabaseConnection, now: &str) -> Result<(), AppError> {
    let templates = list_grade_subject_templates(db).await?;
    let sessions = list_session_subjects(db).await?;
    let template_map = templates
        .into_iter()
        .map(|row| ((row.grade_name, row.subject), (row.start_at, row.end_at)))
        .collect::<std::collections::HashMap<_, _>>();
    let tx = db.begin().await?;
    for session in sessions {
        let Some((start_at, end_at)) =
            template_map.get(&(session.grade_name.clone(), session.subject.clone()))
        else {
            continue;
        };
        if let Some(row) = exam_session_times::Entity::find_by_id(session.session_id)
            .one(&tx)
            .await?
        {
            let mut active = row.into_active_model();
            active.subject = Set(session.subject);
            active.start_at = Set(start_at.clone());
            active.end_at = Set(end_at.clone());
            active.updated_at = Set(now.to_string());
            active.update(&tx).await?;
        } else {
            exam_session_times::ActiveModel {
                session_id: Set(session.session_id),
                subject: Set(session.subject),
                start_at: Set(start_at.clone()),
                end_at: Set(end_at.clone()),
                updated_at: Set(now.to_string()),
            }
            .insert(&tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn load_teacher_rows(db: &DatabaseConnection) -> Result<Vec<TeacherRow>, AppError> {
    let rows = latest_teachers_v2::Entity::find()
        .order_by_asc(latest_teachers_v2::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| TeacherRow {
            id: row.id as i64,
            name: row.teacher_name,
            is_middle_manager: row.is_middle_manager == 1,
        })
        .collect())
}

pub async fn load_teacher_assignment_rows(
    db: &DatabaseConnection,
) -> Result<Vec<TeacherAssignmentRow>, AppError> {
    let rows = latest_teacher_assignments_v2::Entity::find()
        .order_by_asc(latest_teacher_assignments_v2::Column::TeacherId)
        .order_by_asc(latest_teacher_assignments_v2::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| TeacherAssignmentRow {
            teacher_id: row.teacher_id as i64,
            subject: row.subject,
            class_name: row.class_name,
        })
        .collect())
}

pub async fn load_teacher_homeroom_rows(
    db: &DatabaseConnection,
) -> Result<Vec<TeacherHomeroomRow>, AppError> {
    let rows = latest_teacher_homerooms_v2::Entity::find()
        .order_by_asc(latest_teacher_homerooms_v2::Column::TeacherId)
        .order_by_asc(latest_teacher_homerooms_v2::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| TeacherHomeroomRow {
            teacher_id: row.teacher_id as i64,
            class_name: row.class_name,
        })
        .collect())
}

pub async fn load_teacher_grade_subject_rows(
    db: &DatabaseConnection,
) -> Result<Vec<TeacherGradeSubjectRow>, AppError> {
    let assignments = latest_teacher_assignments_v2::Entity::find()
        .order_by_asc(latest_teacher_assignments_v2::Column::TeacherId)
        .order_by_asc(latest_teacher_assignments_v2::Column::Id)
        .all(db)
        .await?;
    let classes = class_configs::Entity::find()
        .filter(class_configs::Column::ConfigType.eq("teaching_class"))
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.class_name, row.grade_name))
        .collect::<std::collections::HashMap<_, _>>();
    Ok(assignments
        .into_iter()
        .map(|row| TeacherGradeSubjectRow {
            teacher_id: row.teacher_id as i64,
            subject: row.subject,
            grade_name: classes.get(&row.class_name).cloned(),
            class_name: row.class_name,
        })
        .collect())
}

pub async fn load_class_subject_rows(
    db: &DatabaseConnection,
) -> Result<Vec<ClassSubjectRow>, AppError> {
    let classes = class_configs::Entity::find()
        .filter(class_configs::Column::ConfigType.eq("teaching_class"))
        .order_by_asc(class_configs::Column::GradeName)
        .order_by_asc(class_configs::Column::ClassName)
        .order_by_asc(class_configs::Column::Id)
        .all(db)
        .await?;
    let subjects = class_config_subjects::Entity::find()
        .order_by_asc(class_config_subjects::Column::Id)
        .all(db)
        .await?;
    let subjects_by_class = subjects.into_iter().fold(
        std::collections::HashMap::<i64, Vec<String>>::new(),
        |mut acc, subject| {
            acc.entry(subject.config_id as i64)
                .or_default()
                .push(subject.subject);
            acc
        },
    );
    let mut rows = Vec::new();
    for class in classes {
        if let Some(subjects) = subjects_by_class.get(&(class.id as i64)) {
            for subject in subjects {
                rows.push(ClassSubjectRow {
                    grade_name: class.grade_name.clone(),
                    class_name: class.class_name.clone(),
                    subject: subject.clone(),
                });
            }
        }
    }
    Ok(rows)
}

pub async fn load_teaching_class_rows(
    db: &DatabaseConnection,
) -> Result<Vec<TeachingClassRow>, AppError> {
    let rows = class_configs::Entity::find()
        .filter(class_configs::Column::ConfigType.eq("teaching_class"))
        .order_by_asc(class_configs::Column::GradeName)
        .order_by_asc(class_configs::Column::ClassName)
        .order_by_asc(class_configs::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| TeachingClassRow {
            id: row.id as i64,
            grade_name: row.grade_name,
            class_name: row.class_name,
            floor: row.floor,
        })
        .collect())
}

pub async fn load_spaces_for_session(
    db: &DatabaseConnection,
    session_id: i64,
) -> Result<Vec<SpaceRow>, AppError> {
    let rows = latest_exam_plan_spaces::Entity::find()
        .filter(latest_exam_plan_spaces::Column::SessionId.eq(session_id))
        .order_by_asc(latest_exam_plan_spaces::Column::SortIndex)
        .order_by_asc(latest_exam_plan_spaces::Column::Id)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| SpaceRow {
            id: row.id,
            space_type: row.space_type,
            space_name: row.space_name,
            original_class_name: row.original_class_name,
            self_study_topic_kind: row.self_study_topic_kind,
            self_study_topic_subjects_json: row.self_study_topic_subjects_json,
            self_study_topic_label: row.self_study_topic_label,
            floor: row.floor,
        })
        .collect())
}

pub async fn get_config(
    db: &DatabaseConnection,
) -> Result<Option<invigilation_config_settings::Model>, AppError> {
    Ok(invigilation_config_settings::Entity::find_by_id(1)
        .one(db)
        .await?)
}

pub async fn upsert_config(
    db: &DatabaseConnection,
    row: invigilation_config_settings::ActiveModel,
) -> Result<(), AppError> {
    if let Some(existing) = invigilation_config_settings::Entity::find_by_id(1)
        .one(db)
        .await?
    {
        let mut active = existing.into_active_model();
        active.default_exam_room_required_count = row.default_exam_room_required_count;
        active.indoor_allowance_per_minute = row.indoor_allowance_per_minute;
        active.outdoor_allowance_per_minute = row.outdoor_allowance_per_minute;
        active.middle_manager_default_enabled = row.middle_manager_default_enabled;
        active.middle_manager_exception_teacher_ids_json =
            row.middle_manager_exception_teacher_ids_json;
        active.self_study_date = row.self_study_date;
        active.self_study_start_time = row.self_study_start_time;
        active.self_study_end_time = row.self_study_end_time;
        active.updated_at = row.updated_at;
        active.update(db).await?;
    } else {
        row.insert(db).await?;
    }
    Ok(())
}

pub async fn update_self_study_class_subjects_json(
    db: &DatabaseConnection,
    json_text: &str,
    now: &str,
) -> Result<(), AppError> {
    if let Some(existing) = invigilation_config_settings::Entity::find_by_id(1)
        .one(db)
        .await?
    {
        let mut active = existing.into_active_model();
        active.self_study_class_subjects_json = Set(json_text.to_string());
        active.updated_at = Set(now.to_string());
        active.update(db).await?;
    } else {
        invigilation_config_settings::ActiveModel {
            id: Set(1),
            default_exam_room_required_count: Set(1),
            indoor_allowance_per_minute: Set(0.5),
            outdoor_allowance_per_minute: Set(0.3),
            middle_manager_default_enabled: Set(0),
            middle_manager_exception_teacher_ids_json: Set("[]".to_string()),
            self_study_date: Set(String::new()),
            self_study_start_time: Set("12:10".to_string()),
            self_study_end_time: Set("13:40".to_string()),
            self_study_class_subjects_json: Set(json_text.to_string()),
            updated_at: Set(now.to_string()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn list_custom_rules(
    db: &DatabaseConnection,
) -> Result<Vec<invigilation_custom_rules::Model>, AppError> {
    Ok(invigilation_custom_rules::Entity::find()
        .order_by_desc(invigilation_custom_rules::Column::Id)
        .all(db)
        .await?)
}

pub async fn replace_custom_rules(
    db: &DatabaseConnection,
    rows: Vec<invigilation_custom_rules::ActiveModel>,
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    invigilation_custom_rules::Entity::delete_many()
        .exec(&tx)
        .await?;
    for row in rows {
        row.insert(&tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn clear_latest_staff_plan_snapshot(db: &DatabaseConnection) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_exam_staff_assignments::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_exam_staff_tasks::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_teacher_duty_stats::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_exam_staff_plan_meta::Entity::delete_many()
        .exec(&tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn persist_plan_snapshot(
    db: &DatabaseConnection,
    meta: PersistedPlanMetaRow,
    tasks: Vec<PersistedTaskRow>,
    duty_stats: Vec<PersistedDutyStatRow>,
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_exam_staff_assignments::Entity::delete_many().exec(&tx).await?;
    latest_exam_staff_tasks::Entity::delete_many().exec(&tx).await?;
    latest_teacher_duty_stats::Entity::delete_many().exec(&tx).await?;
    latest_exam_staff_plan_meta::Entity::delete_many().exec(&tx).await?;

    for task in tasks {
        let teacher_id = task.teacher_id;
        let teacher_name = task.teacher_name.clone();
        let row = latest_exam_staff_tasks::ActiveModel {
            session_id: Set(task.session_id),
            space_id: Set(task.space_id),
            task_source: Set(task.task_source),
            role: Set(task.role),
            grade_name: Set(task.grade_name),
            subject: Set(task.subject),
            space_name: Set(task.space_name),
            floor: Set(task.floor),
            start_at: Set(task.start_at),
            end_at: Set(task.end_at),
            duration_minutes: Set(task.duration_minutes),
            recommended_self_study_topic_kind: Set(task.recommended_self_study_topic_kind),
            recommended_self_study_topic_subjects_json: Set(
                task.recommended_self_study_topic_subjects_json,
            ),
            recommended_self_study_topic_label: Set(task.recommended_self_study_topic_label),
            priority_self_study_chain_json: Set(task.priority_self_study_chain_json),
            assignment_tier: Set(task.assignment_tier),
            status: Set(task.status),
            reason: Set(task.reason),
            allowance_amount: Set(task.allowance_amount),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
        if let (Some(teacher_id), Some(teacher_name)) = (teacher_id, teacher_name) {
            latest_exam_staff_assignments::ActiveModel {
                task_id: Set(row.id),
                teacher_id: Set(teacher_id),
                teacher_name: Set(teacher_name),
                assigned_at: Set(meta.generated_at.clone()),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
    }

    for stat in duty_stats {
        latest_teacher_duty_stats::ActiveModel {
            teacher_id: Set(stat.teacher_id),
            teacher_name: Set(stat.teacher_name),
            indoor_minutes: Set(stat.indoor_minutes),
            outdoor_minutes: Set(stat.outdoor_minutes),
            total_minutes: Set(stat.total_minutes),
            task_count: Set(stat.task_count),
            exam_room_task_count: Set(stat.exam_room_task_count),
            self_study_task_count: Set(stat.self_study_task_count),
            floor_rover_task_count: Set(stat.floor_rover_task_count),
            allowance_total: Set(stat.allowance_total),
            indoor_allowance_total: Set(stat.indoor_allowance_total),
            outdoor_allowance_total: Set(stat.outdoor_allowance_total),
            is_middle_manager: Set(if stat.is_middle_manager { 1 } else { 0 }),
        }
        .insert(&tx)
        .await?;
    }

    latest_exam_staff_plan_meta::ActiveModel {
        id: Set(1),
        generated_at: Set(meta.generated_at),
        session_count: Set(meta.session_count),
        task_count: Set(meta.task_count),
        assigned_count: Set(meta.assigned_count),
        unassigned_count: Set(meta.unassigned_count),
        warning_count: Set(meta.warning_count),
        imbalance_minutes: Set(meta.imbalance_minutes),
        solver_engine: Set(meta.solver_engine),
        optimality_status: Set(meta.optimality_status),
        solve_duration_ms: Set(meta.solve_duration_ms),
        fallback_reason: Set(meta.fallback_reason),
        fallback_pool_assignments: Set(meta.fallback_pool_assignments),
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn latest_plan_meta(
    db: &DatabaseConnection,
) -> Result<Option<latest_exam_staff_plan_meta::Model>, AppError> {
    Ok(latest_exam_staff_plan_meta::Entity::find_by_id(1)
        .one(db)
        .await?)
}

pub async fn list_tasks(
    db: &DatabaseConnection,
    filters: TaskListFilters,
) -> Result<ListResult<TaskListRow>, AppError> {
    let mut query = latest_exam_staff_tasks::Entity::find();
    if let Some(session_id) = filters.session_id {
        query = query.filter(latest_exam_staff_tasks::Column::SessionId.eq(session_id));
    }
    if let Some(role) = filters.role {
        query = query.filter(latest_exam_staff_tasks::Column::Role.eq(role));
    }
    if let Some(status) = filters.status {
        query = query.filter(latest_exam_staff_tasks::Column::Status.eq(status));
    }
    let total = query.clone().count(db).await? as i64;
    let rows = query
        .order_by_asc(latest_exam_staff_tasks::Column::StartAt)
        .order_by_asc(latest_exam_staff_tasks::Column::SessionId)
        .order_by_asc(latest_exam_staff_tasks::Column::Id)
        .limit(filters.page_size as u64)
        .offset(((filters.page - 1) * filters.page_size) as u64)
        .all(db)
        .await?;
    let task_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let assignments = latest_exam_staff_assignments::Entity::find()
        .filter(latest_exam_staff_assignments::Column::TaskId.is_in(task_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.task_id, row))
        .collect::<std::collections::HashMap<_, _>>();
    Ok(ListResult {
        total,
        items: rows
            .into_iter()
            .map(|task| TaskListRow {
                assignment: assignments.get(&task.id).cloned(),
                task,
            })
            .collect(),
    })
}

pub async fn list_duty_stats(
    db: &DatabaseConnection,
    filters: DutyStatFilters,
) -> Result<ListResult<latest_teacher_duty_stats::Model>, AppError> {
    let mut query = latest_teacher_duty_stats::Entity::find();
    if let Some(keyword) = filters.keyword {
        query = query.filter(latest_teacher_duty_stats::Column::TeacherName.contains(&keyword));
    }
    let total = query.clone().count(db).await? as i64;
    let items = query
        .order_by_asc(latest_teacher_duty_stats::Column::TotalMinutes)
        .order_by_asc(latest_teacher_duty_stats::Column::TeacherId)
        .limit(filters.page_size as u64)
        .offset(((filters.page - 1) * filters.page_size) as u64)
        .all(db)
        .await?;
    Ok(ListResult { total, items })
}
