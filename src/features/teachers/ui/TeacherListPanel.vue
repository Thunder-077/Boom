<template>
  <section class="panel" :class="{ dragging: isDragging }">
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-card">
        <strong>松开鼠标开始导入教师名单</strong>
        <span>支持 `.xlsx` / `.xls`</span>
      </div>
    </div>
    <FilterToolbar :items="[]">
      <div class="toolbar-fields">
        <label class="search-field">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79L19 20.5 20.5 19 15.5 14ZM9.5 14A4.5 4.5 0 1 1 14 9.5 4.5 4.5 0 0 1 9.5 14Z" />
          </svg>
          <input class="glass-field" :value="store.viewState.filters.nameKeyword" placeholder="按教师姓名查询" @input="onNameInput" />
        </label>
        <label class="select-field">
          <select class="glass-field" :value="store.viewState.filters.className" @change="onClassChange">
            <option value="">班级</option>
            <option v-for="className in classOptions" :key="className" :value="className">{{ className }}</option>
          </select>
          <span class="material-symbols-rounded select-icon" aria-hidden="true">keyboard_arrow_down</span>
        </label>
        <label class="select-field">
          <select class="glass-field" :value="store.viewState.filters.subject" @change="onSubjectChange">
            <option v-for="option in TEACHER_SUBJECT_OPTIONS" :key="option.value || 'all'" :value="option.value">{{ option.label }}</option>
          </select>
          <span class="material-symbols-rounded select-icon" aria-hidden="true">keyboard_arrow_down</span>
        </label>
      </div>
    </FilterToolbar>

    <p class="import-status" :class="store.viewState.importStatus">
      {{ importStatusLabel }}：{{ importStatusMessage }}
    </p>

    <TableCard title="教师列表" :meta="`共 ${store.viewState.total} 位`">
      <div class="table-scroll">
        <div class="teacher-grid teacher-grid-head">
          <div class="cell head name-col">姓名</div>
          <div class="cell head class-col">班级</div>
          <div class="cell head subject-col">教学科目</div>
          <div class="cell head remark-col">备注</div>
        </div>
        <div
          v-for="(row, index) in store.viewState.rows"
          :key="row.id"
          class="teacher-grid teacher-grid-row"
          :class="{ 'row-alt': index % 2 === 1 }"
        >
          <div class="cell name-col teacher-name">{{ row.teacherName }}</div>
          <div class="cell class-col">
            <div class="tag-row">
              <span v-for="className in row.classNames" :key="className" class="tag-pill">{{ className }}</span>
            </div>
          </div>
          <div class="cell subject-col">{{ row.subjects.map((subject) => TEACHER_SUBJECT_LABELS[subject]).join(" / ") }}</div>
          <div class="cell remark-col">{{ row.remark || "--" }}</div>
        </div>
      </div>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TEACHER_SUBJECT_LABELS, type TeacherSubject } from "../../../entities/teacher/model";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { TEACHER_SUBJECT_OPTIONS, useTeacherStore } from "../store";

const store = useTeacherStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;

const classOptions = computed(() =>
  Array.from(new Set(store.viewState.rows.flatMap((row) => row.classNames))).sort((a, b) => a.localeCompare(b, "zh-CN")),
);

const importStatusLabel = computed(() => {
  if (store.viewState.importStatus === "idle") {
    return "待导入";
  }
  if (store.viewState.importStatus === "importing") {
    return "导入中";
  }
  if (store.viewState.importStatus === "success") {
    return "导入成功";
  }
  return "导入失败";
});

const importStatusMessage = computed(() => {
  if (store.viewState.importStatus === "idle") {
    return "拖拽教师 Excel 文件到页面任意位置即可开始导入";
  }
  return store.viewState.importMessage;
});

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
}

function onClassChange(event: Event) {
  void store.setFilters({ className: (event.target as HTMLSelectElement).value });
}

function onSubjectChange(event: Event) {
  const subject = (event.target as HTMLSelectElement).value as TeacherSubject | "";
  void store.setFilters({ subject });
}

async function handleImport(filePath: string) {
  if (!filePath) {
    return;
  }
  try {
    await store.importExcel(filePath);
  } catch {
    // Import status is already persisted in store.
  }
}

