use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::app_log;
use crate::exam_allocation;
use crate::score::{self, AppError, Subject};

const EXPORT_SHEET_NAME: &str = "监考表";
const ACCOUNTING_SHEET_NAME: &str = "核算";
const LIGHT_BLUE: u32 = 0xDDEBF7;
const ACCOUNTING_HEADER_GRAY: u32 = 0xD9D9D9;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLatestInvigilationScheduleResult {
    file_path: String,
    exported_at: String,
}

#[derive(Debug, Clone)]
struct TaskExportRow {
    session_id: Option<i64>,
    space_id: Option<i64>,
    task_source: String,
    role: String,
    subject: Subject,
    space_name: String,
    floor: String,
    start_at: String,
    end_at: String,
    start_ts: i64,
    recommended_self_study_topic_label: Option<String>,
    teacher_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SlotKey {
    subject_group: String,
    start_at: String,
    end_at: String,
}

#[derive(Debug, Clone)]
struct SlotDef {
    key: SlotKey,
    date_label: String,
    time_label: String,
    left_header: &'static str,
    right_header: &'static str,
    start_ts: i64,
}

#[derive(Debug, Default, Clone)]
struct CellValue {
    left: String,
    exam_count_total: i64,
    counted_exam_spaces: HashSet<(i64, i64)>,
    teachers: Vec<String>,
}

#[derive(Debug, Clone)]
struct AccountingTaskRow {
    teacher_id: i64,
    role: String,
    subject: Subject,
}

#[derive(Debug, Clone)]
struct AccountingTeacherRow {
    teacher_id: i64,
    teacher_name: String,
    group_subject: String,
    is_middle_manager: bool,
    total_minutes: i64,
    indoor_minutes: i64,
    outdoor_minutes: i64,
    allowance_total: f64,
    indoor_allowance_total: f64,
    outdoor_allowance_total: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AccountingConfig {
    outdoor_allowance_per_minute: f64,
    middle_manager_default_enabled: bool,
    middle_manager_exception_teacher_ids: HashSet<i64>,
    total_exam_and_self_study_minutes: i64,
}

fn export_root_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取应用数据目录失败: {e}")))?;
    dir.push("exports");
    fs::create_dir_all(&dir).map_err(|e| AppError::new(format!("创建导出目录失败: {e}")))?;
    Ok(dir)
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.naive_local());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
}

fn is_foreign_subject(subject: Subject) -> bool {
    matches!(subject, Subject::English | Subject::Russian | Subject::Japanese)
}

fn subject_label(subject: Subject) -> &'static str {
    match subject {
        Subject::Chinese => "语文",
        Subject::Geography => "地理",
        Subject::Math => "数学",
        Subject::Biology => "生物",
        Subject::Physics => "物理",
        Subject::English => "英语",
        Subject::Russian => "俄语",
        Subject::Japanese => "日语",
        Subject::History => "历史",
        Subject::Chemistry => "化学",
        Subject::Politics => "思想政治",
    }
}

fn top_subject_group(task_source: &str, subject: Subject) -> String {
    if task_source == "full_self_study" {
        return "自习".to_string();
    }
    if is_foreign_subject(subject) {
        return "外语".to_string();
    }
    subject_label(subject).to_string()
}

fn normalize_room_name(name: &str) -> String {
    if let Some(prefix) = name.strip_suffix('班') {
        return format!("{prefix}场");
    }
    name.to_string()
}

fn period_label(hour: u32, minute: u32) -> &'static str {
    if hour < 12 {
        "上午"
    } else if hour < 18 || (hour == 18 && minute < 30) {
        "下午"
    } else {
        "晚上"
    }
}

fn build_date_label(start_at: &str) -> Result<String, AppError> {
    let dt = parse_datetime(start_at)
        .ok_or_else(|| AppError::new(format!("无法解析监考开始时间: {start_at}")))?;
    Ok(format!(
        "{}日{}",
        dt.day(),
        period_label(dt.hour(), dt.minute())
    ))
}

fn build_time_label(start_at: &str, end_at: &str) -> Result<String, AppError> {
    let start = parse_datetime(start_at)
        .ok_or_else(|| AppError::new(format!("无法解析监考开始时间: {start_at}")))?;
    let end =
        parse_datetime(end_at).ok_or_else(|| AppError::new(format!("无法解析监考结束时间: {end_at}")))?;
    Ok(format!("{}-{}", start.format("%H:%M"), end.format("%H:%M")))
}

fn grade_order_key(name: &str) -> i32 {
    if name.starts_with("高一") {
        1
    } else if name.starts_with("高二") {
        2
    } else if name.starts_with("高三") {
        3
    } else {
        99
    }
}

fn class_number(name: &str) -> Option<i64> {
    let target = name.find('班').or_else(|| name.find('场'))?;
    let mut chars = name[..target].chars().rev().peekable();
    let mut digits = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            digits.push(*ch);
            chars.next();
        } else {
            break;
        }
    }
    if digits.is_empty() {
        return None;
    }
    digits.chars().rev().collect::<String>().parse::<i64>().ok()
}

fn sort_room_names(a: &str, b: &str) -> Ordering {
    grade_order_key(a)
        .cmp(&grade_order_key(b))
        .then(class_number(a).cmp(&class_number(b)))
        .then(a.cmp(b))
}

