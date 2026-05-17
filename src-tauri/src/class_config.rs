use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::score::{AppError, ListResult, Subject};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClassConfigType {
    TeachingClass,
    ExamRoom,
}

impl ClassConfigType {
    pub(crate) fn as_key(self) -> &'static str {
        match self {
            ClassConfigType::TeachingClass => "teaching_class",
            ClassConfigType::ExamRoom => "exam_room",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<Self> {
        match key {
            "teaching_class" => Some(ClassConfigType::TeachingClass),
            "exam_room" => Some(ClassConfigType::ExamRoom),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfigRow {
    pub(crate) id: i64,
    pub(crate) config_type: ClassConfigType,
    pub(crate) grade_name: String,
    pub(crate) class_name: String,
    pub(crate) building: String,
    pub(crate) floor: String,
    pub(crate) room_label: Option<String>,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfigDetail {
    pub(crate) id: i64,
    pub(crate) config_type: ClassConfigType,
    pub(crate) grade_name: String,
    pub(crate) class_name: String,
    pub(crate) building: String,
    pub(crate) floor: String,
    pub(crate) room_label: Option<String>,
    pub(crate) subjects: Vec<Subject>,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClassConfigsParams {
    pub config_type: Option<ClassConfigType>,
    pub grade_name: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertClassConfigPayload {
    pub config_type: ClassConfigType,
    pub grade_name: String,
    pub class_name: String,
    pub building: String,
    pub floor: String,
    pub room_label: Option<String>,
    pub subjects: Option<Vec<Subject>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClassConfigResult {
    pub(crate) id: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub(crate) success: bool,
}

pub(crate) fn validate_payload(payload: &UpsertClassConfigPayload) -> Result<(), AppError> {
    if payload.class_name.trim().is_empty() {
        return Err(AppError::new("班级/教室名称不能为空"));
    }
    if payload.grade_name.trim().is_empty() {
        return Err(AppError::new("年级不能为空"));
    }
    if payload.building.trim().is_empty() {
        return Err(AppError::new("楼栋不能为空"));
    }
    if payload.floor.trim().is_empty() {
        return Err(AppError::new("楼层不能为空"));
    }
    let subjects = payload.subjects.clone().unwrap_or_default();
    if payload.config_type == ClassConfigType::TeachingClass && subjects.is_empty() {
        return Err(AppError::new("教学班至少需要一个科目"));
    }
    if payload.config_type == ClassConfigType::ExamRoom && !subjects.is_empty() {
        return Err(AppError::new("考试教室不允许配置科目"));
    }
    Ok(())
}

pub(crate) fn dedup_subjects(input: Vec<Subject>) -> Vec<Subject> {
    let mut keys = HashSet::new();
    let mut out = Vec::new();
    for subject in input {
        if keys.insert(subject.as_key()) {
            out.push(subject);
        }
    }
    out
}

#[tauri::command]
pub async fn list_class_configs(
    app: AppHandle,
    params: ListClassConfigsParams,
) -> Result<ListResult<ClassConfigRow>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        crate::db::repos::class_config::list(&db, params).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_class_config_detail(app: AppHandle, id: i64) -> Result<ClassConfigDetail, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        crate::db::repos::class_config::get_detail(&db, id).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_class_config(
    app: AppHandle,
    payload: UpsertClassConfigPayload,
) -> Result<CreateClassConfigResult, String> {
    let result: Result<CreateClassConfigResult, AppError> = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        let id = crate::db::repos::class_config::create(&db, &payload).await?;
        Ok(CreateClassConfigResult { id })
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_class_config(
    app: AppHandle,
    id: i64,
    payload: UpsertClassConfigPayload,
) -> Result<SuccessResponse, String> {
    let result: Result<SuccessResponse, AppError> = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        crate::db::repos::class_config::update(&db, id, &payload).await?;
        Ok(SuccessResponse { success: true })
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_class_config(app: AppHandle, id: i64) -> Result<SuccessResponse, String> {
    let result: Result<SuccessResponse, AppError> = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        crate::db::repos::class_config::delete(&db, id).await?;
        Ok(SuccessResponse { success: true })
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_grade_options(app: AppHandle) -> Result<Vec<String>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::class_config::ensure_seeded(&db).await?;
        crate::db::repos::class_config::list_grade_options(&db).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    async fn setup_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        crate::db::migration::Migrator::up(&db, None).await.unwrap();
        db
    }

    #[test]
    fn test_seed_idempotent() {
        tauri::async_runtime::block_on(async {
            let db = setup_db().await;
            crate::db::repos::class_config::ensure_seeded(&db)
                .await
                .unwrap();

            let teaching = crate::db::repos::class_config::list(
                &db,
                ListClassConfigsParams {
                    config_type: Some(ClassConfigType::TeachingClass),
                    grade_name: None,
                    keyword: None,
                },
            )
            .await
            .unwrap();
            let exam_rooms = crate::db::repos::class_config::list(
                &db,
                ListClassConfigsParams {
                    config_type: Some(ClassConfigType::ExamRoom),
                    grade_name: None,
                    keyword: None,
                },
            )
            .await
            .unwrap();
            assert_eq!(teaching.total, 15);
            assert_eq!(exam_rooms.total, 5);

            crate::db::repos::class_config::ensure_seeded(&db)
                .await
                .unwrap();
            let teaching_again = crate::db::repos::class_config::list(
                &db,
                ListClassConfigsParams {
                    config_type: Some(ClassConfigType::TeachingClass),
                    grade_name: None,
                    keyword: None,
                },
            )
            .await
            .unwrap();
            assert_eq!(teaching_again.total, 15);
        });
    }

    #[test]
    fn test_exam_room_reject_subjects() {
        let result = validate_payload(&UpsertClassConfigPayload {
            config_type: ClassConfigType::ExamRoom,
            grade_name: "高一".to_string(),
            class_name: "高一5场".to_string(),
            building: "教学楼C".to_string(),
            floor: "2层".to_string(),
            room_label: Some("C201".to_string()),
            subjects: Some(vec![Subject::Chinese]),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_update_and_delete_subject_relations() {
        tauri::async_runtime::block_on(async {
            let db = setup_db().await;
            let id = crate::db::repos::class_config::create(
                &db,
                &UpsertClassConfigPayload {
                    config_type: ClassConfigType::TeachingClass,
                    grade_name: "高三".to_string(),
                    class_name: "高三1班".to_string(),
                    building: "教学楼D".to_string(),
                    floor: "5层".to_string(),
                    room_label: None,
                    subjects: Some(vec![Subject::Chinese, Subject::Math]),
                },
            )
            .await
            .unwrap();

            crate::db::repos::class_config::update(
                &db,
                id,
                &UpsertClassConfigPayload {
                    config_type: ClassConfigType::TeachingClass,
                    grade_name: "高三".to_string(),
                    class_name: "高三1班".to_string(),
                    building: "教学楼D".to_string(),
                    floor: "6层".to_string(),
                    room_label: None,
                    subjects: Some(vec![Subject::Physics]),
                },
            )
            .await
            .unwrap();

            let detail = crate::db::repos::class_config::get_detail(&db, id)
                .await
                .unwrap();
            assert_eq!(detail.subjects, vec![Subject::Physics]);

            crate::db::repos::class_config::delete(&db, id)
                .await
                .unwrap();
            let deleted = crate::db::repos::class_config::get_detail(&db, id).await;
            assert!(deleted.is_err());
        });
    }
}
