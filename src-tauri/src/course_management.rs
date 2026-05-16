use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use calamine::{open_workbook_auto, Data, Range, Reader};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use regex::Regex;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app_log;
use crate::score::{self, AppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseImportResult {
    imported_at: String,
    entry_count: i64,
    teacher_count: i64,
    admin_class_count: i64,
    foreign_class_count: i64,
    duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseSummary {
    latest_import_id: Option<i64>,
    imported_at: Option<String>,
    entry_count: i64,
    teacher_count: i64,
    admin_class_count: i64,
    foreign_class_count: i64,
    effective_start_date: Option<String>,
    effective_end_date: Option<String>,
    start_week: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseImportBatch {
    id: i64,
    imported_at: String,
    source_file: String,
    entry_count: i64,
    teacher_count: i64,
    admin_class_count: i64,
    foreign_class_count: i64,
    effective_start_date: Option<String>,
    effective_end_date: Option<String>,
    start_week: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseImportSettingsPayload {
    import_id: i64,
    effective_start_date: Option<String>,
    effective_end_date: Option<String>,
    start_week: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseClassOption {
    class_name: String,
    display_name: String,
    class_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseScheduleEntry {
    week_index: i64,
    day_of_week: i64,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
    subject: String,
    teacher_names: Vec<String>,
    class_name: String,
    display_class_name: String,
    class_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoursePeriodSlot {
    week_index: i64,
    day_of_week: i64,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseScheduleQuery {
    view_type: String,
    target: String,
    import_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseScheduleView {
    import_id: i64,
    target: String,
    view_type: String,
    entries: Vec<CourseScheduleEntry>,
    periods: Vec<CoursePeriodSlot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseSubstitutionCandidateQuery {
    import_id: i64,
    teacher_name: String,
    start_date: String,
    end_date: String,
    period_indexes: Option<Vec<i64>>,
    start_period_index: Option<i64>,
    end_period_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseScheduleChange {
    id: i64,
    import_id: i64,
    source_entry_id: i64,
    change_type: String,
    status: String,
    target_date: String,
    source_teacher_name: String,
    actual_teacher_name: String,
    reason: String,
    remark: String,
    created_at: String,
    updated_at: String,
    revoked_at: Option<String>,
    week_index: i64,
    day_of_week: i64,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
    subject: String,
    class_name: String,
    display_class_name: String,
    class_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseSubstitutionCandidate {
    source_entry_id: i64,
    import_id: i64,
    target_date: String,
    week_index: i64,
    day_of_week: i64,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
    subject: String,
    teacher_names: Vec<String>,
    source_teacher_name: String,
    class_name: String,
    display_class_name: String,
    class_type: String,
    existing_change: Option<CourseScheduleChange>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCourseSubstitutionsPayload {
    import_id: i64,
    reason: String,
    remark: String,
    items: Vec<SaveCourseSubstitutionItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCourseSubstitutionItem {
    source_entry_id: i64,
    target_date: String,
    source_teacher_name: String,
    actual_teacher_name: String,
    remark: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkloadQuery {
    import_id: i64,
    start_date: String,
    end_date: String,
    start_period_index: Option<i64>,
    end_period_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkloadDetail {
    teacher_name: String,
    target_date: String,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
    category: String,
    subject: String,
    class_name: String,
    display_class_name: String,
    original_teacher_name: String,
    actual_teacher_name: String,
    is_substitution: bool,
    remark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkloadSummary {
    teacher_name: String,
    morning_reading_count: i64,
    morning_count: i64,
    afternoon_count: i64,
    evening_count: i64,
    substitution_count: i64,
    total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkloadReport {
    import_id: i64,
    start_date: String,
    end_date: String,
    details: Vec<CourseWorkloadDetail>,
    summaries: Vec<CourseWorkloadSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCourseWorkloadResult {
    file_path: String,
    exported_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParsedSubject {
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

impl ParsedSubject {
    fn as_key(self) -> &'static str {
        match self {
            ParsedSubject::Chinese => "chinese",
            ParsedSubject::Math => "math",
            ParsedSubject::English => "english",
            ParsedSubject::Physics => "physics",
            ParsedSubject::Chemistry => "chemistry",
            ParsedSubject::Biology => "biology",
            ParsedSubject::Politics => "politics",
            ParsedSubject::History => "history",
            ParsedSubject::Geography => "geography",
            ParsedSubject::Russian => "russian",
            ParsedSubject::Japanese => "japanese",
            ParsedSubject::Sports => "sports",
            ParsedSubject::Music => "music",
            ParsedSubject::General => "general",
            ParsedSubject::Information => "information",
            ParsedSubject::FineArts => "fine_arts",
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedEntry {
    class_name: String,
    display_class_name: String,
    class_type: String,
    week_index: i64,
    day_of_week: i64,
    day_label: String,
    period_index: i64,
    period_label: String,
    section_label: String,
    subject: String,
    teacher_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct TeacherAssignment {
    teacher_name: String,
    subject: ParsedSubject,
    class_name: String,
}

#[derive(Debug, Clone)]
struct ParsedWorkbook {
    entries: Vec<ParsedEntry>,
    periods: Vec<CoursePeriodSlot>,
    classes: Vec<CourseClassOption>,
    assignments: Vec<TeacherAssignment>,
}

#[derive(Debug, Clone)]
struct DayBlock {
    start_col: usize,
    end_col: usize,
    week_index: i64,
    day_of_week: i64,
    day_label: String,
}

pub fn ensure_schema(conn: &Connection) -> Result<(), AppError> {
    crate::schema::ensure_schema(conn)?;
    ensure_import_setting_columns(conn)?;
    Ok(())
}

fn ensure_import_setting_columns(conn: &Connection) -> Result<(), AppError> {
    let mut stmt = conn.prepare("PRAGMA table_info(course_schedule_imports)")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = HashSet::new();
    for row in rows {
        columns.insert(row?);
    }

    // Existing user databases may already contain course imports, so these
    // metadata columns are added lazily without rebuilding the import table.
    if !columns.contains("effective_start_date") {
        conn.execute(
            "ALTER TABLE course_schedule_imports ADD COLUMN effective_start_date TEXT",
            [],
        )?;
    }
    if !columns.contains("effective_end_date") {
        conn.execute(
            "ALTER TABLE course_schedule_imports ADD COLUMN effective_end_date TEXT",
            [],
        )?;
    }
    if !columns.contains("start_week") {
        conn.execute(
            "ALTER TABLE course_schedule_imports ADD COLUMN start_week INTEGER NOT NULL DEFAULT 1",
            [],
        )?;
    }
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

fn normalize_subject_name(text: &str) -> String {
    text.trim().replace(' ', "")
}

fn parse_subject(text: &str) -> Option<ParsedSubject> {
    match normalize_subject_name(text).as_str() {
        "语文" | "语" => Some(ParsedSubject::Chinese),
        "数学" | "数" => Some(ParsedSubject::Math),
        "外语" | "英语" | "英" => Some(ParsedSubject::English),
        "俄语" | "俄" => Some(ParsedSubject::Russian),
        "日语" | "日" => Some(ParsedSubject::Japanese),
        "物理" | "物" => Some(ParsedSubject::Physics),
        "化学" | "化" => Some(ParsedSubject::Chemistry),
        "生物" | "生" => Some(ParsedSubject::Biology),
        "政治" | "道法" | "政" => Some(ParsedSubject::Politics),
        "历史" | "历" => Some(ParsedSubject::History),
        "地理" | "地" => Some(ParsedSubject::Geography),
        "体育" => Some(ParsedSubject::Sports),
        "音乐" => Some(ParsedSubject::Music),
        "美术" => Some(ParsedSubject::FineArts),
        "信息" => Some(ParsedSubject::Information),
        "通用" => Some(ParsedSubject::General),
        _ => None,
    }
}

fn subject_from_bracket_suffix(text: &str) -> Option<ParsedSubject> {
    let normalized = normalize_subject_name(text);
    let matcher = Regex::new(r"[（(]([^）)]+)[）)]").expect("subject suffix regex should be valid");
    let caps = matcher.captures(&normalized)?;
    let token = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
    parse_subject(token)
}

fn subject_from_schedule_label(text: &str) -> Option<ParsedSubject> {
    parse_subject(text).or_else(|| subject_from_bracket_suffix(text))
}

fn is_schedule_subject(text: &str) -> bool {
    let normalized = normalize_subject_name(text);
    if normalized.is_empty() || normalized.starts_with('=') || normalized.starts_with('#') {
        return false;
    }
    subject_from_schedule_label(&normalized).is_some()
        || matches!(
            normalized.as_str(),
            "班会"
                | "自习"
                | "听力"
                | "练字"
                | "阅读"
                | "活动"
                | "理化"
                | "政史"
                | "地生"
                | "限时"
        )
        || normalized.starts_with("自习（")
        || normalized.starts_with("自习(")
        || normalized.starts_with("限时（")
        || normalized.starts_with("限时(")
        || normalized.starts_with("听力（")
        || normalized.starts_with("听力(")
        || normalized.starts_with("练字（")
        || normalized.starts_with("练字(")
}

fn is_importable_schedule_cell(text: &str) -> bool {
    let normalized = normalize_subject_name(text);
    !normalized.is_empty() && !normalized.starts_with('=') && !normalized.starts_with('#')
}

fn normalize_class_code(token: &str) -> String {
    let trimmed = token.trim();
    let pattern = Regex::new(r"^([123])(\d{2})$").expect("class code regex should be valid");
    if let Some(caps) = pattern.captures(trimmed) {
        let grade = match &caps[1] {
            "1" => "高一",
            "2" => "高二",
            "3" => "高三",
            _ => return trimmed.to_string(),
        };
        let class_no = caps[2].parse::<i32>().unwrap_or(0);
        if class_no > 0 {
            return format!("{grade}{class_no}班");
        }
    }
    trimmed.to_string()
}

fn class_to_teacher_header(class_name: &str) -> String {
    let pattern = Regex::new(r"^([123])(\d{2})$").expect("class header regex should be valid");
    if let Some(caps) = pattern.captures(class_name.trim()) {
        let grade = match &caps[1] {
            "1" => "高一",
            "2" => "高二",
            "3" => "高三",
            _ => return class_name.to_string(),
        };
        let class_no = caps[2].parse::<i32>().unwrap_or(0);
        if class_no > 0 {
            return format!("{grade}（{class_no}）班");
        }
    }
    class_name.to_string()
}

fn is_admin_class(class_name: &str) -> bool {
    Regex::new(r"^[123]\d{2}$")
        .expect("admin class regex should be valid")
        .is_match(class_name.trim())
}

fn class_type_for(class_name: &str) -> &'static str {
    if is_admin_class(class_name) {
        "admin"
    } else if class_name.contains("英语")
        || class_name.contains("俄语")
        || class_name.contains("日语")
        || class_name.contains("（英）")
        || class_name.contains("（俄）")
        || class_name.contains("（日）")
        || class_name.contains("(英)")
        || class_name.contains("(俄)")
        || class_name.contains("(日)")
    {
        "foreign"
    } else {
        "admin"
    }
}

fn split_teacher_names(text: &str) -> Vec<String> {
    text.replace('，', "/")
        .replace('、', "/")
        .replace(',', "/")
        .replace('；', "/")
        .replace(';', "/")
        .split('/')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn should_import_teacher_name(name: &str) -> bool {
    // 走班行政班中常见“岳/厉”这类简称，不应作为真实教师姓名写入教师列表。
    name.chars().count() >= 2
}

fn teacher_surname(name: &str) -> Option<String> {
    name.chars().next().map(|ch| ch.to_string())
}

fn is_foreign_language_subject(subject: ParsedSubject) -> bool {
    matches!(
        subject,
        ParsedSubject::English | ParsedSubject::Russian | ParsedSubject::Japanese
    )
}

fn foreign_subject_from_text(text: &str) -> Option<ParsedSubject> {
    subject_from_schedule_label(text).filter(|subject| is_foreign_language_subject(*subject))
}

fn foreign_subject_from_label_text(text: &str) -> Option<ParsedSubject> {
    let normalized = normalize_subject_name(text);
    if normalized.contains("俄语") || normalized.contains("（俄）") || normalized.contains("(俄)") {
        return Some(ParsedSubject::Russian);
    }
    if normalized.contains("日语") || normalized.contains("（日）") || normalized.contains("(日)") {
        return Some(ParsedSubject::Japanese);
    }
    if normalized.contains("英语") || normalized.contains("（英）") || normalized.contains("(英)") {
        return Some(ParsedSubject::English);
    }
    foreign_subject_from_text(&normalized)
}

fn foreign_subject_for_entry(entry: &ParsedEntry) -> Option<ParsedSubject> {
    foreign_subject_from_text(&entry.subject)
        .or_else(|| foreign_subject_from_label_text(&entry.class_name))
        .or_else(|| foreign_subject_from_label_text(&entry.display_class_name))
}

fn is_generic_foreign_subject_text(text: &str) -> bool {
    let normalized = normalize_subject_name(text);
    if matches!(normalized.as_str(), "外语" | "外") {
        return true;
    }
    let matcher = Regex::new(r"[（(]([^）)]+)[）)]").expect("subject suffix regex should be valid");
    matcher
        .captures(&normalized)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .is_some_and(|token| matches!(token.as_str(), "外语" | "外"))
}

fn grade_digit_from_text(text: &str) -> Option<char> {
    if text.contains("高一") || text.starts_with('1') {
        Some('1')
    } else if text.contains("高二") || text.starts_with('2') {
        Some('2')
    } else if text.contains("高三") || text.starts_with('3') {
        Some('3')
    } else {
        None
    }
}

fn get_cell(range: &Range<Data>, row: usize, col: usize) -> String {
    cell_to_string(range.get((row, col)))
}

fn find_day_blocks(range: &Range<Data>) -> Vec<DayBlock> {
    let mut starts = Vec::new();
    let day_matcher =
        Regex::new(r"星\s*期\s*([一二三四五六日])").expect("day regex should be valid");
    let max_col = range.width();
    for col in 0..max_col {
        let title = get_cell(range, 0, col);
        if let Some(caps) = day_matcher.captures(&title) {
            let day_label = caps.get(1).map(|m| m.as_str()).unwrap_or("一").to_string();
            let day_of_week = match day_label.as_str() {
                "一" => 1,
                "二" => 2,
                "三" => 3,
                "四" => 4,
                "五" => 5,
                "六" => 6,
                "日" => 7,
                _ => 1,
            };
            starts.push((col, day_of_week, format!("星期{day_label}")));
        }
    }

    starts
        .iter()
        .enumerate()
        .map(|(index, (start_col, day_of_week, day_label))| {
            let end_col = starts
                .get(index + 1)
                .map(|(next_start, _, _)| next_start.saturating_sub(1))
                .unwrap_or_else(|| max_col.saturating_sub(1));
            DayBlock {
                start_col: *start_col,
                end_col,
                week_index: (index / 7 + 1) as i64,
                day_of_week: *day_of_week,
                day_label: day_label.clone(),
            }
        })
        .collect()
}

fn parse_teacher_arrangements(
    range: &Range<Data>,
) -> (
    HashMap<(String, String), Vec<String>>,
    Vec<TeacherAssignment>,
) {
    let mut teacher_map = HashMap::new();
    let mut assignments = Vec::new();
    let mut headers = Vec::new();
    let header_row = 1;
    for col in 1..range.width() {
        let header = get_cell(range, header_row, col);
        if !header.is_empty() {
            headers.push((col, header));
        }
    }

    for row in 2..range.height() {
        let subject_name = normalize_subject_name(&get_cell(range, row, 0));
        if subject_name.is_empty() || !is_schedule_subject(&subject_name) {
            continue;
        };
        for (col, class_header) in &headers {
            let teacher_text = get_cell(range, row, *col);
            if teacher_text.is_empty() {
                continue;
            }
            let teachers = split_teacher_names(&teacher_text);
            teacher_map.insert(
                (subject_name.clone(), class_header.clone()),
                teachers.clone(),
            );
            let Some(subject) = subject_from_schedule_label(&subject_name) else {
                continue;
            };
            for teacher_name in teachers {
                if !should_import_teacher_name(&teacher_name) {
                    continue;
                }
                assignments.push(TeacherAssignment {
                    teacher_name,
                    subject,
                    class_name: normalize_class_code(class_header),
                });
            }
        }
    }
    (teacher_map, assignments)
}

fn resolve_teachers(
    teacher_map: &HashMap<(String, String), Vec<String>>,
    class_name: &str,
    subject: &str,
) -> Vec<String> {
    let subject_name = normalize_subject_name(subject);
    let header = class_to_teacher_header(class_name);
    teacher_map
        .get(&(subject_name.clone(), header.clone()))
        .or_else(|| {
            if subject_name == "英语" || subject_name == "俄语" {
                teacher_map.get(&("外语".to_string(), header.clone()))
            } else {
                None
            }
        })
        .cloned()
        .unwrap_or_default()
}

fn parse_total_schedule(
    range: &Range<Data>,
    teacher_map: &HashMap<(String, String), Vec<String>>,
) -> (
    Vec<ParsedEntry>,
    Vec<CoursePeriodSlot>,
    Vec<CourseClassOption>,
) {
    let day_blocks = find_day_blocks(range);
    let mut entries = Vec::new();
    let mut periods = Vec::new();
    let mut class_set = BTreeMap::<(String, String), CourseClassOption>::new();

    for block in day_blocks {
        let class_start = if block.start_col == 0 {
            2
        } else {
            block.start_col
        };
        let class_end = block.end_col;
        let mut current_section = String::new();
        let mut period_index = 0_i64;

        for row in 3..range.height() {
            // 总课表横向按星期展开，但“早上/上午/下午”和节次只在最左侧 A/B 列出现。
            // 后续星期区块从班级列开始，不能按区块起始列读取节次，否则会漏掉周二及之后的课。
            let section = get_cell(range, row, 0);
            if !section.is_empty() {
                current_section = section;
            }
            let period_label = get_cell(range, row, 1);
            if period_label.is_empty() {
                continue;
            }
            period_index += 1;
            periods.push(CoursePeriodSlot {
                week_index: block.week_index,
                day_of_week: block.day_of_week,
                day_label: block.day_label.clone(),
                period_index,
                period_label: period_label.clone(),
                section_label: current_section.clone(),
            });

            for col in class_start..=class_end {
                let class_name = get_cell(range, 2, col);
                if class_name.is_empty() {
                    continue;
                }
                let subject = get_cell(range, row, col);
                if !is_importable_schedule_cell(&subject) {
                    continue;
                }
                let class_type = class_type_for(&class_name).to_string();
                let display_class_name = if is_admin_class(&class_name) {
                    normalize_class_code(&class_name)
                } else {
                    class_name.clone()
                };
                let teacher_names = resolve_teachers(teacher_map, &class_name, &subject);

                class_set
                    .entry((class_type.clone(), class_name.clone()))
                    .or_insert_with(|| CourseClassOption {
                        class_name: class_name.clone(),
                        display_name: display_class_name.clone(),
                        class_type: class_type.clone(),
                    });

                entries.push(ParsedEntry {
                    class_name,
                    display_class_name,
                    class_type,
                    week_index: block.week_index,
                    day_of_week: block.day_of_week,
                    day_label: block.day_label.clone(),
                    period_index,
                    period_label: period_label.clone(),
                    section_label: current_section.clone(),
                    subject: normalize_subject_name(&subject),
                    teacher_names,
                });
            }
        }
    }

    let classes = class_set.into_values().collect();
    (entries, periods, classes)
}

fn foreign_entry_matches_admin_foreign_subject(
    foreign_entry: &ParsedEntry,
    admin_entry: &ParsedEntry,
) -> bool {
    if foreign_entry.class_type != "foreign" || admin_entry.class_type != "admin" {
        return false;
    }
    let Some(foreign_subject) = foreign_subject_for_entry(foreign_entry) else {
        return false;
    };
    if !is_generic_foreign_subject_text(&admin_entry.subject)
        && foreign_subject_for_entry(admin_entry) != Some(foreign_subject)
    {
        return false;
    }
    if grade_digit_from_text(&foreign_entry.class_name) != grade_digit_from_text(&admin_entry.class_name) {
        return false;
    }
    true
}

fn expand_short_foreign_teacher_names(entries: &mut [ParsedEntry]) {
    let foreign_entries = entries
        .iter()
        .filter(|entry| entry.class_type == "foreign")
        .cloned()
        .collect::<Vec<_>>();

    for entry in entries.iter_mut() {
        if entry.class_type != "admin"
            || (foreign_subject_from_text(&entry.subject).is_none()
                && !is_generic_foreign_subject_text(&entry.subject))
        {
            continue;
        }
        let has_short_name = entry
            .teacher_names
            .iter()
            .any(|name| !should_import_teacher_name(name));
        if !has_short_name {
            continue;
        }
        let matched_foreign_teachers = foreign_entries
            .iter()
            .filter(|foreign_entry| {
                foreign_entry.week_index == entry.week_index
                    && foreign_entry.day_of_week == entry.day_of_week
                    && foreign_entry.period_index == entry.period_index
                    && foreign_entry_matches_admin_foreign_subject(foreign_entry, entry)
            })
            .flat_map(|foreign_entry| {
                foreign_entry
                    .teacher_names
                    .iter()
                    .map(|teacher_name| (foreign_entry.subject.clone(), teacher_name.clone()))
                    .collect::<Vec<_>>()
            })
            .filter(|(_, name)| should_import_teacher_name(name))
            .collect::<Vec<_>>();
        if matched_foreign_teachers.is_empty() {
            continue;
        }

        let mut expanded = Vec::new();
        for name in &entry.teacher_names {
            if should_import_teacher_name(name) {
                expanded.push(name.clone());
                continue;
            }
            let matches = matched_foreign_teachers
                .iter()
                .filter(|(_, candidate)| teacher_surname(candidate).as_deref() == Some(name.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if matches.len() == 1 {
                expanded.push(matches[0].1.clone());
            } else {
                expanded.push(name.clone());
            }
        }
        entry.teacher_names = expanded;
    }
}

fn build_teacher_assignments_from_entries(entries: &[ParsedEntry]) -> Vec<TeacherAssignment> {
    let mut seen = BTreeSet::<(String, &'static str, String)>::new();
    let mut assignments = Vec::new();
    for entry in entries {
        let Some(subject) = subject_from_schedule_label(&entry.subject) else {
            continue;
        };
        for teacher_name in &entry.teacher_names {
            if !should_import_teacher_name(teacher_name) {
                continue;
            }
            let class_name = if is_admin_class(&entry.class_name) {
                normalize_class_code(&entry.class_name)
            } else {
                entry.display_class_name.clone()
            };
            if seen.insert((teacher_name.clone(), subject.as_key(), class_name.clone())) {
                assignments.push(TeacherAssignment {
                    teacher_name: teacher_name.clone(),
                    subject,
                    class_name,
                });
            }
        }
    }
    assignments
}

fn validate_no_short_teacher_names(entries: &[ParsedEntry]) -> Result<(), AppError> {
    let unresolved = entries
        .iter()
        .flat_map(|entry| {
            entry
                .teacher_names
                .iter()
                .filter(|name| !should_import_teacher_name(name))
                .map(|name| {
                    format!(
                        "{} {} {} {} {}({})",
                        entry.day_label,
                        entry.section_label,
                        entry.period_label,
                        entry.display_class_name,
                        entry.subject,
                        name
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if unresolved.is_empty() {
        return Ok(());
    }
    let preview = unresolved
        .iter()
        .take(6)
        .cloned()
        .collect::<Vec<_>>()
        .join("；");
    let suffix = if unresolved.len() > 6 {
        format!(" 等 {} 处", unresolved.len())
    } else {
        format!(" 共 {} 处", unresolved.len())
    };
    Err(AppError::new(format!(
        "课表中仍存在未能推导完整姓名的单姓教师：{preview}{suffix}。请检查对应外语教学班任课教师或教师安排表。"
    )))
}

fn parse_course_workbook(file_path: &str) -> Result<ParsedWorkbook, AppError> {
    let mut workbook = open_workbook_auto(file_path)?;
    let teacher_range = workbook
        .worksheet_range("教师安排")
        .map_err(AppError::from)?;
    let total_range = workbook.worksheet_range("总课表").map_err(AppError::from)?;

    let (teacher_map, _assignments) = parse_teacher_arrangements(&teacher_range);
    let (mut entries, periods, classes) = parse_total_schedule(&total_range, &teacher_map);
    expand_short_foreign_teacher_names(&mut entries);
    validate_no_short_teacher_names(&entries)?;
    let assignments = build_teacher_assignments_from_entries(&entries);
    if entries.is_empty() {
        return Err(AppError::new("总课表中没有识别到可导入的课表数据"));
    }

    Ok(ParsedWorkbook {
        entries,
        periods,
        classes,
        assignments,
    })
}

fn persist_course_import(
    conn: &mut Connection,
    imported_at: &str,
    source_file: &str,
    parsed: &ParsedWorkbook,
) -> Result<i64, AppError> {
    let tx = conn.transaction()?;
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

    tx.execute(
        "INSERT INTO course_schedule_imports (imported_at, source_file, entry_count, teacher_count, admin_class_count, foreign_class_count, start_week) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
        params![
            imported_at,
            source_file,
            parsed.entries.len() as i64,
            teacher_count,
            admin_class_count,
            foreign_class_count
        ],
    )?;
    let import_id = tx.last_insert_rowid();

    {
        let mut class_stmt = tx.prepare(
            "INSERT INTO course_schedule_classes (import_id, class_name, display_name, class_type, sort_index) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for (sort_index, class_option) in parsed.classes.iter().enumerate() {
            class_stmt.execute(params![
                import_id,
                class_option.class_name,
                class_option.display_name,
                class_option.class_type,
                sort_index as i64
            ])?;
        }
    }

    {
        let mut period_stmt = tx.prepare(
            "INSERT INTO course_schedule_periods (import_id, week_index, day_of_week, day_label, period_index, period_label, section_label) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        for period in &parsed.periods {
            period_stmt.execute(params![
                import_id,
                period.week_index,
                period.day_of_week,
                period.day_label,
                period.period_index,
                period.period_label,
                period.section_label,
            ])?;
        }
    }

    {
        let mut entry_stmt = tx.prepare(
            "INSERT INTO course_schedule_entries (import_id, class_name, display_class_name, class_type, week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, teacher_search_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )?;
        for entry in &parsed.entries {
            let teacher_text = entry.teacher_names.join("/");
            entry_stmt.execute(params![
                import_id,
                entry.class_name,
                entry.display_class_name,
                entry.class_type,
                entry.week_index,
                entry.day_of_week,
                entry.day_label,
                entry.period_index,
                entry.period_label,
                entry.section_label,
                entry.subject,
                teacher_text,
                teacher_text,
            ])?;
        }
    }

    replace_teacher_assignments_from_schedule(&tx, &parsed.assignments)?;
    tx.commit()?;
    Ok(import_id)
}

fn replace_teacher_assignments_from_schedule(
    tx: &rusqlite::Transaction<'_>,
    assignments: &[TeacherAssignment],
) -> Result<(), AppError> {
    tx.execute("DELETE FROM latest_teacher_assignments_v2", [])?;
    let imported_at = Utc::now().to_rfc3339();
    tx.execute(
        "INSERT OR REPLACE INTO latest_teacher_import_meta (id, imported_at, source_file, row_count) VALUES (1, ?1, '课表导入同步', ?2)",
        params![imported_at, assignments.iter().map(|item| item.teacher_name.clone()).collect::<HashSet<_>>().len() as i64],
    )?;

    let mut teacher_names = BTreeSet::new();
    let mut seen = BTreeSet::<(String, &'static str, String)>::new();
    for assignment in assignments {
        teacher_names.insert(assignment.teacher_name.clone());
        seen.insert((
            assignment.teacher_name.clone(),
            assignment.subject.as_key(),
            assignment.class_name.clone(),
        ));
    }

    remove_teachers_not_in_schedule(tx, &teacher_names)?;

    {
        let mut teacher_stmt = tx.prepare(
            "INSERT OR IGNORE INTO latest_teachers_v2 (teacher_name, remark, is_middle_manager) VALUES (?1, NULL, 0)",
        )?;
        for teacher_name in &teacher_names {
            teacher_stmt.execute(params![teacher_name])?;
        }
    }

    let mut teacher_ids = HashMap::new();
    {
        let mut teacher_query = tx.prepare("SELECT id, teacher_name FROM latest_teachers_v2")?;
        let rows = teacher_query.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, name) = row?;
            teacher_ids.insert(name, id);
        }
    }

    {
        let mut assignment_stmt = tx.prepare(
            "INSERT OR IGNORE INTO latest_teacher_assignments_v2 (teacher_id, subject, class_name) VALUES (?1, ?2, ?3)",
        )?;
        for (teacher_name, subject, class_name) in seen {
            let teacher_id = teacher_ids
                .get(&teacher_name)
                .copied()
                .ok_or_else(|| AppError::new(format!("未找到教师: {teacher_name}")))?;
            assignment_stmt.execute(params![teacher_id, subject, class_name])?;
        }
    }
    Ok(())
}

fn remove_teachers_not_in_schedule(
    tx: &rusqlite::Transaction<'_>,
    teacher_names: &BTreeSet<String>,
) -> Result<(), AppError> {
    if teacher_names.is_empty() {
        tx.execute("DELETE FROM latest_teacher_homerooms_v2", [])?;
        tx.execute("DELETE FROM latest_teachers_v2", [])?;
        return Ok(());
    }

    let placeholders = std::iter::repeat("?")
        .take(teacher_names.len())
        .collect::<Vec<_>>()
        .join(", ");
    let names = teacher_names.iter().map(String::as_str).collect::<Vec<_>>();

    // Teacher import is a snapshot sync: teachers missing from the imported
    // timetable are removed so the system list stays aligned with the course file.
    tx.execute(
        &format!(
            "DELETE FROM latest_teacher_homerooms_v2 WHERE teacher_id IN (SELECT id FROM latest_teachers_v2 WHERE teacher_name NOT IN ({placeholders}))"
        ),
        params_from_iter(names.iter().copied()),
    )?;
    tx.execute(
        &format!("DELETE FROM latest_teachers_v2 WHERE teacher_name NOT IN ({placeholders})"),
        params_from_iter(names.iter().copied()),
    )?;
    Ok(())
}

fn latest_import_id(conn: &Connection) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT id FROM course_schedule_imports ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(AppError::from)
}

fn normalize_optional_date(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn map_import_batch(row: &rusqlite::Row<'_>) -> rusqlite::Result<CourseImportBatch> {
    Ok(CourseImportBatch {
        id: row.get(0)?,
        imported_at: row.get(1)?,
        source_file: row.get(2)?,
        entry_count: row.get(3)?,
        teacher_count: row.get(4)?,
        admin_class_count: row.get(5)?,
        foreign_class_count: row.get(6)?,
        effective_start_date: row.get(7)?,
        effective_end_date: row.get(8)?,
        start_week: row.get(9)?,
    })
}

fn get_import_batch(conn: &Connection, import_id: i64) -> Result<CourseImportBatch, AppError> {
    conn.query_row(
        "SELECT id, imported_at, source_file, entry_count, teacher_count, admin_class_count, foreign_class_count, effective_start_date, effective_end_date, start_week FROM course_schedule_imports WHERE id = ?1",
        params![import_id],
        map_import_batch,
    )
    .map_err(AppError::from)
}

fn list_period_slots(conn: &Connection, import_id: i64) -> Result<Vec<CoursePeriodSlot>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT week_index, day_of_week, day_label, period_index, period_label, section_label FROM course_schedule_periods WHERE import_id = ?1 ORDER BY week_index, day_of_week, period_index",
    )?;
    let rows = stmt.query_map(params![import_id], |row| {
        Ok(CoursePeriodSlot {
            week_index: row.get(0)?,
            day_of_week: row.get(1)?,
            day_label: row.get(2)?,
            period_index: row.get(3)?,
            period_label: row.get(4)?,
            section_label: row.get(5)?,
        })
    })?;
    let mut periods = Vec::new();
    for row in rows {
        periods.push(row?);
    }
    Ok(periods)
}

fn parse_iso_date(value: &str, field_name: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::new(format!("{field_name}格式不正确")))
}

fn date_range_inclusive(start: NaiveDate, end: NaiveDate) -> Result<Vec<NaiveDate>, AppError> {
    if end < start {
        return Err(AppError::new("结束日期不能早于开始日期"));
    }
    let days = (end - start).num_days();
    if days > 62 {
        return Err(AppError::new("单次查询范围不能超过 63 天"));
    }
    Ok((0..=days).map(|offset| start + Duration::days(offset)).collect())
}

fn get_import_anchor(conn: &Connection, import_id: i64) -> Result<(NaiveDate, i64), AppError> {
    let (effective_start_date, start_week): (Option<String>, i64) = conn.query_row(
        "SELECT effective_start_date, start_week FROM course_schedule_imports WHERE id = ?1",
        params![import_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let Some(start_date_text) = effective_start_date else {
        return Err(AppError::new("请先在课务管理中设置该课表批次的生效开始日期"));
    };
    let start_date = parse_iso_date(&start_date_text, "生效开始日期")?;
    Ok((start_date, start_week.max(1)))
}

fn get_schedule_week_count(conn: &Connection, import_id: i64) -> Result<i64, AppError> {
    let week_count = conn
        .query_row(
            "SELECT COALESCE(MAX(week_index), 1) FROM course_schedule_periods WHERE import_id = ?1",
            params![import_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1)
        .max(1);
    Ok(week_count)
}

fn schedule_slot_for_date(
    date: NaiveDate,
    anchor_date: NaiveDate,
    start_week: i64,
    week_count: i64,
) -> Result<(i64, i64), AppError> {
    let elapsed_days = (date - anchor_date).num_days();
    if elapsed_days < 0 {
        return Err(AppError::new("查询日期早于课表批次生效开始日期"));
    }
    let elapsed_weeks = elapsed_days / 7;
    let week_index = ((start_week - 1 + elapsed_weeks) % week_count) + 1;
    let day_of_week = date.weekday().number_from_monday() as i64;
    Ok((week_index, day_of_week))
}

fn period_bounds_for_date(
    date: NaiveDate,
    start_date: NaiveDate,
    end_date: NaiveDate,
    start_period_index: Option<i64>,
    end_period_index: Option<i64>,
) -> (i64, i64) {
    let mut start_period = 1;
    let mut end_period = i64::MAX;
    if date == start_date {
        start_period = start_period_index.unwrap_or(1).max(1);
    }
    if date == end_date {
        end_period = end_period_index.unwrap_or(i64::MAX).max(1);
    }
    if end_period < start_period {
        end_period = start_period;
    }
    (start_period, end_period)
}

fn workload_category(section_label: &str, period_label: &str) -> &'static str {
    let text = format!("{section_label}{period_label}");
    if text.contains("晚") {
        "晚上"
    } else if text.contains("下午") || text.contains("午练") || text.contains("午间") {
        "下午"
    } else if text.contains("上午") || text.contains("大课间") {
        "上午"
    } else if text.contains("早") || text.contains("晨") {
        "早上"
    } else {
        "上午"
    }
}

fn map_course_change_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CourseScheduleChange> {
    Ok(CourseScheduleChange {
        id: row.get(0)?,
        import_id: row.get(1)?,
        source_entry_id: row.get(2)?,
        change_type: row.get(3)?,
        status: row.get(4)?,
        target_date: row.get(5)?,
        source_teacher_name: row.get(6)?,
        actual_teacher_name: row.get(7)?,
        reason: row.get(8)?,
        remark: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        revoked_at: row.get(12)?,
        week_index: row.get(13)?,
        day_of_week: row.get(14)?,
        day_label: row.get(15)?,
        period_index: row.get(16)?,
        period_label: row.get(17)?,
        section_label: row.get(18)?,
        subject: row.get(19)?,
        class_name: row.get(20)?,
        display_class_name: row.get(21)?,
        class_type: row.get(22)?,
    })
}

fn get_active_change_for_slot(
    conn: &Connection,
    source_entry_id: i64,
    target_date: &str,
    source_teacher_name: &str,
) -> Result<Option<CourseScheduleChange>, AppError> {
    conn.query_row(
        "SELECT c.id, c.import_id, c.source_entry_id, c.change_type, c.status, c.target_date, c.source_teacher_name, c.actual_teacher_name, c.reason, c.remark, c.created_at, c.updated_at, c.revoked_at, e.week_index, e.day_of_week, e.day_label, e.period_index, e.period_label, e.section_label, e.subject, e.class_name, e.display_class_name, e.class_type FROM course_schedule_changes c INNER JOIN course_schedule_entries e ON e.id = c.source_entry_id WHERE c.source_entry_id = ?1 AND c.target_date = ?2 AND c.source_teacher_name = ?3 AND c.status = 'active' ORDER BY c.id DESC LIMIT 1",
        params![source_entry_id, target_date, source_teacher_name],
        map_course_change_row,
    )
    .optional()
    .map_err(AppError::from)
}

fn list_changes_for_import(conn: &Connection, import_id: i64) -> Result<Vec<CourseScheduleChange>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.import_id, c.source_entry_id, c.change_type, c.status, c.target_date, c.source_teacher_name, c.actual_teacher_name, c.reason, c.remark, c.created_at, c.updated_at, c.revoked_at, e.week_index, e.day_of_week, e.day_label, e.period_index, e.period_label, e.section_label, e.subject, e.class_name, e.display_class_name, e.class_type FROM course_schedule_changes c INNER JOIN course_schedule_entries e ON e.id = c.source_entry_id WHERE c.import_id = ?1 ORDER BY c.target_date DESC, e.period_index ASC, c.id DESC",
    )?;
    let rows = stmt.query_map(params![import_id], map_course_change_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

fn active_changes_for_date(
    conn: &Connection,
    import_id: i64,
    target_date: &str,
) -> Result<HashMap<(i64, String), CourseScheduleChange>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.import_id, c.source_entry_id, c.change_type, c.status, c.target_date, c.source_teacher_name, c.actual_teacher_name, c.reason, c.remark, c.created_at, c.updated_at, c.revoked_at, e.week_index, e.day_of_week, e.day_label, e.period_index, e.period_label, e.section_label, e.subject, e.class_name, e.display_class_name, e.class_type FROM course_schedule_changes c INNER JOIN course_schedule_entries e ON e.id = c.source_entry_id WHERE c.import_id = ?1 AND c.target_date = ?2 AND c.status = 'active'",
    )?;
    let rows = stmt.query_map(params![import_id, target_date], map_course_change_row)?;
    let mut changes = HashMap::new();
    for row in rows {
        let change = row?;
        changes.insert(
            (change.source_entry_id, change.source_teacher_name.clone()),
            change,
        );
    }
    Ok(changes)
}

fn build_course_workload_report(
    conn: &Connection,
    query: &CourseWorkloadQuery,
) -> Result<CourseWorkloadReport, AppError> {
    if query.import_id <= 0 {
        return Err(AppError::new("请选择课表批次"));
    }
    let start_date = parse_iso_date(&query.start_date, "开始日期")?;
    let end_date = parse_iso_date(&query.end_date, "结束日期")?;
    let dates = date_range_inclusive(start_date, end_date)?;
    let (anchor_date, start_week) = get_import_anchor(conn, query.import_id)?;
    let week_count = get_schedule_week_count(conn, query.import_id)?;

    let mut details = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, class_name, display_class_name, class_type FROM course_schedule_entries WHERE import_id = ?1 AND week_index = ?2 AND day_of_week = ?3 AND period_index BETWEEN ?4 AND ?5 ORDER BY day_of_week, period_index, class_name",
    )?;

    for date in dates {
        let (week_index, day_of_week) =
            schedule_slot_for_date(date, anchor_date, start_week, week_count)?;
        let (start_period, end_period) = period_bounds_for_date(
            date,
            start_date,
            end_date,
            query.start_period_index,
            query.end_period_index,
        );
        let date_text = date.format("%Y-%m-%d").to_string();
        let changes = active_changes_for_date(conn, query.import_id, &date_text)?;
        let rows = stmt.query_map(
            params![
                query.import_id,
                week_index,
                day_of_week,
                start_period,
                end_period
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                ))
            },
        )?;
        for row in rows {
            let (
                entry_id,
                _week_index,
                _day_of_week,
                day_label,
                period_index,
                period_label,
                section_label,
                subject,
                teacher_text,
                class_name,
                display_class_name,
                _class_type,
            ) = row?;
            let category = workload_category(&section_label, &period_label).to_string();
            for original_teacher_name in split_teacher_names(&teacher_text) {
                if !should_import_teacher_name(&original_teacher_name) {
                    continue;
                }
                let change = changes.get(&(entry_id, original_teacher_name.clone()));
                let actual_teacher_name = change
                    .map(|item| item.actual_teacher_name.clone())
                    .unwrap_or_else(|| original_teacher_name.clone());
                let is_substitution = actual_teacher_name != original_teacher_name;
                let remark = if let Some(change) = change {
                    let mut parts = vec![format!("代 {} 老师", change.source_teacher_name)];
                    if !change.reason.trim().is_empty() {
                        parts.push(change.reason.clone());
                    }
                    if !change.remark.trim().is_empty() {
                        parts.push(change.remark.clone());
                    }
                    parts.join(" / ")
                } else {
                    String::new()
                };
                details.push(CourseWorkloadDetail {
                    teacher_name: actual_teacher_name.clone(),
                    target_date: date_text.clone(),
                    day_label: day_label.clone(),
                    period_index,
                    period_label: period_label.clone(),
                    section_label: section_label.clone(),
                    category: category.clone(),
                    subject: subject.clone(),
                    class_name: class_name.clone(),
                    display_class_name: display_class_name.clone(),
                    original_teacher_name: original_teacher_name.clone(),
                    actual_teacher_name,
                    is_substitution,
                    remark,
                });
            }
        }
    }

    details.sort_by(|a, b| {
        a.teacher_name
            .cmp(&b.teacher_name)
            .then(a.target_date.cmp(&b.target_date))
            .then(a.period_index.cmp(&b.period_index))
            .then(a.display_class_name.cmp(&b.display_class_name))
    });

    let mut summary_map = BTreeMap::<String, CourseWorkloadSummary>::new();
    for detail in &details {
        let summary = summary_map
            .entry(detail.teacher_name.clone())
            .or_insert_with(|| CourseWorkloadSummary {
                teacher_name: detail.teacher_name.clone(),
                morning_reading_count: 0,
                morning_count: 0,
                afternoon_count: 0,
                evening_count: 0,
                substitution_count: 0,
                total_count: 0,
            });
        match detail.category.as_str() {
            "早上" => summary.morning_reading_count += 1,
            "下午" => summary.afternoon_count += 1,
            "晚上" => summary.evening_count += 1,
            _ => summary.morning_count += 1,
        }
        if detail.is_substitution {
            summary.substitution_count += 1;
        }
        summary.total_count += 1;
    }

    Ok(CourseWorkloadReport {
        import_id: query.import_id,
        start_date: query.start_date.clone(),
        end_date: query.end_date.clone(),
        details,
        summaries: summary_map.into_values().collect(),
    })
}

fn sanitize_file_name_segment(value: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    value
        .chars()
        .map(|ch| if invalid.contains(&ch) { '_' } else { ch })
        .collect::<String>()
        .trim()
        .to_string()
}

fn course_export_root_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取应用数据目录失败: {e}")))?;
    dir.push("exports");
    dir.push("course-workload");
    Ok(dir)
}

fn build_workload_formats() -> (Format, Format, Format, Format, Format) {
    let title_fmt = Format::new()
        .set_bold()
        .set_font_size(16.)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xDDEBF7))
        .set_border(FormatBorder::Thin);
    let header_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xE2F0D9))
        .set_border(FormatBorder::Thin);
    let teacher_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xF2F2F2))
        .set_border(FormatBorder::Thin);
    let cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let wrap_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_border(FormatBorder::Thin);
    (title_fmt, header_fmt, teacher_fmt, cell_fmt, wrap_fmt)
}

fn write_workload_detail_sheet(
    sheet: &mut Worksheet,
    report: &CourseWorkloadReport,
) -> Result<(), XlsxError> {
    let (title_fmt, header_fmt, teacher_fmt, cell_fmt, wrap_fmt) = build_workload_formats();
    sheet.set_name("课时明细")?;
    sheet.merge_range(
        0,
        0,
        0,
        10,
        &format!("课时明细（{} 至 {}）", report.start_date, report.end_date),
        &title_fmt,
    )?;
    let headers = [
        "教师", "日期", "星期", "节次", "时段", "班级", "科目", "原任课教师", "实际授课教师", "备注", "课时",
    ];
    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(1, col as u16, *header, &header_fmt)?;
    }

    let mut row = 2_u32;
    let mut index = 0_usize;
    while index < report.details.len() {
        let teacher = &report.details[index].teacher_name;
        let group_start = row;
        let mut group_end = row;
        while index < report.details.len() && &report.details[index].teacher_name == teacher {
            let detail = &report.details[index];
            sheet.write_string_with_format(row, 1, &detail.target_date, &cell_fmt)?;
            sheet.write_string_with_format(row, 2, &detail.day_label, &cell_fmt)?;
            sheet.write_string_with_format(row, 3, &detail.period_label, &cell_fmt)?;
            sheet.write_string_with_format(row, 4, &detail.category, &cell_fmt)?;
            sheet.write_string_with_format(row, 5, &detail.display_class_name, &cell_fmt)?;
            sheet.write_string_with_format(row, 6, &detail.subject, &cell_fmt)?;
            sheet.write_string_with_format(row, 7, &detail.original_teacher_name, &cell_fmt)?;
            sheet.write_string_with_format(row, 8, &detail.actual_teacher_name, &cell_fmt)?;
            sheet.write_string_with_format(row, 9, &detail.remark, &wrap_fmt)?;
            sheet.write_number_with_format(row, 10, 1.0, &cell_fmt)?;
            group_end = row;
            row += 1;
            index += 1;
        }
        if group_end > group_start {
            sheet.merge_range(group_start, 0, group_end, 0, teacher, &teacher_fmt)?;
        } else {
            sheet.write_string_with_format(group_start, 0, teacher, &teacher_fmt)?;
        }
    }

    let widths = [12., 12., 10., 10., 10., 14., 12., 12., 12., 26., 8.];
    for (col, width) in widths.iter().enumerate() {
        sheet.set_column_width(col as u16, *width)?;
    }
    Ok(())
}

fn write_workload_summary_sheet(
    sheet: &mut Worksheet,
    report: &CourseWorkloadReport,
) -> Result<(), XlsxError> {
    let (title_fmt, header_fmt, _teacher_fmt, cell_fmt, _wrap_fmt) = build_workload_formats();
    sheet.set_name("分类汇总")?;
    sheet.merge_range(
        0,
        0,
        0,
        6,
        &format!("课时分类汇总（{} 至 {}）", report.start_date, report.end_date),
        &title_fmt,
    )?;
    let headers = ["教师", "早上", "上午", "下午", "晚上", "代课节数", "合计"];
    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(1, col as u16, *header, &header_fmt)?;
    }
    for (idx, summary) in report.summaries.iter().enumerate() {
        let row = 2_u32 + idx as u32;
        sheet.write_string_with_format(row, 0, &summary.teacher_name, &cell_fmt)?;
        sheet.write_number_with_format(row, 1, summary.morning_reading_count as f64, &cell_fmt)?;
        sheet.write_number_with_format(row, 2, summary.morning_count as f64, &cell_fmt)?;
        sheet.write_number_with_format(row, 3, summary.afternoon_count as f64, &cell_fmt)?;
        sheet.write_number_with_format(row, 4, summary.evening_count as f64, &cell_fmt)?;
        sheet.write_number_with_format(row, 5, summary.substitution_count as f64, &cell_fmt)?;
        sheet.write_number_with_format(row, 6, summary.total_count as f64, &cell_fmt)?;
    }
    for (col, width) in [14., 10., 10., 10., 10., 10., 10.].iter().enumerate() {
        sheet.set_column_width(col as u16, *width)?;
    }
    Ok(())
}

fn save_workload_report(
    app: &AppHandle,
    report: &CourseWorkloadReport,
) -> Result<ExportCourseWorkloadResult, AppError> {
    if report.details.is_empty() {
        return Err(AppError::new("暂无可导出的课时明细"));
    }
    let mut workbook = Workbook::new();
    let detail_sheet = workbook.add_worksheet();
    write_workload_detail_sheet(detail_sheet, report)
        .map_err(|e| AppError::new(format!("写入课时明细失败: {e}")))?;
    let summary_sheet = workbook.add_worksheet();
    write_workload_summary_sheet(summary_sheet, report)
        .map_err(|e| AppError::new(format!("写入课时汇总失败: {e}")))?;

    let output_dir = course_export_root_dir(app)?;
    fs::create_dir_all(&output_dir).map_err(|e| AppError::new(format!("创建导出目录失败: {e}")))?;
    let exported_at = Utc::now().to_rfc3339();
    let file_name = sanitize_file_name_segment(&format!(
        "课时统计-{}至{}.xlsx",
        report.start_date, report.end_date
    ));
    let path = output_dir.join(file_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppError::new(format!("覆盖旧课时统计文件失败: {e}")))?;
    }
    workbook
        .save(&path)
        .map_err(|e| AppError::new(format!("保存课时统计文件失败: {e}")))?;
    Ok(ExportCourseWorkloadResult {
        file_path: path.to_string_lossy().to_string(),
        exported_at,
    })
}

#[tauri::command]
pub fn import_course_schedule_from_excel(
    app: AppHandle,
    file_path: String,
) -> Result<CourseImportResult, String> {
    let start = Utc::now();
    let result = (|| -> Result<CourseImportResult, AppError> {
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let parsed = parse_course_workbook(&file_path)?;
        let imported_at = Utc::now().to_rfc3339();
        persist_course_import(&mut conn, &imported_at, &file_path, &parsed)?;
        Ok(CourseImportResult {
            imported_at,
            entry_count: parsed.entries.len() as i64,
            teacher_count: parsed
                .assignments
                .iter()
                .map(|item| item.teacher_name.clone())
                .collect::<HashSet<_>>()
                .len() as i64,
            admin_class_count: parsed
                .classes
                .iter()
                .filter(|item| item.class_type == "admin")
                .count() as i64,
            foreign_class_count: parsed
                .classes
                .iter()
                .filter(|item| item.class_type == "foreign")
                .count() as i64,
            duration_ms: (Utc::now() - start).num_milliseconds(),
        })
    })();
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "course_management.import_course_schedule_from_excel",
            &format!("file_path={file_path} | {e}"),
        );
        e.to_string()
    })
}

#[tauri::command]
pub fn get_course_schedule_summary(app: AppHandle) -> Result<CourseSummary, String> {
    let result = (|| -> Result<CourseSummary, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let Some(import_id) = latest_import_id(&conn)? else {
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
        conn.query_row(
            "SELECT imported_at, entry_count, teacher_count, admin_class_count, foreign_class_count, effective_start_date, effective_end_date, start_week FROM course_schedule_imports WHERE id = ?1",
            params![import_id],
            |row| {
                Ok(CourseSummary {
                    latest_import_id: Some(import_id),
                    imported_at: Some(row.get(0)?),
                    entry_count: row.get(1)?,
                    teacher_count: row.get(2)?,
                    admin_class_count: row.get(3)?,
                    foreign_class_count: row.get(4)?,
                    effective_start_date: row.get(5)?,
                    effective_end_date: row.get(6)?,
                    start_week: row.get(7)?,
                })
            },
        )
        .map_err(AppError::from)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_schedule_classes(
    app: AppHandle,
    class_type: String,
    import_id: Option<i64>,
) -> Result<Vec<CourseClassOption>, String> {
    let result = (|| -> Result<Vec<CourseClassOption>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let selected_import_id = match import_id {
            Some(id) => id,
            None => latest_import_id(&conn)?.unwrap_or(0),
        };
        if selected_import_id == 0 {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT class_name, display_name, class_type FROM course_schedule_classes WHERE import_id = ?1 AND class_type = ?2 ORDER BY sort_index ASC",
        )?;
        let rows = stmt.query_map(params![selected_import_id, class_type], |row| {
            Ok(CourseClassOption {
                class_name: row.get(0)?,
                display_name: row.get(1)?,
                class_type: row.get(2)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_schedule_imports(app: AppHandle) -> Result<Vec<CourseImportBatch>, String> {
    let result = (|| -> Result<Vec<CourseImportBatch>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, imported_at, source_file, entry_count, teacher_count, admin_class_count, foreign_class_count, effective_start_date, effective_end_date, start_week FROM course_schedule_imports ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], map_import_batch)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_course_schedule_import_settings(
    app: AppHandle,
    payload: CourseImportSettingsPayload,
) -> Result<CourseImportBatch, String> {
    let result = (|| -> Result<CourseImportBatch, AppError> {
        if payload.import_id <= 0 {
            return Err(AppError::new("请选择要设置的课表批次"));
        }
        if payload.start_week < 1 {
            return Err(AppError::new("起始周不能小于 1"));
        }
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        conn.execute(
            "UPDATE course_schedule_imports SET effective_start_date = ?1, effective_end_date = ?2, start_week = ?3 WHERE id = ?4",
            params![
                normalize_optional_date(payload.effective_start_date),
                normalize_optional_date(payload.effective_end_date),
                payload.start_week,
                payload.import_id,
            ],
        )?;
        get_import_batch(&conn, payload.import_id)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_course_schedule_import(app: AppHandle, import_id: i64) -> Result<(), String> {
    let result = (|| -> Result<(), AppError> {
        if import_id <= 0 {
            return Err(AppError::new("请选择要删除的课表批次"));
        }
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM course_schedule_entries WHERE import_id = ?1",
            params![import_id],
        )?;
        tx.execute(
            "DELETE FROM course_schedule_periods WHERE import_id = ?1",
            params![import_id],
        )?;
        tx.execute(
            "DELETE FROM course_schedule_classes WHERE import_id = ?1",
            params![import_id],
        )?;
        tx.execute(
            "DELETE FROM course_schedule_imports WHERE id = ?1",
            params![import_id],
        )?;
        tx.commit()?;
        Ok(())
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_schedule_teachers(
    app: AppHandle,
    import_id: Option<i64>,
) -> Result<Vec<String>, String> {
    let result = (|| -> Result<Vec<String>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let selected_import_id = match import_id {
            Some(id) => id,
            None => latest_import_id(&conn)?.unwrap_or(0),
        };
        if selected_import_id == 0 {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT DISTINCT teacher_names FROM course_schedule_entries WHERE import_id = ?1 AND teacher_names <> '' ORDER BY teacher_names ASC",
        )?;
        let rows = stmt.query_map(params![selected_import_id], |row| row.get::<_, String>(0))?;
        let mut names = BTreeSet::new();
        for row in rows {
            for name in split_teacher_names(&row?) {
                names.insert(name);
            }
        }
        Ok(names.into_iter().collect())
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_schedule_periods(
    app: AppHandle,
    import_id: Option<i64>,
) -> Result<Vec<CoursePeriodSlot>, String> {
    let result = (|| -> Result<Vec<CoursePeriodSlot>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let selected_import_id = match import_id {
            Some(id) => id,
            None => latest_import_id(&conn)?.unwrap_or(0),
        };
        if selected_import_id == 0 {
            return Ok(Vec::new());
        }
        let all_periods = list_period_slots(&conn, selected_import_id)?;
        let mut seen = BTreeSet::new();
        let mut unique_periods = Vec::new();
        for period in all_periods {
            if seen.insert(period.period_index) {
                unique_periods.push(period);
            }
        }
        Ok(unique_periods)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_substitution_candidates(
    app: AppHandle,
    query: CourseSubstitutionCandidateQuery,
) -> Result<Vec<CourseSubstitutionCandidate>, String> {
    let result = (|| -> Result<Vec<CourseSubstitutionCandidate>, AppError> {
        if query.import_id <= 0 {
            return Err(AppError::new("请选择课表批次"));
        }
        let teacher_name = query.teacher_name.trim().to_string();
        if teacher_name.is_empty() {
            return Err(AppError::new("请选择需要换课的教师"));
        }
        let start_date = parse_iso_date(&query.start_date, "开始日期")?;
        let end_date = parse_iso_date(&query.end_date, "结束日期")?;
        let dates = date_range_inclusive(start_date, end_date)?;

        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let (anchor_date, start_week) = get_import_anchor(&conn, query.import_id)?;
        let week_count = get_schedule_week_count(&conn, query.import_id)?;

        let mut candidates = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT id, import_id, week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, class_name, display_class_name, class_type FROM course_schedule_entries WHERE import_id = ?1 AND week_index = ?2 AND day_of_week = ?3 AND period_index BETWEEN ?4 AND ?5 AND teacher_search_text LIKE ?6 ORDER BY period_index, class_name",
        )?;
        let selected_period_indexes = query
            .period_indexes
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .copied()
                    .filter(|item| *item > 0)
                    .collect::<BTreeSet<_>>()
            })
            .filter(|items| !items.is_empty());
        for date in dates {
            let (week_index, day_of_week) =
                schedule_slot_for_date(date, anchor_date, start_week, week_count)?;
            let (start_period, end_period) =
                if let Some(period_indexes) = selected_period_indexes.as_ref() {
                    (
                        *period_indexes.iter().next().unwrap_or(&1),
                        *period_indexes.iter().next_back().unwrap_or(&i64::MAX),
                    )
                } else {
                    period_bounds_for_date(
                        date,
                        start_date,
                        end_date,
                        query.start_period_index,
                        query.end_period_index,
                    )
                };
            let date_text = date.format("%Y-%m-%d").to_string();
            let rows = stmt.query_map(
                params![
                    query.import_id,
                    week_index,
                    day_of_week,
                    start_period,
                    end_period,
                    format!("%{teacher_name}%")
                ],
                |row| {
                    let teacher_text: String = row.get(9)?;
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        teacher_text,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                    ))
                },
            )?;
            for row in rows {
                let (
                    source_entry_id,
                    import_id,
                    week_index,
                    day_of_week,
                    day_label,
                    period_index,
                    period_label,
                    section_label,
                    subject,
                    teacher_text,
                    class_name,
                    display_class_name,
                    class_type,
                ) = row?;
                let teacher_names = split_teacher_names(&teacher_text);
                if let Some(period_indexes) = selected_period_indexes.as_ref() {
                    if !period_indexes.contains(&period_index) {
                        continue;
                    }
                }
                if !teacher_names.iter().any(|name| name == &teacher_name) {
                    continue;
                }
                let existing_change =
                    get_active_change_for_slot(&conn, source_entry_id, &date_text, &teacher_name)?;
                candidates.push(CourseSubstitutionCandidate {
                    source_entry_id,
                    import_id,
                    target_date: date_text.clone(),
                    week_index,
                    day_of_week,
                    day_label,
                    period_index,
                    period_label,
                    section_label,
                    subject,
                    teacher_names,
                    source_teacher_name: teacher_name.clone(),
                    class_name,
                    display_class_name,
                    class_type,
                    existing_change,
                });
            }
        }
        Ok(candidates)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_course_substitutions(
    app: AppHandle,
    payload: SaveCourseSubstitutionsPayload,
) -> Result<Vec<CourseScheduleChange>, String> {
    let result = (|| -> Result<Vec<CourseScheduleChange>, AppError> {
        if payload.import_id <= 0 {
            return Err(AppError::new("请选择课表批次"));
        }
        if payload.items.is_empty() {
            return Err(AppError::new("请选择需要保存的换课记录"));
        }
        let mut conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        for item in &payload.items {
            let source_teacher_name = item.source_teacher_name.trim();
            let actual_teacher_name = item.actual_teacher_name.trim();
            if source_teacher_name.is_empty() || actual_teacher_name.is_empty() {
                return Err(AppError::new("原任课教师和代课教师不能为空"));
            }
            if source_teacher_name == actual_teacher_name {
                return Err(AppError::new("代课教师不能与原任课教师相同"));
            }
            parse_iso_date(&item.target_date, "换课日期")?;
            let teacher_text: String = tx.query_row(
                "SELECT teacher_names FROM course_schedule_entries WHERE id = ?1 AND import_id = ?2",
                params![item.source_entry_id, payload.import_id],
                |row| row.get(0),
            )?;
            if !split_teacher_names(&teacher_text)
                .iter()
                .any(|name| name == source_teacher_name)
            {
                return Err(AppError::new(format!(
                    "课次中未找到原任课教师: {source_teacher_name}"
                )));
            }
            let existing_id = tx
                .query_row(
                    "SELECT id FROM course_schedule_changes WHERE import_id = ?1 AND source_entry_id = ?2 AND target_date = ?3 AND source_teacher_name = ?4 AND status = 'active' ORDER BY id DESC LIMIT 1",
                    params![payload.import_id, item.source_entry_id, item.target_date, source_teacher_name],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?;
            let item_remark = item
                .remark
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| payload.remark.trim().to_string());
            if let Some(change_id) = existing_id {
                tx.execute(
                    "UPDATE course_schedule_changes SET actual_teacher_name = ?1, reason = ?2, remark = ?3, updated_at = ?4 WHERE id = ?5",
                    params![
                        actual_teacher_name,
                        payload.reason.trim(),
                        item_remark,
                        now,
                        change_id,
                    ],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO course_schedule_changes (import_id, source_entry_id, change_type, status, target_date, source_teacher_name, actual_teacher_name, reason, remark, created_at, updated_at) VALUES (?1, ?2, 'substitute', 'active', ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                    params![
                        payload.import_id,
                        item.source_entry_id,
                        item.target_date,
                        source_teacher_name,
                        actual_teacher_name,
                        payload.reason.trim(),
                        item_remark,
                        now,
                    ],
                )?;
            }
        }
        tx.commit()?;
        list_changes_for_import(&conn, payload.import_id)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_course_schedule_changes(
    app: AppHandle,
    import_id: Option<i64>,
) -> Result<Vec<CourseScheduleChange>, String> {
    let result = (|| -> Result<Vec<CourseScheduleChange>, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let selected_import_id = match import_id {
            Some(id) => id,
            None => latest_import_id(&conn)?.unwrap_or(0),
        };
        if selected_import_id == 0 {
            return Ok(Vec::new());
        }
        list_changes_for_import(&conn, selected_import_id)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn revoke_course_schedule_change(app: AppHandle, change_id: i64) -> Result<(), String> {
    let result = (|| -> Result<(), AppError> {
        if change_id <= 0 {
            return Err(AppError::new("请选择要撤销的换课记录"));
        }
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE course_schedule_changes SET status = 'revoked', revoked_at = ?1, updated_at = ?1 WHERE id = ?2 AND status = 'active'",
            params![now, change_id],
        )?;
        Ok(())
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_course_workload_report(
    app: AppHandle,
    query: CourseWorkloadQuery,
) -> Result<CourseWorkloadReport, String> {
    let result = (|| -> Result<CourseWorkloadReport, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        build_course_workload_report(&conn, &query)
    })();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_course_workload_report(
    app: AppHandle,
    query: CourseWorkloadQuery,
) -> Result<ExportCourseWorkloadResult, String> {
    let result = (|| -> Result<ExportCourseWorkloadResult, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let report = build_course_workload_report(&conn, &query)?;
        save_workload_report(&app, &report)
    })();
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "course_management.export_course_workload_report",
            &e.to_string(),
        );
        e.to_string()
    })
}

#[tauri::command]
pub fn get_course_schedule_view(
    app: AppHandle,
    query: CourseScheduleQuery,
) -> Result<CourseScheduleView, String> {
    let result = (|| -> Result<CourseScheduleView, AppError> {
        let conn = score::open_connection(&app)?;
        ensure_schema(&conn)?;
        let import_id = query
            .import_id
            .or(latest_import_id(&conn)?)
            .ok_or_else(|| AppError::new("还没有导入课表"))?;
        let target = query.target.trim().to_string();
        if target.is_empty() {
            return Err(AppError::new("请选择要查看的教师或班级"));
        }

        let (sql, params_values): (&str, Vec<String>) = match query.view_type.as_str() {
            "teacher" => (
                "SELECT week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, class_name, display_class_name, class_type FROM course_schedule_entries WHERE import_id = ?1 AND teacher_search_text LIKE ?2 ORDER BY week_index, day_of_week, period_index, class_name",
                vec![import_id.to_string(), format!("%{target}%")],
            ),
            "foreign_class" => (
                "SELECT week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, class_name, display_class_name, class_type FROM course_schedule_entries WHERE import_id = ?1 AND class_type = 'foreign' AND class_name = ?2 ORDER BY week_index, day_of_week, period_index",
                vec![import_id.to_string(), target.clone()],
            ),
            _ => (
                "SELECT week_index, day_of_week, day_label, period_index, period_label, section_label, subject, teacher_names, class_name, display_class_name, class_type FROM course_schedule_entries WHERE import_id = ?1 AND class_type = 'admin' AND class_name = ?2 ORDER BY week_index, day_of_week, period_index",
                vec![import_id.to_string(), target.clone()],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![
                params_values[0].parse::<i64>().unwrap_or(import_id),
                params_values[1]
            ],
            |row| {
                let teacher_text: String = row.get(7)?;
                Ok(CourseScheduleEntry {
                    week_index: row.get(0)?,
                    day_of_week: row.get(1)?,
                    day_label: row.get(2)?,
                    period_index: row.get(3)?,
                    period_label: row.get(4)?,
                    section_label: row.get(5)?,
                    subject: row.get(6)?,
                    teacher_names: split_teacher_names(&teacher_text),
                    class_name: row.get(8)?,
                    display_class_name: row.get(9)?,
                    class_type: row.get(10)?,
                })
            },
        )?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(CourseScheduleView {
            import_id,
            target,
            view_type: query.view_type,
            entries,
            periods: list_period_slots(&conn, import_id)?,
        })
    })();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_normalization() {
        assert_eq!(normalize_class_code("101"), "高一1班");
        assert_eq!(class_to_teacher_header("201"), "高二（1）班");
        assert_eq!(class_type_for("高一英语（1）班"), "foreign");
        assert_eq!(class_type_for("101/104（英）"), "foreign");
        assert_eq!(class_type_for("7/9/10/11（俄）"), "foreign");
    }

    #[test]
    fn test_teacher_name_split_and_filter() {
        assert_eq!(split_teacher_names("岳/厉"), vec!["岳", "厉"]);
        assert!(!should_import_teacher_name("岳"));
        assert!(should_import_teacher_name("岳海霞"));
    }

    #[test]
    fn test_self_study_subject_labels_are_importable() {
        assert!(is_schedule_subject("自习（数）"));
        assert!(is_schedule_subject("限时（历）"));
        assert!(is_schedule_subject("听力（英）"));
        assert!(is_schedule_subject("练字（语）"));
        assert_eq!(
            subject_from_schedule_label("自习（数）"),
            Some(ParsedSubject::Math)
        );
        assert_eq!(
            subject_from_schedule_label("限时（历）"),
            Some(ParsedSubject::History)
        );
    }

    #[test]
    fn test_external_training_labels_are_displayable() {
        assert!(is_importable_schedule_cell("美术集训"));
        assert!(!is_importable_schedule_cell(""));
        assert!(!is_importable_schedule_cell("#NAME?"));
        assert!(!is_importable_schedule_cell("=IFERROR(A1,\"\")"));
    }

    #[test]
    fn test_expand_short_foreign_teacher_names_from_foreign_class() {
        let mut entries = vec![
            ParsedEntry {
                class_name: "101".to_string(),
                display_class_name: "高一1班".to_string(),
                class_type: "admin".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 2,
                period_label: "2".to_string(),
                section_label: "上午".to_string(),
                subject: "英语".to_string(),
                teacher_names: vec!["王".to_string()],
            },
            ParsedEntry {
                class_name: "高一英语（1）班".to_string(),
                display_class_name: "高一英语（1）班".to_string(),
                class_type: "foreign".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 2,
                period_label: "2".to_string(),
                section_label: "上午".to_string(),
                subject: "英语".to_string(),
                teacher_names: vec!["王丽丽".to_string()],
            },
        ];

        expand_short_foreign_teacher_names(&mut entries);

        assert_eq!(entries[0].teacher_names, vec!["王丽丽"]);
    }

    #[test]
    fn test_expand_generic_foreign_subject_from_russian_class() {
        let mut entries = vec![
            ParsedEntry {
                class_name: "102".to_string(),
                display_class_name: "高一2班".to_string(),
                class_type: "admin".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 1,
                period_label: "早读".to_string(),
                section_label: "早上".to_string(),
                subject: "外语".to_string(),
                teacher_names: vec!["厉".to_string()],
            },
            ParsedEntry {
                class_name: "高一俄语（1）班".to_string(),
                display_class_name: "高一俄语（1）班".to_string(),
                class_type: "foreign".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 1,
                period_label: "早读".to_string(),
                section_label: "早上".to_string(),
                subject: "俄语".to_string(),
                teacher_names: vec!["厉明明".to_string()],
            },
        ];

        expand_short_foreign_teacher_names(&mut entries);

        assert_eq!(entries[0].teacher_names, vec!["厉明明"]);
        assert_eq!(entries[0].subject, "外语");
    }

    #[test]
    fn test_expand_generic_foreign_suffix_self_study_from_foreign_class() {
        let mut entries = vec![
            ParsedEntry {
                class_name: "102".to_string(),
                display_class_name: "高一2班".to_string(),
                class_type: "admin".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 13,
                period_label: "晚3".to_string(),
                section_label: "晚上".to_string(),
                subject: "自习（外）".to_string(),
                teacher_names: vec!["岳".to_string()],
            },
            ParsedEntry {
                class_name: "高一英语（1）班".to_string(),
                display_class_name: "高一英语（1）班".to_string(),
                class_type: "foreign".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 13,
                period_label: "晚3".to_string(),
                section_label: "晚上".to_string(),
                subject: "英语".to_string(),
                teacher_names: vec!["岳海霞".to_string()],
            },
        ];

        expand_short_foreign_teacher_names(&mut entries);

        assert_eq!(entries[0].teacher_names, vec!["岳海霞"]);
        assert_eq!(entries[0].subject, "自习（外）");
    }

    #[test]
    fn test_expand_generic_foreign_morning_reading_from_foreign_class_name() {
        let mut entries = vec![
            ParsedEntry {
                class_name: "207".to_string(),
                display_class_name: "高二7班".to_string(),
                class_type: "admin".to_string(),
                week_index: 1,
                day_of_week: 2,
                day_label: "星期二".to_string(),
                period_index: 1,
                period_label: "晨读".to_string(),
                section_label: "早上".to_string(),
                subject: "外语".to_string(),
                teacher_names: vec!["王".to_string()],
            },
            ParsedEntry {
                class_name: "高二英语（1）班".to_string(),
                display_class_name: "高二英语（1）班".to_string(),
                class_type: "foreign".to_string(),
                week_index: 1,
                day_of_week: 2,
                day_label: "星期二".to_string(),
                period_index: 1,
                period_label: "晨读".to_string(),
                section_label: "早上".to_string(),
                subject: "晨读".to_string(),
                teacher_names: vec!["王丽丽".to_string()],
            },
        ];

        expand_short_foreign_teacher_names(&mut entries);

        assert_eq!(entries[0].teacher_names, vec!["王丽丽"]);
        assert_eq!(entries[0].subject, "外语");
    }

    #[test]
    fn test_short_foreign_teacher_name_kept_when_ambiguous() {
        let mut entries = vec![
            ParsedEntry {
                class_name: "101".to_string(),
                display_class_name: "高一1班".to_string(),
                class_type: "admin".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 2,
                period_label: "2".to_string(),
                section_label: "上午".to_string(),
                subject: "英语".to_string(),
                teacher_names: vec!["王".to_string()],
            },
            ParsedEntry {
                class_name: "高一英语（1）班".to_string(),
                display_class_name: "高一英语（1）班".to_string(),
                class_type: "foreign".to_string(),
                week_index: 1,
                day_of_week: 1,
                day_label: "星期一".to_string(),
                period_index: 2,
                period_label: "2".to_string(),
                section_label: "上午".to_string(),
                subject: "英语".to_string(),
                teacher_names: vec!["王丽丽".to_string(), "王晓明".to_string()],
            },
        ];

        expand_short_foreign_teacher_names(&mut entries);

        assert_eq!(entries[0].teacher_names, vec!["王"]);
    }

    #[test]
    fn test_validate_no_short_teacher_names_rejects_unresolved_surname() {
        let entries = vec![ParsedEntry {
            class_name: "101".to_string(),
            display_class_name: "高一1班".to_string(),
            class_type: "admin".to_string(),
            week_index: 1,
            day_of_week: 1,
            day_label: "星期一".to_string(),
            period_index: 2,
            period_label: "2".to_string(),
            section_label: "上午".to_string(),
            subject: "英语".to_string(),
            teacher_names: vec!["王".to_string()],
        }];

        assert!(validate_no_short_teacher_names(&entries).is_err());
    }
}