fn chinese_floor_digit(digit: i32) -> Option<&'static str> {
    match digit {
        1 => Some("一"),
        2 => Some("二"),
        3 => Some("三"),
        4 => Some("四"),
        5 => Some("五"),
        6 => Some("六"),
        7 => Some("七"),
        8 => Some("八"),
        9 => Some("九"),
        _ => None,
    }
}

fn floor_number(value: &str) -> Option<i32> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if !digits.is_empty() {
        return digits.parse::<i32>().ok();
    }
    for (idx, key) in ["一", "二", "三", "四", "五", "六", "七", "八", "九"]
        .iter()
        .enumerate()
    {
        if value.contains(key) {
            return Some(idx as i32 + 1);
        }
    }
    None
}

fn pretty_floor(value: &str) -> String {
    if let Some(number) = floor_number(value) {
        if let Some(label) = chinese_floor_digit(number) {
            return format!("{label}楼");
        }
    }
    if let Some(prefix) = value.strip_suffix('层') {
        return format!("{prefix}楼");
    }
    value.to_string()
}

fn sort_floor_labels(a: &str, b: &str) -> Ordering {
    floor_number(a)
        .cmp(&floor_number(b))
        .then(a.cmp(b))
}

fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum::<usize>()
}

fn join_teacher_names(names: &[String]) -> String {
    names.join("、")
}

fn sanitize_file_name_segment(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn load_exam_title(conn: &rusqlite::Connection) -> Result<String, AppError> {
    let title = conn
        .query_row(
            "SELECT exam_title FROM exam_allocation_settings WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_default();
    Ok(title)
}

fn accounting_subject_headers() -> [&'static str; 13] {
    [
        "语文",
        "数学",
        "外语",
        "历史",
        "地理",
        "生物",
        "政治",
        "物理",
        "化学",
        "自习",
        "长时科目",
        "场内",
        "场外",
    ]
}

fn normalize_subject_group(subject: Subject) -> &'static str {
    match subject {
        Subject::Chinese => "语文",
        Subject::Math => "数学",
        Subject::English | Subject::Russian | Subject::Japanese => "外语",
        Subject::History => "历史",
        Subject::Geography => "地理",
        Subject::Biology => "生物",
        Subject::Politics => "政治",
        Subject::Physics => "物理",
        Subject::Chemistry => "化学",
    }
}

fn accounting_group_rank(label: &str) -> i32 {
    match label {
        "语文" => 1,
        "数学" => 2,
        "外语" => 3,
        "历史" => 4,
        "地理" => 5,
        "生物" => 6,
        "政治" => 7,
        "物理" => 8,
        "化学" => 9,
        _ => 99,
    }
}

fn load_teacher_group_subjects(
    conn: &rusqlite::Connection,
) -> Result<HashMap<i64, String>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT teacher_id, subject
        FROM latest_teacher_assignments_v2
        ORDER BY teacher_id ASC, id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut grouped = HashMap::<i64, Vec<String>>::new();
    for row in rows {
        let (teacher_id, subject_key) = row?;
        if let Some(subject) = Subject::from_key(&subject_key) {
            let label = normalize_subject_group(subject).to_string();
            let entry = grouped.entry(teacher_id).or_default();
            if !entry.iter().any(|item| item == &label) {
                entry.push(label);
            }
        }
    }
    let mut out = HashMap::new();
    for (teacher_id, mut labels) in grouped {
        labels.sort_by(|a, b| {
            accounting_group_rank(a)
                .cmp(&accounting_group_rank(b))
                .then(a.cmp(b))
        });
        let label = if labels.is_empty() {
            "未分科".to_string()
        } else {
            labels.join("、")
        };
        out.insert(teacher_id, label);
    }
    Ok(out)
}

