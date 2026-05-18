use std::collections::{HashMap, HashSet};

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};

use crate::course_management::{
    split_teacher_names, CourseClassOption, CourseImportBatch, CoursePeriodSlot,
    CourseScheduleChange, CourseScheduleEntry, CourseSummary, ParsedWorkbook,
    SaveCourseSubstitutionsPayload,
};
use crate::entity::{
    course_schedule_changes, course_schedule_classes, course_schedule_entries,
    course_schedule_imports, course_schedule_periods,
};
use crate::score::AppError;

const INSERT_CHUNK_SIZE: usize = 500;

#[derive(Clone)]
pub struct CourseEntryRow {
    pub id: i64,
    pub import_id: i64,
    pub week_index: i64,
    pub day_of_week: i64,
    pub day_label: String,
    pub period_index: i64,
    pub period_label: String,
    pub section_label: String,
    pub subject: String,
    pub teacher_names: Vec<String>,
    pub class_name: String,
    pub display_class_name: String,
    pub class_type: String,
}

pub async fn persist_course_import(
    db: &DatabaseConnection,
    imported_at: &str,
    source_file: &str,
    parsed: &ParsedWorkbook,
) -> Result<i64, AppError> {
    let tx = db.begin().await?;
    let teacher_count = parsed
        .assignments
        .iter()
        .map(|item| item.teacher_name.clone())
        .collect::<HashSet<_>>()
        .len() as i64;
    let admin_class_count = parsed
        .classes
        .iter()
        .filter(|item| item.class_type == "admin")
        .count() as i64;
    let foreign_class_count = parsed
        .classes
        .iter()
        .filter(|item| item.class_type == "foreign")
        .count() as i64;

    let import = course_schedule_imports::ActiveModel {
        imported_at: Set(imported_at.to_string()),
        source_file: Set(source_file.to_string()),
        entry_count: Set(parsed.entries.len() as i64),
        teacher_count: Set(teacher_count),
        admin_class_count: Set(admin_class_count),
        foreign_class_count: Set(foreign_class_count),
        effective_start_date: Set(None),
        effective_end_date: Set(None),
        start_week: Set(1),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    let class_rows = parsed
        .classes
        .iter()
        .enumerate()
        .map(
            |(sort_index, class_option)| course_schedule_classes::ActiveModel {
                import_id: Set(import.id),
                class_name: Set(class_option.class_name.clone()),
                display_name: Set(class_option.display_name.clone()),
                class_type: Set(class_option.class_type.clone()),
                sort_index: Set(sort_index as i64),
                ..Default::default()
            },
        )
        .collect::<Vec<_>>();
    for chunk in class_rows.chunks(INSERT_CHUNK_SIZE) {
        course_schedule_classes::Entity::insert_many(chunk.iter().cloned())
            .exec(&tx)
            .await?;
    }

    let period_rows = parsed
        .periods
        .iter()
        .map(|period| course_schedule_periods::ActiveModel {
            import_id: Set(import.id),
            week_index: Set(period.week_index),
            day_of_week: Set(period.day_of_week),
            day_label: Set(period.day_label.clone()),
            period_index: Set(period.period_index),
            period_label: Set(period.period_label.clone()),
            section_label: Set(period.section_label.clone()),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    for chunk in period_rows.chunks(INSERT_CHUNK_SIZE) {
        course_schedule_periods::Entity::insert_many(chunk.iter().cloned())
            .exec(&tx)
            .await?;
    }

    let entry_rows = parsed
        .entries
        .iter()
        .map(|entry| {
            let teacher_text = entry.teacher_names.join("/");
            course_schedule_entries::ActiveModel {
                import_id: Set(import.id),
                class_name: Set(entry.class_name.clone()),
                display_class_name: Set(entry.display_class_name.clone()),
                class_type: Set(entry.class_type.clone()),
                week_index: Set(entry.week_index),
                day_of_week: Set(entry.day_of_week),
                day_label: Set(entry.day_label.clone()),
                period_index: Set(entry.period_index),
                period_label: Set(entry.period_label.clone()),
                section_label: Set(entry.section_label.clone()),
                subject: Set(entry.subject.clone()),
                teacher_names: Set(teacher_text.clone()),
                teacher_search_text: Set(teacher_text),
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();
    for chunk in entry_rows.chunks(INSERT_CHUNK_SIZE) {
        course_schedule_entries::Entity::insert_many(chunk.iter().cloned())
            .exec(&tx)
            .await?;
    }

    tx.commit().await?;
    Ok(import.id)
}

pub async fn latest_import_id(db: &DatabaseConnection) -> Result<Option<i64>, AppError> {
    Ok(course_schedule_imports::Entity::find()
        .order_by_desc(course_schedule_imports::Column::Id)
        .one(db)
        .await?
        .map(|row| row.id))
}

pub async fn summary(db: &DatabaseConnection) -> Result<CourseSummary, AppError> {
    let Some(import) = course_schedule_imports::Entity::find()
        .order_by_desc(course_schedule_imports::Column::Id)
        .one(db)
        .await?
    else {
        return Ok(CourseSummary {
            latest_import_id: None,
            imported_at: None,
            entry_count: 0,
            teacher_count: 0,
            admin_class_count: 0,
            foreign_class_count: 0,
            effective_start_date: None,
            effective_end_date: None,
            start_week: 1,
        });
    };
    Ok(CourseSummary {
        latest_import_id: Some(import.id),
        imported_at: Some(import.imported_at),
        entry_count: import.entry_count,
        teacher_count: import.teacher_count,
        admin_class_count: import.admin_class_count,
        foreign_class_count: import.foreign_class_count,
        effective_start_date: import.effective_start_date,
        effective_end_date: import.effective_end_date,
        start_week: import.start_week,
    })
}

pub async fn list_imports(db: &DatabaseConnection) -> Result<Vec<CourseImportBatch>, AppError> {
    let rows = course_schedule_imports::Entity::find()
        .order_by_desc(course_schedule_imports::Column::Id)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(import_to_batch).collect())
}

pub async fn list_classes(
    db: &DatabaseConnection,
    import_id: i64,
    class_type: &str,
) -> Result<Vec<CourseClassOption>, AppError> {
    let rows = course_schedule_classes::Entity::find()
        .filter(course_schedule_classes::Column::ImportId.eq(import_id))
        .filter(course_schedule_classes::Column::ClassType.eq(class_type))
        .order_by_asc(course_schedule_classes::Column::SortIndex)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| CourseClassOption {
            class_name: row.class_name,
            display_name: row.display_name,
            class_type: row.class_type,
        })
        .collect())
}

pub async fn list_period_slots(
    db: &DatabaseConnection,
    import_id: i64,
) -> Result<Vec<CoursePeriodSlot>, AppError> {
    let rows = course_schedule_periods::Entity::find()
        .filter(course_schedule_periods::Column::ImportId.eq(import_id))
        .order_by_asc(course_schedule_periods::Column::WeekIndex)
        .order_by_asc(course_schedule_periods::Column::DayOfWeek)
        .order_by_asc(course_schedule_periods::Column::PeriodIndex)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| CoursePeriodSlot {
            week_index: row.week_index,
            day_of_week: row.day_of_week,
            day_label: row.day_label,
            period_index: row.period_index,
            period_label: row.period_label,
            section_label: row.section_label,
        })
        .collect())
}

pub async fn update_import_settings(
    db: &DatabaseConnection,
    import_id: i64,
    effective_start_date: Option<String>,
    effective_end_date: Option<String>,
    start_week: i64,
) -> Result<CourseImportBatch, AppError> {
    let row = course_schedule_imports::Entity::find_by_id(import_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("未找到课表批次"))?;
    let mut active = row.into_active_model();
    active.effective_start_date = Set(effective_start_date);
    active.effective_end_date = Set(effective_end_date);
    active.start_week = Set(start_week);
    let updated = active.update(db).await?;
    Ok(import_to_batch(updated))
}

pub async fn delete_import(db: &DatabaseConnection, import_id: i64) -> Result<(), AppError> {
    let tx = db.begin().await?;
    course_schedule_changes::Entity::delete_many()
        .filter(course_schedule_changes::Column::ImportId.eq(import_id))
        .exec(&tx)
        .await?;
    course_schedule_entries::Entity::delete_many()
        .filter(course_schedule_entries::Column::ImportId.eq(import_id))
        .exec(&tx)
        .await?;
    course_schedule_periods::Entity::delete_many()
        .filter(course_schedule_periods::Column::ImportId.eq(import_id))
        .exec(&tx)
        .await?;
    course_schedule_classes::Entity::delete_many()
        .filter(course_schedule_classes::Column::ImportId.eq(import_id))
        .exec(&tx)
        .await?;
    course_schedule_imports::Entity::delete_by_id(import_id)
        .exec(&tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_teacher_texts(
    db: &DatabaseConnection,
    import_id: i64,
) -> Result<Vec<String>, AppError> {
    let rows = course_schedule_entries::Entity::find()
        .filter(course_schedule_entries::Column::ImportId.eq(import_id))
        .filter(course_schedule_entries::Column::TeacherNames.ne(""))
        .order_by_asc(course_schedule_entries::Column::TeacherNames)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|row| row.teacher_names).collect())
}

pub async fn import_anchor(
    db: &DatabaseConnection,
    import_id: i64,
) -> Result<(Option<String>, i64), AppError> {
    let row = course_schedule_imports::Entity::find_by_id(import_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("未找到课表批次"))?;
    Ok((row.effective_start_date, row.start_week.max(1)))
}

pub async fn schedule_week_count(db: &DatabaseConnection, import_id: i64) -> Result<i64, AppError> {
    let count = course_schedule_periods::Entity::find()
        .filter(course_schedule_periods::Column::ImportId.eq(import_id))
        .count(db)
        .await?;
    if count == 0 {
        return Ok(1);
    }
    let rows = course_schedule_periods::Entity::find()
        .filter(course_schedule_periods::Column::ImportId.eq(import_id))
        .order_by_desc(course_schedule_periods::Column::WeekIndex)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.week_index)
        .max()
        .unwrap_or(1)
        .max(1))
}

