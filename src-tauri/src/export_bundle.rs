use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rusqlite::params;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::app_log;
use crate::score::{AppError, Subject};

const SUBJECT_EXPORT_ORDER: [(Subject, &str); 11] = [
    (Subject::Chinese, "语文"),
    (Subject::Math, "数学"),
    (Subject::English, "英语"),
    (Subject::Russian, "俄语"),
    (Subject::Japanese, "日语"),
    (Subject::History, "历史"),
    (Subject::Geography, "地理"),
    (Subject::Biology, "生物"),
    (Subject::Politics, "政治"),
    (Subject::Physics, "物理"),
    (Subject::Chemistry, "化学"),
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLatestExamAllocationBundleResult {
    folder_path: String,
    grade_count: i64,
    file_count: i64,
    exported_at: String,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    subject_label: &'static str,
    start_at: Option<String>,
    end_at: Option<String>,
    start_ts: Option<i64>,
}

#[derive(Debug, Clone)]
struct ExamRow {
    admission_no: String,
    student_name: String,
    class_name: String,
    subject: Subject,
    subject_label: &'static str,
    space_name: String,
    seat_no: i64,
}

#[derive(Debug, Clone)]
struct StudentBase {
    admission_no: String,
    student_name: String,
    class_name: String,
    class_rank: i64,
}

#[derive(Debug, Clone)]
struct TicketExamItem {
    subject_label: &'static str,
    exam_time: String,
    room: String,
    seat: i64,
    start_ts: i64,
}

fn grade_order_key(grade_name: &str) -> (i32, &str) {
    match grade_name {
        "高一" => (1, grade_name),
        "高二" => (2, grade_name),
        "高三" => (3, grade_name),
        _ => (4, grade_name),
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

fn sort_class_like(a: &str, b: &str) -> std::cmp::Ordering {
    class_number(a).cmp(&class_number(b)).then(a.cmp(b))
}

fn subject_label(subject: Subject) -> &'static str {
    SUBJECT_EXPORT_ORDER
        .iter()
        .find(|(s, _)| *s == subject)
        .map(|(_, label)| *label)
        .unwrap_or(subject.as_key())
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.naive_local());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
}

fn format_hm(value: &str) -> Option<String> {
    parse_datetime(value).map(|dt| dt.format("%H:%M").to_string())
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

fn format_ticket_date(start: &str) -> Option<String> {
    let s = parse_datetime(start)?;
    Some(format!("{}月{}日", s.month(), s.day()))
}

fn format_ticket_time(start: &str, end: &str) -> Option<String> {
    let s = parse_datetime(start)?;
    let e = parse_datetime(end)?;
    let left_period = period_label(s.hour(), s.minute());
    let right_period = period_label(e.hour(), e.minute());
    Some(format!(
        "{}{} — {}{}",
        left_period, s.format("%H:%M"), right_period, e.format("%H:%M")
    ))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn strip_notice_leading_index(value: &str) -> String {
    let trimmed = value.trim();
    let chars = trimmed.char_indices().collect::<Vec<_>>();
    let mut end = 0usize;
    let mut has_digit = false;

    for (idx, ch) in &chars {
        if ch.is_ascii_digit() {
            has_digit = true;
            end = idx + ch.len_utf8();
            continue;
        }
        break;
    }

    if !has_digit {
        return trimmed.to_string();
    }

    for (idx, ch) in chars {
        if idx < end {
            continue;
        }
        if matches!(ch, '.' | '．' | '、' | ')' | '）' | ' ' | '\t') {
            end = idx + ch.len_utf8();
            continue;
        }
        break;
    }

    trimmed[end..].trim_start().to_string()
}

fn settings_from_db(conn: &rusqlite::Connection) -> Result<(String, Vec<String>), AppError> {
    conn.query_row(
        "SELECT exam_title, exam_notices_json FROM exam_allocation_settings WHERE id = 1",
        [],
        |row| {
            let title: String = row.get(0)?;
            let notices_json: String = row.get(1)?;
            let notices = serde_json::from_str::<Vec<String>>(&notices_json)
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>();
            Ok((title, notices))
        },
    )
    .map_err(|e| AppError::new(format!("读取月考配置失败: {e}")))
}

fn load_grades(conn: &rusqlite::Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT grade_name FROM latest_exam_plan_sessions")
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(AppError::from)?;
    let mut grades = Vec::new();
    for row in rows {
        grades.push(row.map_err(AppError::from)?);
    }
    grades.sort_by(|a, b| grade_order_key(a).cmp(&grade_order_key(b)));
    Ok(grades)
}

fn load_sessions_for_grade(
    conn: &rusqlite::Connection,
    grade_name: &str,
) -> Result<HashMap<Subject, SessionInfo>, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT s.subject, t.start_at, t.end_at
            FROM latest_exam_plan_sessions s
            LEFT JOIN exam_session_times t ON t.session_id = s.id
            WHERE s.grade_name = ?1
            "#,
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![grade_name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(AppError::from)?;

    let mut map = HashMap::new();
    for row in rows {
        let (subject_key, start_at, end_at) = row.map_err(AppError::from)?;
        let Some(subject) = Subject::from_key(&subject_key) else {
            continue;
        };
        if !SUBJECT_EXPORT_ORDER.iter().any(|(s, _)| *s == subject) {
            continue;
        }
        let start_ts = start_at
            .as_deref()
            .and_then(parse_datetime)
            .map(|dt| dt.and_utc().timestamp_millis());
        map.insert(
            subject,
            SessionInfo {
                subject_label: subject_label(subject),
                start_at,
                end_at,
                start_ts,
            },
        );
    }
    Ok(map)
}

fn load_exam_rows_for_grade(
    conn: &rusqlite::Connection,
    grade_name: &str,
) -> Result<Vec<ExamRow>, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT a.admission_no, a.student_name, a.class_name, s.subject, COALESCE(sp.space_name, ''), COALESCE(a.seat_no, 0)
            FROM latest_exam_plan_student_allocations a
            JOIN latest_exam_plan_sessions s ON s.id = a.session_id
            LEFT JOIN latest_exam_plan_spaces sp ON sp.id = a.space_id
            WHERE s.grade_name = ?1 AND a.allocation_type = 'exam'
            "#,
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![grade_name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(AppError::from)?;

    let mut list = Vec::new();
    for row in rows {
        let (admission_no, student_name, class_name, subject_key, space_name, seat_no) =
            row.map_err(AppError::from)?;
        let Some(subject) = Subject::from_key(&subject_key) else {
            continue;
        };
        list.push(ExamRow {
            admission_no,
            student_name,
            class_name,
            subject,
            subject_label: subject_label(subject),
            space_name,
            seat_no,
        });
    }
    Ok(list)
}

fn load_students_for_grade(
    conn: &rusqlite::Connection,
    grade_name: &str,
) -> Result<Vec<StudentBase>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT admission_no, student_name, class_name, class_rank FROM latest_student_scores WHERE grade_name = ?1",
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![grade_name], |row| {
            Ok(StudentBase {
                admission_no: row.get(0)?,
                student_name: row.get(1)?,
                class_name: row.get(2)?,
                class_rank: row.get(3)?,
            })
        })
        .map_err(AppError::from)?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(AppError::from)?);
    }
    list.sort_by(|a, b| {
        sort_class_like(&a.class_name, &b.class_name)
            .then(a.class_rank.cmp(&b.class_rank))
            .then(a.admission_no.cmp(&b.admission_no))
    });
    Ok(list)
}

