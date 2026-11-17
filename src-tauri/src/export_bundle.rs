use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app_log;
use crate::db::repos::export_bundle::{ExamRow, SessionInfo, StudentBase};
use crate::score::{AppError, Subject};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use serde::Serialize;
use tauri::{AppHandle, Manager};

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
struct TicketExamItem {
    subject_label: &'static str,
    exam_date: String,
    exam_time_range: String,
    room: String,
    seat: i64,
    start_ts: i64,
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
    if hour < 12 || (hour == 12 && minute == 0) {
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
    if left_period == right_period {
        Some(format!(
            "{}{} — {}",
            left_period,
            s.format("%H:%M"),
            e.format("%H:%M")
        ))
    } else {
        Some(format!(
            "{}{} — {}{}",
            left_period,
            s.format("%H:%M"),
            right_period,
            e.format("%H:%M")
        ))
    }
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

fn ordered_subjects_by_exam_time(
    sessions: &HashMap<Subject, SessionInfo>,
) -> Vec<(Subject, &'static str)> {
    let mut list = SUBJECT_EXPORT_ORDER
        .iter()
        .enumerate()
        .map(|(idx, (subject, label))| (idx, *subject, *label))
        .collect::<Vec<_>>();
    list.sort_by(|(a_idx, a_subject, _), (b_idx, b_subject, _)| {
        let a_ts = sessions.get(a_subject).and_then(|s| s.start_ts);
        let b_ts = sessions.get(b_subject).and_then(|s| s.start_ts);
        match (a_ts, b_ts) {
            (Some(a), Some(b)) => a.cmp(&b).then(a_idx.cmp(b_idx)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a_idx.cmp(b_idx),
        }
    });
    list.into_iter()
        .map(|(_, subject, label)| (subject, label))
        .collect::<Vec<_>>()
}

fn apply_roster_column_widths(sheet: &mut Worksheet) -> Result<(), XlsxError> {
    // 按 A4 纵向打印场景调整 6 列宽度，尽量铺满页面。
    let widths = [12.0_f64, 14.0, 18.0, 14.0, 14.0, 10.0];
    for (idx, width) in widths.iter().enumerate() {
        sheet.set_column_width(idx as u16, *width)?;
    }
    Ok(())
}

fn write_roster_header(
    sheet: &mut Worksheet,
    title: &str,
    title_fmt: &Format,
    header_fmt: &Format,
) -> Result<(), XlsxError> {
    sheet.merge_range(0, 0, 0, 5, title, title_fmt)?;
    sheet.write_string_with_format(1, 0, "姓名", header_fmt)?;
    sheet.write_string_with_format(1, 1, "班级", header_fmt)?;
    sheet.write_string_with_format(1, 2, "考号", header_fmt)?;
    sheet.write_string_with_format(1, 3, "考试科目", header_fmt)?;
    sheet.write_string_with_format(1, 4, "考场", header_fmt)?;
    sheet.write_string_with_format(1, 5, "座位号", header_fmt)?;
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
        return "<!DOCTYPE html><html><body><p>准考证模板页面标记顺序错误</p></body></html>"
            .to_string();
    }
    let page_tpl = &template[page_tpl_start..page_end];
    let mut pages_html = String::new();

    for (student, exams) in students {
        // 准考证表格中“考试时间”拆成日期与时段两个子列，便于打印查看。
        let exam_rows = exams
            .iter()
            .map(|item| {
                format!(
                    r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                    escape_html(item.subject_label),
                    escape_html(&item.exam_date),
                    escape_html(&item.exam_time_range),
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

pub(crate) fn export_root_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取应用数据目录失败: {e}")))?;
    dir.push("exports");
    fs::create_dir_all(&dir).map_err(|e| AppError::new(format!("创建导出目录失败: {e}")))?;
    Ok(dir)
}

pub(crate) fn export_batch_dir(root: &Path, exam_title: &str) -> PathBuf {
    let sanitized_title = sanitize_file_name_segment(exam_title);
    if sanitized_title.is_empty() {
        root.join("考场安排")
    } else {
        root.join(format!("{sanitized_title}考场安排"))
    }
}

pub(crate) struct GradeBundleInput {
    pub grade: String,
    pub sessions: HashMap<Subject, SessionInfo>,
    pub rows: Vec<ExamRow>,
    pub students: Vec<StudentBase>,
}

pub(crate) async fn bundle_data_from_draft<C: sea_orm::ConnectionTrait>(
    db: &C,
    draft: &crate::exam_plan_snapshot::ExamPlanDraft,
) -> Result<Vec<GradeBundleInput>, AppError> {
    crate::exam_plan_snapshot::validate_exam_plan_draft(draft)?;
    let mut inputs = Vec::new();
    for grade in ["高一", "高二", "高三"] {
        let mut sessions = HashMap::new();
        let mut rows = Vec::new();
        for (index, session) in draft
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| session.grade_name == grade)
        {
            let id = index as i64 + 1;
            let subject =
                Subject::from_key(&session.subject).ok_or_else(|| AppError::new("草稿科目无效"))?;
            let label = SUBJECT_EXPORT_ORDER
                .iter()
                .find(|(item, _)| *item == subject)
                .unwrap()
                .1;
            let (start, end) = draft
                .windows
                .get(&id)
                .ok_or_else(|| AppError::new("草稿缺少考试时间"))?;
            let format_time = |ts| {
                chrono::DateTime::from_timestamp_millis(ts)
                    .map(|time| time.format("%Y-%m-%dT%H:%M").to_string())
                    .ok_or_else(|| AppError::new("草稿考试时间无效"))
            };
            sessions.insert(
                subject,
                SessionInfo {
                    subject_label: label,
                    start_at: Some(format_time(*start)?),
                    end_at: Some(format_time(*end)?),
                    start_ts: Some(*start),
                },
            );
            for allocation in draft.allocations.iter().filter(|allocation| {
                allocation.session_id == id && allocation.allocation_type == "exam"
            }) {
                let space = allocation
                    .space_id
                    .and_then(|id| draft.spaces.get(id.checked_sub(1)? as usize))
                    .ok_or_else(|| AppError::new("草稿考场不存在"))?;
                rows.push(ExamRow {
                    admission_no: allocation.admission_no.clone(),
                    student_name: allocation.student_name.clone(),
                    class_name: allocation.class_name.clone(),
                    subject,
                    subject_label: label,
                    space_name: space.space_name.clone(),
                    seat_no: allocation.seat_no.unwrap_or_default(),
                });
            }
        }
        if !sessions.is_empty() {
            inputs.push(GradeBundleInput {
                grade: grade.into(),
                sessions,
                rows,
                students: crate::db::repos::export_bundle::students_for_grade(db, grade).await?,
            });
        }
    }
    Ok(inputs)
}

/// 从数据库快照重建考场导出数据，避免读取可能被用户占用的旧表格。
pub(crate) async fn bundle_data_from_snapshot<C: sea_orm::ConnectionTrait>(
    db: &C,
) -> Result<Vec<GradeBundleInput>, AppError> {
    let (plan_sessions, times, spaces, allocations) =
        crate::db::repos::export_bundle::plan_snapshot(db).await?;
    let times: HashMap<_, _> = times.into_iter().map(|row| (row.session_id, row)).collect();
    let spaces: HashMap<_, _> = spaces.into_iter().map(|row| (row.id, row)).collect();
    let mut inputs = Vec::new();
    for grade in ["高一", "高二", "高三"] {
        let mut sessions = HashMap::new();
        let mut rows = Vec::new();
        for session in plan_sessions.iter().filter(|row| row.grade_name == grade) {
            let subject = Subject::from_key(&session.subject)
                .ok_or_else(|| AppError::new("考场快照科目无效"))?;
            let label = SUBJECT_EXPORT_ORDER
                .iter()
                .find(|(item, _)| *item == subject)
                .map(|(_, label)| *label)
                .ok_or_else(|| AppError::new("考场快照科目无效"))?;
            let time = times
                .get(&session.id)
                .ok_or_else(|| AppError::new("考场快照缺少考试时间"))?;
            let start = parse_datetime(&time.start_at)
                .ok_or_else(|| AppError::new("考场快照考试时间无效"))?;
            sessions.insert(
                subject,
                SessionInfo {
                    subject_label: label,
                    start_at: Some(time.start_at.clone()),
                    end_at: Some(time.end_at.clone()),
                    start_ts: Some(start.and_utc().timestamp_millis()),
                },
            );
            for allocation in allocations
                .iter()
                .filter(|row| row.session_id == session.id && row.allocation_type == "exam")
            {
                let space = allocation
                    .space_id
                    .and_then(|id| spaces.get(&id))
                    .ok_or_else(|| AppError::new("考场快照缺少考场"))?;
                if space.session_id != session.id {
                    return Err(AppError::new("考场快照场次不匹配"));
                }
                rows.push(ExamRow {
                    admission_no: allocation.admission_no.clone(),
                    student_name: allocation.student_name.clone(),
                    class_name: allocation.class_name.clone(),
                    subject,
                    subject_label: label,
                    space_name: space.space_name.clone(),
                    seat_no: allocation.seat_no.unwrap_or_default(),
                });
            }
        }
        if !sessions.is_empty() {
            inputs.push(GradeBundleInput {
                grade: grade.into(),
                sessions,
                rows,
                students: crate::db::repos::export_bundle::students_for_grade(db, grade).await?,
            });
        }
    }
    Ok(inputs)
}

pub(crate) async fn regenerate_exam_bundle<C: sea_orm::ConnectionTrait>(
    db: &C,
    dir: &Path,
    plan_revision: &str,
    revision: &(Option<String>, Option<String>),
) -> Result<(), AppError> {
    let settings = crate::db::repos::export_bundle::settings(db).await?;
    let inputs = bundle_data_from_snapshot(db).await?;
    write_exam_bundle_data(
        dir,
        &settings.exam_title,
        &settings.exam_notices,
        &inputs,
        |_, _, _| {},
    )?;
    crate::export_batch::write_batch_identity(dir, plan_revision, revision)
}

pub(crate) fn write_exam_bundle_data<F>(
    batch_dir: &Path,
    exam_title: &str,
    exam_notices: &[String],
    grades: &[GradeBundleInput],
    mut on_grade_done: F,
) -> Result<(), AppError>
where
    F: FnMut(&str, usize, usize),
{
    if exam_title.trim().is_empty() {
        return Err(AppError::new("未配置考试标题，无法导出"));
    }

    if grades.is_empty() {
        return Err(AppError::new("暂无考场分配快照数据，请先执行考场分配"));
    }

    fs::create_dir_all(&batch_dir)
        .map_err(|e| AppError::new(format!("创建导出批次目录失败: {e}")))?;

    let total_grades = grades.len();
    for (grade_index, input) in grades.iter().enumerate() {
        let grade = &input.grade;
        crate::excel_import::normalize_sheet_grade(grade)?;
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

        let sessions = &input.sessions;
        let rows = &input.rows;
        let students = &input.students;

        let mut class_group: HashMap<String, Vec<&ExamRow>> = HashMap::new();
        let mut room_group: HashMap<String, Vec<&ExamRow>> = HashMap::new();
        for row in rows {
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
        let roster_title_fmt = Format::new()
            .set_bold()
            .set_font_size(13.)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border_bottom(FormatBorder::Thin);
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
        let subject_order = ordered_subjects_by_exam_time(&sessions);
        for class_name in classes {
            let mut wb = Workbook::new();
            let class_rows = class_group.get(&class_name).cloned().unwrap_or_default();
            for (subject, label) in &subject_order {
                let mut sheet_rows = class_rows
                    .iter()
                    .copied()
                    .filter(|r| r.subject == *subject)
                    .collect::<Vec<_>>();
                if sheet_rows.is_empty() {
                    continue;
                }
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
                apply_roster_column_widths(sheet)
                    .map_err(|e| AppError::new(format!("设置名册列宽失败: {e}")))?;
                write_roster_header(
                    sheet,
                    &format!("{exam_title}（班级名册）"),
                    &roster_title_fmt,
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
            for (subject, label) in &subject_order {
                let mut sheet_rows = room_rows
                    .iter()
                    .copied()
                    .filter(|r| r.subject == *subject)
                    .collect::<Vec<_>>();
                if sheet_rows.is_empty() {
                    continue;
                }
                sheet_rows.sort_by(|a, b| {
                    a.seat_no
                        .cmp(&b.seat_no)
                        .then(a.admission_no.cmp(&b.admission_no))
                });
                let sheet = wb.add_worksheet();
                sheet
                    .set_name(subject_sheet_name(label))
                    .map_err(|e| AppError::new(format!("设置 Sheet 名失败: {e}")))?;
                apply_roster_column_widths(sheet)
                    .map_err(|e| AppError::new(format!("设置名册列宽失败: {e}")))?;
                write_roster_header(
                    sheet,
                    &format!("{exam_title}（考场名册）"),
                    &roster_title_fmt,
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
        summary_widths.resize(4 + subject_order.len() * 2, display_width("座位号"));
        let group_col = summary_widths.len() as u16;
        summary_widths.push(display_width("普通班"));
        summary_sheet
            .write_string_with_format(0, group_col, "组别", &summary_cell_fmt)
            .map_err(|e| AppError::new(format!("写组别表头失败: {e}")))?;

        for (idx, (subject, label)) in subject_order.iter().enumerate() {
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
        for row in rows {
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
                .write_string_with_format(
                    r,
                    group_col,
                    if stu.exam_group == "comprehensive" {
                        "综评班"
                    } else {
                        "普通班"
                    },
                    &summary_cell_fmt,
                )
                .map_err(|e| AppError::new(format!("写组别失败: {e}")))?;
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

            for (sidx, (subject, _)) in subject_order.iter().enumerate() {
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

        for (subject, label) in &subject_order {
            let mut srows = rows
                .iter()
                .filter(|r| r.subject == *subject)
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
                                exam_date: format_ticket_date(start).unwrap_or_default(),
                                exam_time_range: format_ticket_time(start, end).unwrap_or_default(),
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

    Ok(())
}

pub fn zip_existing_export_bundle(
    app: &AppHandle,
) -> Result<ExportLatestExamAllocationBundleResult, AppError> {
    let exported_at = Utc::now().to_rfc3339();
    let root = export_root_dir(app)?;
    let _operation = crate::export_batch::BatchOperationGuard::acquire(&root)?;
    let db = tauri::async_runtime::block_on(crate::db::connect(app))?;
    use sea_orm::TransactionTrait;
    let (settings, revision, plan_revision) = tauri::async_runtime::block_on(async {
        let tx = db.begin().await?;
        let settings = crate::db::repos::export_bundle::settings(&tx).await?;
        let revision = crate::db::repos::export_bundle::source_revision(&tx).await?;
        let plan_revision = crate::db::repos::export_bundle::plan_revision(&tx).await?;
        tx.commit().await?;
        Ok::<_, AppError>((settings, revision, plan_revision))
    })?;
    let exam_title = settings.exam_title;
    let batch_dir = tauri::async_runtime::block_on(crate::export_batch::current_batch_dir(
        &db,
        &root,
        &exam_title,
    ))?;
    if !batch_dir.exists() {
        return Err(AppError::new("尚未生成导出文件，请先执行考场分配"));
    }
    crate::export_batch::verify_batch_identity(&batch_dir, &plan_revision, &revision.0)?;
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
        let entries =
            fs::read_dir(&current).map_err(|e| AppError::new(format!("读取导出目录失败: {e}")))?;
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

#[cfg(test)]
mod tests {
    use super::format_ticket_time;

    #[test]
    fn test_format_ticket_time_omits_duplicate_period_label() {
        let actual = format_ticket_time("2026-03-25T07:30", "2026-03-25T10:00");
        assert_eq!(actual.as_deref(), Some("上午07:30 — 10:00"));
    }

    #[test]
    fn test_format_ticket_time_keeps_cross_period_label() {
        let actual = format_ticket_time("2026-03-25T11:30", "2026-03-25T13:00");
        assert_eq!(actual.as_deref(), Some("上午11:30 — 下午13:00"));
    }

    #[test]
    fn test_format_ticket_time_treats_noon_boundary_as_morning_end() {
        let actual = format_ticket_time("2026-03-25T10:30", "2026-03-25T12:00");
        assert_eq!(actual.as_deref(), Some("上午10:30 — 12:00"));
    }
}
