use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::entity::{
    class_config_subjects, class_configs, exam_allocation_settings, exam_generation_progress,
    exam_grade_subject_time_templates, invigilation_config_settings, latest_exam_plan_meta,
    latest_exam_plan_sessions, latest_exam_plan_spaces, latest_exam_plan_staff_assignments,
    latest_exam_plan_student_allocations, latest_student_scores, latest_subject_scores,
};
use crate::score::{AppError, ListResult};

#[derive(Debug, Clone)]
pub struct SettingsRow {
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub exam_title: String,
    pub exam_notices_json: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ProgressRow {
    pub status: String,
    pub stage: String,
    pub stage_label: String,
    pub percent: i64,
    pub message: String,
    pub current_grade: Option<String>,
    pub total_grades: i64,
    pub completed_grades: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ClassConfigRow {
    pub grade_name: String,
    pub class_name: String,
    pub building: String,
    pub floor: String,
    pub config_type: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParticipantRow {
    pub admission_no: String,
    pub student_name: String,
    pub class_name: String,
    pub total_score: f64,
    pub score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ActiveGradeSubjectRow {
    pub grade_name: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct SessionInsertRow {
    pub grade_name: String,
    pub subject: String,
    pub is_foreign_group: i64,
    pub foreign_order: Option<i64>,
    pub participant_count: i64,
    pub exam_room_count: i64,
    pub self_study_room_count: i64,
}

#[derive(Debug, Clone)]
pub struct SpaceInsertRow {
    pub session_id: i64,
    pub space_type: String,
    pub space_source: String,
    pub grade_name: String,
    pub subject: String,
    pub space_name: String,
    pub original_class_name: Option<String>,
    pub self_study_topic_kind: Option<String>,
    pub self_study_topic_subjects_json: Option<String>,
    pub self_study_topic_label: Option<String>,
    pub building: String,
    pub floor: String,
    pub capacity: Option<i64>,
    pub sort_index: i64,
}

#[derive(Debug, Clone)]
pub struct StudentAllocationInsertRow {
    pub session_id: i64,
    pub admission_no: String,
    pub student_name: String,
    pub class_name: String,
    pub allocation_type: String,
    pub space_id: Option<i64>,
    pub seat_no: Option<i64>,
    pub subject_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PlanMetaInsertRow {
    pub generated_at: String,
    pub default_capacity: i64,
    pub max_capacity: i64,
    pub grade_count: i64,
    pub session_count: i64,
    pub warning_count: i64,
}

#[derive(Debug, Clone)]
pub struct OverviewCounts {
    pub exam_room_count: i64,
    pub self_study_room_count: i64,
    pub student_allocation_count: i64,
}

#[derive(Debug, Clone)]
pub struct SessionFilters {
    pub grade_name: Option<String>,
    pub subject: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub async fn ensure_defaults(
    db: &DatabaseConnection,
    default_capacity: i64,
    default_max_capacity: i64,
    default_exam_title: &str,
    default_notices_json: &str,
    now: &str,
) -> Result<(), AppError> {
    if exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .is_none()
    {
        exam_allocation_settings::ActiveModel {
            id: Set(1),
            default_capacity: Set(default_capacity),
            max_capacity: Set(default_max_capacity),
            exam_title: Set(default_exam_title.to_string()),
            exam_notices_json: Set(default_notices_json.to_string()),
            updated_at: Set(now.to_string()),
        }
        .insert(db)
        .await?;
    }
    if exam_generation_progress::Entity::find_by_id(1)
        .one(db)
        .await?
        .is_none()
    {
        exam_generation_progress::ActiveModel {
            id: Set(1),
            status: Set("idle".to_string()),
            stage: Set("idle".to_string()),
            stage_label: Set("等待开始".to_string()),
            percent: Set(0),
            message: Set("等待开始分配考场".to_string()),
            current_grade: Set(None),
            total_grades: Set(0),
            completed_grades: Set(0),
            updated_at: Set(now.to_string()),
        }
        .insert(db)
        .await?;
    }
    if invigilation_config_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .is_none()
    {
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
            self_study_class_subjects_json: Set("[]".to_string()),
            updated_at: Set(now.to_string()),
        }
        .insert(db)
        .await?;
    }
    let settings = exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?;
    if let Some(settings) = settings {
        let title_is_empty = settings.exam_title.trim().is_empty();
        let mut active = settings.into_active_model();
        let mut changed = false;
        if title_is_empty {
            active.exam_title = Set(default_exam_title.to_string());
            changed = true;
        }
        if changed {
            active.update(db).await?;
        }
    }
    Ok(())
}

pub async fn get_settings(db: &DatabaseConnection) -> Result<SettingsRow, AppError> {
    let row = exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("考试配置未初始化"))?;
    Ok(SettingsRow {
        default_capacity: row.default_capacity,
        max_capacity: row.max_capacity,
        exam_title: row.exam_title,
        exam_notices_json: row.exam_notices_json,
        updated_at: row.updated_at,
    })
}

pub async fn update_settings(
    db: &DatabaseConnection,
    default_capacity: i64,
    max_capacity: i64,
    exam_title: &str,
    exam_notices_json: &str,
    now: &str,
) -> Result<(), AppError> {
    let row = exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("考试配置未初始化"))?;
    let mut active = row.into_active_model();
    active.default_capacity = Set(default_capacity);
    active.max_capacity = Set(max_capacity);
    active.exam_title = Set(exam_title.to_string());
    active.exam_notices_json = Set(exam_notices_json.to_string());
    active.updated_at = Set(now.to_string());
    active.update(db).await?;
    Ok(())
}

pub async fn replace_default_notices_if_needed(
    db: &DatabaseConnection,
    exam_notices_json: &str,
) -> Result<(), AppError> {
    let row = exam_allocation_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("考试配置未初始化"))?;
    let mut active = row.into_active_model();
    active.exam_notices_json = Set(exam_notices_json.to_string());
    active.update(db).await?;
    Ok(())
}

pub async fn list_class_config_rows(
    db: &DatabaseConnection,
) -> Result<Vec<ClassConfigRow>, AppError> {
    let configs = class_configs::Entity::find()
        .filter(class_configs::Column::ConfigType.is_in(["teaching_class", "exam_room"]))
        .order_by_asc(class_configs::Column::GradeName)
        .order_by_asc(class_configs::Column::ClassName)
        .order_by_asc(class_configs::Column::Id)
        .all(db)
        .await?;
    let subjects = class_config_subjects::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .fold(
            std::collections::HashMap::<i32, Vec<String>>::new(),
            |mut map, row| {
                map.entry(row.config_id).or_default().push(row.subject);
                map
            },
        );
    let mut rows = Vec::new();
    for config in configs {
        let config_subjects = subjects.get(&config.id).cloned().unwrap_or_default();
        if config_subjects.is_empty() {
            rows.push(ClassConfigRow {
                grade_name: config.grade_name.clone(),
                class_name: config.class_name.clone(),
                building: config.building.clone(),
                floor: config.floor.clone(),
                config_type: config.config_type.clone(),
                subject: None,
            });
            continue;
        }
        for subject in config_subjects {
            rows.push(ClassConfigRow {
                grade_name: config.grade_name.clone(),
                class_name: config.class_name.clone(),
                building: config.building.clone(),
                floor: config.floor.clone(),
                config_type: config.config_type.clone(),
                subject: Some(subject),
            });
        }
    }
    Ok(rows)
}

pub async fn list_grade_subject_templates(
    db: &DatabaseConnection,
) -> Result<Vec<crate::db::repos::exam_staff::SessionTemplateRow>, AppError> {
    crate::db::repos::exam_staff::list_grade_subject_templates(db).await
}

#[derive(Debug, Clone)]
pub struct GradeSubjectTemplateSeedRow {
    pub grade_name: String,
    pub subject: String,
    pub start_at: String,
    pub end_at: String,
}

pub async fn seed_grade_subject_templates(
    db: &DatabaseConnection,
    rows: &[GradeSubjectTemplateSeedRow],
    now: &str,
) -> Result<(), AppError> {
    for row in rows {
        if exam_grade_subject_time_templates::Entity::find_by_id((
            row.grade_name.clone(),
            row.subject.clone(),
        ))
        .one(db)
        .await?
        .is_some()
        {
            continue;
        }
        exam_grade_subject_time_templates::ActiveModel {
            grade_name: Set(row.grade_name.clone()),
            subject: Set(row.subject.clone()),
            start_at: Set(row.start_at.clone()),
            end_at: Set(row.end_at.clone()),
            updated_at: Set(now.to_string()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn list_participants(
    db: &DatabaseConnection,
    grade_name: &str,
    subject: &str,
    is_selected: i64,
) -> Result<Vec<ParticipantRow>, AppError> {
    let rows = latest_subject_scores::Entity::find()
        .find_also_related(latest_student_scores::Entity)
        .filter(latest_subject_scores::Column::Subject.eq(subject))
        .filter(latest_subject_scores::Column::IsSelected.eq(is_selected))
        .filter(latest_student_scores::Column::GradeName.eq(grade_name))
        .all(db)
        .await?;
    let mut out = Vec::new();
    for (subject_score, student) in rows {
        let Some(student) = student else {
            continue;
        };
        out.push(ParticipantRow {
            admission_no: student.admission_no,
            student_name: student.student_name,
            class_name: student.class_name,
            total_score: student.total_score,
            score: subject_score.score,
        });
    }
    Ok(out)
}

pub async fn list_active_grade_subjects(
    db: &DatabaseConnection,
) -> Result<Vec<ActiveGradeSubjectRow>, AppError> {
    let rows = latest_subject_scores::Entity::find()
        .find_also_related(latest_student_scores::Entity)
        .filter(latest_subject_scores::Column::IsSelected.eq(1))
        .all(db)
        .await?;
    let mut active = std::collections::HashSet::<(String, String)>::new();
    for (subject_score, student) in rows {
        let Some(student) = student else {
            continue;
        };
        active.insert((student.grade_name, subject_score.subject));
    }
    let mut out = active
        .into_iter()
        .map(|(grade_name, subject)| ActiveGradeSubjectRow {
            grade_name,
            subject,
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
        a.grade_name
            .cmp(&b.grade_name)
            .then(a.subject.cmp(&b.subject))
    });
    Ok(out)
}

pub async fn clear_latest_plan_snapshot(tx: &DatabaseTransaction) -> Result<(), AppError> {
    latest_exam_plan_staff_assignments::Entity::delete_many()
        .exec(tx)
        .await?;
    latest_exam_plan_student_allocations::Entity::delete_many()
        .exec(tx)
        .await?;
    latest_exam_plan_spaces::Entity::delete_many()
        .exec(tx)
        .await?;
    latest_exam_plan_sessions::Entity::delete_many()
        .exec(tx)
        .await?;
    latest_exam_plan_meta::Entity::delete_many()
        .exec(tx)
        .await?;
    Ok(())
}

pub async fn update_progress(db: &DatabaseConnection, row: ProgressRow) -> Result<(), AppError> {
    let existing = exam_generation_progress::Entity::find_by_id(1)
        .one(db)
        .await?;
    if let Some(existing) = existing {
        let mut active = existing.into_active_model();
        active.status = Set(row.status);
        active.stage = Set(row.stage);
        active.stage_label = Set(row.stage_label);
        active.percent = Set(row.percent.clamp(0, 100));
        active.message = Set(row.message);
        active.current_grade = Set(row.current_grade);
        active.total_grades = Set(row.total_grades.max(0));
        active.completed_grades = Set(row.completed_grades.max(0));
        active.updated_at = Set(row.updated_at);
        active.update(db).await?;
    } else {
        exam_generation_progress::ActiveModel {
            id: Set(1),
            status: Set(row.status),
            stage: Set(row.stage),
            stage_label: Set(row.stage_label),
            percent: Set(row.percent.clamp(0, 100)),
            message: Set(row.message),
            current_grade: Set(row.current_grade),
            total_grades: Set(row.total_grades.max(0)),
            completed_grades: Set(row.completed_grades.max(0)),
            updated_at: Set(row.updated_at),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn get_progress(db: &DatabaseConnection) -> Result<ProgressRow, AppError> {
    let row = exam_generation_progress::Entity::find_by_id(1)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("考试生成进度未初始化"))?;
    Ok(ProgressRow {
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
}

pub async fn insert_session(
    tx: &DatabaseTransaction,
    row: SessionInsertRow,
) -> Result<i64, AppError> {
    let model = latest_exam_plan_sessions::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        grade_name: Set(row.grade_name),
        subject: Set(row.subject),
        is_foreign_group: Set(row.is_foreign_group),
        foreign_order: Set(row.foreign_order),
        participant_count: Set(row.participant_count),
        exam_room_count: Set(row.exam_room_count),
        self_study_room_count: Set(row.self_study_room_count),
    }
    .insert(tx)
    .await?;
    Ok(model.id)
}

pub async fn insert_space(tx: &DatabaseTransaction, row: SpaceInsertRow) -> Result<i64, AppError> {
    let model = latest_exam_plan_spaces::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        session_id: Set(row.session_id),
        space_type: Set(row.space_type),
        space_source: Set(row.space_source),
        grade_name: Set(row.grade_name),
        subject: Set(row.subject),
        space_name: Set(row.space_name),
        original_class_name: Set(row.original_class_name),
        self_study_topic_kind: Set(row.self_study_topic_kind),
        self_study_topic_subjects_json: Set(row.self_study_topic_subjects_json),
        self_study_topic_label: Set(row.self_study_topic_label),
        building: Set(row.building),
        floor: Set(row.floor),
        capacity: Set(row.capacity),
        sort_index: Set(row.sort_index),
    }
    .insert(tx)
    .await?;
    Ok(model.id)
}

pub async fn insert_student_allocation(
    tx: &DatabaseTransaction,
    row: StudentAllocationInsertRow,
) -> Result<(), AppError> {
    latest_exam_plan_student_allocations::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        session_id: Set(row.session_id),
        admission_no: Set(row.admission_no),
        student_name: Set(row.student_name),
        class_name: Set(row.class_name),
        allocation_type: Set(row.allocation_type),
        space_id: Set(row.space_id),
        seat_no: Set(row.seat_no),
        subject_score: Set(row.subject_score),
    }
    .insert(tx)
    .await?;
    Ok(())
}

pub async fn insert_plan_meta(
    tx: &DatabaseTransaction,
    row: PlanMetaInsertRow,
) -> Result<(), AppError> {
    latest_exam_plan_meta::ActiveModel {
        id: Set(1),
        generated_at: Set(row.generated_at),
        default_capacity: Set(row.default_capacity),
        max_capacity: Set(row.max_capacity),
        grade_count: Set(row.grade_count),
        session_count: Set(row.session_count),
        warning_count: Set(row.warning_count),
    }
    .insert(tx)
    .await?;
    Ok(())
}

pub async fn latest_plan_meta(
    db: &DatabaseConnection,
) -> Result<Option<latest_exam_plan_meta::Model>, AppError> {
    Ok(latest_exam_plan_meta::Entity::find_by_id(1).one(db).await?)
}

pub async fn overview_counts(db: &DatabaseConnection) -> Result<OverviewCounts, AppError> {
    let spaces = latest_exam_plan_spaces::Entity::find().all(db).await?;
    let exam_room_count = spaces
        .iter()
        .filter(|row| row.space_type == "exam_room")
        .count() as i64;
    let self_study_room_count = spaces
        .iter()
        .filter(|row| row.space_type == "self_study_room")
        .count() as i64;
    let allocations = latest_exam_plan_student_allocations::Entity::find()
        .filter(latest_exam_plan_student_allocations::Column::AllocationType.eq("exam"))
        .all(db)
        .await?;
    let mut admissions = std::collections::HashSet::new();
    for row in allocations {
        admissions.insert(row.admission_no);
    }
    Ok(OverviewCounts {
        exam_room_count,
        self_study_room_count,
        student_allocation_count: admissions.len() as i64,
    })
}

pub async fn list_sessions(
    db: &DatabaseConnection,
    filters: SessionFilters,
) -> Result<ListResult<latest_exam_plan_sessions::Model>, AppError> {
    let mut query = latest_exam_plan_sessions::Entity::find();
    if let Some(grade_name) = filters.grade_name {
        query = query.filter(latest_exam_plan_sessions::Column::GradeName.eq(grade_name));
    }
    if let Some(subject) = filters.subject {
        query = query.filter(latest_exam_plan_sessions::Column::Subject.eq(subject));
    }
    let total = query.clone().count(db).await? as i64;
    let rows = query
        .order_by_asc(latest_exam_plan_sessions::Column::GradeName)
        .order_by_desc(latest_exam_plan_sessions::Column::IsForeignGroup)
        .order_by_asc(latest_exam_plan_sessions::Column::ForeignOrder)
        .order_by_asc(latest_exam_plan_sessions::Column::Subject)
        .order_by_asc(latest_exam_plan_sessions::Column::Id)
        .limit(filters.page_size as u64)
        .offset(((filters.page - 1) * filters.page_size) as u64)
        .all(db)
        .await?;
    Ok(ListResult { items: rows, total })
}

pub async fn get_session(
    db: &DatabaseConnection,
    session_id: i64,
) -> Result<Option<latest_exam_plan_sessions::Model>, AppError> {
    Ok(latest_exam_plan_sessions::Entity::find_by_id(session_id)
        .one(db)
        .await?)
}

pub async fn list_spaces(
    db: &DatabaseConnection,
    session_id: i64,
) -> Result<Vec<latest_exam_plan_spaces::Model>, AppError> {
    Ok(latest_exam_plan_spaces::Entity::find()
        .filter(latest_exam_plan_spaces::Column::SessionId.eq(session_id))
        .order_by_asc(latest_exam_plan_spaces::Column::SortIndex)
        .order_by_asc(latest_exam_plan_spaces::Column::Id)
        .all(db)
        .await?)
}

pub async fn list_student_allocations(
    db: &DatabaseConnection,
    session_id: i64,
) -> Result<Vec<latest_exam_plan_student_allocations::Model>, AppError> {
    Ok(latest_exam_plan_student_allocations::Entity::find()
        .filter(latest_exam_plan_student_allocations::Column::SessionId.eq(session_id))
        .order_by_asc(latest_exam_plan_student_allocations::Column::AllocationType)
        .order_by_asc(latest_exam_plan_student_allocations::Column::SpaceId)
        .order_by_asc(latest_exam_plan_student_allocations::Column::SeatNo)
        .order_by_asc(latest_exam_plan_student_allocations::Column::AdmissionNo)
        .all(db)
        .await?)
}

pub async fn list_staff_assignments(
    db: &DatabaseConnection,
    session_id: i64,
) -> Result<Vec<latest_exam_plan_staff_assignments::Model>, AppError> {
    Ok(latest_exam_plan_staff_assignments::Entity::find()
        .filter(latest_exam_plan_staff_assignments::Column::SessionId.eq(session_id))
        .order_by_asc(latest_exam_plan_staff_assignments::Column::Id)
        .all(db)
        .await?)
}