fn load_accounting_teacher_rows(
    conn: &rusqlite::Connection,
) -> Result<Vec<AccountingTeacherRow>, AppError> {
    let group_subjects = load_teacher_group_subjects(conn)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT
          teacher_id,
          teacher_name,
          total_minutes,
          indoor_minutes,
          outdoor_minutes,
          allowance_total,
          indoor_allowance_total,
          outdoor_allowance_total,
          is_middle_manager
        FROM latest_teacher_duty_stats
        ORDER BY teacher_id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let teacher_id: i64 = row.get(0)?;
        Ok(AccountingTeacherRow {
            teacher_id,
            teacher_name: row.get(1)?,
            group_subject: group_subjects
                .get(&teacher_id)
                .cloned()
                .unwrap_or_else(|| "未分科".to_string()),
            is_middle_manager: row.get::<_, i64>(8)? == 1,
            total_minutes: row.get(2)?,
            indoor_minutes: row.get(3)?,
            outdoor_minutes: row.get(4)?,
            allowance_total: row.get(5)?,
            indoor_allowance_total: row.get(6)?,
            outdoor_allowance_total: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn load_accounting_task_rows(
    conn: &rusqlite::Connection,
) -> Result<Vec<AccountingTaskRow>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT a.teacher_id, t.role, t.subject
        FROM latest_exam_staff_tasks t
        JOIN latest_exam_staff_assignments a ON a.task_id = t.id
        ORDER BY a.teacher_id ASC, t.id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let subject_key: String = row.get(2)?;
        let subject = Subject::from_key(&subject_key).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                2,
                "subject".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;
        Ok(AccountingTaskRow {
            teacher_id: row.get(0)?,
            role: row.get(1)?,
            subject,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn load_accounting_config(conn: &rusqlite::Connection) -> Result<AccountingConfig, AppError> {
    let (outdoor_allowance_per_minute, middle_manager_default_enabled, exception_ids_json, self_study_date, self_study_start_time, self_study_end_time): (
        f64,
        i64,
        String,
        String,
        String,
        String,
    ) = conn.query_row(
        r#"
        SELECT
          outdoor_allowance_per_minute,
          middle_manager_default_enabled,
          middle_manager_exception_teacher_ids_json,
          self_study_date,
          self_study_start_time,
          self_study_end_time
        FROM invigilation_config_settings
        WHERE id = 1
        "#,
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;

    let exception_ids = serde_json::from_str::<Vec<i64>>(&exception_ids_json)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();

    let total_exam_minutes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CAST((julianday(end_at) - julianday(start_at)) * 24 * 60 AS INTEGER)), 0) FROM exam_session_times",
        [],
        |row| row.get(0),
    )?;

    let total_self_study_minutes = if !self_study_date.trim().is_empty()
        && !self_study_start_time.trim().is_empty()
        && !self_study_end_time.trim().is_empty()
    {
        let start = format!("{}T{}", self_study_date.trim(), self_study_start_time.trim());
        let end = format!("{}T{}", self_study_date.trim(), self_study_end_time.trim());
        match (parse_datetime(&start), parse_datetime(&end)) {
            (Some(start_dt), Some(end_dt)) if end_dt > start_dt => {
                (end_dt - start_dt).num_minutes()
            }
            _ => 0,
        }
    } else {
        0
    };

    Ok(AccountingConfig {
        outdoor_allowance_per_minute,
        middle_manager_default_enabled: middle_manager_default_enabled == 1,
        middle_manager_exception_teacher_ids: exception_ids,
        total_exam_and_self_study_minutes: total_exam_minutes + total_self_study_minutes,
    })
}

#[allow(dead_code)]
fn middle_manager_participates(
    teacher_id: i64,
    is_middle_manager: bool,
    config: &AccountingConfig,
) -> bool {
    if !is_middle_manager {
        return true;
    }
    let is_exception = config.middle_manager_exception_teacher_ids.contains(&teacher_id);
    if config.middle_manager_default_enabled {
        !is_exception
    } else {
        is_exception
    }
}

fn load_export_rows(conn: &rusqlite::Connection) -> Result<Vec<TaskExportRow>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
          t.session_id,
          t.space_id,
          t.task_source,
          t.role,
          t.subject,
          t.space_name,
          t.floor,
          t.start_at,
          t.end_at,
          t.recommended_self_study_topic_label,
          a.teacher_name
        FROM latest_exam_staff_tasks t
        LEFT JOIN latest_exam_staff_assignments a ON a.task_id = t.id
        ORDER BY t.start_at ASC, t.id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let subject_key: String = row.get(4)?;
        let subject = Subject::from_key(&subject_key).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                4,
                "subject".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;
        let start_at: String = row.get(7)?;
        let start_ts = parse_datetime(&start_at)
            .map(|dt| dt.and_utc().timestamp())
            .ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("无法解析任务开始时间: {start_at}"),
                    )),
                )
            })?;
        Ok(TaskExportRow {
            session_id: row.get(0)?,
            space_id: row.get(1)?,
            task_source: row.get(2)?,
            role: row.get(3)?,
            subject,
            space_name: row.get(5)?,
            floor: row.get(6)?,
            start_at,
            end_at: row.get(8)?,
            start_ts,
            recommended_self_study_topic_label: row.get(9)?,
            teacher_name: row.get(10)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    if out.is_empty() {
        return Err(AppError::new("暂无监考分配结果，请先执行监考分配"));
    }
    Ok(out)
}

fn load_exam_counts(
    conn: &rusqlite::Connection,
) -> Result<HashMap<(i64, i64), i64>, AppError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT session_id, space_id, COUNT(*)
        FROM latest_exam_plan_student_allocations
        WHERE allocation_type = 'exam' AND space_id IS NOT NULL
        GROUP BY session_id, space_id
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut out = HashMap::new();
    for row in rows {
        let (session_id, space_id, count) = row?;
        out.insert((session_id, space_id), count);
    }
    Ok(out)
}

fn build_slots(rows: &[TaskExportRow]) -> Result<Vec<SlotDef>, AppError> {
    let mut seen = HashSet::<SlotKey>::new();
    let mut slots = Vec::<SlotDef>::new();
    for row in rows {
        let subject_group = top_subject_group(&row.task_source, row.subject);
        let key = SlotKey {
            subject_group: subject_group.clone(),
            start_at: row.start_at.clone(),
            end_at: row.end_at.clone(),
        };
        if seen.insert(key.clone()) {
            let is_self_study_group = subject_group == "自习";
            slots.push(SlotDef {
                key,
                date_label: build_date_label(&row.start_at)?,
                time_label: build_time_label(&row.start_at, &row.end_at)?,
                left_header: if is_self_study_group { "班级" } else { "考生数" },
                right_header: if is_self_study_group { "教师" } else { "监考员" },
                start_ts: row.start_ts,
            });
        }
    }
    slots.sort_by(|a, b| {
        a.start_ts
            .cmp(&b.start_ts)
            .then(a.key.subject_group.cmp(&b.key.subject_group))
    });
    Ok(slots)
}

