<template>
  <section class="dashboard-grid">
    <div class="left-col">
      <ConfigCard title="当前考试配置" description="统一设置考试标题、须知与各科目时间，修改后自动保存本页配置。">
        <div class="field-stack" :style="{ opacity: store.viewState.loading ? 0 : 1, pointerEvents: store.viewState.loading ? 'none' : 'auto', transition: 'opacity 0.3s ease' }">
          <label class="field-block">
            <span class="metric-label">考试标题</span>
            <input
              v-model.trim="capacityForm.examTitle"
              class="glass-field filled-field"
              type="text"
              placeholder="2026 学年春季期末统一考试"
            />
          </label>
          <label class="field-block">
            <span class="metric-label">考生须知</span>
            <textarea
              v-model="capacityForm.examNoticesText"
              class="glass-area filled-area"
              placeholder="请考生提前 30 分钟入场，核对准考证信息；开考 15 分钟后不得进入考场。严禁携带通讯设备与电子资料。"
            />
          </label>
        </div>
        <p class="autosave-note" :class="{ error: !!autoSaveError }" aria-live="polite">
          {{ autoSaveError ? `自动保存失败：${autoSaveError}` : autoSaveText }}
        </p>
      </ConfigCard>

      <TableCard title="考试时间">
        <template #description>
          <p class="table-hint">统一配置各科目考试时间。</p>
        </template>
        <template #actions>
          <button class="secondary-btn" type="button" :disabled="store.viewState.loading" @click="addManualSubjectRow">新增科目</button>
        </template>
        <div class="exam-table-scroll" :style="{ opacity: store.viewState.loading ? 0 : 1, transition: 'opacity 0.3s ease' }">
          <table class="table exam-table">
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
              <tr v-for="item in store.viewState.sessionTimes" :key="item.sessionId">
                <td>{{ SUBJECT_LABELS[item.subject] }}</td>
                <td class="date-cell" :class="{ editing: dateEditState.sessionId === item.sessionId }" @dblclick="beginDateEdit(item.sessionId)">
                  <input
                    v-if="dateEditState.sessionId === item.sessionId"
                    v-model.trim="dateEditState.value"
                    class="month-day-input inline-edit"
                    type="text"
                    placeholder="03-24"
                    autofocus
                    @blur="commitDateEdit(item.sessionId)"
                    @keydown.enter.prevent="commitDateEdit(item.sessionId)"
                    @keydown.esc.prevent="cancelDateEdit"
                  />
                  <button v-else class="date-display-btn" type="button" @dblclick.stop="beginDateEdit(item.sessionId)">
                    {{ formatMonthDay(store.viewState.sessionTimeDrafts[item.sessionId]?.startAt || item.startAt) }}
                  </button>
                </td>
                <td class="time-cell">
                  <input class="time-input" type="time" :value="formatTimeInput(store.viewState.sessionTimeDrafts[item.sessionId]?.startAt)" @input="onTimeInput(item.sessionId, 'startAt', $event)" />
                </td>
                <td class="time-cell">
                  <input class="time-input" type="time" :value="formatTimeInput(store.viewState.sessionTimeDrafts[item.sessionId]?.endAt)" @input="onTimeInput(item.sessionId, 'endAt', $event)" />
                </td>
                <td>
                  <button class="icon-btn" type="button" :disabled="store.viewState.savingTimes" :title="`删除${SUBJECT_LABELS[item.subject]}考试时间配置`" @click="removeExistingSubjectTime(item.subject)">
                    <span class="material-symbols-rounded" aria-hidden="true">delete</span>
                  </button>
                </td>
              </tr>
              <tr v-for="item in manualSubjectRows" :key="item.id">
                <td>
                  <div class="manual-subject-row">
                    <FluentSelect
                      v-model="item.subject"
                      :options="SUBJECT_OPTIONS.map(s => ({ label: SUBJECT_LABELS[s], value: s }))"
                      style="width: 140px; min-height: 38px;"
                    />
                  </div>
                </td>
                <td>
                  <input v-model.trim="item.examMonthDay" class="month-day-input" type="text" placeholder="03-24" />
                </td>
                <td>
                  <input v-model="item.startTime" class="time-input" type="time" />
                </td>
                <td>
                  <input v-model="item.endTime" class="time-input" type="time" />
                </td>
                <td>
                  <button class="icon-btn" type="button" @click="removeManualSubjectRow(item.id)" title="删除该科目时间配置">
                    <span class="material-symbols-rounded" aria-hidden="true">delete</span>
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </TableCard>
    </div>

    <div class="right-col">
      <section class="progress-card card-shell">
        <div class="progress-head">
          <h3>开始分配考场</h3>
          <span class="progress-badge">{{ progressBadgeText }}</span>
        </div>
        <p class="progress-desc">{{ progressDescription }}</p>
        <div class="cta-row">
          <button class="primary-btn" :disabled="store.viewState.generating || isPreparingGenerate" @click="generateExamPlan">
            {{ generateActionText }}
          </button>
          <strong class="percent">{{ progressPercent }}%</strong>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: `${progressPercent}%` }" />
        </div>
        <div class="step-card">
          <span class="metric-label">当前步骤</span>
          <strong class="step-text">{{ progressStepText }}</strong>
        </div>
      </section>

      <ConfigCard title="考场容量配置">
        <div class="field-stack compact" :style="{ opacity: store.viewState.loading ? 0 : 1, pointerEvents: store.viewState.loading ? 'none' : 'auto', transition: 'opacity 0.3s ease' }">
          <label class="metric-field">
            <span class="metric-label">考场默认容量</span>
            <input v-model.number="capacityForm.defaultCapacity" class="metric-input" type="number" min="1" />
          </label>
          <label class="metric-field">
            <span class="metric-label">考场最大容量</span>
            <input v-model.number="capacityForm.maxCapacity" class="metric-input" type="number" min="1" />
          </label>
        </div>
      </ConfigCard>

      <section class="complete-card card-shell">
        <div class="complete-head">
          <h3>{{ completeTitle }}</h3>
          <span class="complete-badge" :class="{ pending: isCompletePending }">{{ completeBadgeText }}</span>
        </div>
        <p class="complete-desc">{{ completeDescription }}</p>
        <div class="complete-meta">
          <div class="complete-summary">
            <span class="metric-label">结果摘要</span>
            <strong v-if="!store.viewState.lastExportFolderPath">{{ completeSummary }}</strong>
            <button v-else class="export-link" type="button" @click="openExportFolder">{{ exportFileName }}</button>
          </div>
          <div class="complete-action">
            <button class="primary-btn export-btn" :disabled="store.viewState.exporting || !store.viewState.overview.generatedAt" @click="exportBundle">
              {{ store.viewState.exporting ? "导出中..." : "导出分配结果" }}
            </button>
          </div>
        </div>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import { Subject } from "../../../entities/score/model";