fn write_common_header(
    sheet: &mut Worksheet,
    row: u32,
    title: &str,
    with_title: bool,
    fmt: &Format,
) -> Result<(), XlsxError> {
    if with_title {
        sheet.merge_range(row, 0, row, 5, title, fmt)?;
        sheet.write_string_with_format(row + 1, 0, "姓名", fmt)?;
        sheet.write_string_with_format(row + 1, 1, "班级", fmt)?;
        sheet.write_string_with_format(row + 1, 2, "考号", fmt)?;
        sheet.write_string_with_format(row + 1, 3, "考试科目", fmt)?;
        sheet.write_string_with_format(row + 1, 4, "考场", fmt)?;
        sheet.write_string_with_format(row + 1, 5, "座位号", fmt)?;
    } else {
        sheet.write_string_with_format(row, 0, "姓名", fmt)?;
        sheet.write_string_with_format(row, 1, "班级", fmt)?;
        sheet.write_string_with_format(row, 2, "考号", fmt)?;
        sheet.write_string_with_format(row, 3, "考试科目", fmt)?;
        sheet.write_string_with_format(row, 4, "考场", fmt)?;
        sheet.write_string_with_format(row, 5, "座位号", fmt)?;
    }
    Ok(())
}

fn write_rows(
    sheet: &mut Worksheet,
    start_row: u32,
    rows: &[&ExamRow],
    fmt: &Format,
) -> Result<(), XlsxError> {
    for (idx, row) in rows.iter().enumerate() {
        let r = start_row + idx as u32;
        sheet.write_string_with_format(r, 0, &row.student_name, fmt)?;
        sheet.write_string_with_format(r, 1, &row.class_name, fmt)?;
        sheet.write_string_with_format(r, 2, &row.admission_no, fmt)?;
        sheet.write_string_with_format(r, 3, row.subject_label, fmt)?;
        sheet.write_string_with_format(r, 4, &row.space_name, fmt)?;
        sheet.write_number_with_format(r, 5, row.seat_no as f64, fmt)?;
    }
    Ok(())
}