fn collect_room_cells(
    rows: &[TaskExportRow],
    slots: &[SlotDef],
    exam_counts: &HashMap<(i64, i64), i64>,
) -> (Vec<String>, HashMap<(String, SlotKey), CellValue>) {
    let slot_lookup: HashSet<SlotKey> = slots.iter().map(|slot| slot.key.clone()).collect();
    let mut room_names = HashSet::<String>::new();
    let mut cells = HashMap::<(String, SlotKey), CellValue>::new();

    for row in rows {
        if row.role == "floor_rover" {
            continue;
        }
        let slot_key = SlotKey {
            subject_group: top_subject_group(&row.task_source, row.subject),
            start_at: row.start_at.clone(),
            end_at: row.end_at.clone(),
        };
        if !slot_lookup.contains(&slot_key) {
            continue;
        }
        let room_name = normalize_room_name(&row.space_name);
        room_names.insert(room_name.clone());
        let entry = cells
            .entry((room_name, slot_key.clone()))
            .or_default();
        match row.role.as_str() {
            "exam_room_invigilator" => {
                if let (Some(session_id), Some(space_id)) = (row.session_id, row.space_id) {
                    if entry.counted_exam_spaces.insert((session_id, space_id)) {
                        if let Some(count) = exam_counts.get(&(session_id, space_id)) {
                            entry.exam_count_total += *count;
                        }
                    }
                }
                entry
                    .teachers
                    .push(row.teacher_name.clone().unwrap_or_else(|| "待分配".to_string()));
            }
            "self_study_supervisor" => {
                if entry.left.is_empty() {
                    entry.left = row
                        .recommended_self_study_topic_label
                        .clone()
                        .unwrap_or_else(|| "自习".to_string());
                }
                entry
                    .teachers
                    .push(row.teacher_name.clone().unwrap_or_else(|| "待分配".to_string()));
            }
            _ => {}
        }
    }

    let mut ordered_rooms = room_names.into_iter().collect::<Vec<_>>();
    ordered_rooms.sort_by(|a, b| sort_room_names(a, b));
    (ordered_rooms, cells)
}

fn collect_floor_cells(
    rows: &[TaskExportRow],
    slots: &[SlotDef],
) -> (Vec<String>, HashMap<(String, SlotKey), CellValue>) {
    let slot_lookup: HashSet<SlotKey> = slots.iter().map(|slot| slot.key.clone()).collect();
    let mut floors = HashSet::<String>::new();
    let mut cells = HashMap::<(String, SlotKey), CellValue>::new();

    for row in rows {
        if row.role != "floor_rover" {
            continue;
        }
        let slot_key = SlotKey {
            subject_group: top_subject_group(&row.task_source, row.subject),
            start_at: row.start_at.clone(),
            end_at: row.end_at.clone(),
        };
        if !slot_lookup.contains(&slot_key) {
            continue;
        }
        let floor_label = pretty_floor(&row.floor);
        floors.insert(floor_label.clone());
        let entry = cells.entry((floor_label.clone(), slot_key)).or_default();
        if entry.left.is_empty() {
            entry.left = floor_label;
        }
        entry
            .teachers
            .push(row.teacher_name.clone().unwrap_or_else(|| "待分配".to_string()));
    }

    let mut ordered_floors = floors.into_iter().collect::<Vec<_>>();
    ordered_floors.sort_by(|a, b| sort_floor_labels(a, b));
    (ordered_floors, cells)
}

fn write_headers(
    sheet: &mut rust_xlsxwriter::Worksheet,
    slots: &[SlotDef],
    header_fmt: &Format,
    plain_header_fmt: &Format,
) -> Result<(), XlsxError> {
    sheet.merge_range(0, 0, 3, 0, "考场", header_fmt)?;

    let mut col = 1_u16;
    let mut subject_index = 0_usize;
    while subject_index < slots.len() {
        let subject_label = slots[subject_index].key.subject_group.clone();
        let group_fmt = if subject_label == "自习" {
            plain_header_fmt
        } else {
            header_fmt
        };
        let subject_start = col;
        let mut date_index = subject_index;
        while date_index < slots.len() && slots[date_index].key.subject_group == subject_label {
            let date_label = slots[date_index].date_label.clone();
            let date_start = col;
            while date_index < slots.len()
                && slots[date_index].key.subject_group == subject_label
                && slots[date_index].date_label == date_label
            {
                sheet.merge_range(
                    2,
                    col,
                    2,
                    col + 1,
                    &slots[date_index].time_label,
                    group_fmt,
                )?;
                sheet.write_string_with_format(3, col, slots[date_index].left_header, group_fmt)?;
                sheet.write_string_with_format(
                    3,
                    col + 1,
                    slots[date_index].right_header,
                    group_fmt,
                )?;
                col += 2;
                date_index += 1;
            }
            sheet.merge_range(1, date_start, 1, col - 1, &date_label, group_fmt)?;
        }
        sheet.merge_range(0, subject_start, 0, col - 1, &subject_label, group_fmt)?;
        subject_index = date_index;
    }

    Ok(())
}