pub async fn list_entries_for_slot(
    db: &DatabaseConnection,
    import_id: i64,
    week_index: i64,
    day_of_week: i64,
    start_period: i64,
    end_period: i64,
) -> Result<Vec<CourseEntryRow>, AppError> {
    let rows = course_schedule_entries::Entity::find()
        .filter(course_schedule_entries::Column::ImportId.eq(import_id))
        .filter(course_schedule_entries::Column::WeekIndex.eq(week_index))
        .filter(course_schedule_entries::Column::DayOfWeek.eq(day_of_week))
        .filter(course_schedule_entries::Column::PeriodIndex.gte(start_period))
        .filter(course_schedule_entries::Column::PeriodIndex.lte(end_period))
        .order_by_asc(course_schedule_entries::Column::DayOfWeek)
        .order_by_asc(course_schedule_entries::Column::PeriodIndex)
        .order_by_asc(course_schedule_entries::Column::ClassName)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(entry_to_row).collect())
}

pub async fn list_entries_for_teacher_slot(
    db: &DatabaseConnection,
    import_id: i64,
    week_index: i64,
    day_of_week: i64,
    start_period: i64,
    end_period: i64,
    teacher_name: &str,
) -> Result<Vec<CourseEntryRow>, AppError> {
    let rows = course_schedule_entries::Entity::find()
        .filter(course_schedule_entries::Column::ImportId.eq(import_id))
        .filter(course_schedule_entries::Column::WeekIndex.eq(week_index))
        .filter(course_schedule_entries::Column::DayOfWeek.eq(day_of_week))
        .filter(course_schedule_entries::Column::PeriodIndex.gte(start_period))
        .filter(course_schedule_entries::Column::PeriodIndex.lte(end_period))
        .filter(course_schedule_entries::Column::TeacherSearchText.contains(teacher_name))
        .order_by_asc(course_schedule_entries::Column::PeriodIndex)
        .order_by_asc(course_schedule_entries::Column::ClassName)
        .all(db)
        .await?;
    Ok(rows.into_iter().map(entry_to_row).collect())
}