fn subject_sheet_name(label: &str) -> String {
    label.chars().take(31).collect::<String>()
}

fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum::<usize>()
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

fn apply_summary_column_widths(sheet: &mut Worksheet, widths: &[usize]) -> Result<(), XlsxError> {
    for (idx, width) in widths.iter().enumerate() {
        let visual = (*width).max(10) as f64 + 2.0;
        sheet.set_column_width(idx as u16, visual)?;
    }
    Ok(())
}

fn build_tickets_html(
    exam_title: &str,
    notices: &[String],
    students: &[(StudentBase, Vec<TicketExamItem>)],
) -> String {
    let template = include_str!("../zhunkaozhengTemplate.html");
    let page_start_marker = "<!--TICKET_PAGE_START-->";
    let page_end_marker = "<!--TICKET_PAGE_END-->";
    let Some(page_start) = template.find(page_start_marker) else {
        return "<!DOCTYPE html><html><body><p>准考证模板缺少 TICKET_PAGE_START 标记</p></body></html>".to_string();
    };
    let Some(page_end) = template.find(page_end_marker) else {
        return "<!DOCTYPE html><html><body><p>准考证模板缺少 TICKET_PAGE_END 标记</p></body></html>".to_string();
    };
    let page_tpl_start = page_start + page_start_marker.len();
    if page_end <= page_tpl_start {
        return "<!DOCTYPE html><html><body><p>准考证模板页面标记顺序错误</p></body></html>".to_string();
    }
    let page_tpl = &template[page_tpl_start..page_end];
    let mut pages_html = String::new();

    for (student, exams) in students {
        // 这里按新版准考证模板生成四列表格：科目、考试时间、考场、座号。
        let exam_rows = exams
            .iter()
            .map(|item| {
                format!(
                    r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                    escape_html(item.subject_label),
                    escape_html(&item.exam_time),
                    escape_html(&item.room),
                    item.seat
                )
            })
            .collect::<Vec<_>>()
            .join("");
        let notice_rows = notices
            .iter()
            .map(|notice| strip_notice_leading_index(notice))
            .filter(|notice| !notice.is_empty())
            .map(|notice| format!(r#"<li>{}</li>"#, escape_html(&notice)))
            .collect::<Vec<_>>()
            .join("");

        let page_html = page_tpl
            .replace("{{EXAM_TITLE}}", &escape_html(exam_title))
            .replace("{{STUDENT_NAME}}", &escape_html(&student.student_name))
            .replace("{{CLASS_NAME}}", &escape_html(&student.class_name))
            .replace("{{ADMISSION_NO}}", &escape_html(&student.admission_no))
            .replace("{{EXAM_ROWS}}", &exam_rows)
            .replace("{{NOTICE_ITEMS}}", &notice_rows);
        pages_html.push_str(&page_html);
    }

    let mut html = String::new();
    html.push_str(&template[..page_start]);
    html.push_str(&pages_html);
    html.push_str(&template[page_end + page_end_marker.len()..]);
    html.replace("{{EXAM_TITLE}}", &escape_html(exam_title))
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

fn export_batch_dir(root: &Path, exam_title: &str) -> PathBuf {
    let sanitized_title = sanitize_file_name_segment(exam_title);
    if sanitized_title.is_empty() {
        root.join("考场安排")
    } else {
        root.join(format!("{sanitized_title}考场安排"))
    }
}

pub fn generate_export_files<F>(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    mut on_grade_done: F,
) -> Result<PathBuf, AppError>
where
    F: FnMut(&str, usize, usize),
{
    let (exam_title, exam_notices) = settings_from_db(conn)?;
    if exam_title.trim().is_empty() {
        return Err(AppError::new("未配置考试标题，无法导出"));
    }

    let grades = load_grades(conn)?;
    if grades.is_empty() {
        return Err(AppError::new("暂无考场分配快照数据，请先执行考场分配"));
    }

    let root = export_root_dir(app)?;
    let batch_dir = export_batch_dir(&root, &exam_title);
    if batch_dir.exists() {
        fs::remove_dir_all(&batch_dir)
            .map_err(|e| AppError::new(format!("清理旧导出目录失败: {e}")))?;
    }
    fs::create_dir_all(&batch_dir)
        .map_err(|e| AppError::new(format!("创建导出批次目录失败: {e}")))?;

    let total_grades = grades.len();
    for (grade_index, grade) in grades.iter().enumerate() {
        let grade_dir = batch_dir.join(grade);
        let class_dir = grade_dir.join("班级名册");
        let room_dir = grade_dir.join("考场名册");
        let ticket_dir = grade_dir.join("准考证");
        fs::create_dir_all(&class_dir)
            .map_err(|e| AppError::new(format!("创建班级名册目录失败: {e}")))?;
        fs::create_dir_all(&room_dir)
            .map_err(|e| AppError::new(format!("创建考场名册目录失败: {e}")))?;
        fs::create_dir_all(&ticket_dir)
            .map_err(|e| AppError::new(format!("创建准考证目录失败: {e}")))?;

        let sessions = load_sessions_for_grade(conn, grade)?;
        let rows = load_exam_rows_for_grade(conn, grade)?;
        let students = load_students_for_grade(conn, grade)?;

        let mut class_group: HashMap<String, Vec<&ExamRow>> = HashMap::new();
        let mut room_group: HashMap<String, Vec<&ExamRow>> = HashMap::new();
        for row in &rows {
            class_group
                .entry(row.class_name.clone())
                .or_default()
                .push(row);
            room_group
                .entry(row.space_name.clone())
                .or_default()
                .push(row);
        }

        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin);
        let cell_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_align(FormatAlign::Center);
        let summary_header_fmt = Format::new()
            .set_bold()
            .set_font_size(11.)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color(Color::RGB(0xD9D9D9))
            .set_border(FormatBorder::Thin);
        let summary_cell_fmt = Format::new()
            .set_font_size(11.)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let mut classes = class_group.keys().cloned().collect::<Vec<_>>();
        classes.sort_by(|a, b| sort_class_like(a, b));
        for class_name in classes {
            let mut wb = Workbook::new();
            let class_rows = class_group.get(&class_name).cloned().unwrap_or_default();
            for (subject, label) in SUBJECT_EXPORT_ORDER {
                let mut sheet_rows = class_rows
                    .iter()
                    .copied()
                    .filter(|r| r.subject == subject)
                    .collect::<Vec<_>>();
                sheet_rows.sort_by(|a, b| {
                    a.space_name
                        .cmp(&b.space_name)
                        .then(a.seat_no.cmp(&b.seat_no))
                        .then(a.admission_no.cmp(&b.admission_no))
                });
                let sheet = wb.add_worksheet();
                sheet
                    .set_name(subject_sheet_name(label))
                    .map_err(|e| AppError::new(format!("设置 Sheet 名失败: {e}")))?;
                write_common_header(
                    sheet,
                    0,
                    &format!("{exam_title}（班级名册）"),
                    true,
                    &header_fmt,
                )
                .map_err(|e| AppError::new(format!("写入班级名册表头失败: {e}")))?;
                write_rows(sheet, 2, &sheet_rows, &cell_fmt)
                    .map_err(|e| AppError::new(format!("写入班级名册数据失败: {e}")))?;
            }
            let path = class_dir.join(format!("{class_name}-班级名册.xlsx"));
            wb.save(&path)
                .map_err(|e| AppError::new(format!("保存班级名册失败: {e}")))?;
        }

        let mut rooms = room_group.keys().cloned().collect::<Vec<_>>();
        rooms.sort_by(|a, b| sort_class_like(a, b));
        for room_name in rooms {
            let mut wb = Workbook::new();
            let room_rows = room_group.get(&room_name).cloned().unwrap_or_default();
            for (subject, label) in SUBJECT_EXPORT_ORDER {
                let mut sheet_rows = room_rows
                    .iter()
                    .copied()
                    .filter(|r| r.subject == subject)
                    .collect::<Vec<_>>();
                sheet_rows.sort_by(|a, b| {
                    a.seat_no
                        .cmp(&b.seat_no)
                        .then(a.admission_no.cmp(&b.admission_no))
                });
                let sheet = wb.add_worksheet();
                sheet
                    .set_name(subject_sheet_name(label))
                    .map_err(|e| AppError::new(format!("设置 Sheet 名失败: {e}")))?;
                write_common_header(
                    sheet,
                    0,
                    &format!("{exam_title}（考场名册）"),
                    true,
                    &header_fmt,
                )
                .map_err(|e| AppError::new(format!("写入考场名册表头失败: {e}")))?;
                write_rows(sheet, 2, &sheet_rows, &cell_fmt)
                    .map_err(|e| AppError::new(format!("写入考场名册数据失败: {e}")))?;
            }
            let path = room_dir.join(format!("考场名册-{room_name}.xlsx"));
            wb.save(&path)
                .map_err(|e| AppError::new(format!("保存考场名册失败: {e}")))?;
        }

        let mut summary_wb = Workbook::new();
        let summary_sheet = summary_wb.add_worksheet();
        summary_sheet
            .set_name("总览")
            .map_err(|e| AppError::new(format!("设置总览 Sheet 名失败: {e}")))?;
        for (col, title) in ["姓名", "班级", "考号", "班级名次"].iter().enumerate() {
            summary_sheet
                .merge_range(0, col as u16, 1, col as u16, *title, &summary_header_fmt)
                .map_err(|e| AppError::new(format!("合并总览基础表头失败: {e}")))?;
        }

        let mut summary_widths = vec![
            display_width("姓名"),
            display_width("班级"),
            display_width("考号"),
            display_width("班级名次"),
        ];
        summary_widths.resize(4 + SUBJECT_EXPORT_ORDER.len() * 2, display_width("座位号"));

        for (idx, (subject, label)) in SUBJECT_EXPORT_ORDER.iter().enumerate() {
            let col = 4_u16 + (idx as u16 * 2);
            let title = if let Some(s) = sessions.get(subject) {
                if let (Some(start), Some(end)) = (s.start_at.as_deref(), s.end_at.as_deref()) {
                    let display = format!(
                        "{}-{}",
                        format_hm(start).unwrap_or_else(|| start.to_string()),
                        format_hm(end).unwrap_or_else(|| end.to_string())
                    );
                    format!("{}（{}）", label, display)
                } else {
                    (*label).to_string()
                }
            } else {
                (*label).to_string()
            };
            summary_sheet
                .merge_range(0, col, 0, col + 1, &title, &summary_header_fmt)
                .map_err(|e| AppError::new(format!("合并总览科目表头失败: {e}")))?;
            summary_sheet
                .write_string_with_format(1, col, "考场号", &summary_header_fmt)
                .map_err(|e| AppError::new(format!("写总览子表头失败: {e}")))?;
            summary_sheet
                .write_string_with_format(1, col + 1, "座位号", &summary_header_fmt)
                .map_err(|e| AppError::new(format!("写总览子表头失败: {e}")))?;
            let title_width = display_width(&title);
            let per_col_width = (title_width / 2)
                .max(display_width("考场号"))
                .max(display_width("座位号"));
            summary_widths[col as usize] = summary_widths[col as usize].max(per_col_width);
            summary_widths[col as usize + 1] = summary_widths[col as usize + 1].max(per_col_width);
        }

        let mut alloc_map: HashMap<(String, Subject), (String, i64)> = HashMap::new();
        for row in &rows {
            alloc_map.insert(
                (row.admission_no.clone(), row.subject),
                (row.space_name.clone(), row.seat_no),
            );
        }
        let exported_student_ids = rows
            .iter()
            .map(|row| row.admission_no.clone())
            .collect::<std::collections::HashSet<_>>();
        let exported_students = students
            .iter()
            .filter(|stu| exported_student_ids.contains(&stu.admission_no))
            .collect::<Vec<_>>();

        for (idx, stu) in exported_students.iter().enumerate() {
            let r = 2 + idx as u32;
            summary_sheet
                .write_string_with_format(r, 0, &stu.student_name, &summary_cell_fmt)
                .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
            summary_sheet
                .write_string_with_format(r, 1, &stu.class_name, &summary_cell_fmt)
                .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
            summary_sheet
                .write_string_with_format(r, 2, &stu.admission_no, &summary_cell_fmt)
                .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
            summary_sheet
                .write_number_with_format(r, 3, stu.class_rank as f64, &summary_cell_fmt)
                .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
            summary_widths[0] = summary_widths[0].max(display_width(&stu.student_name));
            summary_widths[1] = summary_widths[1].max(display_width(&stu.class_name));
            summary_widths[2] = summary_widths[2].max(display_width(&stu.admission_no));
            summary_widths[3] = summary_widths[3].max(display_width(&stu.class_rank.to_string()));

            for (sidx, (subject, _)) in SUBJECT_EXPORT_ORDER.iter().enumerate() {
                let col = 4_u16 + (sidx as u16 * 2);
                if let Some((room, seat)) = alloc_map.get(&(stu.admission_no.clone(), *subject)) {
                    summary_sheet
                        .write_string_with_format(r, col, room, &summary_cell_fmt)
                        .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
                    summary_sheet
                        .write_number_with_format(r, col + 1, *seat as f64, &summary_cell_fmt)
                        .map_err(|e| AppError::new(format!("写总览数据失败: {e}")))?;
                    summary_widths[col as usize] =
                        summary_widths[col as usize].max(display_width(room));
                    summary_widths[col as usize + 1] =
                        summary_widths[col as usize + 1].max(display_width(&seat.to_string()));
                } else {
                    // 未选科目也要显式写入空单元格，确保边框完整显示。
                    summary_sheet
                        .write_string_with_format(r, col, "", &summary_cell_fmt)
                        .map_err(|e| AppError::new(format!("写总览空单元格失败: {e}")))?;
                    summary_sheet
                        .write_string_with_format(r, col + 1, "", &summary_cell_fmt)
                        .map_err(|e| AppError::new(format!("写总览空单元格失败: {e}")))?;
                }
            }
        }
        apply_summary_column_widths(summary_sheet, &summary_widths)
            .map_err(|e| AppError::new(format!("设置总览列宽失败: {e}")))?;

        for (subject, label) in SUBJECT_EXPORT_ORDER {
            let mut srows = rows
                .iter()
                .filter(|r| r.subject == subject)
                .collect::<Vec<_>>();
            srows.sort_by(|a, b| {
                a.space_name
                    .cmp(&b.space_name)
                    .then(a.seat_no.cmp(&b.seat_no))
                    .then(a.admission_no.cmp(&b.admission_no))
            });
            let sheet = summary_wb.add_worksheet();
            sheet
                .set_name(subject_sheet_name(label))
                .map_err(|e| AppError::new(format!("设置科目 Sheet 名失败: {e}")))?;
            write_common_header(sheet, 0, "", false, &header_fmt)
                .map_err(|e| AppError::new(format!("写科目表头失败: {e}")))?;
            write_rows(sheet, 1, &srows, &cell_fmt)
                .map_err(|e| AppError::new(format!("写科目数据失败: {e}")))?;
        }
        summary_wb
            .save(grade_dir.join("考场分配结果汇总.xlsx"))
            .map_err(|e| AppError::new(format!("保存年级汇总失败: {e}")))?;

        let mut ticket_students = Vec::<(StudentBase, Vec<TicketExamItem>)>::new();
        for stu in &exported_students {
            let mut exams = Vec::new();
            for (subject, _) in SUBJECT_EXPORT_ORDER {
                if let Some((room, seat)) = alloc_map.get(&(stu.admission_no.clone(), subject)) {
                    if let Some(session) = sessions.get(&subject) {
                        if let (Some(start), Some(end), Some(start_ts)) = (
                            session.start_at.as_deref(),
                            session.end_at.as_deref(),
                            session.start_ts,
                        ) {
                            exams.push(TicketExamItem {
                                subject_label: session.subject_label,
                                exam_time: format!(
                                    "{} {}",
                                    format_ticket_date(start).unwrap_or_default(),
                                    format_ticket_time(start, end).unwrap_or_default()
                                )
                                .trim()
                                .to_string(),
                                room: room.clone(),
                                seat: *seat,
                                start_ts,
                            });
                        }
                    }
                }
            }
            exams.sort_by(|a, b| {
                a.start_ts
                    .cmp(&b.start_ts)
                    .then(a.subject_label.cmp(b.subject_label))
            });
            if !exams.is_empty() {
                ticket_students.push(((*stu).clone(), exams));
            }
        }
        let html = build_tickets_html(&exam_title, &exam_notices, &ticket_students);
        let html_path = ticket_dir.join("准考证.html");
        fs::write(&html_path, html)
            .map_err(|e| AppError::new(format!("写入准考证 HTML 失败: {e}")))?;
        on_grade_done(grade, grade_index + 1, total_grades);
    }

    Ok(batch_dir)
}

pub fn zip_existing_export_bundle(
    app: &AppHandle,
) -> Result<ExportLatestExamAllocationBundleResult, AppError> {
    let exported_at = Utc::now().to_rfc3339();
    let root = export_root_dir(app)?;
    let conn = crate::score::open_connection(app)?;
    let (exam_title, _) = settings_from_db(&conn)?;
    let batch_dir = export_batch_dir(&root, &exam_title);
    if !batch_dir.exists() {
        return Err(AppError::new("尚未生成导出文件，请先执行考场分配"));
    }
    let grade_count = fs::read_dir(&batch_dir)
        .map_err(|e| AppError::new(format!("读取导出目录失败: {e}")))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count() as i64;
    let file_count = count_files_recursively(&batch_dir)?;
    Ok(ExportLatestExamAllocationBundleResult {
        folder_path: batch_dir.to_string_lossy().to_string(),
        grade_count,
        file_count,
        exported_at,
    })
}

fn count_files_recursively(dir: &Path) -> Result<i64, AppError> {
    let mut count = 0_i64;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .map_err(|e| AppError::new(format!("读取导出目录失败: {e}")))?;
        for entry in entries {
            let entry = entry.map_err(|e| AppError::new(format!("读取目录项失败: {e}")))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                count += 1;
            }
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn export_latest_exam_allocation_bundle(
    app: AppHandle,
) -> Result<ExportLatestExamAllocationBundleResult, String> {
    let result = zip_existing_export_bundle(&app);
    result.map_err(|e| {
        app_log::log_error(
            &app,
            "export_bundle.export_latest_exam_allocation_bundle",
            &e.to_string(),
        );
        e.to_string()
    })
}
