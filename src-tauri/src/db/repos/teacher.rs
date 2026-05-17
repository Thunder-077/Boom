use std::collections::{HashMap, HashSet};

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, Set,
    TransactionTrait, ColumnTrait, QueryFilter,
};

use crate::entity::{
    latest_teacher_assignments_v2, latest_teacher_homerooms_v2, latest_teacher_import_meta,
    latest_teachers_v2,
};
use crate::score::{AppError, ListResult};
use crate::teacher::{
    AggregatedTeacher, TeacherListParams, TeacherRow, TeacherSubject, TeacherSummary,
};

pub struct ScheduleTeacherAssignment {
    pub teacher_name: String,
    pub subject_key: String,
    pub class_name: String,
}

pub async fn replace_all(
    db: &DatabaseConnection,
    imported_at: &str,
    source_file: &str,
    teachers: &[AggregatedTeacher],
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_teacher_assignments_v2::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_teacher_homerooms_v2::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_teachers_v2::Entity::delete_many().exec(&tx).await?;
    latest_teacher_import_meta::Entity::delete_many()
        .exec(&tx)
        .await?;

    latest_teacher_import_meta::ActiveModel {
        id: Set(1),
        imported_at: Set(imported_at.to_string()),
        source_file: Set(source_file.to_string()),
        row_count: Set(teachers.len() as i32),
    }
    .insert(&tx)
    .await?;

    for teacher in teachers {
        let row = latest_teachers_v2::ActiveModel {
            teacher_name: Set(teacher.teacher_name.clone()),
            remark: Set(teacher.remark.clone()),
            is_middle_manager: Set(if teacher.is_middle_manager { 1 } else { 0 }),
            ..Default::default()
        }
        .insert(&tx)
        .await?;

        for (subject, class_name) in &teacher.assignments {
            latest_teacher_assignments_v2::ActiveModel {
                teacher_id: Set(row.id),
                subject: Set(subject.as_key().to_string()),
                class_name: Set(class_name.clone()),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }

        for class_name in &teacher.homeroom_classes {
            latest_teacher_homerooms_v2::ActiveModel {
                teacher_id: Set(row.id),
                class_name: Set(class_name.clone()),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn sync_from_course_schedule(
    db: &DatabaseConnection,
    assignments: &[ScheduleTeacherAssignment],
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_teacher_assignments_v2::Entity::delete_many()
        .exec(&tx)
        .await?;

    latest_teacher_import_meta::Entity::delete_by_id(1)
        .exec(&tx)
        .await?;
    let imported_at = chrono::Utc::now().to_rfc3339();
    latest_teacher_import_meta::ActiveModel {
        id: Set(1),
        imported_at: Set(imported_at),
        source_file: Set("课表导入同步".to_string()),
        row_count: Set(unique_teacher_names(assignments).len() as i32),
    }
    .insert(&tx)
    .await?;

    let teacher_names = unique_teacher_names(assignments);
    remove_teachers_not_in_schedule(&tx, &teacher_names).await?;

    for teacher_name in &teacher_names {
        let exists = latest_teachers_v2::Entity::find()
            .filter(latest_teachers_v2::Column::TeacherName.eq(teacher_name))
            .one(&tx)
            .await?
            .is_some();
        if !exists {
            latest_teachers_v2::ActiveModel {
                teacher_name: Set(teacher_name.clone()),
                remark: Set(None),
                is_middle_manager: Set(0),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
    }

    let teachers = latest_teachers_v2::Entity::find().all(&tx).await?;
    let teacher_ids = teachers
        .into_iter()
        .map(|teacher| (teacher.teacher_name, teacher.id))
        .collect::<HashMap<_, _>>();
    let mut seen = HashSet::new();
    for assignment in assignments {
        if !seen.insert((
            assignment.teacher_name.clone(),
            assignment.subject_key.clone(),
            assignment.class_name.clone(),
        )) {
            continue;
        }
        let teacher_id = teacher_ids
            .get(&assignment.teacher_name)
            .copied()
            .ok_or_else(|| AppError::new(format!("未找到教师: {}", assignment.teacher_name)))?;
        latest_teacher_assignments_v2::ActiveModel {
            teacher_id: Set(teacher_id),
            subject: Set(assignment.subject_key.clone()),
            class_name: Set(assignment.class_name.clone()),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list(
    db: &DatabaseConnection,
    params: TeacherListParams,
) -> Result<ListResult<TeacherRow>, AppError> {
    let teachers = latest_teachers_v2::Entity::find()
        .order_by_asc(latest_teachers_v2::Column::Id)
        .all(db)
        .await?;
    let assignments = latest_teacher_assignments_v2::Entity::find()
        .order_by_asc(latest_teacher_assignments_v2::Column::Id)
        .all(db)
        .await?;

    let mut assignments_by_teacher: HashMap<i32, Vec<latest_teacher_assignments_v2::Model>> =
        HashMap::new();
    for assignment in assignments {
        assignments_by_teacher
            .entry(assignment.teacher_id)
            .or_default()
            .push(assignment);
    }

    let name_keyword = params
        .name_keyword
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let class_name_filter = params
        .class_name
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut items = Vec::new();
    for teacher in teachers {
        if let Some(keyword) = &name_keyword {
            if !teacher.teacher_name.contains(keyword) {
                continue;
            }
        }

        let teacher_assignments = assignments_by_teacher
            .get(&teacher.id)
            .cloned()
            .unwrap_or_default();

        if let Some(class_name) = &class_name_filter {
            if !teacher_assignments
                .iter()
                .any(|assignment| assignment.class_name.contains(class_name))
            {
                continue;
            }
        }
        if let Some(subject) = params.subject {
            if !teacher_assignments
                .iter()
                .any(|assignment| assignment.subject == subject.as_key())
            {
                continue;
            }
        }

        items.push(row_to_teacher_row(teacher, teacher_assignments));
    }

    Ok(ListResult {
        total: items.len() as i64,
        items,
    })
}

pub async fn summary(db: &DatabaseConnection) -> Result<TeacherSummary, AppError> {
    let imported_at = latest_teacher_import_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.imported_at);
    let teacher_count = latest_teachers_v2::Entity::find().count(db).await? as i64;
    Ok(TeacherSummary {
        imported_at,
        teacher_count,
    })
}

fn row_to_teacher_row(
    teacher: latest_teachers_v2::Model,
    assignments: Vec<latest_teacher_assignments_v2::Model>,
) -> TeacherRow {
    let mut subjects = Vec::new();
    let mut subject_keys = HashSet::new();
    let mut class_names = Vec::new();
    let mut class_keys = HashSet::new();

    for assignment in assignments {
        if let Some(subject) = TeacherSubject::from_key(&assignment.subject) {
            if subject_keys.insert(subject.as_key()) {
                subjects.push(subject);
            }
        }
        if class_keys.insert(assignment.class_name.clone()) {
            class_names.push(assignment.class_name);
        }
    }

    TeacherRow {
        id: i64::from(teacher.id),
        teacher_name: teacher.teacher_name,
        subjects,
        class_names,
        remark: teacher.remark,
        is_middle_manager: teacher.is_middle_manager == 1,
    }
}

fn unique_teacher_names(assignments: &[ScheduleTeacherAssignment]) -> HashSet<String> {
    assignments
        .iter()
        .map(|assignment| assignment.teacher_name.clone())
        .collect()
}

async fn remove_teachers_not_in_schedule<C>(
    db: &C,
    teacher_names: &HashSet<String>,
) -> Result<(), AppError>
where
    C: sea_orm::ConnectionTrait,
{
    let existing_teachers = latest_teachers_v2::Entity::find().all(db).await?;
    let removed_teacher_ids = existing_teachers
        .into_iter()
        .filter(|teacher| !teacher_names.contains(&teacher.teacher_name))
        .map(|teacher| teacher.id)
        .collect::<Vec<_>>();

    for teacher_id in removed_teacher_ids {
        latest_teacher_homerooms_v2::Entity::delete_many()
            .filter(latest_teacher_homerooms_v2::Column::TeacherId.eq(teacher_id))
            .exec(db)
            .await?;
        latest_teachers_v2::Entity::delete_by_id(teacher_id)
            .exec(db)
            .await?;
    }

    Ok(())
}
