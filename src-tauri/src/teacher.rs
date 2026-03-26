use std::collections::{HashMap, HashSet};

use calamine::{open_workbook_auto, Data, Reader};
use chrono::Utc;
use regex::Regex;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::app_log;
use crate::score::{self, AppError, ListResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TeacherSubject {
    Chinese,
    Math,
    English,
    Physics,
    Chemistry,
    Biology,
    Politics,
    History,
    Geography,
    Russian,
    Japanese,
    Sports,
    Music,
    General,
    Information,
    FineArts,
}

impl TeacherSubject {
    fn as_key(self) -> &'static str {
        match self {
            TeacherSubject::Chinese => "chinese",
            TeacherSubject::Math => "math",
            TeacherSubject::English => "english",
            TeacherSubject::Physics => "physics",
            TeacherSubject::Chemistry => "chemistry",
            TeacherSubject::Biology => "biology",
            TeacherSubject::Politics => "politics",
            TeacherSubject::History => "history",
            TeacherSubject::Geography => "geography",
            TeacherSubject::Russian => "russian",
            TeacherSubject::Japanese => "japanese",
            TeacherSubject::Sports => "sports",
            TeacherSubject::Music => "music",
            TeacherSubject::General => "general",
            TeacherSubject::Information => "information",
            TeacherSubject::FineArts => "fine_arts",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "chinese" => Some(TeacherSubject::Chinese),
            "math" => Some(TeacherSubject::Math),
            "english" => Some(TeacherSubject::English),
            "physics" => Some(TeacherSubject::Physics),
            "chemistry" => Some(TeacherSubject::Chemistry),
            "biology" => Some(TeacherSubject::Biology),
            "politics" => Some(TeacherSubject::Politics),
            "history" => Some(TeacherSubject::History),
            "geography" => Some(TeacherSubject::Geography),
            "russian" => Some(TeacherSubject::Russian),
            "japanese" => Some(TeacherSubject::Japanese),
            "sports" => Some(TeacherSubject::Sports),
            "music" => Some(TeacherSubject::Music),
            "general" => Some(TeacherSubject::General),
            "information" => Some(TeacherSubject::Information),
            "fine_arts" => Some(TeacherSubject::FineArts),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherImportResult {
    imported_at: String,
    row_count: i64,
    duration_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherSummary {
    imported_at: Option<String>,
    teacher_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherRow {
    id: i64,
    teacher_name: String,
    subjects: Vec<TeacherSubject>,
    class_names: Vec<String>,
    remark: Option<String>,
    is_middle_manager: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherListParams {
    pub name_keyword: Option<String>,
    pub class_name: Option<String>,
    pub subject: Option<TeacherSubject>,
}

#[derive(Debug)]
struct ParsedTeacherRow {
    teacher_name: String,
    subject: TeacherSubject,
    class_names: Vec<String>,
    remark: Option<String>,
}

#[derive(Debug, Default)]
struct AggregatedTeacher {
    teacher_name: String,
    assignments: HashSet<(TeacherSubject, String)>,
    remark: Option<String>,
    is_middle_manager: bool,
    homeroom_classes: HashSet<String>,
}

pub fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    crate::schema::ensure_schema(conn)?;
    Ok(())
}

fn cell_to_string(cell: Option<&Data>) -> String {
    match cell {
        Some(Data::String(s)) => s.trim().to_string(),
        Some(Data::Float(v)) => {
            if (v.fract().abs()) < 1e-9 {
                format!("{:.0}", v)
            } else {
                v.to_string()
            }
        }
        Some(Data::Int(v)) => v.to_string(),
        Some(Data::Bool(v)) => v.to_string(),
        Some(Data::DateTimeIso(s)) => s.trim().to_string(),
        Some(Data::DurationIso(s)) => s.trim().to_string(),
        Some(Data::Empty) | None => String::new(),
        Some(other) => other.to_string().trim().to_string(),
    }
}

fn parse_teacher_subject(text: &str) -> Option<TeacherSubject> {
    match text.trim() {
        "语文" => Some(TeacherSubject::Chinese),
        "数学" => Some(TeacherSubject::Math),
        "英语" => Some(TeacherSubject::English),
        "物理" => Some(TeacherSubject::Physics),
        "化学" => Some(TeacherSubject::Chemistry),
        "生物" => Some(TeacherSubject::Biology),
        "政治" => Some(TeacherSubject::Politics),
        "历史" => Some(TeacherSubject::History),
        "地理" => Some(TeacherSubject::Geography),
        "俄语" => Some(TeacherSubject::Russian),
        "日语" => Some(TeacherSubject::Japanese),
        "体育" => Some(TeacherSubject::Sports),
        "音乐" => Some(TeacherSubject::Music),
        "通用" => Some(TeacherSubject::General),
        "信息" => Some(TeacherSubject::Information),
        "美术" => Some(TeacherSubject::FineArts),
        _ => None,
    }
}

fn normalize_class_code(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let with_class =
        Regex::new(r"^([123]\d{2})班$").expect("class code with 班 regex should be valid");
    if let Some(caps) = with_class.captures(trimmed) {
        return normalize_class_code(&caps[1]);
    }
    if trimmed.contains('班') {
        return trimmed.to_string();
    }

    let pattern = Regex::new(r"^([123])(\d{2})$").expect("class code regex should be valid");
    if let Some(caps) = pattern.captures(trimmed) {
        let grade = match &caps[1] {
            "1" => "高一",
            "2" => "高二",
            "3" => "高三",
            _ => return trimmed.to_string(),
        };
        let class_num = caps[2].parse::<i32>().unwrap_or(0);
        if class_num > 0 {
            return format!("{grade}{class_num}班");
        }
    }
    trimmed.to_string()
}

fn class_sort_key(name: &str) -> (i32, i32, String) {
    let matcher = Regex::new(r"^高([一二三])(\d+)班$").expect("class sort regex should be valid");
    if let Some(caps) = matcher.captures(name) {
        let grade = match &caps[1] {
            "一" => 1,
            "二" => 2,
            "三" => 3,
            _ => 99,
        };
        let class_no = caps[2].parse::<i32>().unwrap_or(999);
        return (grade, class_no, name.to_string());
    }
    (99, 999, name.to_string())
}

fn parse_class_names(text: &str) -> Vec<String> {
    text.replace('，', ",")
        .replace('、', ",")
        .replace('；', ",")
        .replace(';', ",")
        .replace('/', ",")
        .replace(' ', ",")
        .split(',')
        .map(normalize_class_code)
        .filter(|v| !v.is_empty())
        .collect()
}

fn parse_homeroom_classes(remark: &str) -> Vec<String> {
    if !remark.contains("班主任") {
        return Vec::new();
    }
    let matcher = Regex::new(r"(高[一二三]\d+班|[123]\d{2}班|[123]\d{2})")
        .expect("homeroom class matcher regex should be valid");
    let mut set = HashSet::new();
    for caps in matcher.captures_iter(remark) {
        let token = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let normalized = normalize_class_code(token);
        if !normalized.is_empty() {
            set.insert(normalized);
        }
    }
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

fn is_middle_manager(remark: Option<&String>) -> bool {
    remark.is_some_and(|value| value.contains("中层"))
}

fn build_teacher_remark(
    homeroom_classes: &HashSet<String>,
    is_middle_manager: bool,
    fallback: Option<&String>,
) -> Option<String> {
    let mut parts = Vec::new();

    let mut classes = homeroom_classes.iter().cloned().collect::<Vec<_>>();
    classes.sort_by_key(|name| class_sort_key(name));
    for class_name in classes {
        parts.push(format!("{class_name}班主任"));
    }
    if is_middle_manager {
        parts.push("中层领导".to_string());
    }

    if !parts.is_empty() {
        return Some(parts.join("，"));
    }

    fallback
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Clone, Copy)]
struct TeacherHeaderIndexes {
    teacher_name: usize,
    subject: usize,
    class_name: usize,
    remark: Option<usize>,
}

fn normalize_header_name(text: &str) -> String {
    text.trim().replace(' ', "").replace('\n', "")
}

fn matches_header(header: &str, aliases: &[&str]) -> bool {
    aliases.iter().any(|alias| header == *alias)
}

fn detect_header_indexes(row: &[Data]) -> Result<TeacherHeaderIndexes, AppError> {
    let headers = row
        .iter()
        .enumerate()
        .map(|(idx, cell)| (idx, normalize_header_name(&cell_to_string(Some(cell)))))
        .collect::<Vec<_>>();

    let teacher_name = headers
        .iter()
        .find(|(_, text)| matches_header(text, &["教师姓名", "姓名", "老师姓名"]))
        .map(|(idx, _)| *idx)
        .ok_or_else(|| AppError::new("教师 Excel 缺少“教师姓名”列"))?;
    let subject = headers
        .iter()
        .find(|(_, text)| matches_header(text, &["任教学科", "教学科目", "科目", "学科"]))
        .map(|(idx, _)| *idx)
        .ok_or_else(|| AppError::new("教师 Excel 缺少“任教学科”列"))?;
    let class_name = headers
        .iter()
        .find(|(_, text)| matches_header(text, &["任教班级", "班级", "任课班级"]))
        .map(|(idx, _)| *idx)
        .ok_or_else(|| AppError::new("教师 Excel 缺少“任教班级”列"))?;
    let remark = headers
        .iter()
        .find(|(_, text)| matches_header(text, &["备注", "说明"]))
        .map(|(idx, _)| *idx);

    Ok(TeacherHeaderIndexes {
        teacher_name,
        subject,
        class_name,
        remark,
    })
}

fn parse_teacher_excel(file_path: &str) -> Result<Vec<ParsedTeacherRow>, AppError> {
    let mut workbook = open_workbook_auto(file_path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::new("Excel 文件未找到工作表"))?
        .map_err(AppError::from)?;

    let mut rows = range.rows();
    let first_row = rows
        .next()
        .ok_or_else(|| AppError::new("Excel 文件为空，缺少表头"))?;
    let (header_indexes, data_start_row_no) = match detect_header_indexes(first_row) {
        Ok(indexes) => (indexes, 2_usize),
        Err(_) => {
            let second_row = rows
                .next()
                .ok_or_else(|| AppError::new("教师 Excel 缺少表头行，请检查第二行是否为表头"))?;
            let indexes = detect_header_indexes(second_row)?;
            (indexes, 3_usize)
        }
    };

    let mut out = Vec::new();
    for (offset, row) in rows.enumerate() {
        let row_no = offset + data_start_row_no;
        let teacher_name = cell_to_string(row.get(header_indexes.teacher_name));
        let subject_text = cell_to_string(row.get(header_indexes.subject));
        let class_text = cell_to_string(row.get(header_indexes.class_name));
        let remark = header_indexes
            .remark
            .map(|idx| cell_to_string(row.get(idx)))
            .unwrap_or_default();

        if teacher_name.is_empty() && subject_text.is_empty() && class_text.is_empty() {
            continue;
        }
        if teacher_name.is_empty() {
            return Err(AppError::new(format!("第 {} 行教师姓名不能为空", row_no)));
        }
        let subject = parse_teacher_subject(&subject_text).ok_or_else(|| {
            AppError::new(format!("第 {} 行任教学科不合法: {}", row_no, subject_text))
        })?;
        let class_names = parse_class_names(&class_text);
        if class_names.is_empty() {
            return Err(AppError::new(format!("第 {} 行任教班级不能为空", row_no)));
        }
        out.push(ParsedTeacherRow {
            teacher_name,
            subject,
            class_names,
            remark: if remark.is_empty() {
                None
            } else {
                Some(remark)
            },
        });
    }
    if out.is_empty() {
        return Err(AppError::new("Excel 没有可导入的教师数据"));
    }
    Ok(out)
}

fn aggregate_rows(rows: Vec<ParsedTeacherRow>) -> Vec<AggregatedTeacher> {
    let mut map: HashMap<String, AggregatedTeacher> = HashMap::new();
    for row in rows {
        let row_is_middle_manager = is_middle_manager(row.remark.as_ref());
        let row_homerooms = row
            .remark
            .as_ref()
            .map(|v| parse_homeroom_classes(v))
            .unwrap_or_default();
        let entry = map
            .entry(row.teacher_name.clone())
            .or_insert_with(|| AggregatedTeacher {
                teacher_name: row.teacher_name.clone(),
                assignments: HashSet::new(),
                remark: row.remark.clone(),
                is_middle_manager: row_is_middle_manager,
                homeroom_classes: HashSet::new(),
            });

        if entry.remark.is_none() && row.remark.is_some() {
            entry.remark = row.remark.clone();
        }
        if row_is_middle_manager {
            entry.is_middle_manager = true;
        }
        for class_name in row_homerooms {
            entry.homeroom_classes.insert(class_name);
        }
        for class_name in row.class_names {
            entry.assignments.insert((row.subject, class_name));
        }

        entry.remark = build_teacher_remark(
            &entry.homeroom_classes,
            entry.is_middle_manager,
            entry.remark.as_ref().or(row.remark.as_ref()),
        );
    }
    let mut items: Vec<AggregatedTeacher> = map.into_values().collect();
    items.sort_by(|a, b| a.teacher_name.cmp(&b.teacher_name));
    items
}

fn persist_teachers(
    conn: &mut Connection,
    imported_at: &str,
    source_file: &str,
    teachers: &[AggregatedTeacher],
) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM latest_teacher_assignments_v2", [])?;
    tx.execute("DELETE FROM latest_teacher_homerooms_v2", [])?;
    tx.execute("DELETE FROM latest_teachers_v2", [])?;
    tx.execute("DELETE FROM latest_teacher_import_meta", [])?;
    tx.execute(
        "INSERT INTO latest_teacher_import_meta (id, imported_at, source_file, row_count) VALUES (1, ?1, ?2, ?3)",
        params![imported_at, source_file, teachers.len() as i64],
    )?;

    for teacher in teachers {
        tx.execute(
            "INSERT INTO latest_teachers_v2 (teacher_name, remark, is_middle_manager) VALUES (?1, ?2, ?3)",
            params![
                teacher.teacher_name,
                teacher.remark,
                if teacher.is_middle_manager { 1_i64 } else { 0_i64 }
            ],
        )?;
        let teacher_id = tx.last_insert_rowid();
        for (subject, class_name) in &teacher.assignments {
            tx.execute(
                "INSERT INTO latest_teacher_assignments_v2 (teacher_id, subject, class_name) VALUES (?1, ?2, ?3)",
                params![teacher_id, subject.as_key(), class_name],
            )?;
        }
        for class_name in &teacher.homeroom_classes {
            tx.execute(
                "INSERT INTO latest_teacher_homerooms_v2 (teacher_id, class_name) VALUES (?1, ?2)",
                params![teacher_id, class_name],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn import_teachers_from_excel(
    app: AppHandle,
    file_path: String,
) -> Result<TeacherImportResult, String> {
    let start = Utc::now();
    let result = (|| -> Result<TeacherImportResult, AppError> {
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let rows = parse_teacher_excel(&file_path)?;
        let teachers = aggregate_rows(rows);
        let imported_at = Utc::now().to_rfc3339();
        persist_teachers(&mut conn, &imported_at, &file_path, &teachers)?;
        Ok(TeacherImportResult {
            imported_at,
            row_count: teachers.len() as i64,
            duration_ms: (Utc::now() - start).num_milliseconds(),
        })
    })();
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "teacher.import_teachers_from_excel",
            &format!("file_path={file_path} | {e}"),
        );
        e.to_string()
    })
}

#[tauri::command]
pub fn list_latest_teachers(
    app: AppHandle,
    params: TeacherListParams,
) -> Result<ListResult<TeacherRow>, String> {
    let result = (|| -> Result<ListResult<TeacherRow>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;

        let mut where_sql = String::from(" WHERE 1=1 ");
        let mut values: Vec<Value> = Vec::new();

        if let Some(name_keyword) = params
            .name_keyword
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            where_sql.push_str(" AND t.teacher_name LIKE ? ");
            values.push(Value::Text(format!("%{name_keyword}%")));
        }
        if let Some(class_name) = params
            .class_name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            where_sql.push_str(" AND EXISTS (SELECT 1 FROM latest_teacher_assignments_v2 ta WHERE ta.teacher_id = t.id AND ta.class_name LIKE ?) ");
            values.push(Value::Text(format!("%{class_name}%")));
        }
        if let Some(subject) = params.subject {
            where_sql.push_str(" AND EXISTS (SELECT 1 FROM latest_teacher_assignments_v2 ta WHERE ta.teacher_id = t.id AND ta.subject = ?) ");
            values.push(Value::Text(subject.as_key().to_string()));
        }

        let total_sql = format!("SELECT COUNT(*) FROM latest_teachers_v2 t {where_sql}");
        let total: i64 = conn.query_row(&total_sql, params_from_iter(values.iter()), |row| {
            row.get(0)
        })?;

        let list_sql = format!("SELECT t.id, t.teacher_name, t.remark, COALESCE(t.is_middle_manager, 0) FROM latest_teachers_v2 t {where_sql} ORDER BY t.id ASC");
        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(values.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)? == 1,
            ))
        })?;

        let mut items = Vec::new();
        for row in rows {
            let (id, teacher_name, remark, is_middle_manager) = row?;

            let mut assignment_stmt = conn.prepare(
                "SELECT subject, class_name FROM latest_teacher_assignments_v2 WHERE teacher_id = ?1 ORDER BY id ASC",
            )?;
            let assignment_rows = assignment_stmt.query_map(params![id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;

            let mut subjects = Vec::new();
            let mut subject_keys = HashSet::new();
            let mut class_names = Vec::new();
            let mut class_keys = HashSet::new();

            for assignment in assignment_rows {
                let (subject_key, class_name) = assignment?;
                if let Some(subject) = TeacherSubject::from_key(&subject_key) {
                    if subject_keys.insert(subject.as_key()) {
                        subjects.push(subject);
                    }
                }
                if class_keys.insert(class_name.clone()) {
                    class_names.push(class_name);
                }
            }

            items.push(TeacherRow {
                id,
                teacher_name,
                subjects,
                class_names,
                remark,
                is_middle_manager,
            });
        }

        Ok(ListResult { items, total })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_latest_teacher_summary(app: AppHandle) -> Result<TeacherSummary, String> {
    let result = (|| -> Result<TeacherSummary, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let imported_at = conn
            .query_row(
                "SELECT imported_at FROM latest_teacher_import_meta WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok();
        let teacher_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM latest_teachers_v2", [], |row| {
                row.get(0)
            })?;
        Ok(TeacherSummary {
            imported_at,
            teacher_count,
        })
    })();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_class_code() {
        assert_eq!(normalize_class_code("202"), "高二2班");
        assert_eq!(normalize_class_code("202班"), "高二2班");
        assert_eq!(normalize_class_code("210"), "高二10班");
        assert_eq!(normalize_class_code("高二3班"), "高二3班");
    }

    #[test]
    fn test_parse_class_names() {
        let classes = parse_class_names("202/210，211");
        assert_eq!(classes, vec!["高二2班", "高二10班", "高二11班"]);
    }

    #[test]
    fn test_subject_extended() {
        assert_eq!(parse_teacher_subject("体育"), Some(TeacherSubject::Sports));
        assert_eq!(parse_teacher_subject("音乐"), Some(TeacherSubject::Music));
        assert_eq!(
            parse_teacher_subject("信息"),
            Some(TeacherSubject::Information)
        );
        assert_eq!(parse_teacher_subject("通用"), Some(TeacherSubject::General));
        assert_eq!(
            parse_teacher_subject("美术"),
            Some(TeacherSubject::FineArts)
        );
    }

    #[test]
    fn test_parse_homeroom_classes_and_middle_manager() {
        let classes = parse_homeroom_classes("202班班主任，中层领导");
        assert_eq!(classes, vec!["高二2班"]);
        assert!(is_middle_manager(Some(
            &"202班班主任，中层领导".to_string()
        )));
    }

    #[test]
    fn test_parse_homeroom_classes_with_slash_and_middle_manager() {
        let classes = parse_homeroom_classes("中层领导/202班主任");
        assert_eq!(classes, vec!["高二2班"]);
        assert!(is_middle_manager(Some(&"中层领导/202班主任".to_string())));
    }

    #[test]
    fn test_build_teacher_remark_normalized() {
        let classes = HashSet::from(["高一2班".to_string(), "高一10班".to_string()]);
        let remark = build_teacher_remark(&classes, true, Some(&"中层领导/102班主任".to_string()));
        assert_eq!(
            remark,
            Some("高一2班班主任，高一10班班主任，中层领导".to_string())
        );
    }
}
