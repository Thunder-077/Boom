use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::class_config::{
    dedup_subjects, validate_payload, ClassConfigDetail, ClassConfigRow, ClassConfigType,
    ListClassConfigsParams, UpsertClassConfigPayload,
};
use crate::entity::{class_config_subjects, class_configs};
use crate::score::{AppError, ListResult, Subject};

pub async fn ensure_seeded(db: &DatabaseConnection) -> Result<(), AppError> {
    let count = class_configs::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let tx = db.begin().await?;
    let now = Utc::now().to_rfc3339();

    let grade1 = vec![
        Subject::Chinese,
        Subject::Math,
        Subject::English,
        Subject::Russian,
        Subject::Physics,
        Subject::Chemistry,
        Subject::Biology,
        Subject::History,
        Subject::Geography,
        Subject::Politics,
    ];
    for idx in 1..=4 {
        insert_in_tx(
            &tx,
            &UpsertClassConfigPayload {
                config_type: ClassConfigType::TeachingClass,
                grade_name: "高一".to_string(),
                class_name: format!("高一{}班", idx),
                building: "向远楼".to_string(),
                floor: "3层".to_string(),
                room_label: None,
                subjects: Some(grade1.clone()),
            },
            &now,
        )
        .await?;
    }

    let grade2: [(&str, Vec<Subject>); 11] = [
        (
            "高二1班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::Physics,
                Subject::Chemistry,
                Subject::Biology,
                Subject::Russian,
            ],
        ),
        (
            "高二2班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::Physics,
                Subject::Chemistry,
                Subject::Biology,
                Subject::English,
            ],
        ),
        (
            "高二3班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::Physics,
                Subject::Chemistry,
                Subject::Geography,
                Subject::Russian,
            ],
        ),
        (
            "高二4班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::Physics,
                Subject::Chemistry,
                Subject::Geography,
                Subject::English,
            ],
        ),
        (
            "高二5班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Biology,
                Subject::Geography,
                Subject::Russian,
            ],
        ),
        (
            "高二6班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Biology,
                Subject::Geography,
                Subject::English,
            ],
        ),
        (
            "高二7班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Politics,
                Subject::Geography,
                Subject::Russian,
            ],
        ),
        (
            "高二8班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Politics,
                Subject::Geography,
                Subject::English,
                Subject::Russian,
            ],
        ),
        (
            "高二9班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Politics,
                Subject::Geography,
                Subject::English,
                Subject::Russian,
            ],
        ),
        (
            "高二10班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Politics,
                Subject::Geography,
                Subject::English,
                Subject::Russian,
            ],
        ),
        (
            "高二11班",
            vec![
                Subject::Chinese,
                Subject::Math,
                Subject::History,
                Subject::Politics,
                Subject::Geography,
                Subject::English,
                Subject::Russian,
            ],
        ),
    ];

    for (name, subjects) in grade2 {
        insert_in_tx(
            &tx,
            &UpsertClassConfigPayload {
                config_type: ClassConfigType::TeachingClass,
                grade_name: "高二".to_string(),
                class_name: name.to_string(),
                building: "教学楼B".to_string(),
                floor: "4层".to_string(),
                room_label: None,
                subjects: Some(subjects),
            },
            &now,
        )
        .await?;
    }

    for (grade_name, class_name, building, floor) in [
        ("高一", "高一5场", "向远楼", "5层"),
        ("高一", "高一6场", "向远楼", "5层"),
        ("高二", "高二12场", "向远楼", "2层"),
        ("高二", "高二13场", "向远楼", "2层"),
        ("高二", "高二14场", "向远楼", "4层"),
    ] {
        insert_in_tx(
            &tx,
            &UpsertClassConfigPayload {
                config_type: ClassConfigType::ExamRoom,
                grade_name: grade_name.to_string(),
                class_name: class_name.to_string(),
                building: building.to_string(),
                floor: floor.to_string(),
                room_label: None,
                subjects: Some(Vec::new()),
            },
            &now,
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list(
    db: &DatabaseConnection,
    params: ListClassConfigsParams,
) -> Result<ListResult<ClassConfigRow>, AppError> {
    let config_type = params.config_type.unwrap_or(ClassConfigType::TeachingClass);
    let condition = build_list_condition(config_type, &params);
    let total = class_configs::Entity::find()
        .filter(condition.clone())
        .count(db)
        .await? as i64;
    let rows = class_configs::Entity::find()
        .filter(condition)
        .order_by_asc(class_configs::Column::GradeName)
        .order_by_asc(class_configs::Column::ClassName)
        .order_by_asc(class_configs::Column::Id)
        .all(db)
        .await?;

    Ok(ListResult {
        total,
        items: rows
            .into_iter()
            .map(row_to_list_item)
            .collect::<Result<_, _>>()?,
    })
}

pub async fn get_detail(db: &DatabaseConnection, id: i64) -> Result<ClassConfigDetail, AppError> {
    let row = class_configs::Entity::find_by_id(id as i32)
        .one(db)
        .await?
        .ok_or_else(|| AppError::new("配置不存在"))?;
    let subjects = class_config_subjects::Entity::find()
        .filter(class_config_subjects::Column::ConfigId.eq(row.id))
        .order_by_asc(class_config_subjects::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .filter_map(|item| Subject::from_key(&item.subject))
        .collect();

    row_to_detail(row, subjects)
}

pub async fn create(
    db: &DatabaseConnection,
    payload: &UpsertClassConfigPayload,
) -> Result<i64, AppError> {
    let tx = db.begin().await?;
    let id = insert_in_tx(&tx, payload, &Utc::now().to_rfc3339()).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    payload: &UpsertClassConfigPayload,
) -> Result<(), AppError> {
    validate_payload(payload)?;
    let tx = db.begin().await?;
    let existing = class_configs::Entity::find_by_id(id as i32)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::new("配置不存在"))?;
    let mut active: class_configs::ActiveModel = existing.into();
    active.config_type = Set(payload.config_type.as_key().to_string());
    active.grade_name = Set(payload.grade_name.trim().to_string());
    active.class_name = Set(payload.class_name.trim().to_string());
    active.building = Set(payload.building.trim().to_string());
    active.floor = Set(payload.floor.trim().to_string());
    active.room_label = Set(trim_optional(payload.room_label.as_ref()));
    active.updated_at = Set(Utc::now().to_rfc3339());
    active.update(&tx).await?;
    replace_subjects(&tx, id as i32, payload).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    let tx = db.begin().await?;
    class_config_subjects::Entity::delete_many()
        .filter(class_config_subjects::Column::ConfigId.eq(id as i32))
        .exec(&tx)
        .await?;
    let result = class_configs::Entity::delete_by_id(id as i32)
        .exec(&tx)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::new("配置不存在"));
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_grade_options(db: &DatabaseConnection) -> Result<Vec<String>, AppError> {
    let rows = class_configs::Entity::find()
        .filter(class_configs::Column::ConfigType.eq(ClassConfigType::TeachingClass.as_key()))
        .order_by_asc(class_configs::Column::GradeName)
        .all(db)
        .await?;
    let mut items = Vec::new();
    for row in rows {
        if !items.contains(&row.grade_name) {
            items.push(row.grade_name);
        }
    }
    Ok(items)
}

async fn insert_in_tx<C>(
    db: &C,
    payload: &UpsertClassConfigPayload,
    now: &str,
) -> Result<i64, AppError>
where
    C: sea_orm::ConnectionTrait,
{
    validate_payload(payload)?;
    let row = class_configs::ActiveModel {
        config_type: Set(payload.config_type.as_key().to_string()),
        grade_name: Set(payload.grade_name.trim().to_string()),
        class_name: Set(payload.class_name.trim().to_string()),
        building: Set(payload.building.trim().to_string()),
        floor: Set(payload.floor.trim().to_string()),
        room_label: Set(trim_optional(payload.room_label.as_ref())),
        created_at: Set(now.to_string()),
        updated_at: Set(now.to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    replace_subjects(db, row.id, payload).await?;
    Ok(i64::from(row.id))
}

async fn replace_subjects<C>(
    db: &C,
    config_id: i32,
    payload: &UpsertClassConfigPayload,
) -> Result<(), AppError>
where
    C: sea_orm::ConnectionTrait,
{
    class_config_subjects::Entity::delete_many()
        .filter(class_config_subjects::Column::ConfigId.eq(config_id))
        .exec(db)
        .await?;
    for subject in dedup_subjects(payload.subjects.clone().unwrap_or_default()) {
        class_config_subjects::ActiveModel {
            config_id: Set(config_id),
            subject: Set(subject.as_key().to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn build_list_condition(
    config_type: ClassConfigType,
    params: &ListClassConfigsParams,
) -> Condition {
    let mut condition =
        Condition::all().add(class_configs::Column::ConfigType.eq(config_type.as_key()));

    if config_type == ClassConfigType::TeachingClass {
        if let Some(grade_name) = params
            .grade_name
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            condition = condition.add(class_configs::Column::GradeName.eq(grade_name));
        }
    }
    if let Some(keyword) = params
        .keyword
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        condition = condition.add(class_configs::Column::ClassName.contains(keyword));
    }
    condition
}

fn row_to_list_item(row: class_configs::Model) -> Result<ClassConfigRow, AppError> {
    Ok(ClassConfigRow {
        id: i64::from(row.id),
        config_type: ClassConfigType::from_key(&row.config_type)
            .ok_or_else(|| AppError::new("配置类型错误"))?,
        grade_name: row.grade_name,
        class_name: row.class_name,
        building: row.building,
        floor: row.floor,
        room_label: row.room_label,
        updated_at: row.updated_at,
    })
}

fn row_to_detail(
    row: class_configs::Model,
    subjects: Vec<Subject>,
) -> Result<ClassConfigDetail, AppError> {
    Ok(ClassConfigDetail {
        id: i64::from(row.id),
        config_type: ClassConfigType::from_key(&row.config_type)
            .ok_or_else(|| AppError::new("配置类型错误"))?,
        grade_name: row.grade_name,
        class_name: row.class_name,
        building: row.building,
        floor: row.floor,
        room_label: row.room_label,
        subjects,
        updated_at: row.updated_at,
    })
}

fn trim_optional(value: Option<&String>) -> Option<String> {
    value
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
}
