import { useEffect, useMemo, useState } from "react";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import type { CourseSubstitutionCandidate } from "../../../entities/course-management/model";
import { useAppDialog } from "../../../shared/ui/appDialog";
import { FluentSelect, InfoHint, TableCard } from "../../../widgets/common/index.react";
import { useReactCourseManagementStore } from "../store";

const REASON_OPTIONS = [
  { label: "请假", value: "请假" },
  { label: "公出", value: "公出" },
  { label: "培训", value: "培训" },
  { label: "临时换课", value: "临时换课" },
  { label: "其他", value: "其他" },
];

function excelFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || path;
}

function formatDate(value: string) {
  const date = new Date(`${value}T00:00:00`);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return `${month}月${day}日（${weekdays[date.getDay()]}）`;
}

function normalizePeriodText(value: string) {
  return value.replace(/\s+/g, "");
}

function candidateKey(item: CourseSubstitutionCandidate) {
  return `${item.targetDate}:${item.sourceEntryId}:${item.sourceTeacherName}`;
}

export default function CourseSubstitutionPanel() {
  const store = useReactCourseManagementStore();
  const { state } = store;
  const dialog = useAppDialog();
  const today = new Date().toISOString().slice(0, 10);
  const [queryTeacher, setQueryTeacher] = useState("");
  const [startDate, setStartDate] = useState(today);
  const [endDate, setEndDate] = useState(today);
  const [reason, setReason] = useState("请假");
  const [bulkTeacher, setBulkTeacher] = useState("");
  const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set());
  const [selectedPeriodIndexes, setSelectedPeriodIndexes] = useState<Set<number>>(new Set());
  const [draftTeachers, setDraftTeachers] = useState<Record<string, string>>({});
  const [expandedTeachers, setExpandedTeachers] = useState<Set<string>>(new Set());
  const [isSearching, setIsSearching] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [feedbackType, setFeedbackType] = useState<"info" | "success" | "warning" | "error">("info");
  const [feedbackMessage, setFeedbackMessage] = useState("选择教师和日期范围后，查询该教师涉及的课次并逐节指定代课教师。");

  const importOptions = useMemo(
    () => state.imports.map((item) => ({ label: excelFileName(item.sourceFile), value: item.id })),
    [state.imports],
  );
  const teacherOptions = useMemo(() => state.teachers.map((teacher) => ({ label: teacher, value: teacher })), [state.teachers]);
  const periodOptions = useMemo(
    () => state.periods.map((period) => ({
      label: period.sectionLabel ? `${period.sectionLabel} ${period.periodLabel}` : period.periodLabel,
      value: period.periodIndex,
      sectionLabel: period.sectionLabel,
      periodLabel: period.periodLabel,
    })),
    [state.periods],
  );
  const changesGroupedByTeacher = useMemo(() => {
    const map = new Map<string, typeof state.scheduleChanges>();
    for (const change of state.scheduleChanges) {
      const key = change.sourceTeacherName;
      const current = map.get(key) ?? [];
      map.set(key, [...current, change]);
    }
    return Array.from(map.entries()).sort(([a], [b]) => a.localeCompare(b, "zh-CN"));
  }, [state.scheduleChanges]);
  const allSelected = state.substitutionCandidates.length > 0
    && state.substitutionCandidates.every((item) => selectedKeys.has(candidateKey(item)));
  const saveableCount = state.substitutionCandidates.filter((item) => {
    const key = candidateKey(item);
    const teacher = draftTeachers[key] ?? item.existingChange?.actualTeacherName ?? "";
    return selectedKeys.has(key) && teacher && teacher !== item.sourceTeacherName;
  }).length;

  useEffect(() => {
    if (hasDesktopRuntime()) {
      void store.loadOptions();
    }
  }, []);

  useEffect(() => {
    setSelectedPeriodIndexes(new Set());
  }, [state.selectedImportId]);

  useEffect(() => {
    if (selectedPeriodIndexes.size === 0 && periodOptions.length > 0) {
      setSelectedPeriodIndexes(new Set(periodOptions.map((item) => item.value)));
    }
  }, [periodOptions, selectedPeriodIndexes.size]);

  function substituteOptionsFor(sourceTeacher: string) {
    return teacherOptions.filter((item) => item.value !== sourceTeacher);
  }

  function periodMatchesGroup(
    period: { value: number; label: string; sectionLabel: string; periodLabel: string },
    group: "early" | "morning" | "afternoon" | "evening",
  ) {
    const section = normalizePeriodText(period.sectionLabel);
    const labelText = normalizePeriodText(`${period.periodLabel}${period.label}`);
    if (group === "afternoon" && labelText.includes("午练")) return true;
    if (section) {
      if (group === "early") return section.includes("早");
      if (group === "morning") return section.includes("上午");
      if (group === "afternoon") return section.includes("下午");
      return section.includes("晚");
    }
    if (group === "evening") return labelText.includes("晚");
    if (group === "afternoon") return labelText.includes("下午") || labelText.includes("午练") || labelText.includes("午间");
    if (group === "early") return labelText.includes("早上") || labelText.includes("晨读") || labelText.includes("早读");
    return labelText.includes("上午") || labelText.includes("大课间");
  }

  async function searchCandidates() {
    if (!queryTeacher || !startDate || !endDate) {
      setFeedbackType("error");
      setFeedbackMessage("请选择换课教师和日期范围。");
      return;
    }
    if (selectedPeriodIndexes.size === 0) {
      setFeedbackType("error");
      setFeedbackMessage("请至少选择一个涉及节次。");
      return;
    }
    setIsSearching(true);
    try {
      const candidates = await store.findSubstitutionCandidates({
        teacherName: queryTeacher,
        startDate,
        endDate,
        periodIndexes: Array.from(selectedPeriodIndexes).sort((a, b) => a - b),
      });
      setSelectedKeys(new Set());
      const drafts: Record<string, string> = {};
      for (const item of candidates) {
        if (item.existingChange) drafts[candidateKey(item)] = item.existingChange.actualTeacherName;
      }
      setDraftTeachers(drafts);
      setFeedbackType(candidates.length > 0 ? "success" : "warning");
      setFeedbackMessage(candidates.length > 0 ? `找到 ${candidates.length} 节相关课程。` : "该时间范围内没有找到该教师的课程。");
    } catch (error) {
      setFeedbackType("error");
      setFeedbackMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSearching(false);
    }
  }

  async function saveSelected() {
    const items = state.substitutionCandidates
      .filter((item) => selectedKeys.has(candidateKey(item)))
      .map((item) => ({
        sourceEntryId: item.sourceEntryId,
        targetDate: item.targetDate,
        sourceTeacherName: item.sourceTeacherName,
        actualTeacherName: draftTeachers[candidateKey(item)] ?? item.existingChange?.actualTeacherName ?? "",
      }))
      .filter((item) => item.actualTeacherName && item.actualTeacherName !== item.sourceTeacherName);
    if (items.length === 0) {
      setFeedbackType("error");
      setFeedbackMessage("请至少为一节课指定有效的代课教师。");
      return;
    }
    setIsSaving(true);
    try {
      await store.saveSubstitutions({ reason, remark: "", items });
      setFeedbackType("success");
      setFeedbackMessage(`已保存 ${items.length} 条调代课记录。`);
      await searchCandidates();
    } catch (error) {
      setFeedbackType("error");
      setFeedbackMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function revokeChange(changeId: number) {
    const confirmed = await dialog.confirm({
      tone: "warning",
      icon: "undo",
      title: "删除调代课记录",
      summary: "确定删除这条调代课记录吗？删除后该记录将无法恢复。",
      confirmText: "确认删除",
      cancelText: "取消",
    });
    if (!confirmed) return;
    try {
      await store.revokeScheduleChange(changeId);
      setFeedbackType("success");
      setFeedbackMessage("调代课记录已删除。");
    } catch (error) {
      setFeedbackType("error");
      setFeedbackMessage(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <section className="panel">
      <div className="course-substitution-workspace">
        <TableCard title="新建调代课" meta={`已查询 ${state.substitutionCandidates.length} 节课`}>
          <div className="substitution-form">
            <div className="form-grid course-substitution-grid">
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
                <span>换课教师</span>
                <FluentSelect modelValue={queryTeacher} options={teacherOptions} placeholder="选择教师" searchable onUpdateModelValue={(value) => setQueryTeacher(String(value))} />
              </label>
              <label className="control-field">
                <span>开始日期</span>
                <input className="glass-input" type="date" value={startDate} onChange={(event) => setStartDate(event.target.value)} />
              </label>
              <label className="control-field">
                <span>结束日期</span>
                <input className="glass-input" type="date" value={endDate} onChange={(event) => setEndDate(event.target.value)} />
              </label>
              <label className="control-field">
                <span>原因</span>
                <FluentSelect modelValue={reason} options={REASON_OPTIONS} onUpdateModelValue={(value) => setReason(String(value))} />
              </label>
              <button className="action-btn primary" type="button" disabled={isSearching} onClick={() => void searchCandidates()}>
                <span className="material-symbols-rounded" aria-hidden="true">search</span>
                查询课次
              </button>
            </div>

            <InfoHint type={feedbackType} text={feedbackMessage} />

            <div className="period-picker">
              <div className="period-picker-head">
                <span>涉及节次</span>
                <div className="period-actions">
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set(periodOptions.map((item) => item.value)))}>全部</button>
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set(periodOptions.filter((item) => periodMatchesGroup(item, "early")).map((item) => item.value)))}>早上</button>
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set(periodOptions.filter((item) => periodMatchesGroup(item, "morning")).map((item) => item.value)))}>上午</button>
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set(periodOptions.filter((item) => periodMatchesGroup(item, "afternoon")).map((item) => item.value)))}>下午</button>
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set(periodOptions.filter((item) => periodMatchesGroup(item, "evening")).map((item) => item.value)))}>晚上</button>
                  <button type="button" disabled={periodOptions.length === 0} onClick={() => setSelectedPeriodIndexes(new Set())}>清空</button>
                </div>
              </div>
              {periodOptions.length === 0 ? (
                <div className="period-empty">请选择已导入并设置节次的课表批次</div>
              ) : (
                <div className="period-buttons">
                  {periodOptions.map((period) => (
                    <button
                      key={period.value}
                      type="button"
                      className={selectedPeriodIndexes.has(period.value) ? "active" : ""}
                      onClick={() => setSelectedPeriodIndexes((current) => {
                        const next = new Set(current);
                        if (next.has(period.value)) next.delete(period.value);
                        else next.add(period.value);
                        return next;
                      })}
                    >
                      {period.label}
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="bulk-row">
              <FluentSelect modelValue={bulkTeacher} options={teacherOptions} placeholder="批量指定代课教师" searchable onUpdateModelValue={(value) => setBulkTeacher(String(value))} />
              <button
                className="action-btn secondary"
                type="button"
                disabled={!bulkTeacher || selectedKeys.size === 0}
                onClick={() => setDraftTeachers((current) => {
                  const next = { ...current };
                  for (const item of state.substitutionCandidates) {
                    const key = candidateKey(item);
                    if (selectedKeys.has(key) && item.sourceTeacherName !== bulkTeacher) {
                      next[key] = bulkTeacher;
                    }
                  }
                  return next;
                })}
              >
                <span className="material-symbols-rounded" aria-hidden="true">group_add</span>
                批量指定
              </button>
              <button className="action-btn primary" type="button" disabled={isSaving || saveableCount === 0} onClick={() => void saveSelected()}>
                <span className="material-symbols-rounded" aria-hidden="true">save</span>
                保存生效
              </button>
            </div>

            <div className="candidate-table-wrap">
              <table className="data-table">
                <thead>
                  <tr>
                    <th className="check-col">
                      <input
                        type="checkbox"
                        checked={allSelected}
                        onChange={() => setSelectedKeys(allSelected ? new Set() : new Set(state.substitutionCandidates.map(candidateKey)))}
                      />
                    </th>
                    <th>日期</th>
                    <th>节次</th>
                    <th>班级</th>
                    <th>科目</th>
                    <th>原任课</th>
                    <th>代课教师</th>
                    <th>状态</th>
                  </tr>
                </thead>
                <tbody>
                  {state.substitutionCandidates.length === 0 ? (
                    <tr>
                      <td colSpan={8} className="empty-cell">按教师和日期范围查询需要处理的课次</td>
                    </tr>
                  ) : null}
                  {state.substitutionCandidates.map((item) => {
                    const key = candidateKey(item);
                    return (
                      <tr key={key}>
                        <td className="check-col">
                          <input
                            type="checkbox"
                            checked={selectedKeys.has(key)}
                            onChange={() => setSelectedKeys((current) => {
                              const next = new Set(current);
                              if (next.has(key)) next.delete(key);
                              else next.add(key);
                              return next;
                            })}
                          />
                        </td>
                        <td>{formatDate(item.targetDate)}</td>
                        <td>
                          <strong>{item.periodLabel}</strong>
                          <span>{item.sectionLabel}</span>
                        </td>
                        <td>{item.displayClassName}</td>
                        <td>{item.subject}</td>
                        <td>{item.sourceTeacherName}</td>
                        <td>
                          <FluentSelect
                            modelValue={draftTeachers[key] ?? item.existingChange?.actualTeacherName ?? ""}
                            options={substituteOptionsFor(item.sourceTeacherName)}
                            placeholder="选择代课教师"
                            searchable
                            onUpdateModelValue={(value) => {
                              const teacher = String(value);
                              setDraftTeachers((current) => ({ ...current, [key]: teacher }));
                              if (teacher) {
                                setSelectedKeys((current) => new Set([...current, key]));
                              }
                            }}
                          />
                        </td>
                        <td>
                          <span className={`status-pill ${item.existingChange ? "active" : ""}`}>
                            {item.existingChange ? `已由 ${item.existingChange.actualTeacherName} 代课` : "待安排"}
                          </span>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </TableCard>

        <TableCard title="已生效记录" meta={`${state.scheduleChanges.length} 条记录`}>
          <div className="change-list">
            {state.scheduleChanges.length === 0 ? (
              <div className="empty-state">
                <span className="material-symbols-rounded" aria-hidden="true">event_available</span>
                <strong>暂无调代课记录</strong>
              </div>
            ) : null}
            {changesGroupedByTeacher.map(([teacherName, changes]) => (
              <div key={teacherName} className="teacher-group">
                <button
                  className={`teacher-group-header ${expandedTeachers.has(teacherName) ? "expanded" : ""}`}
                  type="button"
                  onClick={() => setExpandedTeachers((current) => {
                    const next = new Set(current);
                    if (next.has(teacherName)) next.delete(teacherName);
                    else next.add(teacherName);
                    return next;
                  })}
                >
                  <span className="material-symbols-rounded expand-icon" aria-hidden="true">expand_more</span>
                  <span className="teacher-name">{teacherName}</span>
                  <span className="group-stats">{changes.length} 条记录</span>
                </button>
                {expandedTeachers.has(teacherName) ? (
                  <div className="teacher-group-body">
                    {changes.map((change) => (
                      <div key={change.id} className="change-row">
                        <div className="change-main">
                          <strong>{formatDate(change.targetDate)} {change.periodLabel} {change.displayClassName} {change.subject}</strong>
                          <span>代课：{change.actualTeacherName}</span>
                          <small>{change.reason || "未填写原因"}{change.remark ? ` / ${change.remark}` : ""}</small>
                        </div>
                        <div className="change-actions">
                          <button className="icon-btn danger" type="button" onClick={() => void revokeChange(change.id)}>
                            <span className="material-symbols-rounded" aria-hidden="true">undo</span>
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : null}
              </div>
            ))}
          </div>
        </TableCard>
      </div>
    </section>
  );
}
