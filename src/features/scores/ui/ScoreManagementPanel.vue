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
        <label>
          <select class="glass-field title-select">
            <option>考试标题</option>
          </select>
        </label>
        <label class="search-field">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79L19 20.5 20.5 19 15.5 14ZM9.5 14A4.5 4.5 0 1 1 14 9.5 4.5 4.5 0 0 1 9.5 14Z" />
          </svg>
          <input class="glass-field" :value="store.viewState.filters.nameKeyword" placeholder="按姓名筛选" @input="onNameInput" />
        </label>
      </div>
    </FilterToolbar>

    <InfoHint :text="importHintText" />
    <p v-if="store.viewState.importStatus !== 'idle'" class="import-status" :class="store.viewState.importStatus">
      {{ store.viewState.importMessage }}
    </p>

    <TableCard title="考试成绩列表" :meta="`已同步 ${store.viewState.total} 条`">
      <div class="table-scroll">
      <table class="table score-table">
        <thead>
          <tr>
            <th>姓名</th>
            <th>准考证号</th>
            <th>班级</th>
            <th>科目</th>
            <th>分数</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, index) in store.viewState.rows" :key="row.admissionNo" :class="rowClass(index)">
            <td class="emphasis">{{ row.studentName }}</td>
            <td>{{ row.admissionNo }}</td>
            <td>{{ row.className }}</td>
            <td>总分</td>
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
              <select v-model="item.state" class="glass-field small" :disabled="detailState.mode === 'view'">
                <option value="scored">有成绩</option>
                <option value="absent">缺考</option>
                <option value="not_selected">未选考</option>
              </select>
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
import type { ScoreCellState, ScoreDetail, ScoreUpdatePayload } from "../../../entities/score/model";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
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

const importHintText = computed(() => {
  if (isDragging.value) {
    return "松开鼠标即可导入成绩 Excel 文件";
  }
  return "可将 Excel 文件拖拽到页面任意位置导入成绩数据";
});

function rowClass(index: number) {
  return index % 2 === 1 ? "row-alt" : "";
}

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
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

.title-select {
  width: 220px;
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
  width: 220px;
  padding-left: 42px;
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
  font-size: 18px;
  font-weight: 700;
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
  background: #f8fbff;
}

.detail-mask {
  position: fixed;
  inset: 0;
  background: rgba(17, 21, 26, 0.42);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 40;
}

.detail-card {
  width: 780px;
  max-height: 86vh;
  overflow: auto;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.detail-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.detail-head h3 {
  margin: 0;
}

.close-btn {
  border: 0;
  background: transparent;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  color: var(--color-text-muted);
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
  border-radius: 14px;
  padding: 12px;
  display: grid;
  gap: 8px;
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
  font-size: 13px;
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
