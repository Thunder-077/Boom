import { useEffect, useMemo, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TEACHER_SUBJECT_LABELS } from "../../../entities/teacher/model";
import {
  FilterToolbar,
  FluentSelect,
  InfoHint,
  Pagination,
  TableCard,
  Tag,
} from "../../../widgets/common/index.react";
import { TEACHER_SUBJECT_OPTIONS, useReactTeacherStore } from "../store";

const gradeRankMap: Record<string, number> = { 高一: 1, 高二: 2, 高三: 3 };

function extractClassSortNumber(className: string) {
  const match = className.match(/(\d+)/g);
  return match && match.length > 0 ? Number(match[match.length - 1]) : Number.POSITIVE_INFINITY;
}

function extractGradeName(className: string) {
  const match = className.match(/^(高[一二三]|初[一二三]|初中[一二三]|高中[一二三])/);
  return match?.[0] ?? "";
}

function compareClasses(a: string, b: string) {
  const gradeA = extractGradeName(a);
  const gradeB = extractGradeName(b);
  const gradeDiff = (gradeRankMap[gradeA] ?? 99) - (gradeRankMap[gradeB] ?? 99);
  if (gradeDiff !== 0) return gradeDiff;
  const classDiff = extractClassSortNumber(a) - extractClassSortNumber(b);
  if (classDiff !== 0) return classDiff;
  return a.localeCompare(b, "zh-CN", { numeric: true });
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

export default function TeacherListPanel() {
  const store = useReactTeacherStore();
  const { state } = store;
  const [isDragging, setIsDragging] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 15;

  const classOptions = useMemo(
    () => Array.from(new Set(state.rows.flatMap((row) => row.classNames))).sort(compareClasses),
    [state.rows],
  );
  const totalRows = state.rows.length;
  const pagedRows = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    return state.rows.slice(start, start + pageSize);
  }, [currentPage, state.rows]);

  useEffect(() => {
    setCurrentPage(1);
  }, [state.filters]);

  useEffect(() => {
    let unlistenDragDrop: (() => void) | null = null;

    async function bindWindowEvents() {
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
      ? "拖拽教师 Excel 文件到页面任意位置即可开始导入"
      : state.importMessage;
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
            <strong>松开鼠标开始导入教师名单</strong>
            <span>支持 `.xlsx` / `.xls`</span>
          </div>
        </div>
      ) : null}

      <FilterToolbar items={[]}>
        <div className="toolbar-fields">
          <label className="search-field">
            <span className="material-symbols-rounded search-icon" aria-hidden="true">search</span>
            <input
              className="glass-field"
              value={state.filters.nameKeyword ?? ""}
              placeholder="按教师姓名查询"
              onChange={(event) => void store.setFilters({ nameKeyword: event.target.value })}
            />
          </label>
          <FluentSelect
            modelValue={state.filters.className ?? ""}
            options={[{ label: "班级", value: "" }, ...classOptions.map((className) => ({ label: className, value: className }))]}
            onUpdateModelValue={(value) => void store.setFilters({ className: value as string })}
            className="filter-select"
          />
          <FluentSelect
            modelValue={state.filters.subject ?? ""}
            options={TEACHER_SUBJECT_OPTIONS}
            onUpdateModelValue={(value) => void store.setFilters({ subject: value as never })}
            className="filter-select"
          />
        </div>
      </FilterToolbar>

      <InfoHint
        className="import-status"
        type={importHintType}
        text={`${importStatusLabel}：${importStatusMessage}`}
      />

      <TableCard title="教师列表" meta={`共 ${state.total} 位`}>
        <div className="table-scroll">
          <div className="teacher-grid teacher-grid-head">
            <div className="cell head name-col">姓名</div>
            <div className="cell head class-col">班级</div>
            <div className="cell head subject-col">教学科目</div>
            <div className="cell head remark-col">备注</div>
          </div>
          {pagedRows.map((row, index) => (
            <div
              key={row.id}
              className={`teacher-grid teacher-grid-row ${index % 2 === 1 ? "row-alt" : ""}`.trim()}
            >
              <div className="cell name-col teacher-name">{row.teacherName}</div>
              <div className="cell class-col">
                <div className="tag-row">
                  {row.classNames.map((className) => (
                    <Tag key={className} size="sm">{className}</Tag>
                  ))}
                </div>
              </div>
              <div className="cell subject-col">
                {row.subjects.map((subject) => TEACHER_SUBJECT_LABELS[subject]).join(" / ")}
              </div>
              <div className="cell remark-col">{row.remark || "--"}</div>
            </div>
          ))}
        </div>
        <Pagination
          currentPage={currentPage}
          pageSize={pageSize}
          total={totalRows}
          onChange={setCurrentPage}
        />
      </TableCard>
    </section>
  );
}
