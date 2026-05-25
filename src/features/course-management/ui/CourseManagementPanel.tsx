import { useEffect, useMemo, useState } from "react";
import type { LucideIcon } from "lucide-react";
import { Building2, CalendarX, Save, Trash2, Users, UserCheck } from "lucide-react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import type { CoursePeriodSlot, CourseScheduleEntry, CourseViewType } from "../../../entities/course-management/model";
import { useAppDialog } from "../../../shared/ui/appDialog";
import { FilterToolbar, FluentSelect, InfoHint, TableCard } from "../../../widgets/common/index.react";
import { useReactCourseManagementStore } from "../store";

const DAYS = [
  { value: 1, label: "周一" },
  { value: 2, label: "周二" },
  { value: 3, label: "周三" },
  { value: 4, label: "周四" },
  { value: 5, label: "周五" },
  { value: 6, label: "周六" },
  { value: 7, label: "周日" },
] as const;

const VIEW_TYPE_OPTIONS: Array<{ value: CourseViewType; label: string; icon: LucideIcon }> = [
  { value: "admin_class", label: "行政班", icon: Building2 },
  { value: "foreign_class", label: "教学班", icon: Users },
  { value: "teacher", label: "教师", icon: UserCheck },
];

function normalizeDroppedPath(rawPath: string) {
  const trimmed = rawPath.trim();
  if (!trimmed.startsWith("file://")) return trimmed;
  try {
    const url = new URL(trimmed);
    return decodeURIComponent(url.pathname)
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  } catch {
    return decodeURIComponent(trimmed.replace(/^file:\/\//i, ""))
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  }
}

function pickExcelPath(paths: string[]) {
  for (const rawPath of paths) {
    const normalized = normalizeDroppedPath(rawPath);
    const lowerPath = normalized.toLowerCase();
    if (lowerPath.endsWith(".xlsx") || lowerPath.endsWith(".xls")) {
      return normalized;
    }
  }
  return undefined;
}

function sectionToneClass(section: string) {
  const normalized = section.replace(/\s+/g, "");
  if (normalized.includes("早")) return "tone-early";
  if (normalized.includes("上午")) return "tone-morning";
  if (normalized.includes("下午")) return "tone-afternoon";
  if (normalized.includes("晚")) return "tone-evening";
  return "tone-default";
}

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function excelFileName(path: string, fallback = "") {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || fallback || path;
}

export default function CourseManagementPanel() {
  const store = useReactCourseManagementStore();
  const { state } = store;
  const dialog = useAppDialog();
  const [isDragging, setIsDragging] = useState(false);
  const [selectedWeekIndex, setSelectedWeekIndex] = useState(1);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [isDeletingImport, setIsDeletingImport] = useState(false);

  const targetOptions = useMemo(() => {
    if (state.viewType === "teacher") {
      return state.teachers.map((teacher) => ({ label: teacher, value: teacher }));
    }
    if (state.viewType === "foreign_class") {
      return state.foreignClasses.map((item) => ({ label: item.displayName, value: item.className }));
    }
    return state.adminClasses.map((item) => ({ label: item.displayName, value: item.className }));
  }, [state.adminClasses, state.foreignClasses, state.teachers, state.viewType]);

  const importOptions = useMemo(
    () => state.imports.map((item) => ({ label: excelFileName(item.sourceFile), value: item.id })),
    [state.imports],
  );

  const selectedBatch = state.selectedImport;
  const scheduleTitle = (targetOptions.find((item) => item.value === state.target)?.label || state.target || "课表");
  const scheduleMeta = `双周循环，共 ${state.schedule?.entries.length ?? 0} 条记录`;

  const weekIndexes = useMemo(() => {
    const periodWeeks = state.schedule?.periods ?? [];
    const source = periodWeeks.length > 0 ? periodWeeks : (state.schedule?.entries ?? []);
    return Array.from(new Set(source.map((entry) => entry.weekIndex))).sort((a, b) => a - b);
  }, [state.schedule]);

  useEffect(() => {
    if (weekIndexes.length > 0 && !weekIndexes.includes(selectedWeekIndex)) {
      setSelectedWeekIndex(weekIndexes[0]);
    }
  }, [selectedWeekIndex, weekIndexes]);

  const currentWeekEntries = useMemo(
    () => (state.schedule?.entries ?? []).filter((entry) => entry.weekIndex === selectedWeekIndex),
    [selectedWeekIndex, state.schedule?.entries],
  );

  const currentWeekPeriods = useMemo<readonly CoursePeriodSlot[]>(
    () => (state.schedule?.periods ?? []).filter((period) => period.weekIndex === selectedWeekIndex),
    [selectedWeekIndex, state.schedule?.periods],
  );

  const periodRows = useMemo(() => {
    const map = new Map<number, { index: number; label: string; section: string }>();
    for (const period of currentWeekPeriods) {
      map.set(period.periodIndex, { index: period.periodIndex, label: period.periodLabel, section: period.sectionLabel });
    }
    if (map.size === 0) {
      for (const entry of currentWeekEntries) {
        map.set(entry.periodIndex, { index: entry.periodIndex, label: entry.periodLabel, section: entry.sectionLabel });
      }
    }
    const rows = Array.from(map.values()).sort((a, b) => a.index - b.index).map((period) => ({
      ...period,
      isSectionStart: false,
      sectionSpan: 1,
    }));
    let index = 0;
    // Preserve stable section grouping so row spans remain deterministic.
    while (index < rows.length) {
      const section = rows[index].section;
      let span = 1;
      while (index + span < rows.length && rows[index + span].section === section) {
        span += 1;
      }
      rows[index].isSectionStart = true;
      rows[index].sectionSpan = span;
      index += span;
    }
    return rows;
  }, [currentWeekEntries, currentWeekPeriods]);

  useEffect(() => {
    let unlistenDragDrop: (() => void) | null = null;

    async function bindWindowEvents() {
      if (!hasDesktopRuntime()) {
        return;
      }
      const appWindow = getCurrentWebviewWindow();
      unlistenDragDrop = await appWindow.onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          setIsDragging(true);
          return;
        }
        if (event.payload.type === "leave") {
          setIsDragging(false);
          return;
        }
        if (event.payload.type === "drop") {
          setIsDragging(false);
          const excelPath = pickExcelPath(event.payload.paths);
          if (excelPath) {
            void store.importExcel(excelPath);
            return;
          }
          store.setImportFeedback("error", "已收到拖拽，但未识别到 Excel 文件");
        }
      });
      await store.loadOptions();
    }

    void bindWindowEvents();
    return () => {
      unlistenDragDrop?.();
    };
  }, []);

  function entriesFor(weekIndex: number, dayOfWeek: number, periodIndex: number): readonly CourseScheduleEntry[] {
    return currentWeekEntries.filter(
      (entry) => entry.weekIndex === weekIndex && entry.dayOfWeek === dayOfWeek && entry.periodIndex === periodIndex,
    );
  }

  async function saveImportSettings() {
    setIsSavingSettings(true);
    try {
      await store.saveSelectedImportSettings();
      store.setImportFeedback("success", "课表批次设置已保存");
    } catch (error) {
      store.setImportFeedback("error", error instanceof Error ? error.message : String(error));
    } finally {
      setIsSavingSettings(false);
    }
  }

  async function deleteImportBatch() {
    if (!selectedBatch) return;
    const confirmed = await dialog.confirm({
      tone: "danger",
      title: "删除课表批次",
      summary: `确定删除 ${formatDate(selectedBatch.importedAt)} 导入的全部课表数据吗？删除后该批次的课表、节次与调代课引用都将不可恢复。`,
      details: [excelFileName(selectedBatch.sourceFile), `导入时间：${formatDate(selectedBatch.importedAt)}`],
      confirmText: "确认删除",
      cancelText: "取消",
    });
    if (!confirmed) return;
    setIsDeletingImport(true);
    try {
      await store.deleteSelectedImport();
      store.setImportFeedback("success", "已删除该导入批次的课表数据");
    } catch (error) {
      store.setImportFeedback("error", error instanceof Error ? error.message : String(error));
    } finally {
      setIsDeletingImport(false);
    }
  }

  const importStatusLabel =
    state.importStatus === "idle"
      ? "待导入"
      : state.importStatus === "importing"
        ? "导入中"
        : state.importStatus === "success"
          ? "导入成功"
          : "导入失败";
  const importStatusMessage = state.importStatus === "idle" ? "拖拽 Excel 导入，历史保留" : state.importMessage;
  const importHintType =
    state.importStatus === "success"
      ? "success"
      : state.importStatus === "error"
        ? "error"
        : state.importStatus === "importing"
          ? "warning"
          : "info";

  return (
    <section className={`panel ${isDragging ? "dragging" : ""}`}>
      {isDragging ? (
        <div className="drag-overlay">
          <div className="drag-card">
            <strong>松开鼠标开始导入课表</strong>
            <span>支持 `.xlsx` / `.xls`，每次导入都会保留历史批次</span>
          </div>
        </div>
      ) : null}

      <FilterToolbar items={[]}>
        <div className="toolbar-fields">
          <div className="segmented" role="tablist" aria-label="课表查看方式">
            {VIEW_TYPE_OPTIONS.map((option) => (
              <button
                key={option.value}
                type="button"
                className={state.viewType === option.value ? "active" : ""}
                onClick={() => void store.setViewType(option.value)}
              >
                <option.icon size={18} />
                {option.label}
              </button>
            ))}
          </div>
          <FluentSelect
            modelValue={state.target}
            options={targetOptions}
            onUpdateModelValue={(value) => void store.setTarget(String(value))}
            className="target-select"
          />
        </div>
      </FilterToolbar>

      <div className="import-management">
        <InfoHint className="import-status" type={importHintType} text={`${importStatusLabel}：${importStatusMessage}`} />
        <div className="import-controls-section">
          <span className="controls-label">批次设置</span>
          <div className="import-controls">
            <label className="control-field batch-field">
              <span>课表批次</span>
              <FluentSelect
                modelValue={state.selectedImportId ?? ""}
                options={importOptions}
                placeholder="未导入"
                onUpdateModelValue={(value) => void store.setSelectedImport(Number(value))}
              />
            </label>
            <label className="control-field">
              <span>生效开始</span>
              <input
                className="glass-input"
                type="date"
                value={state.settingsDraft.effectiveStartDate}
                disabled={!state.selectedImportId}
                onChange={(event) => store.setSettingsDraft({ effectiveStartDate: event.target.value })}
              />
            </label>
            <label className="control-field">
              <span>生效结束</span>
              <input
                className="glass-input"
                type="date"
                value={state.settingsDraft.effectiveEndDate}
                disabled={!state.selectedImportId}
                onChange={(event) => store.setSettingsDraft({ effectiveEndDate: event.target.value })}
              />
            </label>
            <label className="control-field week-field">
              <span>当前从第几周开始</span>
              <input
                className="glass-input"
                type="number"
                min="1"
                step="1"
                value={state.settingsDraft.startWeek}
                disabled={!state.selectedImportId}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  store.setSettingsDraft({ startWeek: Number.isFinite(value) ? Math.max(1, Math.floor(value)) : 1 });
                }}
              />
            </label>
            <button className="action-btn primary" type="button" disabled={!state.selectedImportId || isSavingSettings} onClick={() => void saveImportSettings()}>
              <Save size={18} />
              <span>保存</span>
            </button>
            <button className="action-btn danger" type="button" disabled={!state.selectedImportId || isDeletingImport} onClick={() => void deleteImportBatch()}>
              <Trash2 size={18} />
              <span>删除</span>
            </button>
          </div>
        </div>
      </div>

      <TableCard title={scheduleTitle === "课表" ? scheduleTitle : `${scheduleTitle}课表`} meta={scheduleMeta}>
        {!state.schedule || state.schedule.entries.length === 0 ? (
          <div className="empty-state">
            <CalendarX size={24} />
            <strong>{state.selectedImportId ? "当前条件下暂无课表" : "请先拖拽导入课表 Excel"}</strong>
          </div>
        ) : (
          <div className="schedule-area">
            <div className="week-switch" role="tablist" aria-label="周次切换">
              {weekIndexes.map((weekIndex) => (
                <button
                  key={weekIndex}
                  type="button"
                  className={selectedWeekIndex === weekIndex ? "active" : ""}
                  onClick={() => setSelectedWeekIndex(weekIndex)}
                >
                  第 {weekIndex} 周
                </button>
              ))}
            </div>
            <div className="schedule-table-scroll">
              <div className="schedule-grid" style={{ ["--period-count" as string]: String(periodRows.length) }}>
                <div className="corner-cell">节次</div>
                {DAYS.map((day) => (
                  <div key={day.value} className="day-head">{day.label}</div>
                ))}
                {periodRows.map((period) => (
                  <div key={`row-${selectedWeekIndex}-${period.index}`} className="schedule-row-fragment">
                    {period.isSectionStart ? (
                      <div
                        className={`section-cell ${sectionToneClass(period.section)}`}
                        style={{ gridRow: `span ${period.sectionSpan}` }}
                      >
                        <span>{period.section}</span>
                      </div>
                    ) : null}
                    <div className={`period-cell ${sectionToneClass(period.section)}`}>
                      <strong>{period.label}</strong>
                    </div>
                    {DAYS.map((day) => (
                      <div key={`${selectedWeekIndex}-${period.index}-${day.value}`} className="lesson-cell">
                        {entriesFor(selectedWeekIndex, day.value, period.index).map((entry) => (
                          <div
                            key={`${entry.className}-${entry.subject}-${entry.periodIndex}-${entry.dayOfWeek}`}
                            className={`lesson ${sectionToneClass(period.section)}`}
                          >
                            <strong>{entry.subject}</strong>
                            {state.viewType === "teacher" ? (
                              <span>{entry.displayClassName}</span>
                            ) : (
                              <span>{entry.teacherNames.join(" / ") || "--"}</span>
                            )}
                          </div>
                        ))}
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </TableCard>
    </section>
  );
}
