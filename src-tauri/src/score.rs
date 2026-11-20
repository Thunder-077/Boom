use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use calamine::{open_workbook_auto, Data, Reader};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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

impl From<sea_orm::DbErr> for AppError {
    fn from(value: sea_orm::DbErr) -> Self {
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
pub(crate) struct ParsedSubjectScore {
    pub(crate) subject: Subject,
    pub(crate) score: Option<f64>,
    pub(crate) state: ScoreCellState,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedStudent {
    pub(crate) admission_no: String,
    pub(crate) class_name: String,
    pub(crate) grade_name: String,
    pub(crate) student_name: String,
    pub(crate) subject_combination: String,
    pub(crate) language: String,
    pub(crate) total_score: f64,
    pub(crate) selected_subject_count: i64,
    pub(crate) class_rank: i64,
    pub(crate) grade_rank: i64,
    pub(crate) subjects: Vec<ParsedSubjectScore>,
    pub(crate) remark: Option<String>,
    pub(crate) exam_group: crate::exam_group::ExamGroup,
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
    pub(crate) admission_no: String,
    pub(crate) class_name: String,
    pub(crate) grade_name: String,
    pub(crate) student_name: String,
    pub(crate) subject_combination: String,
    pub(crate) language: String,
    pub(crate) total_score: f64,
    pub(crate) class_rank: i64,
    pub(crate) grade_rank: i64,
    pub(crate) selected_subject_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestSummary {
    pub(crate) imported_at: Option<String>,
    pub(crate) student_count: i64,
    pub(crate) class_count: i64,
    pub(crate) grade_count: i64,
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
    pub(crate) subject: Subject,
    pub(crate) score: Option<f64>,
    pub(crate) state: ScoreCellState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDetail {
    pub(crate) admission_no: String,
    pub(crate) class_name: String,
    pub(crate) grade_name: String,
    pub(crate) student_name: String,
    pub(crate) subject_combination: String,
    pub(crate) language: String,
    pub(crate) total_score: f64,
    pub(crate) class_rank: i64,
    pub(crate) grade_rank: i64,
    pub(crate) selected_subject_count: i64,
    pub(crate) subjects: Vec<ScoreSubjectItem>,
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
        return Err(AppError::new(format!(
            "第 {} 行选科组合为空",
            row_index + 1
        )));
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

fn parse_score_cell(
    cell: Option<&Data>,
    row_index: usize,
    subject_header: &str,
    is_selected: bool,
) -> Result<ParsedSubjectScoreState, AppError> {
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
    if comprehensive_class_name(class_name) {
        return "高三".to_string();
    }
    let matcher = Regex::new(r"高[一二三]").expect("regex for grade should be valid");
    if let Some(m) = matcher.find(class_name) {
        return m.as_str().to_string();
    }
    "未知年级".to_string()
}

fn comprehensive_class_name(class_name: &str) -> bool {
    class_name.strip_prefix("综评").and_then(|name| name.strip_suffix('班'))
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) && number.parse::<u64>().is_ok_and(|value| value > 0))
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

pub(crate) fn parse_excel_rows(file_path: &str) -> Result<Vec<ParsedStudent>, AppError> {
    let mut workbook = open_workbook_auto(file_path)?;
    let mut students = Vec::new();
    let mut seen = HashMap::<String, crate::excel_import::ImportLocation>::new();
    let mut groups = HashMap::new();
    // 先校验全部Sheet，任何错误均不得进入替换数据库的事务。
    for name in workbook.sheet_names().to_vec() {
        let range = workbook.worksheet_range(&name)?;
        if range.rows().all(|row| {
            row.iter()
                .all(|cell| cell_to_trimmed_string(Some(cell)).is_empty())
        }) {
            continue;
        }
        let grade = crate::excel_import::normalize_sheet_grade(&name)?;
        let first_row = range.start().map_or(0, |(r, _)| r) as usize;
        let parsed = parse_score_sheet(&range, &grade).map_err(|error| {
            AppError::new(format!("Sheet《{name}》第{}行起：{error}", first_row + 1))
        })?;
        for (student, row_number) in parsed {
            let location = crate::excel_import::ImportLocation {
                sheet_name: name.clone(),
                row_number,
            };
            if let Some(previous) = seen.insert(student.admission_no.clone(), location.clone()) {
                return Err(AppError::new(format!("{location}与{previous}准考证号重复")));
            }
            if grade == "高三" {
                let key = student.class_name.clone();
                if let Some((group, previous)) =
                    groups.insert(key, (student.exam_group, location.clone()))
                {
                    if group != student.exam_group {
                        return Err(AppError::new(format!(
                            "{location}与{previous}同班综评标记不一致"
                        )));
                    }
                }
            }
            students.push(student);
        }
    }
    if students.is_empty() {
        return Err(AppError::new("Excel 没有可导入的数据行"));
    }
    Ok(students)
}

fn parse_score_sheet(
    range: &calamine::Range<Data>,
    grade: &str,
) -> Result<Vec<(ParsedStudent, usize)>, AppError> {
    let mut rows_iter = range.rows();
    let header_row = rows_iter
        .next()
        .ok_or_else(|| AppError::new("Excel 文件为空，缺少表头"))?;
    validate_header(header_row)?;
    let remark_column = header_row
        .iter()
        .position(|cell| cell_to_trimmed_string(Some(cell)) == "备注");
    let first_row = range.start().map_or(0, |(r, _)| r) as usize;

    let mut students = Vec::new();
    for (offset, row) in rows_iter.enumerate() {
        let excel_row_index = first_row + offset + 1;
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
        if extract_grade_name(&class_name) != grade {
            return Err(AppError::new(format!(
                "第{}行班级年级与Sheet年级不一致",
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
            let parsed = parse_score_cell(
                row.get(column_index),
                excel_row_index,
                header_name,
                is_selected,
            )?;
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

        let remark = remark_column
            .map(|column| cell_to_trimmed_string(row.get(column)))
            .filter(|text| !text.is_empty());
        let exam_group = if grade == "高三" && (comprehensive_class_name(&class_name) || remark.as_deref() == Some("综评班")) {
            crate::exam_group::ExamGroup::Comprehensive
        } else {
            crate::exam_group::ExamGroup::Regular
        };
        students.push((
            ParsedStudent {
                admission_no,
                class_name: class_name.clone(),
                grade_name: grade.to_string(),
                student_name,
                subject_combination: combination_text,
                language: language_text,
                total_score,
                selected_subject_count,
                class_rank: 0,
                grade_rank: 0,
                subjects,
                remark,
                exam_group,
            },
            excel_row_index + 1,
        ));
    }
    if students.is_empty() {
        return Err(AppError::new("Excel 没有可导入的数据行"));
    }
    Ok(students)
}

#[cfg(test)]
pub(crate) mod multi_sheet_tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use std::path::PathBuf;

    pub(crate) fn fixture(kind: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "boom-score-{kind}-{}.xlsx",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut book = Workbook::new();
        for (g, count) in [(1, 2), (2, 3), (3, 4)] {
            let sheet = book.add_worksheet();
            sheet
                .set_name(format!("高{}", ["一", "二", "三"][g - 1]))
                .unwrap();
            for (col, header) in FIXED_HEADERS
                .iter()
                .chain(std::iter::once(&"备注"))
                .enumerate()
            {
                sheet.write_string(0, col as u16, *header).unwrap();
            }
            for i in 0..count {
                let mut values = vec![
                    format!("g{g}-{i}"),
                    format!(
                        "高{}{}班",
                        ["一", "二", "三"][g - 1],
                        if g == 3 && i > 1 { 11 } else { 1 }
                    ),
                    "匿名学生".into(),
                    "物化生".into(),
                    "英语".into(),
                ];
                values.extend((0..11).map(|j| if j < 6 { "10" } else { "" }.to_string()));
                values.push(if g == 3 && i > 1 {
                    "综评班".into()
                } else {
                    String::new()
                });
                if kind == "pipeline" && g == 3 && i == 1 {
                    values[4] = "俄语".into();
                    values[7] = String::new();
                    values[14] = "10".into();
                }
                if kind == "named_comprehensive" && g == 3 && i > 1 {
                    values[1] = format!("综评{}班", i - 1);
                    values[16] = String::new();
                }
                if kind == "duplicate" && g == 3 && i == 0 {
                    values[0] = "g1-0".into();
                }
                if kind == "grade" && g == 3 && i == 0 {
                    values[1] = "高一1班".into();
                }
                if kind == "mixed" && g == 3 && i == 2 {
                    values[16] = String::new();
                }
                if kind == "required" && g == 3 && i == 0 {
                    values[2] = String::new();
                }
                for (col, value) in values.iter().enumerate() {
                    sheet
                        .write_string((i + 1) as u32, col as u16, value)
                        .unwrap();
                }
            }
            if kind == "header" && g == 3 {
                sheet.write_string(0, 0, "错误表头").unwrap();
            }
        }
        book.add_worksheet().set_name("空白").unwrap();
        book.save(&path).unwrap();
        path
    }

    #[test]
    fn test_import_named_comprehensive_classes_without_remarks() {
        let path = fixture("named_comprehensive");
        let students = parse_excel_rows(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        for name in ["综评1班", "综评2班"] {
            let student = students.iter().find(|student| student.class_name == name).unwrap();
            assert_eq!(student.grade_name, "高三");
            assert_eq!(student.exam_group, crate::exam_group::ExamGroup::Comprehensive);
        }
    }

    #[test]
    fn test_score_import_reads_all_grade_sheets() {
        let path = fixture("ok");
        let mut students = parse_excel_rows(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(students.len(), 9);
        apply_ranks(&mut students);
        assert_eq!(
            students.iter().filter(|s| s.grade_name == "高三").count(),
            4
        );
        assert_eq!(
            students
                .iter()
                .filter(|s| s.exam_group == crate::exam_group::ExamGroup::Comprehensive)
                .count(),
            2
        );
        assert!(students.iter().all(|s| s.grade_rank == 1));
    }

    #[test]
    fn test_score_import_reports_source_for_bad_later_sheets() {
        for kind in ["duplicate", "grade", "mixed", "required", "header"] {
            let path = fixture(kind);
            let result = parse_excel_rows(path.to_str().unwrap());
            std::fs::remove_file(path).unwrap();
            let error = result.expect_err(kind).to_string();
            assert!(error.contains("高三"), "{kind}: {error}");
            assert!(error.contains("行"), "{kind}: {error}");
        }
    }

    #[test]
    fn test_score_import_failure_preserves_snapshot() {
        tauri::async_runtime::block_on(async {
            use sea_orm::{Database, EntityTrait};
            use sea_orm_migration::MigratorTrait;
            let db = Database::connect("sqlite::memory:").await.unwrap();
            crate::db::migration::Migrator::up(&db, None).await.unwrap();
            let good = fixture("ok");
            assert_eq!(
                import_scores_into_db(&db, good.to_str().unwrap(), "old")
                    .await
                    .unwrap(),
                9
            );
            std::fs::remove_file(good).unwrap();
            let before = crate::entity::latest_student_scores::Entity::find()
                .all(&db)
                .await
                .unwrap();
            for kind in ["duplicate", "grade", "mixed", "required", "header"] {
                let bad = fixture(kind);
                assert!(import_scores_into_db(&db, bad.to_str().unwrap(), "new")
                    .await
                    .is_err());
                std::fs::remove_file(bad).unwrap();
                assert_eq!(
                    crate::entity::latest_student_scores::Entity::find()
                        .all(&db)
                        .await
                        .unwrap(),
                    before
                );
                assert_eq!(
                    crate::entity::latest_import_meta::Entity::find_by_id(1)
                        .one(&db)
                        .await
                        .unwrap()
                        .unwrap()
                        .imported_at,
                    "old"
                );
            }
        });
    }
}

fn assign_competition_rank(
    students: &mut [ParsedStudent],
    groups: HashMap<String, Vec<usize>>,
    is_class: bool,
) {
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
        class_groups
            .entry(student.class_name.clone())
            .or_default()
            .push(idx);
        grade_groups
            .entry(student.grade_name.clone())
            .or_default()
            .push(idx);
    }
    assign_competition_rank(students, class_groups, true);
    assign_competition_rank(students, grade_groups, false);
}

#[derive(Debug, Clone)]
pub(crate) struct RankRow {
    pub(crate) admission_no: String,
    pub(crate) class_name: String,
    pub(crate) grade_name: String,
    pub(crate) total_score: f64,
    pub(crate) class_rank: i64,
    pub(crate) grade_rank: i64,
}

pub(crate) fn assign_rank_rows(rows: &mut [RankRow]) {
    let mut class_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut grade_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        class_groups
            .entry(row.class_name.clone())
            .or_default()
            .push(idx);
        grade_groups
            .entry(row.grade_name.clone())
            .or_default()
            .push(idx);
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

/// 解析、校验全部年级后才替换快照；调用方无需启动桌面窗口即可验证原子导入。
pub(crate) async fn import_scores_into_db(
    db: &sea_orm::DatabaseConnection,
    file_path: &str,
    imported_at: &str,
) -> Result<usize, AppError> {
    let mut students = parse_excel_rows(file_path)?;
    apply_ranks(&mut students);
    let tx = sea_orm::TransactionTrait::begin(db).await?;
    crate::db::repos::score::persist_latest_snapshot(&tx, file_path, imported_at, &students)
        .await?;
    tx.commit().await?;
    Ok(students.len())
}

#[tauri::command]
pub async fn import_scores_from_excel(
    app: AppHandle,
    file_path: String,
) -> Result<ImportResult, String> {
    let start = Utc::now();
    let result: Result<ImportResult, AppError> = async {
        let db = crate::db::connect(&app).await?;
        let imported_at = Utc::now().to_rfc3339();
        let row_count = import_scores_into_db(&db, &file_path, &imported_at).await?;
        Ok(ImportResult {
            imported_at,
            row_count: row_count as i64,
            warning_count: 0,
            duration_ms: (Utc::now() - start).num_milliseconds(),
        })
    }
    .await;
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "score.import_scores_from_excel",
            &format!("file_path={file_path} | {e}"),
        );
        e.to_string()
    })
}

#[tauri::command]
pub async fn list_latest_score_rows(
    app: AppHandle,
    params: ScoreListParams,
) -> Result<ListResult<ScoreRow>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::score::list(&db, params).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_score_detail(app: AppHandle, admission_no: String) -> Result<ScoreDetail, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::score::get_detail(&db, &admission_no).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_score_row(
    app: AppHandle,
    payload: UpdateScorePayload,
) -> Result<SuccessResponse, String> {
    let result: Result<SuccessResponse, AppError> = async {
        let db = crate::db::connect(&app).await?;
        let admission_no = payload.admission_no.trim().to_string();
        let class_name = payload.class_name.trim().to_string();
        let student_name = payload.student_name.trim().to_string();
        if admission_no.is_empty() || class_name.is_empty() || student_name.is_empty() {
            return Err(AppError::new("准考证号、班级、姓名不能为空"));
        }

        if !crate::db::repos::score::exists(&db, &admission_no).await? {
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
                    let score = item.score.ok_or_else(|| {
                        AppError::new(format!("{}成绩不能为空", subject.as_key()))
                    })?;
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
        crate::db::repos::score::update_student_scores(
            &db,
            &admission_no,
            &class_name,
            &grade_name,
            &student_name,
            total_score,
            selected_subject_count,
            &normalized,
        )
        .await?;
        Ok(SuccessResponse { success: true })
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_score_grade_options(app: AppHandle) -> Result<Vec<String>, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::score::list_grade_options(&db).await
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_latest_summary(app: AppHandle) -> Result<LatestSummary, String> {
    let result = async {
        let db = crate::db::connect(&app).await?;
        crate::db::repos::score::summary(&db).await
    }
    .await;
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

        let absent =
            parse_score_cell(Some(&Data::String("-".to_string())), 1, "物理", true).unwrap();
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

        let dash =
            parse_score_cell(Some(&Data::String("-".to_string())), 1, "化学", false).unwrap();
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
        assert!(matches!(
            parse_language("英语", 0).unwrap(),
            Subject::English
        ));
        assert!(matches!(
            parse_language("俄语", 0).unwrap(),
            Subject::Russian
        ));
        assert!(matches!(
            parse_language("日语", 0).unwrap(),
            Subject::Japanese
        ));
        assert!(parse_language("法语", 0).is_err());
    }
}