fn write_room_section(
    sheet: &mut rust_xlsxwriter::Worksheet,
    start_row: u32,
    room_names: &[String],
    slots: &[SlotDef],
    cells: &HashMap<(String, SlotKey), CellValue>,
    body_fmt: &Format,
    wrap_fmt: &Format,
    plain_body_fmt: &Format,
    plain_wrap_fmt: &Format,
) -> Result<u32, XlsxError> {
    let mut row = start_row;
    for room_name in room_names {
        sheet.write_string_with_format(row, 0, room_name, body_fmt)?;
        let mut col = 1_u16;
        for slot in slots {
            let cell = cells.get(&(room_name.clone(), slot.key.clone()));
            let left = if let Some(item) = cell {
                if !item.left.is_empty() {
                    item.left.clone()
                } else if item.exam_count_total > 0 {
                    item.exam_count_total.to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let right = cell
                .map(|item| join_teacher_names(&item.teachers))
                .unwrap_or_default();
            let is_empty_cell = left.is_empty() && right.is_empty();
            let is_self_study_cell = slot.key.subject_group == "自习"
                || cell
                    .map(|item| !item.left.is_empty())
                    .unwrap_or(false);
            let left_fmt = if is_self_study_cell {
                plain_body_fmt
            } else if is_empty_cell {
                plain_body_fmt
            } else {
                body_fmt
            };
            let right_fmt = if is_self_study_cell {
                plain_wrap_fmt
            } else if is_empty_cell {
                plain_wrap_fmt
            } else {
                wrap_fmt
            };
            sheet.write_string_with_format(row, col, &left, left_fmt)?;
            sheet.write_string_with_format(row, col + 1, &right, right_fmt)?;
            col += 2;
        }
        row += 1;
    }
    Ok(row)
}

fn write_floor_section(
    sheet: &mut rust_xlsxwriter::Worksheet,
    start_row: u32,
    floors: &[String],
    slots: &[SlotDef],
    cells: &HashMap<(String, SlotKey), CellValue>,
    body_fmt: &Format,
    wrap_fmt: &Format,
    group_fmt: &Format,
    plain_body_fmt: &Format,
    plain_wrap_fmt: &Format,
) -> Result<u32, XlsxError> {
    if floors.is_empty() {
        return Ok(start_row);
    }
    let end_row = start_row + floors.len() as u32 - 1;
    if floors.len() == 1 {
        sheet.write_string_with_format(start_row, 0, "流动监考", group_fmt)?;
    } else {
        sheet.merge_range(start_row, 0, end_row, 0, "流动监考", group_fmt)?;
    }

    let mut row = start_row;
    for floor in floors {
        let mut col = 1_u16;
        for slot in slots {
            let cell = cells.get(&(floor.clone(), slot.key.clone()));
            let left = cell.map(|item| item.left.as_str()).unwrap_or("");
            let right = cell
                .map(|item| join_teacher_names(&item.teachers))
                .unwrap_or_default();
            let is_empty_cell = left.is_empty() && right.is_empty();
            let left_fmt = if is_empty_cell { plain_body_fmt } else { body_fmt };
            let right_fmt = if is_empty_cell { plain_wrap_fmt } else { wrap_fmt };
            sheet.write_string_with_format(row, col, left, left_fmt)?;
            sheet.write_string_with_format(row, col + 1, &right, right_fmt)?;
            col += 2;
        }
        row += 1;
    }
    Ok(row)
}

fn apply_column_widths(
    sheet: &mut rust_xlsxwriter::Worksheet,
    room_names: &[String],
    floors: &[String],
    slots: &[SlotDef],
    room_cells: &HashMap<(String, SlotKey), CellValue>,
    floor_cells: &HashMap<(String, SlotKey), CellValue>,
) -> Result<(), XlsxError> {
    let mut widths = vec![display_width("流动监考").max(display_width("考场")); 1 + slots.len() * 2];
    for room in room_names {
        widths[0] = widths[0].max(display_width(room));
    }
    for floor in floors {
        widths[0] = widths[0].max(display_width(floor));
    }

    for (slot_index, slot) in slots.iter().enumerate() {
        let left_col = 1 + slot_index * 2;
        let right_col = left_col + 1;
        widths[left_col] = widths[left_col].max(display_width(slot.left_header));
        widths[right_col] = widths[right_col].max(display_width(slot.right_header));
        let merged_hint = display_width(&slot.time_label)
            .max(display_width(&slot.date_label))
            .max(display_width(&slot.key.subject_group));
        let per_col_hint = ((merged_hint + 1) / 2).max(4);
        widths[left_col] = widths[left_col].max(per_col_hint);
        widths[right_col] = widths[right_col].max(per_col_hint);

        for room in room_names {
            if let Some(cell) = room_cells.get(&(room.clone(), slot.key.clone())) {
                let left = if !cell.left.is_empty() {
                    cell.left.clone()
                } else if cell.exam_count_total > 0 {
                    cell.exam_count_total.to_string()
                } else {
                    String::new()
                };
                let right = join_teacher_names(&cell.teachers);
                widths[left_col] = widths[left_col].max(display_width(&left));
                widths[right_col] = widths[right_col].max(display_width(&right));
            }
        }

        for floor in floors {
            if let Some(cell) = floor_cells.get(&(floor.clone(), slot.key.clone())) {
                widths[left_col] = widths[left_col].max(display_width(&cell.left));
                widths[right_col] =
                    widths[right_col].max(display_width(&join_teacher_names(&cell.teachers)));
            }
        }
    }

    for (index, width) in widths.iter().enumerate() {
        let visual = (*width).max(6) as f64 + 2.0;
        sheet.set_column_width(index as u16, visual)?;
    }
    Ok(())
}

fn write_accounting_sheet(
    sheet: &mut rust_xlsxwriter::Worksheet,
    teacher_rows: &[AccountingTeacherRow],
    task_rows: &[AccountingTaskRow],
    config: &AccountingConfig,
) -> Result<(), AppError> {
    let headers = [
        "科目",
        "教师",
        "序号",
        "语文",
        "数学",
        "外语",
        "历史",
        "地理",
        "生物",
        "政治",
        "物理",
        "化学",
        "自习",
        "长时科目",
        "场内",
        "场外",
        "合计",
        "总时长",
        "场内时长",
        "场外时长",
        "场内津贴",
        "场外津贴",
        "总津贴",
    ];
    let header_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(ACCOUNTING_HEADER_GRAY))
        .set_border(FormatBorder::Thin);
    let body_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let decimal_fmt = body_fmt.clone().set_num_format("0.00");

    for (col, header) in headers.iter().enumerate() {
        sheet
            .write_string_with_format(0, col as u16, *header, &header_fmt)
            .map_err(|e| AppError::new(format!("写入核算表头失败: {e}")))?;
    }

    let mut flags_by_teacher = HashMap::<i64, HashSet<&'static str>>::new();
    for task in task_rows {
        let entry = flags_by_teacher.entry(task.teacher_id).or_default();
        entry.insert(normalize_subject_group(task.subject));
        if task.role == "self_study_supervisor" {
            entry.insert("自习");
        }
        if task.role == "floor_rover" {
            entry.insert("场外");
        } else {
            entry.insert("场内");
        }
    }

    let mut rows = teacher_rows.to_vec();
    rows.sort_by(|a, b| {
        accounting_group_rank(&a.group_subject)
            .cmp(&accounting_group_rank(&b.group_subject))
            .then(a.group_subject.cmp(&b.group_subject))
            .then(a.teacher_name.cmp(&b.teacher_name))
            .then(a.teacher_id.cmp(&b.teacher_id))
    });

    let subject_headers = accounting_subject_headers();
    let mut widths = headers.iter().map(|item| display_width(item)).collect::<Vec<_>>();
    let mut row_index = 1_u32;
    let mut index_in_group = 1_i64;
    let mut group_start = 1_u32;
    let mut current_group: Option<String> = None;

    for teacher in &rows {
        let teacher_group = teacher.group_subject.clone();
        if current_group.as_ref() != Some(&teacher_group) {
            if let Some(group_label) = current_group.take() {
                widths[0] = widths[0].max(display_width(&group_label));
                let end_row = row_index - 1;
                if group_start == end_row {
                    sheet
                        .write_string_with_format(group_start, 0, &group_label, &body_fmt)
                        .map_err(|e| AppError::new(format!("写入核算科目分组失败: {e}")))?;
                } else {
                    sheet
                        .merge_range(group_start, 0, end_row, 0, &group_label, &body_fmt)
                        .map_err(|e| AppError::new(format!("合并核算科目分组失败: {e}")))?;
                }
            }
            current_group = Some(teacher_group.clone());
            group_start = row_index;
            index_in_group = 1;
        }

        let flags = flags_by_teacher
            .get(&teacher.teacher_id)
            .cloned()
            .unwrap_or_default();
        let mut outdoor_minutes = teacher.outdoor_minutes;
        let mut total_minutes = teacher.total_minutes;
        let mut outdoor_allowance_total = teacher.outdoor_allowance_total;
        let mut allowance_total = teacher.allowance_total;
        if teacher.is_middle_manager {
            let baseline_total_minutes =
                config.total_exam_and_self_study_minutes.max(teacher.total_minutes);
            let supplemental_outdoor_minutes =
                (baseline_total_minutes - teacher.total_minutes).max(0);
            outdoor_minutes = teacher.outdoor_minutes + supplemental_outdoor_minutes;
            total_minutes = baseline_total_minutes;
            outdoor_allowance_total =
                teacher.outdoor_allowance_total
                    + supplemental_outdoor_minutes as f64 * config.outdoor_allowance_per_minute;
            allowance_total = teacher.indoor_allowance_total + outdoor_allowance_total;
        }
        let has_long_subject = flags.contains("语文") || flags.contains("数学") || flags.contains("外语");
        let mut flag_values = Vec::<i64>::new();
        for label in subject_headers {
            let value = i64::from(flags.contains(label));
            flag_values.push(value);
        }
        flag_values.push(i64::from(has_long_subject));
        flag_values.push(i64::from(flags.contains("场内")));
        flag_values.push(i64::from(flags.contains("场外")));
        let total_flags = flag_values.iter().sum::<i64>();

        sheet
            .write_string_with_format(row_index, 1, &teacher.teacher_name, &body_fmt)
            .map_err(|e| AppError::new(format!("写入核算教师失败: {e}")))?;
        sheet
            .write_number_with_format(row_index, 2, index_in_group as f64, &body_fmt)
            .map_err(|e| AppError::new(format!("写入核算序号失败: {e}")))?;
        widths[1] = widths[1].max(display_width(&teacher.teacher_name));
        widths[2] = widths[2].max(display_width(&index_in_group.to_string()));

        for (offset, value) in flag_values.iter().enumerate() {
            let col = 3 + offset as u16;
            sheet
                .write_number_with_format(row_index, col, *value as f64, &body_fmt)
                .map_err(|e| AppError::new(format!("写入核算标记失败: {e}")))?;
            widths[col as usize] = widths[col as usize].max(display_width(&value.to_string()));
        }

        let total_col = 16_u16;
        let integer_values = [
            (total_col, total_flags),
            (17_u16, total_minutes),
            (18_u16, teacher.indoor_minutes),
            (19_u16, outdoor_minutes),
        ];
        for (col, value) in integer_values {
            sheet
                .write_number_with_format(row_index, col, value as f64, &body_fmt)
                .map_err(|e| AppError::new(format!("写入核算统计失败: {e}")))?;
            widths[col as usize] = widths[col as usize].max(display_width(&value.to_string()));
        }

        let decimal_values = [
            (20_u16, teacher.indoor_allowance_total),
            (21_u16, outdoor_allowance_total),
            (22_u16, allowance_total),
        ];
        for (col, value) in decimal_values {
            sheet
                .write_number_with_format(row_index, col, value, &decimal_fmt)
                .map_err(|e| AppError::new(format!("写入核算津贴失败: {e}")))?;
            widths[col as usize] = widths[col as usize].max(display_width(&format!("{value:.2}")));
        }

        row_index += 1;
        index_in_group += 1;
    }

    if let Some(group_label) = current_group {
        widths[0] = widths[0].max(display_width(&group_label));
        let end_row = row_index.saturating_sub(1);
        if group_start == end_row {
            sheet
                .write_string_with_format(group_start, 0, &group_label, &body_fmt)
                .map_err(|e| AppError::new(format!("写入核算科目分组失败: {e}")))?;
        } else {
            sheet
                .merge_range(group_start, 0, end_row, 0, &group_label, &body_fmt)
                .map_err(|e| AppError::new(format!("合并核算科目分组失败: {e}")))?;
        }
    }

    for (index, width) in widths.iter().enumerate() {
        let visual = (*width).max(6) as f64 + 2.0;
        sheet
            .set_column_width(index as u16, visual)
            .map_err(|e| AppError::new(format!("设置核算列宽失败: {e}")))?;
    }

    Ok(())
}

fn build_formats() -> (Format, Format, Format, Format, Format, Format, Format) {
    let header_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let plain_header_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let body_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(LIGHT_BLUE))
        .set_border(FormatBorder::Thin);
    let wrap_fmt = body_fmt.clone();
    let plain_body_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let plain_wrap_fmt = plain_body_fmt.clone();
    // 流动监考第一列需要和主体区域保持同样底色，便于打印阅读。
    let flow_group_fmt = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(LIGHT_BLUE))
        .set_border(FormatBorder::Thin);
    (
        header_fmt,
        plain_header_fmt,
        body_fmt,
        wrap_fmt,
        plain_body_fmt,
        plain_wrap_fmt,
        flow_group_fmt,
    )
}

