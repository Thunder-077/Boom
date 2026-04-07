<template>
  <section class="panel" :class="{ dragging: isDragging }">
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-card">
        <strong>松开鼠标开始导入成绩表</strong>
        <span>支持 `.xlsx` / `.xls`</span>
      </div>
    </div>
    <FilterToolbar :items="[]">
      <div class="toolbar-fields">
        <FluentSelect
          :model-value="store.viewState.filters.gradeName ?? ''"
          :options="[{ label: '全部年级', value: '' }, { label: '高一', value: '高一' }, { label: '高二', value: '高二' }, { label: '高三', value: '高三' }]"
          @update:model-value="store.setFilters({ gradeName: $event as string })"
          style="width: 220px;"
        />
        <label class="filter-search">
          <span class="material-symbols-rounded filter-search-icon" aria-hidden="true">search</span>
          <input :value="store.viewState.filters.nameKeyword" placeholder="按姓名筛选" @input="onNameInput" />
        </label>
      </div>
    </FilterToolbar>

    <InfoHint
      class="import-status"
      :type="store.viewState.importStatus === 'success' ? 'success' : store.viewState.importStatus === 'error' ? 'error' : store.viewState.importStatus === 'importing' ? 'warning' : 'info'"
      :text="importStatusLabel + '：' + importStatusMessage"
    />

    <TableCard title="考试成绩列表" :meta="`已同步 ${store.viewState.total} 条`">
      <div class="table-scroll">
      <table class="table score-table">
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
          <tr v-for="(row, index) in store.viewState.rows" :key="row.admissionNo" :class="rowClass(index)">
            <td class="emphasis">{{ row.studentName }}</td>
            <td>{{ row.admissionNo }}</td>
            <td>{{ row.className }}</td>
            <td>{{ formatSubjectSelection(row) }}</td>
            <td class="score-cell">{{ row.totalScore.toFixed(0) }}</td>
            <td class="link-cell">
              <button class="link-btn" type="button" @click="openDetail(row.admissionNo, 'view')">查看</button>
              <span class="sep">/</span>
              <button class="link-btn" type="button" @click="openDetail(row.admissionNo, 'edit')">编辑</button>
            </td>
          </tr>
        </tbody>
      </table>
      </div>
      <div v-if="store.viewState.totalPages > 1" class="pagination-row">
        <span class="page-meta">共 {{ store.viewState.total }} 条成绩记录</span>
        <div class="pagination-actions">
          <button class="page-btn" type="button" :disabled="store.viewState.page === 1" @click="goPage(store.viewState.page - 1)">上一页</button>
          <button v-for="page in visiblePages" :key="page" class="page-btn" :class="{ active: page === store.viewState.page }" type="button" @click="goPage(page)">{{ page }}</button>
          <button class="page-btn" type="button" :disabled="store.viewState.page === store.viewState.totalPages" @click="goPage(store.viewState.page + 1)">下一页</button>
        </div>
      </div>
    </TableCard>

    <div v-if="detailState.visible" class="detail-mask" @click.self="closeDetail">
      <section class="detail-card card-shell">
        <div class="detail-head">
          <h3>{{ detailState.mode === 'view' ? "查看成绩" : "编辑成绩" }}</h3>
          <button class="close-btn" type="button" @click="closeDetail">×</button>
        </div>
        <div v-if="detailState.loading" class="detail-loading">加载中...</div>
        <div v-else-if="detailState.error" class="detail-error">{{ detailState.error }}</div>
        <template v-else-if="detailState.form">
          <div class="detail-meta">
            <label class="meta-field">
              <span>姓名</span>
              <input v-model.trim="detailState.form.studentName" class="glass-field" :disabled="detailState.mode === 'view'" />
            </label>
            <label class="meta-field">
              <span>班级</span>
              <input v-model.trim="detailState.form.className" class="glass-field" :disabled="detailState.mode === 'view'" />
            </label>
            <label class="meta-field readonly">
              <span>准考证号</span>
              <input :value="detailState.form.admissionNo" class="glass-field" disabled />
            </label>
          </div>
          <div class="subject-list">
            <div v-for="item in detailState.form.subjects" :key="item.subject" class="subject-row">
              <strong>{{ SUBJECT_LABELS[item.subject] }}</strong>
              <FluentSelect
                v-model="item.state"
                :options="[{ label: '有成绩', value: 'scored' }, { label: '缺考', value: 'absent' }, { label: '未选考', value: 'not_selected' }]"
                :disabled="detailState.mode === 'view'"
                style="width: 130px; min-height: 38px;"
              />
              <input
                v-model.number="item.score"
                class="glass-field score-input"
                type="number"
                min="0"
                step="0.5"
                :disabled="detailState.mode === 'view' || item.state !== 'scored'"
                :placeholder="item.state === 'scored' ? '输入分数' : '--'"
              />
            </div>
          </div>
          <div class="detail-actions">
            <button class="secondary-btn" type="button" @click="closeDetail">关闭</button>
            <button
              v-if="detailState.mode === 'edit'"
              class="primary-btn"
              type="button"
              :disabled="detailState.saving"
              @click="saveDetail"
            >
              {{ detailState.saving ? "保存中..." : "保存" }}
            </button>
          </div>
        </template>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import type { ScoreCellState, ScoreDetail, ScoreRow, ScoreUpdatePayload } from "../../../entities/score/model";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useScoreStore } from "../store";

