use std::collections::HashMap;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

use crate::entity::{latest_import_meta, latest_student_scores, latest_subject_scores};
use crate::score::{
    assign_rank_rows, AppError, LatestSummary, ListResult, ParsedStudent, RankRow, ScoreCellState,
    ScoreDetail, ScoreListParams, ScoreRow, ScoreSubjectItem, Subject,
};

pub async fn persist_latest_snapshot(
    db: &DatabaseConnection,
    source_file: &str,
    imported_at: &str,
    students: &[ParsedStudent],
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_subject_scores::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_student_scores::Entity::delete_many()
        .exec(&tx)
        .await?;
    latest_import_meta::Entity::delete_many().exec(&tx).await?;

    latest_import_meta::ActiveModel {
        id: Set(1),
        imported_at: Set(imported_at.to_string()),
        source_file: Set(source_file.to_string()),
        row_count: Set(students.len() as i64),
    }
    .insert(&tx)
    .await?;

    for student in students {
        latest_student_scores::ActiveModel {
            admission_no: Set(student.admission_no.clone()),
            class_name: Set(student.class_name.clone()),
            grade_name: Set(student.grade_name.clone()),
            student_name: Set(student.student_name.clone()),
            subject_combination: Set(student.subject_combination.clone()),
            language: Set(student.language.clone()),
            total_score: Set(student.total_score),
            class_rank: Set(student.class_rank),
            grade_rank: Set(student.grade_rank),
            selected_subject_count: Set(student.selected_subject_count),
        }
        .insert(&tx)
        .await?;

        for subject in &student.subjects {
            latest_subject_scores::ActiveModel {
                admission_no: Set(student.admission_no.clone()),
                subject: Set(subject.subject.as_key().to_string()),
                score: Set(subject.score),
                is_selected: Set(
                    if matches!(
                        subject.state,
                        ScoreCellState::Scored | ScoreCellState::Absent
                    ) {
                        1
                    } else {
                        0
                    },
                ),
                is_absent: Set(if matches!(subject.state, ScoreCellState::Absent) {
                    1
                } else {
                    0
                }),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list(
    db: &DatabaseConnection,
    params: ScoreListParams,
) -> Result<ListResult<ScoreRow>, AppError> {
    let mut query = latest_student_scores::Entity::find();

    if let Some(keyword) = normalize_filter(params.name_keyword.as_deref()) {
        query = query.filter(latest_student_scores::Column::StudentName.contains(&keyword));
    }
    if let Some(class_name) = normalize_filter(params.class_name.as_deref()) {
        query = query.filter(latest_student_scores::Column::ClassName.contains(&class_name));
    }
    if let Some(grade_name) = normalize_filter(params.grade_name.as_deref()) {
        query = query.filter(latest_student_scores::Column::GradeName.eq(grade_name));
    }

    let total = query.clone().count(db).await? as i64;
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).clamp(1, 500);
    let offset = (page - 1) * page_size;
    let rows = query
        .order_by_asc(latest_student_scores::Column::GradeName)
        .order_by_asc(latest_student_scores::Column::ClassName)
        .order_by_asc(latest_student_scores::Column::ClassRank)
        .order_by_asc(latest_student_scores::Column::AdmissionNo)
        .limit(page_size as u64)
        .offset(offset as u64)
        .all(db)
        .await?;

    Ok(ListResult {
        items: rows.into_iter().map(student_to_score_row).collect(),
        total,
    })
}

pub async fn get_detail(
    db: &DatabaseConnection,
    admission_no: &str,
) -> Result<ScoreDetail, AppError> {
    let student = latest_student_scores::Entity::find_by_id(admission_no.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("未找到该成绩记录"))?;
    let subject_rows = latest_subject_scores::Entity::find()
        .filter(latest_subject_scores::Column::AdmissionNo.eq(student.admission_no.clone()))
        .all(db)
        .await?;

    let mut subjects_by_key = HashMap::new();
    for row in subject_rows {
        if let Some(item) = subject_row_to_item(row) {
            subjects_by_key.insert(item.subject.as_key().to_string(), item);
        }
    }

    let mut subjects = Vec::new();
    for subject in ordered_subjects() {
        subjects.push(
            subjects_by_key
                .remove(subject.as_key())
                .unwrap_or(ScoreSubjectItem {
                    subject,
                    score: None,
                    state: ScoreCellState::NotSelected,
                }),
        );
    }

    Ok(ScoreDetail {
        admission_no: student.admission_no,
        class_name: student.class_name,
        grade_name: student.grade_name,
        student_name: student.student_name,
        subject_combination: student.subject_combination,
        language: student.language,
        total_score: student.total_score,
        class_rank: student.class_rank,
        grade_rank: student.grade_rank,
        selected_subject_count: student.selected_subject_count,
        subjects,
    })
}

pub async fn exists(db: &DatabaseConnection, admission_no: &str) -> Result<bool, AppError> {
    Ok(
        latest_student_scores::Entity::find_by_id(admission_no.to_string())
            .one(db)
            .await?
            .is_some(),
    )
}

pub async fn update_student_scores(
    db: &DatabaseConnection,
    admission_no: &str,
    class_name: &str,
    grade_name: &str,
    student_name: &str,
    total_score: f64,
    selected_subject_count: i64,
    subjects: &[ScoreSubjectItem],
) -> Result<(), AppError> {
    let tx = db.begin().await?;
    latest_student_scores::ActiveModel {
        admission_no: Set(admission_no.to_string()),
        class_name: Set(class_name.to_string()),
        grade_name: Set(grade_name.to_string()),
        student_name: Set(student_name.to_string()),
        total_score: Set(total_score),
        selected_subject_count: Set(selected_subject_count),
        ..Default::default()
    }
    .update(&tx)
    .await?;

    latest_subject_scores::Entity::delete_many()
        .filter(latest_subject_scores::Column::AdmissionNo.eq(admission_no.to_string()))
        .exec(&tx)
        .await?;
    for item in subjects {
        latest_subject_scores::ActiveModel {
            admission_no: Set(admission_no.to_string()),
            subject: Set(item.subject.as_key().to_string()),
            score: Set(item.score),
            is_selected: Set(if matches!(item.state, ScoreCellState::NotSelected) {
                0
            } else {
                1
            }),
            is_absent: Set(if matches!(item.state, ScoreCellState::Absent) {
                1
            } else {
                0
            }),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
    }

    // 单条成绩变更后，班级排名和年级排名都可能受到影响，需要在同一事务内重算。
    recompute_ranks(&tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn summary(db: &DatabaseConnection) -> Result<LatestSummary, AppError> {
    let imported_at = latest_import_meta::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.imported_at);
    let student_count = latest_student_scores::Entity::find().count(db).await? as i64;
    let students = latest_student_scores::Entity::find().all(db).await?;
    let class_count = students
        .iter()
        .map(|row| row.class_name.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;
    let grade_count = students
        .iter()
        .map(|row| row.grade_name.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;

    Ok(LatestSummary {
        imported_at,
        student_count,
        class_count,
        grade_count,
    })
}

async fn recompute_ranks<C>(db: &C) -> Result<(), AppError>
where
    C: sea_orm::ConnectionTrait,
{
    let rows = latest_student_scores::Entity::find()
        .order_by_asc(latest_student_scores::Column::AdmissionNo)
        .all(db)
        .await?;
    let mut rank_rows = rows
        .into_iter()
        .map(|row| RankRow {
            admission_no: row.admission_no,
            class_name: row.class_name,
            grade_name: row.grade_name,
            total_score: row.total_score,
            class_rank: 0,
            grade_rank: 0,
        })
        .collect::<Vec<_>>();
    assign_rank_rows(&mut rank_rows);
    for row in rank_rows {
        latest_student_scores::ActiveModel {
            admission_no: Set(row.admission_no),
            class_rank: Set(row.class_rank),
            grade_rank: Set(row.grade_rank),
            ..Default::default()
        }
        .update(db)
        .await?;
    }
    Ok(())
}

fn normalize_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn ordered_subjects() -> [Subject; 11] {
    [
        Subject::Chinese,
        Subject::Math,
        Subject::English,
        Subject::Physics,
        Subject::Chemistry,
        Subject::Biology,
        Subject::Politics,
        Subject::History,
        Subject::Geography,
        Subject::Russian,
        Subject::Japanese,
    ]
}

fn student_to_score_row(row: latest_student_scores::Model) -> ScoreRow {
    ScoreRow {
        admission_no: row.admission_no,
        class_name: row.class_name,
        grade_name: row.grade_name,
        student_name: row.student_name,
        subject_combination: row.subject_combination,
        language: row.language,
        total_score: row.total_score,
        class_rank: row.class_rank,
        grade_rank: row.grade_rank,
        selected_subject_count: row.selected_subject_count,
    }
}

fn subject_row_to_item(row: latest_subject_scores::Model) -> Option<ScoreSubjectItem> {
    let subject = Subject::from_key(&row.subject)?;
    let state = if row.is_selected == 0 {
        ScoreCellState::NotSelected
    } else if row.is_absent == 1 {
        ScoreCellState::Absent
    } else {
        ScoreCellState::Scored
    };
    Some(ScoreSubjectItem {
        subject,
        score: row.score,
        state,
    })
}
