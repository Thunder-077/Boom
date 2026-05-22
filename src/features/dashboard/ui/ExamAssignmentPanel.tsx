import { useEffect, useMemo, useRef, useState } from "react";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import { Subject } from "../../../entities/score/model";
import { revealInExplorer } from "../../../shared/utils/appLog";
import { ConfigCard, FluentSelect, TableCard } from "../../../widgets/common/index.react";
import { useReactExamAllocationStore } from "../store";

const GENERATION_STAGE_ORDER: Record<string, number> = {
  loading_config: 1,
  clearing_snapshot: 2,
  building_sessions: 3,
  allocating_rooms: 4,
  finalizing_results: 5,
  exporting_files: 6,
};
const TOTAL_GENERATION_STAGES = 6;
const SUBJECT_OPTIONS: Subject[] = Object.values(Subject);
const DISPLAY_SUBJECT_OPTIONS: Subject[] = SUBJECT_OPTIONS.filter(
  (subject) => subject !== Subject.Russian && subject !== Subject.Japanese,
);

interface ManualSubjectRow {
  id: number;
  subject: Subject;
  examMonthDay: string;
  startTime: string;
  endTime: string;
}

function formatDate(value?: string | null) {
  if (!value) {
    return "--";
  }
  return value.replace("T", " ").slice(0, 10);
}

function formatMonthDay(value?: string | null) {
  if (!value) {
    return "--";
  }
  const full = value.replace("T", " ").slice(0, 10);
  if (full.length !== 10) {
    return "--";
  }
  return full.slice(5, 10);
}