fn build_workbook_from_connection(conn: &rusqlite::Connection) -> Result<Workbook, AppError> {
    let rows = load_export_rows(&conn)?;
    let exam_counts = load_exam_counts(&conn)?;
    let slots = build_slots(&rows)?;
    let (room_names, room_cells) = collect_room_cells(&rows, &slots, &exam_counts);
    let (floors, floor_cells) = collect_floor_cells(&rows, &slots);
    let accounting_teacher_rows = load_accounting_teacher_rows(conn)?;
    let accounting_task_rows = load_accounting_task_rows(conn)?;
    let accounting_config = load_accounting_config(conn)?;

    if room_names.is_empty() && floors.is_empty() {
        return Err(AppError::new("暂无可导出的监考任务"));
    }

    let (
        header_fmt,
        plain_header_fmt,
        body_fmt,
        wrap_fmt,
        plain_body_fmt,
        plain_wrap_fmt,
        flow_group_fmt,
    ) = build_formats();
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet
        .set_name(EXPORT_SHEET_NAME)
        .map_err(|e| AppError::new(format!("设置监考表 Sheet 名失败: {e}")))?;
    write_headers(sheet, &slots, &header_fmt, &plain_header_fmt)
        .map_err(|e| AppError::new(format!("写入监考表头失败: {e}")))?;
    let next_row = write_room_section(
        sheet,
        4,
        &room_names,
        &slots,
        &room_cells,
        &body_fmt,
        &wrap_fmt,
        &plain_body_fmt,
        &plain_wrap_fmt,
    )
    .map_err(|e| AppError::new(format!("写入考场安排失败: {e}")))?;
    let _ = write_floor_section(
        sheet,
        next_row,
        &floors,
        &slots,
        &floor_cells,
        &body_fmt,
        &wrap_fmt,
        &flow_group_fmt,
        &plain_body_fmt,
        &plain_wrap_fmt,
    )
    .map_err(|e| AppError::new(format!("写入流动监考失败: {e}")))?;
    apply_column_widths(sheet, &room_names, &floors, &slots, &room_cells, &floor_cells)
        .map_err(|e| AppError::new(format!("设置列宽失败: {e}")))?;

    let accounting_sheet = workbook.add_worksheet();
    accounting_sheet
        .set_name(ACCOUNTING_SHEET_NAME)
        .map_err(|e| AppError::new(format!("设置核算 Sheet 名失败: {e}")))?;
    write_accounting_sheet(
        accounting_sheet,
        &accounting_teacher_rows,
        &accounting_task_rows,
        &accounting_config,
    )?;

    Ok(workbook)
}

