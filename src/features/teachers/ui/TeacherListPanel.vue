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
        <FluentSelect
          :model-value="store.viewState.filters.className ?? ''"
          :options="[{ label: '班级', value: '' }, ...classOptions.map(c => ({ label: c, value: c }))]"
          @update:model-value="store.setFilters({ className: $event as string })"
          style="width: 150px;"
        />
        <FluentSelect
          :model-value="store.viewState.filters.subject ?? ''"
          :options="TEACHER_SUBJECT_OPTIONS"
          @update:model-value="store.setFilters({ subject: $event as any })"
          style="width: 150px;"
        />
      </div>
    </FilterToolbar>

    <InfoHint
      class="import-status"
      :type="store.viewState.importStatus === 'success' ? 'success' : store.viewState.importStatus === 'error' ? 'error' : store.viewState.importStatus === 'importing' ? 'warning' : 'info'"
      :text="importStatusLabel + '：' + importStatusMessage"
    />

    <TableCard title="教师列表" :meta="`共 ${store.viewState.total} 位`">
      <div class="table-scroll">
        <div class="teacher-grid teacher-grid-head">
          <div class="cell head name-col">姓名</div>
          <div class="cell head class-col">班级</div>
          <div class="cell head subject-col">教学科目</div>
          <div class="cell head remark-col">备注</div>
        </div>
        <div
          v-for="(row, index) in pagedRows"
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
      <div v-if="totalPages > 1" class="pagination-row">
        <span class="page-meta">共 {{ totalRows }} 位教师，本页 {{ pageStart }} - {{ pageEnd }}</span>
        <div class="pagination-actions">
          <button class="page-btn" type="button" :disabled="currentPage === 1" @click="goToPrevPage">上一页</button>
          <button v-for="page in visiblePages" :key="page" class="page-btn" :class="{ active: page === currentPage }" type="button" @click="goToPage(page)">{{ page }}</button>
          <button class="page-btn" type="button" :disabled="currentPage === totalPages" @click="goToNextPage">下一页</button>
        </div>
      </div>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TEACHER_SUBJECT_LABELS } from "../../../entities/teacher/model";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { TEACHER_SUBJECT_OPTIONS, useTeacherStore } from "../store";

const store = useTeacherStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;

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

const classOptions = computed(() =>
  Array.from(new Set(store.viewState.rows.flatMap((row) => row.classNames))).sort(compareClasses),
);

const currentPage = ref(1);
const pageSize = ref(5);

const totalRows = computed(() => store.viewState.rows.length);
const totalPages = computed(() => Math.max(1, Math.ceil(totalRows.value / pageSize.value)));

const pagedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return store.viewState.rows.slice(start, start + pageSize.value);
});

const pageStart = computed(() => totalRows.value === 0 ? 0 : (currentPage.value - 1) * pageSize.value + 1);
const pageEnd = computed(() => Math.min(currentPage.value * pageSize.value, totalRows.value));

const visiblePages = computed(() => {
  const current = currentPage.value;
  const total = totalPages.value;
  const pages = new Set([1, Math.max(1, current - 1), current, Math.min(total, current + 1), total]);
  return Array.from(pages).filter((p) => p >= 1 && p <= total).sort((a, b) => a - b);
});

function goToPage(page: number) {
  currentPage.value = page;
}

function goToPrevPage() {
  if (currentPage.value > 1) currentPage.value--;
}

function goToNextPage() {
  if (currentPage.value < totalPages.value) currentPage.value++;
}

watch(
  () => store.viewState.filters,
  () => {
    currentPage.value = 1;
  },
  { deep: true }
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
  flex-direction: column;
  min-height: 0;
}

.panel.dragging :deep(.toolbar) {
  border-color: var(--accent-border-strong);
  background: rgba(var(--accent-rgb), 0.08);
}

.drag-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  border-radius: 24px;
  background: rgba(var(--accent-rgb), 0.08);
  border: 2px dashed rgba(var(--accent-rgb), 0.34);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drag-card {
  min-width: 280px;
  padding: 20px 24px;
  border-radius: 22px;
  background: var(--surface-panel-strong);
  box-shadow: var(--shadow-medium);
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
  background: var(--surface-elevated);
  border-bottom: 1px solid var(--color-border-soft);
}

.teacher-grid-row {
  min-height: 72px;
  background: var(--surface-panel-strong);
  border-bottom: 1px solid var(--border-default);
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
  font-size: 12px;
  font-weight: 700;
  color: var(--text-secondary);
  letter-spacing: 0.04em;
  text-transform: uppercase;
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
  background: var(--surface-elevated);
}

.import-status {
  margin: -10px 4px 0;
}

.pagination-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-top: 1px solid var(--border-default);
  background: var(--surface-panel);
}

.page-meta {
  color: var(--color-text-muted);
  font-size: 13px;
}

.pagination-actions {
  display: flex;
  gap: 8px;
}

.page-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 8px;
  border-radius: 10px;
  border: 1px solid var(--color-border-soft);
  background: var(--surface-panel-strong);
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  transition: all 0.2s;
}

.page-btn:hover:not(:disabled) {
  background: rgba(var(--accent-rgb), 0.1);
  border-color: var(--accent-border-strong);
  color: var(--accent-primary);
}

.page-btn.active {
  background: var(--accent-primary);
  color: #fff;
  border-color: var(--accent-primary);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