import { revealInExplorer } from "../../../shared/utils/appLog";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useExamAllocationStore } from "../store";

const store = useExamAllocationStore();
const GENERATION_STAGE_ORDER: Record<string, number> = {
  loading_config: 1,
  clearing_snapshot: 2,
  building_sessions: 3,
  allocating_rooms: 4,
  finalizing_results: 5,
  exporting_files: 6,
};
const TOTAL_GENERATION_STAGES = 6;
const capacityForm = reactive({
  defaultCapacity: 40,
  maxCapacity: 41,
  examTitle: "",
  examNoticesText: "",
});
const SUBJECT_OPTIONS: Subject[] = Object.values(Subject);
const manualSubjectRows = reactive<Array<{ id: number; subject: Subject; examMonthDay: string; startTime: string; endTime: string }>>([]);
const dateEditState = reactive<{ sessionId: number | null; value: string }>({
  sessionId: null,
  value: "",
});
let manualSubjectRowId = 1;
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
const autoSaveReady = ref(false);
const autoSaving = ref(false);
const autoSaveError = ref("");
const autoSavedAt = ref(0);
const autoSaveDirty = ref(false);
const suppressAutoSave = ref(false);
const isPreparingGenerate = ref(false);

const progressPercent = computed(() => {
  if (store.viewState.generating || store.viewState.generationProgress.status === "running") {
    return store.viewState.generationProgress.percent;
  }
  if (store.viewState.generationProgress.status === "completed") {
    return 100;
  }
  return store.viewState.overview.generatedAt ? 100 : 0;
});