fn save_workbook_to_dir(
    mut workbook: Workbook,
    output_dir: &Path,
    exam_title: &str,
) -> Result<ExportLatestInvigilationScheduleResult, AppError> {
    let exported_at = Utc::now().to_rfc3339();
    fs::create_dir_all(output_dir)
        .map_err(|e| AppError::new(format!("创建导出目录失败: {e}")))?;
    let sanitized_title = sanitize_file_name_segment(exam_title);
    let file_name = if sanitized_title.is_empty() {
        "监考表.xlsx".to_string()
    } else {
        format!("{sanitized_title}监考表.xlsx")
    };
    let path = output_dir.join(file_name);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| AppError::new(format!("覆盖旧监考表失败: {e}")))?;
    }
    workbook
        .save(&path)
        .map_err(|e| AppError::new(format!("保存监考表失败: {e}")))?;

    Ok(ExportLatestInvigilationScheduleResult {
        file_path: path.to_string_lossy().to_string(),
        exported_at,
    })
}

fn export_schedule_internal(app: &AppHandle) -> Result<ExportLatestInvigilationScheduleResult, AppError> {
    let conn = score::open_connection(app)?;
    exam_allocation::ensure_schema(&conn)?;
    let workbook = build_workbook_from_connection(&conn)?;
    let exam_title = load_exam_title(&conn)?;
    let output_dir = export_root_dir(app)?;
    save_workbook_to_dir(workbook, &output_dir, &exam_title)
}

