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
          class="grade-select"
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
      <Pagination
        :currentPage="store.viewState.page"
        :pageSize="store.viewState.pageSize ?? 20"
        :total="store.viewState.total"
        @update:currentPage="goPage"
      />
    </TableCard>

    <div v-if="detailState.visible" class="detail-mask" @click.self="closeDetail">
      <section class="detail-card card-shell">
        <div class="detail-head">
          <h3>{{ detailState.mode === 'view' ? "查看成绩" : "编辑成绩" }}</h3>
          <button class="close-btn" type="button" @click="closeDetail">
            <span class="material-symbols-rounded" aria-hidden="true">close</span>
          </button>
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
                class="state-select"
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
import { Pagination } from "@/widgets/common";
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
  gap: var(--space-lg);
  position: relative;
  min-width: 900px;
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
  border-color: var(--accent-primary);
  background: var(--accent-softer);
}

.drag-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  border-radius: var(--radius-lg);
  background: rgba(var(--accent-rgb), 0.08);
  border: 2px dashed var(--border-strong);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drag-card {
  min-width: 280px;
  padding: var(--space-xl) var(--space-2xl);
  border-radius: var(--radius-lg);
  background: var(--surface-panel);
  box-shadow: var(--shadow-medium);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  text-align: center;
}

.drag-card strong {
  font-size: var(--font-size-lg);
  color: var(--accent-primary);
}

.drag-card span {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.toolbar-fields {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.grade-select {
  width: 220px;
}

.state-select {
  width: 130px;
}

.filter-search {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 220px;
  height: 42px;
  padding: 0 var(--space-lg);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-input);
  gap: 10px;
}

.filter-search-icon {
  color: var(--text-secondary);
  font-size: var(--font-size-xl);
  font-family: "Material Symbols Rounded";
  flex-shrink: 0;
}

.filter-search input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: var(--font-size-base);
  outline: none;
  min-width: 0;
}

.filter-search input::placeholder {
  color: var(--text-tertiary);
}

.filter-search:focus-within {
  border-color: rgba(var(--accent-rgb), 0.5);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
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
  font-size: var(--font-size-2xl);
  font-weight: 700;
  color: var(--accent-primary);
  font-family: var(--font-mono);
}

.link-cell {
  color: var(--accent-primary);
}

.link-btn {
  border: 0;
  background: transparent;
  color: var(--accent-primary);
  cursor: pointer;
  font-size: 14px;
}

.sep {
  margin: 0 6px;
  color: var(--text-tertiary);
}

.emphasis {
  font-weight: 600;
}

.row-alt {
  background: var(--surface-table-stripe);
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
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  border-radius: var(--radius-lg);
}

.detail-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.detail-head h3 {
  margin: 0;
  font-size: var(--font-size-title-sm);
  font-weight: 700;
}

.close-btn {
  border: 0;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background-color var(--transition-base) var(--transition-ease);
}

.close-btn .material-symbols-rounded {
  font-size: 20px;
}

.close-btn:hover {
  background: var(--accent-softer);
}

.detail-meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-md);
}

.meta-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.meta-field span {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
}

.subject-list {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: var(--space-md);
  display: grid;
  gap: var(--space-md);
}

.subject-row {
  display: grid;
  grid-template-columns: 88px 140px 1fr;
  align-items: center;
  gap: var(--space-md);
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
  gap: var(--space-md);
}

.detail-loading,
.detail-error {
  color: var(--text-secondary);
  font-size: 14px;
}

.import-status {
  margin: calc(-1 * var(--space-xs)) var(--space-xs) 0;
}
</style>