function normalizeMonthDay(value: string) {
  const matched = value.trim().match(/^(\d{1,2})[-/](\d{1,2})$/);
  if (!matched) {
    return null;
  }
  const month = Number(matched[1]);
  const day = Number(matched[2]);
  if (!Number.isInteger(month) || !Number.isInteger(day) || month < 1 || month > 12 || day < 1 || day > 31) {
    return null;
  }
  return `${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

function resolveFullDateFromMonthDay(monthDay: string, fallbackDate: string) {
  const normalized = normalizeMonthDay(monthDay);
  if (!normalized) {
    throw new Error("考试日期格式应为 MM-DD（例如 03-24）");
  }
  const year = fallbackDate.slice(0, 4);
  return `${year}-${normalized}`;
}

function formatTimeInput(value?: string | null) {
  if (!value) {
    return "";
  }
  return value.replace("T", " ").slice(11, 16);
}

function examTimeSubjectLabel(subject: Subject) {
  if (subject === Subject.English || subject === Subject.Russian || subject === Subject.Japanese) {
    return "外语";
  }
  return SUBJECT_LABELS[subject];
}

export default function ExamAssignmentPanel() {
  const store = useReactExamAllocationStore();
  const { state } = store;
  const [capacityForm, setCapacityForm] = useState({
    defaultCapacity: 40,
    maxCapacity: 41,
    examTitle: "",
    examNoticesText: "",
  });
  const [manualSubjectRows, setManualSubjectRows] = useState<ManualSubjectRow[]>([]);
  const [dateEditState, setDateEditState] = useState<{ sessionId: number | null; value: string }>({
    sessionId: null,
    value: "",
  });
  const [autoSaveReady, setAutoSaveReady] = useState(false);
  const [autoSaving, setAutoSaving] = useState(false);
  const [autoSaveError, setAutoSaveError] = useState("");
  const [autoSavedAt, setAutoSavedAt] = useState(0);
  const [autoSaveDirty, setAutoSaveDirty] = useState(false);
  const [isPreparingGenerate, setIsPreparingGenerate] = useState(false);
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const suppressAutoSaveRef = useRef(false);
  const manualSubjectRowIdRef = useRef(1);
  const prevManualSignatureRef = useRef("");
  const prevFieldSignatureRef = useRef("");
  const prevDraftSignatureRef = useRef("");

  const sessionTimeGradeOptions = useMemo(
    () => state.sessionTimeGradeOptions.map((grade) => ({ label: grade, value: grade })),
    [state.sessionTimeGradeOptions],
  );

  const progressPercent = useMemo(() => {
    if (state.generating || state.generationProgress.status === "running") {
      return state.generationProgress.percent;
    }
    if (state.generationProgress.status === "completed") {
      return 100;
    }
    return state.overview.generatedAt ? 100 : 0;
  }, [state.generationProgress, state.generating, state.overview.generatedAt]);

  const progressBadgeText = useMemo(() => {
    if (state.generationProgress.status === "error") return "失败";
    if (state.generating || state.generationProgress.status === "running") {
      return state.generationProgress.stageLabel || "执行中";
    }
    if (state.overview.generatedAt) return "已完成";
    return "待执行";
  }, [state.generationProgress, state.generating, state.overview.generatedAt]);

  const progressDescription = useMemo(() => {
    const progress = state.generationProgress;
    const stageIndex = GENERATION_STAGE_ORDER[progress.stage];
    if (progress.status === "running") {
      const parts = [
        stageIndex ? `当前执行第 ${stageIndex}/${TOTAL_GENERATION_STAGES} 阶段：${progress.stageLabel}` : "当前正在执行考场分配",
      ];
      if (progress.currentGrade) {
        parts.push(`当前年级：${progress.currentGrade}`);
      }
      if (progress.totalGrades > 0) {
        parts.push(`年级进度：${progress.completedGrades}/${progress.totalGrades}`);
      }
      return parts.join("，");
    }
    return "点击按钮即可开始分配考场 ~~";
  }, [state.generationProgress]);

  const progressStepText = useMemo(() => {
    const progress = state.generationProgress;
    const stageIndex = GENERATION_STAGE_ORDER[progress.stage];
    if (progress.status === "error") {
      return progress.message || "分配过程中出现错误，请查看日志。";
    }
    if (state.generating || progress.status === "running") {
      const stepPrefix = stageIndex
        ? `第 ${stageIndex}/${TOTAL_GENERATION_STAGES} 阶段 · ${progress.stageLabel}`
        : progress.stageLabel || "执行中";
      return progress.message ? `${stepPrefix}：${progress.message}` : stepPrefix;
    }
    if (state.overview.generatedAt) {
      return "考场分配完成，点击导出打开结果目录。";
    }
    return "等待开始，系统将按当前配置自动排考场。";
  }, [state.generating, state.generationProgress, state.overview.generatedAt]);

  const completeBadgeText = useMemo(() => {
    if (state.generationProgress.status === "error") return "失败";
    if (state.exporting) return "导出中";
    if (state.generating) return "执行中";
    if (state.overview.generatedAt) return "已完成";
    return "未开始";
  }, [state.exporting, state.generating, state.generationProgress.status, state.overview.generatedAt]);

  const exportFileName = useMemo(() => {
    const raw = state.lastExportFolderPath;
    if (!raw) {
      return "";
    }
    const matched = raw.match(/[^\\/]+$/);
    return matched?.[0] ?? "考场安排";
  }, [state.lastExportFolderPath]);

  const autoSaveText = useMemo(() => {
    if (state.loading) return "正在加载配置...";
    if (autoSaving) return "正在自动保存...";
    if (autoSavedAt > 0) {
      return `已自动保存（${new Date(autoSavedAt).toLocaleTimeString("zh-CN", { hour12: false })}）`;
    }
    return "修改后自动保存";
  }, [autoSavedAt, autoSaving, state.loading]);

  const generateActionText = useMemo(() => {
    if (state.generating) return "分配中...";
    if (isPreparingGenerate) return "保存配置中...";
    return "开始分配考场";
  }, [isPreparingGenerate, state.generating]);

  function getDraftDate(sessionId: number) {
    const draft = state.sessionTimeDrafts[sessionId];
    const source = draft?.startAt || draft?.endAt;
    if (source && source.length >= 10) {
      return source.slice(0, 10);
    }
    return new Date().toISOString().slice(0, 10);
  }

  function clearAutoSaveTimer() {
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = null;
    }
  }

  function scheduleAutoSave(delay = 700) {
    if (!autoSaveReady || suppressAutoSaveRef.current) return;
    setAutoSaveDirty(true);
    clearAutoSaveTimer();
    autoSaveTimerRef.current = setTimeout(() => {
      void flushAutoSave();
    }, delay);
  }

  async function persistDrafts(options: { strictManualRows?: boolean; clearManualRows?: boolean } = {}) {
    const { strictManualRows = true, clearManualRows = true } = options;
    const examNotices = capacityForm.examNoticesText
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0);

    await store.saveSettings(
      capacityForm.defaultCapacity,
      capacityForm.maxCapacity,
      capacityForm.examTitle,
      examNotices,
    );

    const extraItems: Array<{ sessionId: number; gradeName: string; subject: Subject; startAt: string; endAt: string }> = [];
    for (const row of manualSubjectRows) {
      if (!row.examMonthDay || !row.startTime || !row.endTime) {
        if (strictManualRows) {
          throw new Error(`请先完整填写 ${examTimeSubjectLabel(row.subject)} 的考试日期（月-日）、开始时间和结束时间`);
        }
        continue;
      }
      const existing = state.sessionTimes.find((item) => item.subject === row.subject);
      if (existing) {
        const fallbackDate = getDraftDate(existing.sessionId);
        const targetDate = resolveFullDateFromMonthDay(row.examMonthDay, fallbackDate);
        store.setSessionTimeDraft(existing.sessionId, "startAt", `${targetDate}T${row.startTime}`);
        store.setSessionTimeDraft(existing.sessionId, "endAt", `${targetDate}T${row.endTime}`);
        continue;
      }
      const targetDate = resolveFullDateFromMonthDay(row.examMonthDay, new Date().toISOString().slice(0, 10));
      extraItems.push({
        sessionId: -100 - manualSubjectRows.findIndex((item) => item.id === row.id),
        gradeName: state.selectedSessionTimeGradeName,
        subject: row.subject,
        startAt: `${targetDate}T${row.startTime}`,
        endAt: `${targetDate}T${row.endTime}`,
      });
    }

    await store.saveSessionTimes(extraItems);
    if (clearManualRows) {
      setManualSubjectRows([]);
    }
  }

  async function flushAutoSave() {
    if (!autoSaveReady || suppressAutoSaveRef.current || !autoSaveDirty) return;
    if (state.generating || state.saving || state.savingTimes) {
      scheduleAutoSave(400);
      return;
    }
    setAutoSaveDirty(false);
    setAutoSaving(true);
    setAutoSaveError("");
    suppressAutoSaveRef.current = true;
    try {
      await persistDrafts({ strictManualRows: false, clearManualRows: false });
      setAutoSavedAt(Date.now());
    } catch (error) {
      setAutoSaveDirty(true);
      setAutoSaveError(error instanceof Error ? error.message : String(error));
      scheduleAutoSave(1200);
    } finally {
      suppressAutoSaveRef.current = false;
      setAutoSaving(false);
    }
  }

  async function generateExamPlan() {
    if (state.generating || isPreparingGenerate) return;
    setIsPreparingGenerate(true);
    setAutoSaveError("");
    clearAutoSaveTimer();
    setAutoSaveDirty(false);
    suppressAutoSaveRef.current = true;
    try {
      await persistDrafts();
    } finally {
      suppressAutoSaveRef.current = false;
      setIsPreparingGenerate(false);
    }
    await store.generate();
  }

  async function exportBundle() {
    await store.exportLatestBundle();
  }

  async function openExportFolder() {
    if (!state.lastExportFolderPath) {
      return;
    }
    await revealInExplorer(state.lastExportFolderPath);
  }

  useEffect(() => {
    void store.loadAll();
    return () => {
      clearAutoSaveTimer();
    };
  }, []);

  useEffect(() => {
    if (state.loading) {
      return;
    }
    // Keep the initial React form in sync with the loaded persisted config
    // before auto-save starts observing local edits.
    suppressAutoSaveRef.current = true;
    setCapacityForm({
      defaultCapacity: state.settings.defaultCapacity,
      maxCapacity: state.settings.maxCapacity,
      examTitle: state.settings.examTitle ?? "",
      examNoticesText: (state.settings.examNotices ?? []).join("\n"),
    });
    suppressAutoSaveRef.current = false;
    setAutoSaveReady(true);
  }, [state.loading, state.settings]);

  useEffect(() => {
    if (!autoSaveReady || suppressAutoSaveRef.current) return;
    const signature = JSON.stringify([
      capacityForm.examTitle,
      capacityForm.examNoticesText,
      capacityForm.defaultCapacity,
      capacityForm.maxCapacity,
    ]);
    if (!prevFieldSignatureRef.current) {
      prevFieldSignatureRef.current = signature;
      return;
    }
    if (signature !== prevFieldSignatureRef.current) {
      prevFieldSignatureRef.current = signature;
      setAutoSaveError("");
      scheduleAutoSave();
    }
  }, [autoSaveReady, capacityForm]);

  useEffect(() => {
    if (!autoSaveReady || suppressAutoSaveRef.current) return;
    const signature = JSON.stringify(state.sessionTimeDrafts);
    if (!prevDraftSignatureRef.current) {
      prevDraftSignatureRef.current = signature;
      return;
    }
    if (signature !== prevDraftSignatureRef.current) {
      prevDraftSignatureRef.current = signature;
      setAutoSaveError("");
      scheduleAutoSave(850);
    }
  }, [autoSaveReady, state.sessionTimeDrafts]);

  useEffect(() => {
    if (!autoSaveReady || suppressAutoSaveRef.current) return;
    const signature = JSON.stringify(
      manualSubjectRows
        .filter((row) => row.examMonthDay.trim() && row.startTime.trim() && row.endTime.trim())
        .map((row) => ({
          id: row.id,
          subject: row.subject,
          examMonthDay: row.examMonthDay.trim(),
          startTime: row.startTime.trim(),
          endTime: row.endTime.trim(),
        })),
    );
    if (!prevManualSignatureRef.current) {
      prevManualSignatureRef.current = signature;
      return;
    }
    if (signature !== prevManualSignatureRef.current) {
      prevManualSignatureRef.current = signature;
      setAutoSaveError("");
      scheduleAutoSave(850);
    }
  }, [autoSaveReady, manualSubjectRows]);

  useEffect(() => {
    function onGlobalPointerDown(event: PointerEvent) {
      if (dateEditState.sessionId === null) {
        return;
      }
      const target = event.target as HTMLElement | null;
      if (target?.closest(".date-cell.editing")) {
        return;
      }
      const sessionId = dateEditState.sessionId;
      const normalized = normalizeMonthDay(dateEditState.value);
      if (!normalized) {
        setDateEditState({ sessionId: null, value: "" });
        return;
      }
      const fallbackDate = getDraftDate(sessionId);
      const targetDate = resolveFullDateFromMonthDay(normalized, fallbackDate);
      const draft = state.sessionTimeDrafts[sessionId];
      const startTime = formatTimeInput(draft?.startAt) || "08:00";
      const endTime = formatTimeInput(draft?.endAt) || "10:00";
      store.setSessionTimeDraft(sessionId, "startAt", `${targetDate}T${startTime}`);
      store.setSessionTimeDraft(sessionId, "endAt", `${targetDate}T${endTime}`);
      setDateEditState({ sessionId: null, value: "" });
    }

    window.addEventListener("pointerdown", onGlobalPointerDown, true);
    return () => {
      window.removeEventListener("pointerdown", onGlobalPointerDown, true);
    };
  }, [dateEditState, state.sessionTimeDrafts]);

  return (
    <section className="exam-dashboard-grid">
      <div className="exam-dashboard-left-col">
        <ConfigCard title="当前考试配置">
          {state.errorMessage ? (
            <p className="page-error-note" aria-live="polite">
              数据加载异常：{state.errorMessage}
            </p>
          ) : null}
          <div
            className="field-stack"
            style={{
              opacity: state.loading ? 0 : 1,
              pointerEvents: state.loading ? "none" : "auto",
              transition: "opacity 0.3s ease",
            }}
          >
            <label className="field-block">
              <span className="metric-label">考试标题</span>
              <input
                value={capacityForm.examTitle}
                className="glass-field filled-field"
                type="text"
                placeholder="2026 学年春季期末统一考试"
                onChange={(event) => setCapacityForm((current) => ({ ...current, examTitle: event.target.value.trimStart() }))}
              />
            </label>
            <label className="field-block">
              <span className="metric-label">考生须知</span>
              <textarea
                value={capacityForm.examNoticesText}
                className="glass-area filled-area"
                placeholder="请考生提前 30 分钟入场，核对准考证信息；开考 15 分钟后不得进入考场。严禁携带通讯设备与电子资料。"
                onChange={(event) => setCapacityForm((current) => ({ ...current, examNoticesText: event.target.value }))}
              />
            </label>
          </div>
          <p className={`autosave-note ${autoSaveError ? "error" : ""}`.trim()} aria-live="polite">
            {autoSaveError ? `自动保存失败：${autoSaveError}` : autoSaveText}
          </p>
        </ConfigCard>

        <TableCard
          title="考试时间"
          actions={(
            <>
              <FluentSelect
                modelValue={state.selectedSessionTimeGradeName}
                options={sessionTimeGradeOptions}
                className="exam-grade-select"
                onUpdateModelValue={(value) => {
                  if (typeof value === "string") {
                    void store.setSessionTimeGrade(value);
                  }
                }}
              />
              <button className="secondary-btn" type="button" disabled={state.loading} onClick={() => {
                const used = new Set<Subject>([
                  ...state.sessionTimes.map((item) => item.subject),
                  ...manualSubjectRows.map((item) => item.subject),
                ]);
                const nextSubject = DISPLAY_SUBJECT_OPTIONS.find((subject) => !used.has(subject)) ?? DISPLAY_SUBJECT_OPTIONS[0];
                setManualSubjectRows((current) => [
                  ...current,
                  {
                    id: manualSubjectRowIdRef.current++,
                    subject: nextSubject,
                    examMonthDay: formatMonthDay(new Date().toISOString().slice(0, 10)),
                    startTime: "",
                    endTime: "",
                  },
                ]);
              }}
              >
                新增科目
              </button>
            </>
          )}
        >
          <div className="exam-table-scroll" style={{ opacity: state.loading ? 0 : 1, transition: "opacity 0.3s ease" }}>
            <table className="table exam-table">
              <thead>
                <tr>
                  <th>科目</th>
                  <th>考试日期</th>
                  <th>开始时间</th>
                  <th>结束时间</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {state.sessionTimes.map((item) => (
                  <tr key={item.sessionId}>
                    <td>{examTimeSubjectLabel(item.subject)}</td>
                    <td className={`date-cell ${dateEditState.sessionId === item.sessionId ? "editing" : ""}`.trim()}>
                      {dateEditState.sessionId === item.sessionId ? (
                        <input
                          value={dateEditState.value}
                          className="month-day-input inline-edit"
                          type="text"
                          placeholder="03-24"
                          autoFocus
                          onChange={(event) => setDateEditState({ sessionId: item.sessionId, value: event.target.value.trim() })}
                          onBlur={() => {
                            const normalized = normalizeMonthDay(dateEditState.value);
                            if (!normalized) {
                              setDateEditState({ sessionId: null, value: "" });
                              return;
                            }
                            const fallbackDate = getDraftDate(item.sessionId);
                            const targetDate = resolveFullDateFromMonthDay(normalized, fallbackDate);
                            const draft = state.sessionTimeDrafts[item.sessionId];
                            const startTime = formatTimeInput(draft?.startAt) || "08:00";
                            const endTime = formatTimeInput(draft?.endAt) || "10:00";
                            store.setSessionTimeDraft(item.sessionId, "startAt", `${targetDate}T${startTime}`);
                            store.setSessionTimeDraft(item.sessionId, "endAt", `${targetDate}T${endTime}`);
                            setDateEditState({ sessionId: null, value: "" });
                          }}
                          onKeyDown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              const normalized = normalizeMonthDay(dateEditState.value);
                              if (!normalized) {
                                setDateEditState({ sessionId: null, value: "" });
                                return;
                              }
                              const fallbackDate = getDraftDate(item.sessionId);
                              const targetDate = resolveFullDateFromMonthDay(normalized, fallbackDate);
                              const draft = state.sessionTimeDrafts[item.sessionId];
                              const startTime = formatTimeInput(draft?.startAt) || "08:00";
                              const endTime = formatTimeInput(draft?.endAt) || "10:00";
                              store.setSessionTimeDraft(item.sessionId, "startAt", `${targetDate}T${startTime}`);
                              store.setSessionTimeDraft(item.sessionId, "endAt", `${targetDate}T${endTime}`);
                              setDateEditState({ sessionId: null, value: "" });
                            }
                            if (event.key === "Escape") {
                              event.preventDefault();
                              setDateEditState({ sessionId: null, value: "" });
                            }
                          }}
                        />
                      ) : (
                        <button
                          className="date-display-btn"
                          type="button"
                          onDoubleClick={() => {
                            const current = state.sessionTimeDrafts[item.sessionId];
                            const fromStart = formatMonthDay(current?.startAt);
                            const fromEnd = formatMonthDay(current?.endAt);
                            const monthDay = fromStart !== "--" ? fromStart : fromEnd;
                            setDateEditState({
                              sessionId: item.sessionId,
                              value: monthDay === "--" ? "" : monthDay,
                            });
                          }}
                        >
                          {formatMonthDay(state.sessionTimeDrafts[item.sessionId]?.startAt || item.startAt)}
                        </button>
                      )}
                    </td>
                    <td className="time-cell">
                      <input
                        className="time-input"
                        type="time"
                        value={formatTimeInput(state.sessionTimeDrafts[item.sessionId]?.startAt)}
                        onChange={(event) => {
                          const current = state.sessionTimeDrafts[item.sessionId];
                          const datePart = formatDate(current?.startAt || current?.endAt) || new Date().toISOString().slice(0, 10);
                          store.setSessionTimeDraft(item.sessionId, "startAt", `${datePart}T${event.target.value}`);
                        }}
                      />
                    </td>
                    <td className="time-cell">
                      <input
                        className="time-input"
                        type="time"
                        value={formatTimeInput(state.sessionTimeDrafts[item.sessionId]?.endAt)}
                        onChange={(event) => {
                          const current = state.sessionTimeDrafts[item.sessionId];
                          const datePart = formatDate(current?.startAt || current?.endAt) || new Date().toISOString().slice(0, 10);
                          store.setSessionTimeDraft(item.sessionId, "endAt", `${datePart}T${event.target.value}`);
                        }}
                      />
                    </td>
                    <td>
                      <button
                        className="icon-btn"
                        type="button"
                        disabled={state.savingTimes}
                        title={`删除${examTimeSubjectLabel(item.subject)}考试时间配置`}
                        onClick={() => void store.deleteSessionTime(item.subject)}
                      >
                        <span className="material-symbols-rounded" aria-hidden="true">delete</span>
                      </button>
                    </td>
                  </tr>
                ))}
                {manualSubjectRows.map((item) => (
                  <tr key={item.id}>
                    <td>
                      <div className="manual-subject-row">
                        <FluentSelect
                          modelValue={item.subject}
                          options={DISPLAY_SUBJECT_OPTIONS.map((subject) => ({ label: examTimeSubjectLabel(subject), value: subject }))}
                          className="exam-subject-select"
                          onUpdateModelValue={(value) => setManualSubjectRows((current) => current.map((row) => row.id === item.id ? { ...row, subject: value as Subject } : row))}
                        />
                      </div>
                    </td>
                    <td>
                      <input
                        value={item.examMonthDay}
                        className="month-day-input"
                        type="text"
                        placeholder="03-24"
                        onChange={(event) => setManualSubjectRows((current) => current.map((row) => row.id === item.id ? { ...row, examMonthDay: event.target.value.trim() } : row))}
                      />
                    </td>
                    <td>
                      <input
                        value={item.startTime}
                        className="time-input"
                        type="time"
                        onChange={(event) => setManualSubjectRows((current) => current.map((row) => row.id === item.id ? { ...row, startTime: event.target.value } : row))}
                      />
                    </td>
                    <td>
                      <input
                        value={item.endTime}
                        className="time-input"
                        type="time"
                        onChange={(event) => setManualSubjectRows((current) => current.map((row) => row.id === item.id ? { ...row, endTime: event.target.value } : row))}
                      />
                    </td>
                    <td>
                      <button className="icon-btn" type="button" title="删除该科目时间配置" onClick={() => setManualSubjectRows((current) => current.filter((row) => row.id !== item.id))}>
                        <span className="material-symbols-rounded" aria-hidden="true">delete</span>
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </TableCard>
      </div>

      <div className="exam-dashboard-right-col">
        <section className="progress-card card-shell">
          <div className="progress-head">
            <h3>开始分配考场</h3>
            <span className="progress-badge">{progressBadgeText}</span>
          </div>
          <p className="progress-desc">{progressDescription}</p>
          <div className="hero-metrics">
            <div className="hero-metric">
              <span>考场数量</span>
              <strong>{state.overview.examRoomCount || "--"}</strong>
            </div>
            <div className="hero-metric">
              <span>考生数量</span>
              <strong>{state.overview.studentAllocationCount || "--"}</strong>
            </div>
          </div>
          <div className="cta-row">
            <button className="primary-btn" disabled={state.generating || isPreparingGenerate} onClick={() => void generateExamPlan()}>
              {generateActionText}
            </button>
            <strong className="percent">{progressPercent}%</strong>
          </div>
          <div className="progress-track">
            <div className="progress-fill" style={{ width: `${progressPercent}%` }} />
          </div>
          <div className="step-card">
            <span className="metric-label">当前步骤</span>
            <strong className="step-text">{progressStepText}</strong>
          </div>
        </section>

        <section className="overview-card card-shell">
          <h2 className="ov-main-title">配置与结果</h2>

          <h3 className="ov-section-title">考场容量参数</h3>
          <div className="ov-capacity-grid">
            <div className="ov-capacity-box">
              <div className="ov-box-label">考场默认容量</div>
              <div className="ov-box-value">
                <input
                  value={capacityForm.defaultCapacity}
                  className="ov-capacity-input"
                  type="number"
                  min="1"
                  onChange={(event) => setCapacityForm((current) => ({ ...current, defaultCapacity: Number(event.target.value) || 1 }))}
                />
                <span className="ov-box-unit">人</span>
              </div>
            </div>
            <div className="ov-capacity-box">
              <div className="ov-box-label">考场最大容量</div>
              <div className="ov-box-value">
                <input
                  value={capacityForm.maxCapacity}
                  className="ov-capacity-input"
                  type="number"
                  min="1"
                  onChange={(event) => setCapacityForm((current) => ({ ...current, maxCapacity: Number(event.target.value) || 1 }))}
                />
                <span className="ov-box-unit">人</span>
              </div>
            </div>
          </div>

          <hr className="ov-divider" />

          <h3 className="ov-section-title">结果中心</h3>
          <div className="ov-status-list">
            <div className="ov-status-item">
              <span className="ov-status-label">任务状态</span>
              <span className={`ov-badge ${state.generationProgress.status === "error" ? "status-error" : state.generating || state.exporting || !state.overview.generatedAt ? "status-pending" : "status-success"}`}>{completeBadgeText}</span>
            </div>
            <div className="ov-status-item">
              <span className="ov-status-label">已生成结果</span>
              <span className="ov-status-value">{state.overview.generatedAt ? "可导出" : "未生成"}</span>
            </div>
            <div className="ov-status-item">
              <span className="ov-status-label">结果摘要</span>
              {state.lastExportFolderPath ? (
                <button className="ov-result-link" type="button" onClick={() => void openExportFolder()}>{exportFileName}</button>
              ) : (
                <span className="ov-status-value ov-status-muted">尚未导出分配文件</span>
              )}
            </div>
          </div>

          <button className="ov-btn-block" disabled={state.exporting || !state.overview.generatedAt} onClick={() => void exportBundle()}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            {state.exporting ? "导出中..." : "导出分配结果"}
          </button>
        </section>
      </div>
    </section>
  );
}
