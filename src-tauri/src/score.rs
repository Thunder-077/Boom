use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::path::PathBuf;

use calamine::{open_workbook_auto, Data, Reader};
use chrono::Utc;
use regex::Regex;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, Transaction};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app_log;

const FIXED_HEADERS: [&str; 16] = [
    "准考证号",
    "班级",
    "姓名",
    "选科组合",
    "语种",
    "语文",
    "数学",
    "英语",
    "物理",
    "化学",
    "生物",
    "政治",
    "历史",
    "地理",
    "俄语",
    "日语",
];

const SUBJECT_COLUMNS: [(usize, Subject, &str); 11] = [
    (5, Subject::Chinese, "语文"),
    (6, Subject::Math, "数学"),
    (7, Subject::English, "英语"),
    (8, Subject::Physics, "物理"),
    (9, Subject::Chemistry, "化学"),
    (10, Subject::Biology, "生物"),
    (11, Subject::Politics, "政治"),
    (12, Subject::History, "历史"),
    (13, Subject::Geography, "地理"),
    (14, Subject::Russian, "俄语"),
    (15, Subject::Japanese, "日语"),
];

#[derive(Debug)]
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::new(format!("数据库操作失败: {value}"))
    }
}

impl From<calamine::Error> for AppError {
    fn from(value: calamine::Error) -> Self {
        Self::new(format!("Excel 解析失败: {value}"))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
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
}

impl Subject {
    pub fn as_key(self) -> &'static str {
        match self {
            Subject::Chinese => "chinese",
            Subject::Math => "math",
            Subject::English => "english",
            Subject::Physics => "physics",
            Subject::Chemistry => "chemistry",
            Subject::Biology => "biology",
            Subject::Politics => "politics",
            Subject::History => "history",
            Subject::Geography => "geography",
            Subject::Russian => "russian",
            Subject::Japanese => "japanese",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "chinese" => Some(Subject::Chinese),
            "math" => Some(Subject::Math),
            "english" => Some(Subject::English),
            "physics" => Some(Subject::Physics),
            "chemistry" => Some(Subject::Chemistry),
            "biology" => Some(Subject::Biology),
            "politics" => Some(Subject::Politics),
            "history" => Some(Subject::History),
            "geography" => Some(Subject::Geography),
            "russian" => Some(Subject::Russian),
            "japanese" => Some(Subject::Japanese),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreCellState {
    Scored,
    NotSelected,
    Absent,
}

#[derive(Debug, Clone)]
struct ParsedSubjectScore {
    subject: Subject,
    score: Option<f64>,
    state: ScoreCellState,
}

#[derive(Debug, Clone)]
struct ParsedStudent {
    admission_no: String,
    class_name: String,
    grade_name: String,
    student_name: String,
    subject_combination: String,
    language: String,
    total_score: f64,
    selected_subject_count: i64,
    class_rank: i64,
    grade_rank: i64,
    subjects: Vec<ParsedSubjectScore>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    imported_at: String,
    row_count: i64,
    warning_count: i64,
    duration_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRow {
    admission_no: String,
    class_name: String,
    grade_name: String,
    student_name: String,
    subject_combination: String,
    language: String,
    total_score: f64,
    class_rank: i64,
    grade_rank: i64,
    selected_subject_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestSummary {
    imported_at: Option<String>,
    student_count: i64,
    class_count: i64,
    grade_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreListParams {
    pub name_keyword: Option<String>,
    pub class_name: Option<String>,
    pub grade_name: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreSubjectItem {
    subject: Subject,
    score: Option<f64>,
    state: ScoreCellState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDetail {
    admission_no: String,
    class_name: String,
    grade_name: String,
    student_name: String,
    subject_combination: String,
    language: String,
    total_score: f64,
    class_rank: i64,
    grade_rank: i64,
    selected_subject_count: i64,
    subjects: Vec<ScoreSubjectItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScorePayload {
    admission_no: String,
    class_name: String,
    student_name: String,
    subjects: Vec<ScoreSubjectItem>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    success: bool,
}

pub fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取应用数据目录失败: {e}")))?;
    create_dir_all(&dir).map_err(|e| AppError::new(format!("创建应用数据目录失败: {e}")))?;
    dir.push("scores.sqlite3");
    Ok(dir)
}

pub fn open_connection(app: &AppHandle) -> Result<Connection, AppError> {
    let path = db_path(app)?;
    Connection::open(path).map_err(AppError::from)
}

pub fn init_schema(conn: &Connection) -> Result<(), AppError> {
    crate::schema::ensure_schema(conn)
}

fn cell_to_trimmed_string(cell: Option<&Data>) -> String {
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

fn parse_subject_combination(text: &str, row_index: usize) -> Result<HashSet<Subject>, AppError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(format!("第 {} 行选科组合为空", row_index + 1)));
    }
    let mut subjects: HashSet<Subject> = HashSet::new();
    // 所有学生默认选语文和数学
    subjects.insert(Subject::Chinese);
    subjects.insert(Subject::Math);

    if trimmed == "全科" {
        subjects.insert(Subject::Physics);
        subjects.insert(Subject::Chemistry);
        subjects.insert(Subject::Biology);
        subjects.insert(Subject::Politics);
        subjects.insert(Subject::History);
        subjects.insert(Subject::Geography);
        return Ok(subjects);
    }

    for ch in trimmed.chars() {
        let subject = match ch {
            '物' => Subject::Physics,
            '化' => Subject::Chemistry,
            '生' => Subject::Biology,
            '政' => Subject::Politics,
            '史' => Subject::History,
            '地' => Subject::Geography,
            _ => {
                return Err(AppError::new(format!(
                    "第 {} 行选科组合包含无法识别的字符: '{}'",
                    row_index + 1,
                    ch
                )));
            }
        };
        subjects.insert(subject);
    }
    Ok(subjects)
}

fn parse_language(text: &str, row_index: usize) -> Result<Subject, AppError> {
    let trimmed = text.trim();
    match trimmed {
        "英语" => Ok(Subject::English),
        "俄语" => Ok(Subject::Russian),
        "日语" => Ok(Subject::Japanese),
        _ => Err(AppError::new(format!(
            "第 {} 行语种无法识别: '{}'",
            row_index + 1,
            trimmed
        ))),
    }
}

fn parse_score_cell(cell: Option<&Data>, row_index: usize, subject_header: &str, is_selected: bool) -> Result<ParsedSubjectScoreState, AppError> {
    let text = cell_to_trimmed_string(cell);

    if is_selected {
        // 已选科目
        if text.is_empty() {
            return Err(AppError::new(format!(
                "第 {} 行选了{}但成绩为空",
                row_index + 1,
                subject_header
            )));
        }
        if text == "-" {
            return Ok(ParsedSubjectScoreState {
                score: Some(0.0),
                state: ScoreCellState::Absent,
                selected: true,
            });
        }
        let parsed = text.parse::<f64>().map_err(|_| {
            AppError::new(format!(
                "第 {} 行科目 {} 成绩格式错误: {}",
                row_index + 1,
                subject_header,
                text
            ))
        })?;
        Ok(ParsedSubjectScoreState {
            score: Some(parsed),
            state: ScoreCellState::Scored,
            selected: true,
        })
    } else {
        // 未选科目
        if text.is_empty() || text == "-" {
            return Ok(ParsedSubjectScoreState {
                score: None,
                state: ScoreCellState::NotSelected,
                selected: false,
            });
        }
        // 未选但有成绩 → 报错
        Err(AppError::new(format!(
            "第 {} 行未选{}但有成绩: {}",
            row_index + 1,
            subject_header,
            text
        )))
    }
}

struct ParsedSubjectScoreState {
    score: Option<f64>,
    state: ScoreCellState,
    selected: bool,
}

fn extract_grade_name(class_name: &str) -> String {
    let matcher = Regex::new(r"高[一二三]").expect("regex for grade should be valid");
    if let Some(m) = matcher.find(class_name) {
        return m.as_str().to_string();
    }
    "未知年级".to_string()
}

fn validate_header(header_row: &[Data]) -> Result<(), AppError> {
    let parsed_headers: Vec<String> = header_row
        .iter()
        .take(FIXED_HEADERS.len())
        .map(|c| cell_to_trimmed_string(Some(c)))
        .collect();
    if parsed_headers.len() != FIXED_HEADERS.len() {
        return Err(AppError::new("Excel 表头列数不正确"));
    }
    for (index, expected) in FIXED_HEADERS.iter().enumerate() {
        if parsed_headers[index] != *expected {
            return Err(AppError::new(format!(
                "Excel 表头不匹配: 第 {} 列应为 '{}'，实际为 '{}'",
                index + 1,
                expected,
                parsed_headers[index]
            )));
        }
    }
    Ok(())
}

fn parse_excel_rows(file_path: &str) -> Result<Vec<ParsedStudent>, AppError> {
    let mut workbook = open_workbook_auto(file_path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::new("Excel 文件未找到工作表"))?
        .map_err(AppError::from)?;
    let mut rows_iter = range.rows();
    let header_row = rows_iter.next().ok_or_else(|| AppError::new("Excel 文件为空，缺少表头"))?;
    validate_header(header_row)?;

    let mut students = Vec::new();
    for (offset, row) in rows_iter.enumerate() {
        let excel_row_index = offset + 1;
        let admission_no = cell_to_trimmed_string(row.get(0));
        let class_name = cell_to_trimmed_string(row.get(1));
        let student_name = cell_to_trimmed_string(row.get(2));
        if admission_no.is_empty() && class_name.is_empty() && student_name.is_empty() {
            continue;
        }
        if admission_no.is_empty() || class_name.is_empty() || student_name.is_empty() {
            return Err(AppError::new(format!(
                "第 {} 行缺少必填字段（准考证号/班级/姓名）",
                excel_row_index + 1
            )));
        }

        let combination_text = cell_to_trimmed_string(row.get(3));
        let language_text = cell_to_trimmed_string(row.get(4));

        let mut selected_subjects = parse_subject_combination(&combination_text, excel_row_index)?;
        let lang_subject = parse_language(&language_text, excel_row_index)?;
        selected_subjects.insert(lang_subject);

        let mut subjects = Vec::new();
        let mut total_score = 0.0;
        let mut selected_subject_count = 0_i64;
        for (column_index, subject, header_name) in SUBJECT_COLUMNS {
            let is_selected = selected_subjects.contains(&subject);
            let parsed = parse_score_cell(row.get(column_index), excel_row_index, header_name, is_selected)?;
            if parsed.selected {
                selected_subject_count += 1;
                total_score += parsed.score.unwrap_or(0.0);
            }
            subjects.push(ParsedSubjectScore {
                subject,
                score: parsed.score,
                state: parsed.state,
            });
        }

        students.push(ParsedStudent {
            admission_no,
            class_name: class_name.clone(),
            grade_name: extract_grade_name(&class_name),
            student_name,
            subject_combination: combination_text,
            language: language_text,
            total_score,
            selected_subject_count,
            class_rank: 0,
            grade_rank: 0,
            subjects,
        });
    }
    if students.is_empty() {
        return Err(AppError::new("Excel 没有可导入的数据行"));
    }
    Ok(students)
}

fn assign_competition_rank(students: &mut [ParsedStudent], groups: HashMap<String, Vec<usize>>, is_class: bool) {
    for (_, mut indexes) in groups {
        indexes.sort_by(|a, b| {
            students[*b]
                .total_score
                .partial_cmp(&students[*a].total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(students[*a].admission_no.cmp(&students[*b].admission_no))
        });
        let mut current_rank = 1_i64;
        let mut previous_score: Option<f64> = None;
        for (position, index) in indexes.iter().enumerate() {
            let total = students[*index].total_score;
            if let Some(prev) = previous_score {
                if (prev - total).abs() > 1e-9 {
                    current_rank = (position + 1) as i64;
                }
            }
            previous_score = Some(total);
            if is_class {
                students[*index].class_rank = current_rank;
            } else {
                students[*index].grade_rank = current_rank;
            }
        }
    }
}

fn apply_ranks(students: &mut [ParsedStudent]) {
    let mut class_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut grade_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, student) in students.iter().enumerate() {
        class_groups.entry(student.class_name.clone()).or_default().push(idx);
        grade_groups.entry(student.grade_name.clone()).or_default().push(idx);
    }
    assign_competition_rank(students, class_groups, true);
    assign_competition_rank(students, grade_groups, false);
}

#[derive(Debug, Clone)]
struct RankRow {
    admission_no: String,
    class_name: String,
    grade_name: String,
    total_score: f64,
    class_rank: i64,
    grade_rank: i64,
}

fn assign_rank_rows(rows: &mut [RankRow]) {
    let mut class_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut grade_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        class_groups.entry(row.class_name.clone()).or_default().push(idx);
        grade_groups.entry(row.grade_name.clone()).or_default().push(idx);
    }
    for (_, mut indexes) in class_groups {
        indexes.sort_by(|a, b| {
            rows[*b]
                .total_score
                .partial_cmp(&rows[*a].total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(rows[*a].admission_no.cmp(&rows[*b].admission_no))
        });
        let mut current_rank = 1_i64;
        let mut previous_score: Option<f64> = None;
        for (position, index) in indexes.iter().enumerate() {
            let total = rows[*index].total_score;
            if let Some(prev) = previous_score {
                if (prev - total).abs() > 1e-9 {
                    current_rank = (position + 1) as i64;
                }
            }
            previous_score = Some(total);
            rows[*index].class_rank = current_rank;
        }
    }
    for (_, mut indexes) in grade_groups {
        indexes.sort_by(|a, b| {
            rows[*b]
                .total_score
                .partial_cmp(&rows[*a].total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(rows[*a].admission_no.cmp(&rows[*b].admission_no))
        });
        let mut current_rank = 1_i64;
        let mut previous_score: Option<f64> = None;
        for (position, index) in indexes.iter().enumerate() {
            let total = rows[*index].total_score;
            if let Some(prev) = previous_score {
                if (prev - total).abs() > 1e-9 {
                    current_rank = (position + 1) as i64;
                }
            }
            previous_score = Some(total);
            rows[*index].grade_rank = current_rank;
        }
    }
}

fn recompute_ranks_tx(tx: &Transaction<'_>) -> Result<(), AppError> {
    let mut stmt = tx.prepare(
        "SELECT admission_no, class_name, grade_name, total_score FROM latest_student_scores ORDER BY admission_no ASC",
    )?;
    let rows_iter = stmt.query_map([], |row| {
        Ok(RankRow {
            admission_no: row.get(0)?,
            class_name: row.get(1)?,
            grade_name: row.get(2)?,
            total_score: row.get(3)?,
            class_rank: 0,
            grade_rank: 0,
        })
    })?;
    let mut rows = Vec::new();
    for row in rows_iter {
        rows.push(row?);
    }
    assign_rank_rows(&mut rows);
    for row in rows {
        tx.execute(
            "UPDATE latest_student_scores SET class_rank = ?1, grade_rank = ?2 WHERE admission_no = ?3",
            params![row.class_rank, row.grade_rank, row.admission_no],
        )?;
    }
    Ok(())
}

fn persist_latest_snapshot(conn: &mut Connection, source_file: &str, imported_at: &str, students: &[ParsedStudent]) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM latest_subject_scores", [])?;
    tx.execute("DELETE FROM latest_student_scores", [])?;
    tx.execute("DELETE FROM latest_import_meta", [])?;
    tx.execute(
        "INSERT INTO latest_import_meta (id, imported_at, source_file, row_count) VALUES (1, ?1, ?2, ?3)",
        params![imported_at, source_file, students.len() as i64],
    )?;

    for student in students {
        tx.execute(
            r#"
            INSERT INTO latest_student_scores (
              admission_no, class_name, grade_name, student_name,
              subject_combination, language,
              total_score, class_rank, grade_rank, selected_subject_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                student.admission_no,
                student.class_name,
                student.grade_name,
                student.student_name,
                student.subject_combination,
                student.language,
                student.total_score,
                student.class_rank,
                student.grade_rank,
                student.selected_subject_count
            ],
        )?;

        for subject in &student.subjects {
            tx.execute(
                "INSERT INTO latest_subject_scores (admission_no, subject, score, is_selected, is_absent) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    student.admission_no,
                    subject.subject.as_key(),
                    subject.score,
                    matches!(subject.state, ScoreCellState::Scored | ScoreCellState::Absent),
                    matches!(subject.state, ScoreCellState::Absent)
                ],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn import_scores_from_excel(app: AppHandle, file_path: String) -> Result<ImportResult, String> {
    let start = Utc::now();
    let result = (|| -> Result<ImportResult, AppError> {
        let mut conn = open_connection(&app)?;
        init_schema(&conn)?;
        let mut students = parse_excel_rows(&file_path)?;
        apply_ranks(&mut students);
        let imported_at = Utc::now().to_rfc3339();
        persist_latest_snapshot(&mut conn, &file_path, &imported_at, &students)?;
        Ok(ImportResult {
            imported_at,
            row_count: students.len() as i64,
            warning_count: 0,
            duration_ms: (Utc::now() - start).num_milliseconds(),
        })
    })();
    result.map_err(|e| {
        app_log::log_error(&app, "score.import_scores_from_excel", &format!("file_path={file_path} | {e}"));
        e.to_string()
    })
}

#[tauri::command]
pub fn list_latest_score_rows(app: AppHandle, params: ScoreListParams) -> Result<ListResult<ScoreRow>, String> {
    let result = (|| -> Result<ListResult<ScoreRow>, AppError> {
        let conn = open_connection(&app)?;
        init_schema(&conn)?;
        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<Value> = Vec::new();

        if let Some(keyword) = params.name_keyword.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
            where_clauses.push("student_name LIKE ?".to_string());
            bind_values.push(Value::Text(format!("%{keyword}%")));
        }
        if let Some(class_name) = params.class_name.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
            where_clauses.push("class_name LIKE ?".to_string());
            bind_values.push(Value::Text(format!("%{class_name}%")));
        }
        if let Some(grade_name) = params.grade_name.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
            where_clauses.push("grade_name = ?".to_string());
            bind_values.push(Value::Text(grade_name.to_string()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };
        let total_sql = format!("SELECT COUNT(*) FROM latest_student_scores{where_sql}");
        let total: i64 = conn.query_row(&total_sql, params_from_iter(bind_values.iter()), |row| row.get(0))?;

        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(50).clamp(1, 500);
        let offset = (page - 1) * page_size;

        let mut list_bind_values = bind_values;
        list_bind_values.push(Value::Integer(page_size));
        list_bind_values.push(Value::Integer(offset));

        let list_sql = format!(
            r#"
            SELECT admission_no, class_name, grade_name, student_name, subject_combination, language, total_score, class_rank, grade_rank, selected_subject_count
            FROM latest_student_scores
            {where_sql}
            ORDER BY grade_name ASC, class_name ASC, class_rank ASC, admission_no ASC
            LIMIT ? OFFSET ?
            "#
        );

        let mut stmt = conn.prepare(&list_sql)?;
        let rows = stmt.query_map(params_from_iter(list_bind_values.iter()), |row| {
            Ok(ScoreRow {
                admission_no: row.get(0)?,
                class_name: row.get(1)?,
                grade_name: row.get(2)?,
                student_name: row.get(3)?,
                subject_combination: row.get(4)?,
                language: row.get(5)?,
                total_score: row.get(6)?,
                class_rank: row.get(7)?,
                grade_rank: row.get(8)?,
                selected_subject_count: row.get(9)?,
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
pub fn get_score_detail(app: AppHandle, admission_no: String) -> Result<ScoreDetail, String> {
    let result = (|| -> Result<ScoreDetail, AppError> {
        let conn = open_connection(&app)?;
        init_schema(&conn)?;
        let student = conn
            .query_row(
                r#"
                SELECT admission_no, class_name, grade_name, student_name, subject_combination, language, total_score, class_rank, grade_rank, selected_subject_count
                FROM latest_student_scores
                WHERE admission_no = ?1
                "#,
                params![admission_no],
                |row| {
                    Ok(ScoreDetail {
                        admission_no: row.get(0)?,
                        class_name: row.get(1)?,
                        grade_name: row.get(2)?,
                        student_name: row.get(3)?,
                        subject_combination: row.get(4)?,
                        language: row.get(5)?,
                        total_score: row.get(6)?,
                        class_rank: row.get(7)?,
                        grade_rank: row.get(8)?,
                        selected_subject_count: row.get(9)?,
                        subjects: Vec::new(),
                    })
                },
            )
            .map_err(|_| AppError::new("未找到该成绩记录"))?;

        let mut map: HashMap<Subject, ScoreSubjectItem> = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT subject, score, is_selected, is_absent FROM latest_subject_scores WHERE admission_no = ?1",
        )?;
        let rows = stmt.query_map(params![student.admission_no.clone()], |row| {
            let subject_key: String = row.get(0)?;
            let score: Option<f64> = row.get(1)?;
            let is_selected: i64 = row.get(2)?;
            let is_absent: i64 = row.get(3)?;
            Ok((subject_key, score, is_selected, is_absent))
        })?;
        for row in rows {
            let (subject_key, score, is_selected, is_absent) = row?;
            let Some(subject) = Subject::from_key(&subject_key) else {
                continue;
            };
            let state = if is_selected == 0 {
                ScoreCellState::NotSelected
            } else if is_absent == 1 {
                ScoreCellState::Absent
            } else {
                ScoreCellState::Scored
            };
            map.insert(
                subject,
                ScoreSubjectItem {
                    subject,
                    score,
                    state,
                },
            );
        }

        let mut subjects = Vec::new();
        for (_, subject, _) in SUBJECT_COLUMNS {
            if let Some(item) = map.get(&subject) {
                subjects.push(item.clone());
            } else {
                subjects.push(ScoreSubjectItem {
                    subject,
                    score: None,
                    state: ScoreCellState::NotSelected,
                });
            }
        }
        Ok(ScoreDetail { subjects, ..student })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_score_row(app: AppHandle, payload: UpdateScorePayload) -> Result<SuccessResponse, String> {
    let result = (|| -> Result<SuccessResponse, AppError> {
        let mut conn = open_connection(&app)?;
        init_schema(&conn)?;

        let admission_no = payload.admission_no.trim().to_string();
        let class_name = payload.class_name.trim().to_string();
        let student_name = payload.student_name.trim().to_string();
        if admission_no.is_empty() || class_name.is_empty() || student_name.is_empty() {
            return Err(AppError::new("准考证号、班级、姓名不能为空"));
        }

        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM latest_student_scores WHERE admission_no = ?1",
            params![admission_no.clone()],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(AppError::new("未找到要更新的成绩记录"));
        }

        let mut subject_map: HashMap<Subject, ScoreSubjectItem> = HashMap::new();
        for item in payload.subjects {
            subject_map.insert(item.subject, item);
        }

        let mut normalized = Vec::new();
        let mut total_score = 0.0_f64;
        let mut selected_subject_count = 0_i64;
        for (_, subject, _) in SUBJECT_COLUMNS {
            let mut item = subject_map.remove(&subject).unwrap_or(ScoreSubjectItem {
                subject,
                score: None,
                state: ScoreCellState::NotSelected,
            });
            match item.state {
                ScoreCellState::NotSelected => {
                    item.score = None;
                }
                ScoreCellState::Absent => {
                    item.score = Some(0.0);
                    selected_subject_count += 1;
                }
                ScoreCellState::Scored => {
                    let score = item
                        .score
                        .ok_or_else(|| AppError::new(format!("{}成绩不能为空", subject.as_key())))?;
                    if score < 0.0 {
                        return Err(AppError::new(format!("{}成绩不能小于 0", subject.as_key())));
                    }
                    selected_subject_count += 1;
                    total_score += score;
                }
            }
            normalized.push(item);
        }

        let grade_name = extract_grade_name(&class_name);
        let tx = conn.transaction()?;
        tx.execute(
            r#"
            UPDATE latest_student_scores
            SET class_name = ?1, grade_name = ?2, student_name = ?3, total_score = ?4, selected_subject_count = ?5
            WHERE admission_no = ?6
            "#,
            params![
                class_name,
                grade_name,
                student_name,
                total_score,
                selected_subject_count,
                admission_no.clone()
            ],
        )?;
        tx.execute(
            "DELETE FROM latest_subject_scores WHERE admission_no = ?1",
            params![admission_no.clone()],
        )?;
        for item in normalized {
            tx.execute(
                "INSERT INTO latest_subject_scores (admission_no, subject, score, is_selected, is_absent) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    admission_no.clone(),
                    item.subject.as_key(),
                    item.score,
                    !matches!(item.state, ScoreCellState::NotSelected),
                    matches!(item.state, ScoreCellState::Absent)
                ],
            )?;
        }
        recompute_ranks_tx(&tx)?;
        tx.commit()?;
        Ok(SuccessResponse { success: true })
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_latest_summary(app: AppHandle) -> Result<LatestSummary, String> {
    let result = (|| -> Result<LatestSummary, AppError> {
        let conn = open_connection(&app)?;
        init_schema(&conn)?;
        let imported_at = conn
            .query_row("SELECT imported_at FROM latest_import_meta WHERE id = 1", [], |row| row.get::<_, String>(0))
            .ok();
        let student_count: i64 = conn.query_row("SELECT COUNT(*) FROM latest_student_scores", [], |row| row.get(0))?;
        let class_count: i64 = conn.query_row("SELECT COUNT(DISTINCT class_name) FROM latest_student_scores", [], |row| row.get(0))?;
        let grade_count: i64 = conn.query_row("SELECT COUNT(DISTINCT grade_name) FROM latest_student_scores", [], |row| row.get(0))?;
        Ok(LatestSummary {
            imported_at,
            student_count,
            class_count,
            grade_count,
        })
    })();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_score_cell_selected() {
        let scored = parse_score_cell(Some(&Data::Float(88.5)), 1, "数学", true).unwrap();
        assert!(matches!(scored.state, ScoreCellState::Scored));
        assert_eq!(scored.score, Some(88.5));

        let absent = parse_score_cell(Some(&Data::String("-".to_string())), 1, "物理", true).unwrap();
        assert!(matches!(absent.state, ScoreCellState::Absent));
        assert_eq!(absent.score, Some(0.0));

        // 已选但为空 → 报错
        let err = parse_score_cell(Some(&Data::Empty), 1, "语文", true);
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_score_cell_not_selected() {
        let empty = parse_score_cell(Some(&Data::Empty), 1, "化学", false).unwrap();
        assert!(matches!(empty.state, ScoreCellState::NotSelected));
        assert_eq!(empty.score, None);

        let dash = parse_score_cell(Some(&Data::String("-".to_string())), 1, "化学", false).unwrap();
        assert!(matches!(dash.state, ScoreCellState::NotSelected));
        assert_eq!(dash.score, None);

        // 未选但有成绩 → 报错
        let err = parse_score_cell(Some(&Data::Float(90.0)), 1, "化学", false);
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_subject_combination() {
        let full = parse_subject_combination("全科", 0).unwrap();
        assert!(full.contains(&Subject::Physics));
        assert!(full.contains(&Subject::Chemistry));
        assert!(full.contains(&Subject::Biology));
        assert!(full.contains(&Subject::Politics));
        assert!(full.contains(&Subject::History));
        assert!(full.contains(&Subject::Geography));
        assert!(full.contains(&Subject::Chinese));
        assert!(full.contains(&Subject::Math));

        let partial = parse_subject_combination("物化生", 0).unwrap();
        assert!(partial.contains(&Subject::Physics));
        assert!(partial.contains(&Subject::Chemistry));
        assert!(partial.contains(&Subject::Biology));
        assert!(!partial.contains(&Subject::History));
        assert!(partial.contains(&Subject::Chinese));
        assert!(partial.contains(&Subject::Math));

        let err = parse_subject_combination("物X化", 0);
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_language() {
        assert!(matches!(parse_language("英语", 0).unwrap(), Subject::English));
        assert!(matches!(parse_language("俄语", 0).unwrap(), Subject::Russian));
        assert!(matches!(parse_language("日语", 0).unwrap(), Subject::Japanese));
        assert!(parse_language("法语", 0).is_err());
    }
}