pub async fn list_entries_for_view(
    db: &DatabaseConnection,
    import_id: i64,
    view_type: &str,
    target: &str,
) -> Result<Vec<CourseScheduleEntry>, AppError> {
    let mut query = course_schedule_entries::Entity::find()
        .filter(course_schedule_entries::Column::ImportId.eq(import_id));
    query = match view_type {
        "teacher" => {
            query.filter(course_schedule_entries::Column::TeacherSearchText.contains(target))
        }
        "foreign_class" => query
            .filter(course_schedule_entries::Column::ClassType.eq("foreign"))
            .filter(course_schedule_entries::Column::ClassName.eq(target)),
        _ => query
            .filter(course_schedule_entries::Column::ClassType.eq("admin"))
            .filter(course_schedule_entries::Column::ClassName.eq(target)),
    };
    let rows = query
        .order_by_asc(course_schedule_entries::Column::WeekIndex)
        .order_by_asc(course_schedule_entries::Column::DayOfWeek)
        .order_by_asc(course_schedule_entries::Column::PeriodIndex)
        .order_by_asc(course_schedule_entries::Column::ClassName)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| CourseScheduleEntry {
            week_index: row.week_index,
            day_of_week: row.day_of_week,
            day_label: row.day_label,
            period_index: row.period_index,
            period_label: row.period_label,
            section_label: row.section_label,
            subject: row.subject,
            teacher_names: split_teacher_names(&row.teacher_names),
            class_name: row.class_name,
            display_class_name: row.display_class_name,
            class_type: row.class_type,
        })
        .collect())
}

