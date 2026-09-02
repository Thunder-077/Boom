import { useEffect, useState } from "react";
import { Search, X } from "lucide-react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import type { ScoreCellState, ScoreDetail, ScoreRow, ScoreUpdatePayload } from "../../../entities/score/model";
import { FilterToolbar, FluentSelect, InfoHint, Pagination, TableCard } from "../../../widgets/common/index.react";
import { useReactScoreStore } from "../store";

const GRADE_OPTIONS = [
  { label: "全部年级", value: "" },
  { label: "高一", value: "高一" },
  { label: "高二", value: "高二" },
  { label: "高三", value: "高三" },
] as const;

const SCORE_STATE_OPTIONS = [
  { label: "有成绩", value: "scored" },
  { label: "缺考", value: "absent" },
  { label: "未选考", value: "not_selected" },
] as const;

const LANGUAGE_SHORT: Record<string, string> = { 英语: "英", 俄语: "俄", 日语: "日" };

function formatSubjectSelection(row: ScoreRow) {
  if (row.subjectCombination === "全科") return "全科";
  const langShort = LANGUAGE_SHORT[row.language] ?? row.language;
  return `语数${langShort}${row.subjectCombination}`;
}

function normalizeDroppedPath(rawPath: string) {
  const trimmed = rawPath.trim();
  if (!trimmed.startsWith("file://")) {
    return trimmed;
  }
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

export default function ScoreManagementPanel() {
  const store = useReactScoreStore();
  const { state } = store;
  const [isDragging, setIsDragging] = useState(false);
  const [detailState, setDetailState] = useState<{
    visible: boolean;
    mode: "view" | "edit";
    loading: boolean;
    saving: boolean;
    error: string;
    form: ScoreDetail | null;
  }>({
    visible: false,
    mode: "view",
    loading: false,
    saving: false,
    error: "",
    form: null,
  });

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
          const excelFilePath = pickExcelPath(event.payload.paths);
          if (excelFilePath) {
            void store.importExcel(excelFilePath);
            return;
          }
          store.setImportFeedback("error", "已收到拖拽，但未识别到可导入的 Excel 文件路径");
        }
      });
      await store.load();
    }

    void bindWindowEvents();
    return () => {
      unlistenDragDrop?.();
    };
  }, []);

  const importStatusLabel =
    state.importStatus === "idle"
      ? "待导入"
      : state.importStatus === "importing"
        ? "导入中"
        : state.importStatus === "success"
          ? "导入成功"
          : "导入失败";
  const importStatusMessage =
    state.importStatus === "idle"
      ? "拖拽成绩 Excel 文件到页面任意位置即可开始导入"
      : state.importMessage;
  const importHintType =
    state.importStatus === "success"
      ? "success"
      : state.importStatus === "error"
        ? "error"
        : state.importStatus === "importing"
          ? "warning"
          : "info";

  async function openDetail(admissionNo: string, mode: "view" | "edit") {
    setDetailState({
      visible: true,
      mode,
      loading: true,
      saving: false,
      error: "",
      form: null,
    });
    try {
      const form = await store.getDetail(admissionNo);
      setDetailState({
        visible: true,
        mode,
        loading: false,
        saving: false,
        error: "",
        form,
      });
    } catch (error) {
      setDetailState({
        visible: true,
        mode,
        loading: false,
        saving: false,
        error: error instanceof Error ? error.message : String(error),
        form: null,
      });
    }
  }

  function closeDetail() {
    setDetailState({
      visible: false,
      mode: "view",
      loading: false,
      saving: false,
      error: "",
      form: null,
    });
  }

  async function saveDetail() {
    if (!detailState.form) {
      return;
    }
    setDetailState((current) => ({ ...current, saving: true, error: "" }));
    try {
      const subjects = detailState.form.subjects.map((item) => ({
        subject: item.subject,
        state: item.state as ScoreCellState,
        score:
          item.state === "scored" && item.score !== null && Number.isFinite(Number(item.score))
            ? Number(item.score)
            : null,
      }));
      const payload: ScoreUpdatePayload = {
        admissionNo: detailState.form.admissionNo,
        className: detailState.form.className,
        studentName: detailState.form.studentName,
        subjects,
      };
      await store.updateScore(payload);
      await openDetail(detailState.form.admissionNo, "view");
    } catch (error) {
      setDetailState((current) => ({
        ...current,
        saving: false,
        error: error instanceof Error ? error.message : String(error),
      }));
    }
  }

  return (
    <section className={`panel ${isDragging ? "dragging" : ""}`}>
      {isDragging ? (
        <div className="drag-overlay">
          <div className="drag-card">
            <strong>松开鼠标开始导入成绩表</strong>
            <span>支持 `.xlsx` / `.xls`</span>
          </div>
        </div>
      ) : null}

      <FilterToolbar items={[]}>
        <div className="toolbar-fields">
          <FluentSelect
            modelValue={state.filters.gradeName ?? ""}
            options={[
              { label: "全部年级", value: "" },
              ...state.gradeOptions.map((grade) => ({ label: grade, value: grade })),
            ]}
            onUpdateModelValue={(value) => void store.setFilters({ gradeName: value as string })}
            className="grade-select"
          />
          <label className="filter-search">
            <Search size={18} className="filter-search-icon" />
            <input
              value={state.filters.nameKeyword ?? ""}
              placeholder="按姓名筛选"
              onChange={(event) => void store.setFilters({ nameKeyword: event.target.value })}
            />
          </label>
        </div>
      </FilterToolbar>

      <InfoHint className="import-status" type={importHintType} text={`${importStatusLabel}：${importStatusMessage}`} />

      <TableCard title="考试成绩列表" meta={`已同步 ${state.total} 条`}>
        <div className="table-scroll">
          <table className="table score-table">
            <thead>
              <tr>
                <th>姓名</th>
                <th>准考证号</th>
                <th>班级</th>
                <th>选科</th>
                <th>分数</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {state.rows.map((row, index) => (
                <tr key={row.admissionNo} className={index % 2 === 1 ? "row-alt" : ""}>
                  <td className="emphasis">{row.studentName}</td>
                  <td>{row.admissionNo}</td>
                  <td>{row.className}</td>
                  <td>{formatSubjectSelection(row)}</td>
                  <td className="score-cell">{row.totalScore.toFixed(0)}</td>
                  <td className="link-cell">
                    <button className="link-btn" type="button" onClick={() => void openDetail(row.admissionNo, "view")}>查看</button>
                    <span className="sep">/</span>
                    <button className="link-btn" type="button" onClick={() => void openDetail(row.admissionNo, "edit")}>编辑</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <Pagination
          currentPage={state.page}
          pageSize={state.pageSize ?? 20}
          total={state.total}
          onChange={(page) => void store.setPage(page)}
        />
      </TableCard>

      {detailState.visible ? (
        <div className="detail-mask" onClick={(event) => {
          if (event.target === event.currentTarget) {
            closeDetail();
          }
        }}>
          <section className="detail-card card-shell">
            <div className="detail-head">
              <h3>{detailState.mode === "view" ? "查看成绩" : "编辑成绩"}</h3>
              <button className="close-btn" type="button" onClick={closeDetail}>
                <X size={18} />
              </button>
            </div>

            {detailState.loading ? (
              <div className="detail-loading">加载中...</div>
            ) : detailState.error ? (
              <div className="detail-error">{detailState.error}</div>
            ) : detailState.form ? (
              <>
                <div className="detail-meta">
                  <label className="meta-field">
                    <span>姓名</span>
                    <input
                      value={detailState.form.studentName}
                      className="glass-field"
                      disabled={detailState.mode === "view"}
                      onChange={(event) => setDetailState((current) => current.form ? ({
                        ...current,
                        form: { ...current.form, studentName: event.target.value },
                      }) : current)}
                    />
                  </label>
                  <label className="meta-field">
                    <span>班级</span>
                    <input
                      value={detailState.form.className}
                      className="glass-field"
                      disabled={detailState.mode === "view"}
                      onChange={(event) => setDetailState((current) => current.form ? ({
                        ...current,
                        form: { ...current.form, className: event.target.value },
                      }) : current)}
                    />
                  </label>
                  <label className="meta-field readonly">
                    <span>准考证号</span>
                    <input value={detailState.form.admissionNo} className="glass-field" disabled />
                  </label>
                </div>

                <div className="subject-list">
                  {detailState.form.subjects.map((item, index) => (
                    <div key={item.subject} className="subject-row">
                      <strong>{SUBJECT_LABELS[item.subject]}</strong>
                      <FluentSelect
                        modelValue={item.state}
                        options={[...SCORE_STATE_OPTIONS]}
                        disabled={detailState.mode === "view"}
                        className="state-select"
                        onUpdateModelValue={(value) => setDetailState((current) => {
                          if (!current.form) return current;
                          const subjects = [...current.form.subjects];
                          subjects[index] = {
                            ...subjects[index],
                            state: value as ScoreCellState,
                          };
                          return { ...current, form: { ...current.form, subjects } };
                        })}
                      />
                      <input
                        value={item.score ?? ""}
                        className="glass-field score-input"
                        type="number"
                        min="0"
                        step="0.5"
                        disabled={detailState.mode === "view" || item.state !== "scored"}
                        placeholder={item.state === "scored" ? "输入分数" : "--"}
                        onChange={(event) => setDetailState((current) => {
                          if (!current.form) return current;
                          const subjects = [...current.form.subjects];
                          subjects[index] = {
                            ...subjects[index],
                            score: event.target.value === "" ? null : Number(event.target.value),
                          };
                          return { ...current, form: { ...current.form, subjects } };
                        })}
                      />
                    </div>
                  ))}
                </div>

                <div className="detail-actions">
                  <button className="secondary-btn" type="button" onClick={closeDetail}>关闭</button>
                  {detailState.mode === "edit" ? (
                    <button className="primary-btn" type="button" disabled={detailState.saving} onClick={() => void saveDetail()}>
                      {detailState.saving ? "保存中..." : "保存"}
                    </button>
                  ) : null}
                </div>
              </>
            ) : null}
          </section>
        </div>
      ) : null}
    </section>
  );
}