const progressBadgeText = computed(() => {
  if (store.viewState.generationProgress.status === "error") {
    return "失败";
  }
  if (store.viewState.generating || store.viewState.generationProgress.status === "running") {
    return store.viewState.generationProgress.stageLabel || "执行中";
  }
  if (store.viewState.overview.generatedAt) {
    return "已完成";
  }
  return "待执行";
});

const progressDescription = computed(() => {
  const progress = store.viewState.generationProgress;
  const stageIndex = GENERATION_STAGE_ORDER[progress.stage];
  if (store.viewState.generationProgress.status === "running") {
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
  return "系统会自动为所有年级的学生分配考场，并生成相关导出文件。";
});

const progressStepText = computed(() => {
  const progress = store.viewState.generationProgress;
  const stageIndex = GENERATION_STAGE_ORDER[progress.stage];
  if (store.viewState.generationProgress.status === "error") {
    return progress.message || "分配过程中出现错误，请查看日志。";
  }
  if (store.viewState.generating || store.viewState.generationProgress.status === "running") {
    const stepPrefix = stageIndex
      ? `第 ${stageIndex}/${TOTAL_GENERATION_STAGES} 阶段 · ${progress.stageLabel}`
      : progress.stageLabel || "执行中";
    return progress.message ? `${stepPrefix}：${progress.message}` : stepPrefix;
  }
  if (store.viewState.overview.generatedAt) {
    return "考场分配完成，点击导出打开结果目录。";
  }
  return "等待开始，系统将按当前配置自动排考场。";
});

const completeBadgeText = computed(() => {
  if (store.viewState.exporting) {
    return "导出中";
  }
  if (store.viewState.generating) {
    return "执行中";
  }
  if (store.viewState.overview.generatedAt) {
    return "已完成";
  }
  return "未开始";
});

const completeTitle = computed(() => {
  if (store.viewState.generating) {
    return "分配进行中";
  }
  if (store.viewState.overview.generatedAt) {
    return "分配已完成";
  }
  return "等待分配";
});

const completeDescription = computed(() => {
  if (!store.viewState.overview.generatedAt) {
    return "尚未生成考场分配结果，完成配置后点击“开始分配考场”即可执行。";
  }
  return `完成 ${store.viewState.overview.examRoomCount} 个考场与 ${store.viewState.overview.studentAllocationCount} 名考生自动分配。`;
});

const completeSummary = computed(() => "尚未导出分配文件");
const exportFileName = computed(() => {
  const raw = store.viewState.lastExportFolderPath;
  if (!raw) {
    return "";
  }
  const matched = raw.match(/[^\\/]+$/);
  return matched?.[0] ?? "考场安排";
});

const isCompletePending = computed(() => store.viewState.generating || !store.viewState.overview.generatedAt);

function addManualSubjectRow() {
  const used = new Set<Subject>([
    ...store.viewState.sessionTimes.map((item) => item.subject),
    ...manualSubjectRows.map((item) => item.subject),
  ]);
  const nextSubject = SUBJECT_OPTIONS.find((subject) => !used.has(subject)) ?? SUBJECT_OPTIONS[0];
  manualSubjectRows.push({
    id: manualSubjectRowId++,
    subject: nextSubject,
    examMonthDay: formatMonthDay(new Date().toISOString().slice(0, 10)),
    startTime: "",
    endTime: "",
  });
}

function removeManualSubjectRow(id: number) {
  const index = manualSubjectRows.findIndex((item) => item.id === id);
  if (index >= 0) {
    manualSubjectRows.splice(index, 1);
  }
}

function getDraftDate(sessionId: number): string {
  const draft = store.viewState.sessionTimeDrafts[sessionId];
  const source = draft?.startAt || draft?.endAt;
  if (source && source.length >= 10) {
    return source.slice(0, 10);
  }
  return new Date().toISOString().slice(0, 10);
}

const isApplyingConfig = computed(() => store.viewState.saving || store.viewState.savingTimes);
const autoSaveText = computed(() => {
  if (store.viewState.loading) return "正在加载配置...";
  if (autoSaving.value) return "正在自动保存...";
  if (autoSavedAt.value > 0) {
    return `已自动保存（${new Date(autoSavedAt.value).toLocaleTimeString("zh-CN", { hour12: false })}）`;
  }
  return "修改后自动保存";
});
const completeManualRowsSignature = computed(() =>
  JSON.stringify(
    manualSubjectRows
      .filter((row) => row.examMonthDay.trim() && row.startTime.trim() && row.endTime.trim())
      .map((row) => ({
        id: row.id,
        subject: row.subject,
        examMonthDay: row.examMonthDay.trim(),
        startTime: row.startTime.trim(),
        endTime: row.endTime.trim(),
      })),
  ),
);

const generateActionText = computed(() => {
  if (store.viewState.generating) {
    return "分配中...";
  }
  if (isPreparingGenerate.value) {
    return "保存配置中...";
  }
  return "开始分配考场";
});

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

function normalizeMonthDay(value: string): string | null {
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

function resolveFullDateFromMonthDay(monthDay: string, fallbackDate: string): string {
  const normalized = normalizeMonthDay(monthDay);
  if (!normalized) {
    throw new Error(`考试日期格式应为 MM-DD（例如 03-24）`);
  }
  const year = fallbackDate.slice(0, 4);
  return `${year}-${normalized}`;
}

function formatTimeInput(value?: string) {
  if (!value) {
    return "";
  }
  return value.replace("T", " ").slice(11, 16);
}

function onTimeInput(sessionId: number, field: "startAt" | "endAt", event: Event) {
  const current = store.viewState.sessionTimeDrafts[sessionId];
  const raw = (event.target as HTMLInputElement).value;
  const datePart = formatDate(current?.startAt || current?.endAt) || new Date().toISOString().slice(0, 10);
  store.setSessionTimeDraft(sessionId, field, `${datePart}T${raw}`);
}

function beginDateEdit(sessionId: number) {
  const current = store.viewState.sessionTimeDrafts[sessionId];
  const fromStart = formatMonthDay(current?.startAt);
  const fromEnd = formatMonthDay(current?.endAt);
  const monthDay = fromStart !== "--" ? fromStart : fromEnd;
  dateEditState.sessionId = sessionId;
  dateEditState.value = monthDay === "--" ? "" : monthDay;
}

function cancelDateEdit() {
  dateEditState.sessionId = null;
  dateEditState.value = "";
}

function commitDateEdit(sessionId: number) {
  if (dateEditState.sessionId !== sessionId) {
    return;
  }
  const normalized = normalizeMonthDay(dateEditState.value);
  if (!normalized) {
    cancelDateEdit();
    return;
  }
  const fallbackDate = getDraftDate(sessionId);
  const targetDate = resolveFullDateFromMonthDay(normalized, fallbackDate);
  const draft = store.viewState.sessionTimeDrafts[sessionId];
  const startTime = formatTimeInput(draft?.startAt) || "08:00";
  const endTime = formatTimeInput(draft?.endAt) || "10:00";
  store.setSessionTimeDraft(sessionId, "startAt", `${targetDate}T${startTime}`);
  store.setSessionTimeDraft(sessionId, "endAt", `${targetDate}T${endTime}`);
  cancelDateEdit();
}

function onGlobalPointerDown(event: PointerEvent) {
  if (dateEditState.sessionId === null) {
    return;
  }
  const target = event.target as HTMLElement | null;
  if (target?.closest(".date-cell.editing")) {
    return;
  }
  commitDateEdit(dateEditState.sessionId);
}

async function persistDrafts(options: { strictManualRows?: boolean; clearManualRows?: boolean } = {}) {
  const { strictManualRows = true, clearManualRows = true } = options;
  const examNotices = capacityForm.examNoticesText
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  await store.saveSettings(capacityForm.defaultCapacity, capacityForm.maxCapacity, capacityForm.examTitle, examNotices);

  const extraItems: Array<{ sessionId: number; subject: Subject; startAt: string; endAt: string }> = [];
  for (const row of manualSubjectRows) {
    if (!row.examMonthDay || !row.startTime || !row.endTime) {
      if (strictManualRows) {
        throw new Error(`请先完整填写 ${SUBJECT_LABELS[row.subject]} 的考试日期（月-日）、开始时间和结束时间`);
      }
      continue;
    }
    const existing = store.viewState.sessionTimes.find((item) => item.subject === row.subject);
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
      subject: row.subject,
      startAt: `${targetDate}T${row.startTime}`,
      endAt: `${targetDate}T${row.endTime}`,
    });
  }

  await store.saveSessionTimes(extraItems);
  if (clearManualRows) {
    manualSubjectRows.splice(0, manualSubjectRows.length);
  }
}