function normalizeDroppedPath(rawPath: string): string {
  const trimmed = rawPath.trim();
  if (!trimmed.startsWith("file://")) {
    return trimmed;
  }
  try {
    const url = new URL(trimmed);
    const decoded = decodeURIComponent(url.pathname);
    const normalized = decoded
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
    return normalized;
  } catch {
    const withoutScheme = trimmed.replace(/^file:\/\//i, "");
    const decoded = decodeURIComponent(withoutScheme);
    return decoded
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  }
}

function pickExcelPath(paths: string[]): string | undefined {
  for (const rawPath of paths) {
    const normalized = normalizeDroppedPath(rawPath);
    const lowerPath = normalized.toLowerCase();
    if (lowerPath.endsWith(".xlsx") || lowerPath.endsWith(".xls")) {
      return normalized;
    }
  }
  return undefined;
}

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow();
  unlistenDragDrop = await appWindow.onDragDropEvent((event) => {
    if (event.payload.type === "enter" || event.payload.type === "over") {
      isDragging.value = true;
      return;
    }
    if (event.payload.type === "leave") {
      isDragging.value = false;
      return;
    }
    if (event.payload.type === "drop") {
      isDragging.value = false;
      const excelPath = pickExcelPath(event.payload.paths);
      if (excelPath) {
        void handleImport(excelPath);
        return;
      }
      store.setImportFeedback("error", "已收到拖拽，但未识别到可导入的 Excel 文件路径");
    }
  });
  await store.load();
});

onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
    unlistenDragDrop = null;
  }
});
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  gap: 22px;
  position: relative;
}

.panel :deep(.table-card) {
  flex: 1;
  min-height: 0;
}

.panel :deep(.table-card .content) {
  display: flex;
  min-height: 0;
}

.panel.dragging :deep(.toolbar) {
  border-color: #b9d6ff;
  background: rgba(232, 242, 255, 0.92);
}

.drag-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  border-radius: 20px;
  background: rgba(15, 108, 189, 0.08);
  border: 2px dashed #7fb1ea;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drag-card {
  min-width: 280px;
  padding: 18px 22px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 18px 36px rgba(15, 108, 189, 0.14);
  display: flex;
  flex-direction: column;
  gap: 6px;
  text-align: center;
}

.drag-card strong {
  font-size: 15px;
  color: var(--color-brand);
}

.drag-card span {
  font-size: 13px;
  color: var(--color-text-muted);
}

.toolbar-fields {
  display: flex;
  gap: 12px;
}

.toolbar-fields label {
  display: block;
}

.search-field {
  position: relative;
}

.search-field svg {
  position: absolute;
  left: 14px;
  top: 12px;
  width: 18px;
  height: 18px;
  fill: var(--color-text-muted);
}

.search-field input {
  width: 260px;
  padding-left: 42px;
}

.select-field {
  position: relative;
}

.select-field .glass-field {
  width: 150px;
  appearance: none;
  padding-right: 36px;
}

.select-icon {
  position: absolute;
  right: 12px;
  top: 12px;
  color: var(--color-text-muted);
  font-size: 18px;
  pointer-events: none;
  font-family: "Material Symbols Rounded";
}

.table-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.teacher-grid {
  display: grid;
  grid-template-columns: 180px minmax(0, 1.2fr) 160px minmax(0, 0.9fr);
  align-items: center;
}

.teacher-grid-head {
  min-height: 48px;
  background: #fafcff;
  border-bottom: 1px solid var(--color-border-soft);
}

.teacher-grid-row {
  min-height: 72px;
  background: #fff;
  border-bottom: 1px solid var(--color-border-soft);
}

.cell {
  min-height: inherit;
  display: flex;
  align-items: center;
  padding: 0 18px;
  font-size: 14px;
  color: var(--color-text);
  min-width: 0;
}

.head {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.teacher-name {
  font-weight: 600;
}

.subject-col {
  word-break: break-all;
}

.remark-col {
  word-break: break-word;
  color: var(--color-text-muted);
}

.tag-row {
  display: flex;
  align-items: center;
  align-content: center;
  flex-wrap: wrap;
  gap: 10px;
  width: 100%;
  padding: 10px 0;
}

.row-alt {
  background: #f8fbff;
}

.import-status {
  margin: -10px 4px 0;
  font-size: 13px;
}

.import-status.idle {
  color: var(--color-text-muted);
}

.import-status.importing {
  color: var(--color-brand);
}

.import-status.success {
  color: var(--color-success);
}

.import-status.error {
  color: var(--color-danger);
}
</style>