pub async fn active_changes_for_date(
    db: &DatabaseConnection,
    import_id: i64,
    target_date: &str,
) -> Result<HashMap<(i64, String), CourseScheduleChange>, AppError> {
    let changes = course_schedule_changes::Entity::find()
        .filter(course_schedule_changes::Column::ImportId.eq(import_id))
        .filter(course_schedule_changes::Column::TargetDate.eq(target_date))
        .filter(course_schedule_changes::Column::Status.eq("active"))
        .all(db)
        .await?;
    let mut result = HashMap::new();
    for change in changes_with_entries(db, changes).await? {
        result.insert(
            (change.source_entry_id, change.source_teacher_name.clone()),
            change,
        );
    }
    Ok(result)
}

pub async fn active_change_for_slot(
    db: &DatabaseConnection,
    source_entry_id: i64,
    target_date: &str,
    source_teacher_name: &str,
) -> Result<Option<CourseScheduleChange>, AppError> {
    let changes = course_schedule_changes::Entity::find()
        .filter(course_schedule_changes::Column::SourceEntryId.eq(source_entry_id))
        .filter(course_schedule_changes::Column::TargetDate.eq(target_date))
        .filter(course_schedule_changes::Column::SourceTeacherName.eq(source_teacher_name))
        .filter(course_schedule_changes::Column::Status.eq("active"))
        .order_by_desc(course_schedule_changes::Column::Id)
        .all(db)
        .await?;
    Ok(changes_with_entries(db, changes).await?.into_iter().next())
}

pub async fn list_changes_for_import(
    db: &DatabaseConnection,
    import_id: i64,
) -> Result<Vec<CourseScheduleChange>, AppError> {
    let changes = course_schedule_changes::Entity::find()
        .filter(course_schedule_changes::Column::ImportId.eq(import_id))
        .order_by_desc(course_schedule_changes::Column::TargetDate)
        .order_by_asc(course_schedule_changes::Column::Id)
        .all(db)
        .await?;
    let mut rows = changes_with_entries(db, changes).await?;
    rows.sort_by(|a, b| {
        b.target_date
            .cmp(&a.target_date)
            .then(a.period_index.cmp(&b.period_index))
            .then(b.id.cmp(&a.id))
    });
    Ok(rows)
}