async function removeExistingSubjectTime(subject: Subject) {
  await store.deleteSessionTime(subject);
}

async function generateExamPlan() {
  if (store.viewState.generating || isPreparingGenerate.value) return;
  isPreparingGenerate.value = true;
  autoSaveError.value = "";
  if (autoSaveTimer) {
    clearTimeout(autoSaveTimer);
    autoSaveTimer = null;
  }
  autoSaveDirty.value = false;
  suppressAutoSave.value = true;
  try {
    await persistDrafts();
  } finally {
    suppressAutoSave.value = false;
    isPreparingGenerate.value = false;
  }
  await store.generate();
}
function scheduleAutoSave(delay = 700) {
  if (!autoSaveReady.value || suppressAutoSave.value) return;
  autoSaveDirty.value = true;
  if (autoSaveTimer) {
    clearTimeout(autoSaveTimer);
  }
  autoSaveTimer = setTimeout(() => {
    void flushAutoSave();
  }, delay);
}

async function flushAutoSave() {
  if (!autoSaveReady.value || suppressAutoSave.value || !autoSaveDirty.value) return;
  if (store.viewState.generating || isApplyingConfig.value) {
    scheduleAutoSave(400);
    return;
  }
  autoSaveDirty.value = false;
  autoSaving.value = true;
  autoSaveError.value = "";
  suppressAutoSave.value = true;
  try {
    await persistDrafts({ strictManualRows: false, clearManualRows: false });
    autoSavedAt.value = Date.now();
  } catch (error) {
    autoSaveDirty.value = true;
    autoSaveError.value = error instanceof Error ? error.message : String(error);
    scheduleAutoSave(1200);
  } finally {
    suppressAutoSave.value = false;
    autoSaving.value = false;
  }
}

