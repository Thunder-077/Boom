use std::collections::HashSet;

use chrono::Utc;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, Transaction};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::score::{self, AppError, ListResult, Subject};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClassConfigType {
    TeachingClass,
    ExamRoom,
}

impl ClassConfigType {
    fn as_key(self) -> &'static str {
        match self {
            ClassConfigType::TeachingClass => "teaching_class",
            ClassConfigType::ExamRoom => "exam_room",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
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
    id: i64,
    config_type: ClassConfigType,
    grade_name: String,
    class_name: String,
    building: String,
    floor: String,
    room_label: Option<String>,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfigDetail {
    id: i64,
    config_type: ClassConfigType,
    grade_name: String,
    class_name: String,
    building: String,
    floor: String,
    room_label: Option<String>,
    subjects: Vec<Subject>,
    updated_at: String,
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
    id: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    success: bool,
}

pub fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    crate::schema::ensure_schema(conn)?;
    seed_default_class_configs(conn)?;
    Ok(())
}

fn seed_default_class_configs(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM class_configs", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    let tx = conn.unchecked_transaction()?;

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
        insert_class_config_tx(
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
        )?;
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
        insert_class_config_tx(
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
        )?;
    }

    let exam_rooms = [
        ("高一", "高一5场", "向远楼", "5层"),
        ("高一", "高一6场", "向远楼", "5层"),
        ("高二", "高二12场", "向远楼", "2层"),
        ("高二", "高二13场", "向远楼", "2层"),
        ("高二", "高二14场", "向远楼", "4层"),
    ];

    for (grade_name, class_name, building, floor) in exam_rooms {
        insert_class_config_tx(
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
        )?;
    }

    tx.commit()?;
    Ok(())
}

fn validate_payload(payload: &UpsertClassConfigPayload) -> Result<(), AppError> {
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

fn dedup_subjects(input: Vec<Subject>) -> Vec<Subject> {
    let mut keys = HashSet::new();
    let mut out = Vec::new();
    for subject in input {
        if keys.insert(subject.as_key()) {
            out.push(subject);
        }
    }
    out
}

fn insert_class_config_tx(
    tx: &Transaction<'_>,
    payload: &UpsertClassConfigPayload,
    now: &str,
) -> Result<i64, AppError> {
    validate_payload(payload)?;
    tx.execute(
        r#"
        INSERT INTO class_configs (config_type, grade_name, class_name, building, floor, room_label, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            payload.config_type.as_key(),
            payload.grade_name.trim(),
            payload.class_name.trim(),
            payload.building.trim(),
            payload.floor.trim(),
            payload.room_label.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()),
            now,
            now
        ],
    )?;
    let id = tx.last_insert_rowid();

    let subjects = dedup_subjects(payload.subjects.clone().unwrap_or_default());
    for subject in subjects {
        tx.execute(
            "INSERT INTO class_config_subjects (config_id, subject) VALUES (?1, ?2)",
            params![id, subject.as_key()],
        )?;
    }
    Ok(id)
}