const store = useScoreStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;
const detailState = reactive<{
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
    return "拖拽成绩 Excel 文件到页面任意位置即可开始导入";
  }
  return store.viewState.importMessage;
});

const LANGUAGE_SHORT: Record<string, string> = { "英语": "英", "俄语": "俄", "日语": "日" };

function formatSubjectSelection(row: ScoreRow): string {
  if (row.subjectCombination === "全科") return "全科";
  const langShort = LANGUAGE_SHORT[row.language] ?? row.language;
  return "语数" + langShort + row.subjectCombination;
}

function rowClass(index: number) {
  return index % 2 === 1 ? "row-alt" : "";
}

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
}

function goPage(page: number) {
  void store.setPage(page);
}

const visiblePages = computed(() => {
  const total = store.viewState.totalPages;
  const current = store.viewState.page;
  const maxVisible = 7;
  if (total <= maxVisible) {
    return Array.from({ length: total }, (_, i) => i + 1);
  }
  const half = Math.floor(maxVisible / 2);
  let start = Math.max(1, current - half);
  let end = start + maxVisible - 1;
  if (end > total) {
    end = total;
    start = end - maxVisible + 1;
  }
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
});

async function openDetail(admissionNo: string, mode: "view" | "edit") {
  detailState.visible = true;
  detailState.mode = mode;
  detailState.loading = true;
  detailState.error = "";
  detailState.form = null;
  try {
    detailState.form = await store.getDetail(admissionNo);
  } catch (error) {
    detailState.error = error instanceof Error ? error.message : String(error);
  } finally {
    detailState.loading = false;
  }
}

function closeDetail() {
  detailState.visible = false;
  detailState.loading = false;
  detailState.saving = false;
  detailState.error = "";
  detailState.form = null;
}

async function saveDetail() {
  if (!detailState.form) {
    return;
  }
  detailState.saving = true;
  detailState.error = "";
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
    detailState.error = error instanceof Error ? error.message : String(error);
  } finally {
    detailState.saving = false;
  }
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
      const excelFilePath = pickExcelPath(event.payload.paths);
      if (excelFilePath) {
        void handleImport(excelFilePath);
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
  align-items: center;
  gap: 12px;
}

/* Filter fields – matches .pen scoreFilter spec */
.filter-search {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 220px;
  height: 42px;
  padding: 0 14px;
  border: 1px solid var(--color-border-soft);
  border-radius: 16px;
  background: var(--surface-input);
}

/* Search */
.filter-search {
  gap: 10px;
}

.filter-search-icon {
  color: var(--color-text-muted);
  font-size: 18px;
  font-family: "Material Symbols Rounded";
  flex-shrink: 0;
}

.filter-search input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--color-text);
  font-size: 14px;
  outline: none;
  min-width: 0;
}

.filter-search input::placeholder {
  color: var(--color-text-muted);
}

.filter-search:focus-within {
  border-color: rgba(var(--accent-rgb), 0.3);
  box-shadow: 0 0 0 4px var(--accent-focus-ring);
}

.score-table tbody tr {
  height: 58px;
}

.table-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.score-cell {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent-primary-strong);
  font-family: var(--font-mono);
}

.link-cell {
  color: var(--color-brand);
}

.link-btn {
  border: 0;
  background: transparent;
  color: var(--color-brand);
  cursor: pointer;
  font-size: 14px;
}

.sep {
  margin: 0 6px;
  color: var(--color-text-muted);
}

.emphasis {
  font-weight: 600;
}

.row-alt {
  background: var(--surface-elevated);
}

/* Pagination */
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

.detail-mask {
  position: fixed;
  inset: 0;
  background: var(--surface-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 40;
}

.detail-card {
  width: 780px;
  max-height: 86vh;
  overflow: auto;
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  border-radius: 26px;
}

.detail-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.detail-head h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
}

.close-btn {
  border: 0;
  background: transparent;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  color: var(--color-text-muted);
  width: 32px;
  height: 32px;
  border-radius: 10px;
}

.close-btn:hover {
  background: rgba(var(--accent-rgb), 0.08);
}

.detail-meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.meta-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.meta-field span {
  font-size: 12px;
  color: var(--color-text-muted);
}

.subject-list {
  border: 1px solid var(--color-border-soft);
  border-radius: 18px;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.subject-row {
  display: grid;
  grid-template-columns: 88px 140px 1fr;
  align-items: center;
  gap: 10px;
}

.small {
  min-height: 38px;
}

.score-input {
  min-height: 38px;
}

.detail-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.detail-loading,
.detail-error {
  color: var(--color-text-muted);
  font-size: 14px;
}

.import-status {
  margin: -10px 4px 0;
}

</style>