async function exportBundle() {
  await store.exportLatestBundle();
}

async function openExportFolder() {
  const target = store.viewState.lastExportFolderPath;
  if (!target) {
    return;
  }
  await revealInExplorer(target);
}

watch(
  () => [capacityForm.examTitle, capacityForm.examNoticesText, capacityForm.defaultCapacity, capacityForm.maxCapacity],
  () => {
    if (!autoSaveReady.value || suppressAutoSave.value) return;
    autoSaveError.value = "";
    scheduleAutoSave();
  },
);

watch(
  () => store.viewState.sessionTimeDrafts,
  () => {
    if (!autoSaveReady.value || suppressAutoSave.value) return;
    autoSaveError.value = "";
    scheduleAutoSave(850);
  },
  { deep: true },
);

watch(completeManualRowsSignature, (next, prev) => {
  if (!autoSaveReady.value || suppressAutoSave.value) return;
  if (next === prev) return;
  autoSaveError.value = "";
  scheduleAutoSave(850);
});

onMounted(async () => {
  await store.loadAll();
  capacityForm.defaultCapacity = store.viewState.settings.defaultCapacity;
  capacityForm.maxCapacity = store.viewState.settings.maxCapacity;
  capacityForm.examTitle = store.viewState.settings.examTitle ?? "";
  capacityForm.examNoticesText = (store.viewState.settings.examNotices ?? []).join("\n");
  autoSaveReady.value = true;
  window.addEventListener("pointerdown", onGlobalPointerDown, true);
});

onUnmounted(() => {
  if (autoSaveTimer) {
    clearTimeout(autoSaveTimer);
    autoSaveTimer = null;
  }
  window.removeEventListener("pointerdown", onGlobalPointerDown, true);
});
</script>

<style scoped>
.dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.75fr) minmax(320px, 1fr);
  gap: 22px;
}

.exam-table-scroll {
  max-height: 260px;
  overflow-y: auto;
  padding-right: 6px;
}