pub async fn save_substitutions(
    db: &DatabaseConnection,
    payload: &SaveCourseSubstitutionsPayload,
    now: &str,
) -> Result<Vec<CourseScheduleChange>, AppError> {
    let tx = db.begin().await?;
    for item in &payload.items {
        let source_teacher_name = item.source_teacher_name.trim();
        let actual_teacher_name = item.actual_teacher_name.trim();
        let entry = course_schedule_entries::Entity::find_by_id(item.source_entry_id)
            .filter(course_schedule_entries::Column::ImportId.eq(payload.import_id))
            .one(&tx)
            .await?
            .ok_or_else(|| AppError::new("未找到课次"))?;
        if !split_teacher_names(&entry.teacher_names)
            .iter()
            .any(|name| name == source_teacher_name)
        {
            return Err(AppError::new(format!(
                "课次中未找到原任课教师: {source_teacher_name}"
            )));
        }
        let item_remark = item
            .remark
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| payload.remark.trim().to_string());
        let existing = course_schedule_changes::Entity::find()
            .filter(course_schedule_changes::Column::ImportId.eq(payload.import_id))
            .filter(course_schedule_changes::Column::SourceEntryId.eq(item.source_entry_id))
            .filter(course_schedule_changes::Column::TargetDate.eq(&item.target_date))
            .filter(course_schedule_changes::Column::SourceTeacherName.eq(source_teacher_name))
            .filter(course_schedule_changes::Column::Status.eq("active"))
            .order_by_desc(course_schedule_changes::Column::Id)
            .one(&tx)
            .await?;
        if let Some(row) = existing {
            let mut active = row.into_active_model();
            active.actual_teacher_name = Set(actual_teacher_name.to_string());
            active.reason = Set(payload.reason.trim().to_string());
            active.remark = Set(item_remark);
            active.updated_at = Set(now.to_string());
            active.update(&tx).await?;
        } else {
            course_schedule_changes::ActiveModel {
                import_id: Set(payload.import_id),
                source_entry_id: Set(item.source_entry_id),
                change_type: Set("substitute".to_string()),
                status: Set("active".to_string()),
                target_date: Set(item.target_date.clone()),
                source_teacher_name: Set(source_teacher_name.to_string()),
                actual_teacher_name: Set(actual_teacher_name.to_string()),
                reason: Set(payload.reason.trim().to_string()),
                remark: Set(item_remark),
                created_at: Set(now.to_string()),
                updated_at: Set(now.to_string()),
                revoked_at: Set(None),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
    }
    tx.commit().await?;
    list_changes_for_import(db, payload.import_id).await
}

pub async fn revoke_change(db: &DatabaseConnection, change_id: i64) -> Result<(), AppError> {
    if let Some(row) = course_schedule_changes::Entity::find_by_id(change_id)
        .filter(course_schedule_changes::Column::Status.eq("active"))
        .one(db)
        .await?
    {
        course_schedule_changes::Entity::delete_by_id(row.id)
            .exec(db)
            .await?;
    }
    Ok(())
}

fn import_to_batch(row: course_schedule_imports::Model) -> CourseImportBatch {
    CourseImportBatch {
        id: row.id,
        imported_at: row.imported_at,
        source_file: row.source_file,
        entry_count: row.entry_count,
        teacher_count: row.teacher_count,
        admin_class_count: row.admin_class_count,
        foreign_class_count: row.foreign_class_count,
        effective_start_date: row.effective_start_date,
        effective_end_date: row.effective_end_date,
        start_week: row.start_week,
    }
}

fn entry_to_row(row: course_schedule_entries::Model) -> CourseEntryRow {
    CourseEntryRow {
        id: row.id,
        import_id: row.import_id,
        week_index: row.week_index,
        day_of_week: row.day_of_week,
        day_label: row.day_label,
        period_index: row.period_index,
        period_label: row.period_label,
        section_label: row.section_label,
        subject: row.subject,
        teacher_names: split_teacher_names(&row.teacher_names),
        class_name: row.class_name,
        display_class_name: row.display_class_name,
        class_type: row.class_type,
    }
}

async fn changes_with_entries(
    db: &DatabaseConnection,
    changes: Vec<course_schedule_changes::Model>,
) -> Result<Vec<CourseScheduleChange>, AppError> {
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let entry_ids = changes
        .iter()
        .map(|change| change.source_entry_id)
        .collect::<HashSet<_>>();
    let entries = course_schedule_entries::Entity::find()
        .filter(course_schedule_entries::Column::Id.is_in(entry_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|entry| (entry.id, entry))
        .collect::<HashMap<_, _>>();
    let mut rows = Vec::new();
    for change in changes {
        let entry = entries
            .get(&change.source_entry_id)
            .ok_or_else(|| AppError::new("换课记录关联课次不存在"))?;
        rows.push(CourseScheduleChange {
            id: change.id,
            import_id: change.import_id,
            source_entry_id: change.source_entry_id,
            change_type: change.change_type,
            status: change.status,
            target_date: change.target_date,
            source_teacher_name: change.source_teacher_name,
            actual_teacher_name: change.actual_teacher_name,
            reason: change.reason,
            remark: change.remark,
            created_at: change.created_at,
            updated_at: change.updated_at,
            revoked_at: change.revoked_at,
            week_index: entry.week_index,
            day_of_week: entry.day_of_week,
            day_label: entry.day_label.clone(),
            period_index: entry.period_index,
            period_label: entry.period_label.clone(),
            section_label: entry.section_label.clone(),
            subject: entry.subject.clone(),
            class_name: entry.class_name.clone(),
            display_class_name: entry.display_class_name.clone(),
            class_type: entry.class_type.clone(),
        });
    }
    Ok(rows)
}
