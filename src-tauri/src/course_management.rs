use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use calamine::{open_workbook_auto, Data, Range, Reader};
use chrono::Utc;
use regex::Regex;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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

fn parse_course_workbook(file_path: &str) -> Result<ParsedWorkbook, AppError> {
    let mut workbook = open_workbook_auto(file_path)?;
    let teacher_range = workbook
        .worksheet_range("教师安排")
        .map_err(AppError::from)?;
    let total_range = workbook.worksheet_range("总课表").map_err(AppError::from)?;

    let (teacher_map, assignments) = parse_teacher_arrangements(&teacher_range);
    let (entries, periods, classes) = parse_total_schedule(&total_range, &teacher_map);
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
}