.exam-table-scroll thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: #f7f9fc;
}

.global-hint {
  grid-column: 1 / -1;
  margin: 0;
  padding: 10px 12px;
  border: 1px solid #dce8f8;
  border-radius: 12px;
  background: #f7fbff;
  color: var(--color-text-muted);
  font-size: 13px;
}

.export-link {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--color-brand);
  font: inherit;
  font-weight: 700;
  text-decoration: underline;
  cursor: pointer;
}


.left-col,
.right-col {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.left-col {
  gap: 18px;
}

.right-col {
  gap: 12px;
}

.field-stack {
  display: grid;
  gap: 12px;
}

.field-stack.compact {
  gap: 10px;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.autosave-note {
  margin: 6px 0 0;
  font-size: 12px;
  color: #5f738d;
}

.autosave-note.error {
  color: var(--color-danger);
}

.filled-field::placeholder,
.filled-area::placeholder {
  color: rgba(28, 31, 35, 0.72);
}

.progress-card {
  padding: 18px;
  border-radius: 22px;
  border-color: #d9e8ff;
  background: linear-gradient(135deg, rgba(247, 251, 255, 0.8), rgba(233, 243, 255, 0.8));
  box-shadow: 0 14px 40px var(--color-shadow-strong);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.progress-head h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}

.progress-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 8px 12px;
  border-radius: 999px;
  border: 1px solid #cfe0fb;
  background: rgba(255, 255, 255, 0.6);
  color: var(--color-brand);
  font-size: 12px;
  font-weight: 700;
}

.progress-desc,
.complete-desc {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 14px;
  line-height: 1.45;
}

.cta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.percent {
  color: var(--color-brand);
  font-size: 28px;
}

.progress-track {
  height: 14px;
  border-radius: 999px;
  background: #dcebff;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #0f6cbd, #2e86de);
}

.step-card {
  border: 1px solid #e4eeff;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.56);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.step-text {
  font-size: 14px;
}

.metric-input {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
}

.metric-input:focus {
  outline: none;
}

.table-hint {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.complete-card {
  padding: 12px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.72);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.complete-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.complete-head h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
}

.complete-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 8px 12px;
  border-radius: 999px;
  background: var(--color-success-soft);
  color: var(--color-success);
  font-size: 12px;
  font-weight: 700;
}

.complete-badge.pending {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.complete-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.complete-summary {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.complete-summary strong {
  font-size: 14px;
}

.complete-action {
  display: flex;
  justify-content: flex-end;
}

.export-btn {
  width: 164px;
}

.exam-table tbody tr {
  height: 58px;
}

.time-input {
  width: 88px;
  border: 0;
  background: transparent;
  border-radius: 8px;
  padding: 4px 8px;
}

.month-day-input {
  width: 72px;
  border: 0;
  background: transparent;
}

.month-day-input.inline-edit {
  width: 84px;
  border: 1px solid #b9d6ff;
  border-radius: 8px;
  background: #f4f9ff;
  padding: 4px 8px;
  box-shadow: 0 0 0 2px rgba(185, 214, 255, 0.35);
}

.time-input:focus {
  outline: none;
  border: 1px solid #b9d6ff;
  background: #f4f9ff;
  box-shadow: 0 0 0 2px rgba(185, 214, 255, 0.35);
}

.month-day-input:focus {
  outline: none;
}

.date-cell {
  width: 110px;
  border-radius: 10px;
  transition: background-color 0.15s ease;
}

.date-cell.editing {
  background: #eef5ff;
}

.date-display-btn {
  border: 0;
  background: transparent;
  color: var(--color-text);
  font: inherit;
  cursor: text;
  padding: 0;
}

.time-cell {
  border-radius: 10px;
  transition: background-color 0.15s ease;
}

.time-cell:focus-within {
  background: #eef5ff;
}


.manual-subject-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.subject-select {
  min-width: 110px;
  border: 1px solid var(--color-border-soft);
  border-radius: 10px;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.9);
}

.icon-btn {
  border: 0;
  background: transparent;
  color: #c26868;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.icon-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
}

.icon-btn.disabled {
  color: var(--color-text-muted);
  cursor: not-allowed;
}

</style>