fn update_class_config_tx(
    tx: &Transaction<'_>,
    id: i64,
    payload: &UpsertClassConfigPayload,
    now: &str,
) -> Result<(), AppError> {
    validate_payload(payload)?;
    let affected = tx.execute(
        r#"
        UPDATE class_configs
        SET config_type = ?1, grade_name = ?2, class_name = ?3, building = ?4, floor = ?5, room_label = ?6, updated_at = ?7
        WHERE id = ?8
        "#,
        params![
            payload.config_type.as_key(),
            payload.grade_name.trim(),
            payload.class_name.trim(),
            payload.building.trim(),
            payload.floor.trim(),
            payload.room_label.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()),
            now,
            id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::new("配置不存在"));
    }
    tx.execute(
        "DELETE FROM class_config_subjects WHERE config_id = ?1",
        params![id],
    )?;
    for subject in dedup_subjects(payload.subjects.clone().unwrap_or_default()) {
        tx.execute(
            "INSERT INTO class_config_subjects (config_id, subject) VALUES (?1, ?2)",
            params![id, subject.as_key()],
        )?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_class_configs(
    app: AppHandle,
    params: ListClassConfigsParams,
) -> Result<ListResult<ClassConfigRow>, String> {
    let result = (|| -> Result<ListResult<ClassConfigRow>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;

        let config_type = params.config_type.unwrap_or(ClassConfigType::TeachingClass);
        let mut where_parts = vec!["config_type = ?".to_string()];
        let mut values = vec![Value::Text(config_type.as_key().to_string())];

        if config_type == ClassConfigType::TeachingClass {
            if let Some(grade_name) = params
                .grade_name
                .as_ref()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                where_parts.push("grade_name = ?".to_string());
                values.push(Value::Text(grade_name.to_string()));
            }
        }
        if let Some(keyword) = params
            .keyword
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            where_parts.push("class_name LIKE ?".to_string());
            values.push(Value::Text(format!("%{keyword}%")));
        }

        let where_sql = format!(" WHERE {}", where_parts.join(" AND "));
        let count_sql = format!("SELECT COUNT(*) FROM class_configs{where_sql}");
        let total: i64 = conn.query_row(&count_sql, params_from_iter(values.iter()), |row| {
            row.get(0)
        })?;

        let list_sql = format!(
            "SELECT id, config_type, grade_name, class_name, building, floor, room_label, updated_at FROM class_configs{where_sql} ORDER BY grade_name ASC, class_name ASC, id ASC"
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(values.iter()), |row| {
            let type_key: String = row.get(1)?;
            let config_type = ClassConfigType::from_key(&type_key).ok_or_else(|| {
                rusqlite::Error::InvalidColumnType(
                    1,
                    "config_type".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;
            Ok(ClassConfigRow {
                id: row.get(0)?,
                config_type,
                grade_name: row.get(2)?,
                class_name: row.get(3)?,
                building: row.get(4)?,
                floor: row.get(5)?,
                room_label: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(ListResult { items, total })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_class_config_detail(app: AppHandle, id: i64) -> Result<ClassConfigDetail, String> {
    let result = (|| -> Result<ClassConfigDetail, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;

        let mut stmt = conn.prepare(
            "SELECT id, config_type, grade_name, class_name, building, floor, room_label, updated_at FROM class_configs WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        let row = rows.next()?.ok_or_else(|| AppError::new("配置不存在"))?;
        let type_key: String = row.get(1)?;
        let config_type =
            ClassConfigType::from_key(&type_key).ok_or_else(|| AppError::new("配置类型错误"))?;

        let mut subject_stmt = conn.prepare(
            "SELECT subject FROM class_config_subjects WHERE config_id = ?1 ORDER BY id ASC",
        )?;
        let subject_rows = subject_stmt.query_map(params![id], |srow| srow.get::<_, String>(0))?;
        let mut subjects = Vec::new();
        for key in subject_rows {
            if let Some(subject) = Subject::from_key(&key?) {
                subjects.push(subject);
            }
        }

        Ok(ClassConfigDetail {
            id: row.get(0)?,
            config_type,
            grade_name: row.get(2)?,
            class_name: row.get(3)?,
            building: row.get(4)?,
            floor: row.get(5)?,
            room_label: row.get(6)?,
            subjects,
            updated_at: row.get(7)?,
        })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_class_config(
    app: AppHandle,
    payload: UpsertClassConfigPayload,
) -> Result<CreateClassConfigResult, String> {
    let result = (|| -> Result<CreateClassConfigResult, AppError> {
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        let id = insert_class_config_tx(&tx, &payload, &now)?;
        tx.commit()?;
        Ok(CreateClassConfigResult { id })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_class_config(
    app: AppHandle,
    id: i64,
    payload: UpsertClassConfigPayload,
) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        update_class_config_tx(&tx, id, &payload, &now)?;
        tx.commit()?;
        Ok(SuccessResponse { success: true })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_class_config(app: AppHandle, id: i64) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM class_config_subjects WHERE config_id = ?1",
            params![id],
        )?;
        let affected = tx.execute("DELETE FROM class_configs WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::new("配置不存在"));
        }
        tx.commit()?;
        Ok(SuccessResponse { success: true })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_grade_options(app: AppHandle) -> Result<Vec<String>, String> {
    let result = (|| -> Result<Vec<String>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT grade_name FROM class_configs WHERE config_type = 'teaching_class' ORDER BY grade_name ASC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_seed_idempotent() {
        let conn = setup_conn();
        let teaching_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_configs WHERE config_type = 'teaching_class'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let exam_room_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_configs WHERE config_type = 'exam_room'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(teaching_count, 15);
        assert_eq!(exam_room_count, 5);
        seed_default_class_configs(&conn).unwrap();
        let teaching_count2: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_configs WHERE config_type = 'teaching_class'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let exam_room_count2: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_configs WHERE config_type = 'exam_room'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(teaching_count2, 15);
        assert_eq!(exam_room_count2, 5);
    }

    #[test]
    fn test_exam_room_reject_subjects() {
        let conn = setup_conn();
        let tx = conn.unchecked_transaction().unwrap();
        let now = Utc::now().to_rfc3339();
        let result = insert_class_config_tx(
            &tx,
            &UpsertClassConfigPayload {
                config_type: ClassConfigType::ExamRoom,
                grade_name: "高一".to_string(),
                class_name: "高一5场".to_string(),
                building: "教学楼C".to_string(),
                floor: "2层".to_string(),
                room_label: Some("C201".to_string()),
                subjects: Some(vec![Subject::Chinese]),
            },
            &now,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_and_delete_subject_relations() {
        let conn = setup_conn();
        let tx = conn.unchecked_transaction().unwrap();
        let now = Utc::now().to_rfc3339();
        let id = insert_class_config_tx(
            &tx,
            &UpsertClassConfigPayload {
                config_type: ClassConfigType::TeachingClass,
                grade_name: "高三".to_string(),
                class_name: "高三1班".to_string(),
                building: "教学楼D".to_string(),
                floor: "5层".to_string(),
                room_label: None,
                subjects: Some(vec![Subject::Chinese, Subject::Math]),
            },
            &now,
        )
        .unwrap();
        tx.commit().unwrap();

        let tx2 = conn.unchecked_transaction().unwrap();
        update_class_config_tx(
            &tx2,
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
            &Utc::now().to_rfc3339(),
        )
        .unwrap();
        tx2.commit().unwrap();

        let subject_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_config_subjects WHERE config_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(subject_count, 1);

        let tx3 = conn.unchecked_transaction().unwrap();
        tx3.execute(
            "DELETE FROM class_config_subjects WHERE config_id = ?1",
            params![id],
        )
        .unwrap();
        tx3.execute("DELETE FROM class_configs WHERE id = ?1", params![id])
            .unwrap();
        tx3.commit().unwrap();

        let row_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM class_configs WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(row_count, 0);
    }
}