pub fn export_latest_invigilation_schedule(
    app: AppHandle,
) -> Result<ExportLatestInvigilationScheduleResult, String> {
    let result = export_schedule_internal(&app);
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "export_invigilation.export_latest_invigilation_schedule",
            &e.to_string(),
        );
        e.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_normalize_room_name_turns_class_into_room() {
        assert_eq!(normalize_room_name("高一1班"), "高一1场");
        assert_eq!(normalize_room_name("高一5场"), "高一5场");
    }

    #[test]
    fn test_top_subject_group_collapses_foreign_and_full_self_study() {
        assert_eq!(top_subject_group("exam", Subject::English), "外语");
        assert_eq!(top_subject_group("exam", Subject::Russian), "外语");
        assert_eq!(top_subject_group("full_self_study", Subject::Math), "自习");
        assert_eq!(top_subject_group("exam", Subject::Politics), "思想政治");
    }

    #[test]
    fn test_pretty_floor_formats_numeric_floor() {
        assert_eq!(pretty_floor("2层"), "二楼");
        assert_eq!(pretty_floor("五楼"), "五楼");
        assert_eq!(pretty_floor("临时"), "临时");
    }

    #[test]
    fn test_sanitize_file_name_segment_removes_windows_invalid_chars() {
        assert_eq!(
            sanitize_file_name_segment(" 2026/03:月考? "),
            "2026_03_月考_"
        );
    }

    #[test]
    #[ignore = "manual integration test against the real sqlite database"]
    fn test_export_latest_invigilation_schedule_manual() {
        let db_path = std::env::var("ACADEMIC_REAL_DB_PATH")
            .expect("ACADEMIC_REAL_DB_PATH must point to scores.sqlite3");
        let db_path = PathBuf::from(db_path);
        let conn = Connection::open(&db_path).expect("open real sqlite db");
        crate::schema::ensure_schema(&conn).expect("ensure schema");

        let workbook = build_workbook_from_connection(&conn).expect("build workbook from real db");
        let exam_title = load_exam_title(&conn).expect("load exam title");
        let export_dir = db_path
            .parent()
            .expect("db parent")
            .join("exports");
        let result =
            save_workbook_to_dir(workbook, &export_dir, &exam_title).expect("save real workbook");

        println!(
            "REAL_DB_EXPORT_INVIGILATION {}",
            serde_json::to_string(&result).expect("serialize export result")
        );
        assert!(PathBuf::from(&result.file_path).is_file());
    }
}
