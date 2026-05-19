<template>
  <section class="dashboard-grid">
    <div class="left-col">
      <ConfigCard title="当前考试配置">
        <p v-if="store.viewState.errorMessage" class="page-error-note" aria-live="polite">
          数据加载异常：{{ store.viewState.errorMessage }}
        </p>
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
        <template #actions>
          <FluentSelect
            :model-value="store.viewState.selectedSessionTimeGradeName"
            :options="sessionTimeGradeOptions"
            class="grade-select"
            @update:model-value="onSessionTimeGradeChange"
          />
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
                <td>{{ examTimeSubjectLabel(item.subject) }}</td>
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
                  <button class="icon-btn" type="button" :disabled="store.viewState.savingTimes" :title="`删除${examTimeSubjectLabel(item.subject)}考试时间配置`" @click="removeExistingSubjectTime(item.subject)">
                    <span class="material-symbols-rounded" aria-hidden="true">delete</span>
                  </button>
                </td>
              </tr>
              <tr v-for="item in manualSubjectRows" :key="item.id">
                <td>
                  <div class="manual-subject-row">
                    <FluentSelect
                      v-model="item.subject"
                      :options="DISPLAY_SUBJECT_OPTIONS.map(s => ({ label: examTimeSubjectLabel(s), value: s }))"
                      class="subject-select"
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
        <div class="hero-metrics">
          <div class="hero-metric">
            <span>考场数量</span>
            <strong>{{ store.viewState.overview.examRoomCount || "--" }}</strong>
          </div>
          <div class="hero-metric">
            <span>考生数量</span>
            <strong>{{ store.viewState.overview.studentAllocationCount || "--" }}</strong>
          </div>
        </div>
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

      <section class="overview-card card-shell">
        <h2 class="ov-main-title">配置与结果</h2>

        <h3 class="ov-section-title">考场容量参数</h3>
        <div class="ov-capacity-grid">
          <div class="ov-capacity-box">
            <div class="ov-box-label">考场默认容量</div>
            <div class="ov-box-value">
              <input v-model.number="capacityForm.defaultCapacity" class="ov-capacity-input" type="number" min="1" />
              <span class="ov-box-unit">人</span>
            </div>
          </div>
          <div class="ov-capacity-box">
            <div class="ov-box-label">考场最大容量</div>
            <div class="ov-box-value">
              <input v-model.number="capacityForm.maxCapacity" class="ov-capacity-input" type="number" min="1" />
              <span class="ov-box-unit">人</span>
            </div>
          </div>
        </div>

        <hr class="ov-divider" />

        <h3 class="ov-section-title">结果中心</h3>
        <div class="ov-status-list">
          <div class="ov-status-item">
            <span class="ov-status-label">任务状态</span>
            <span class="ov-badge" :class="getStatusClass()">{{ completeBadgeText }}</span>
          </div>
          <div class="ov-status-item">
            <span class="ov-status-label">已生成结果</span>
            <span class="ov-status-value">{{ store.viewState.overview.generatedAt ? "可导出" : "未生成" }}</span>
          </div>
          <div class="ov-status-item">
            <span class="ov-status-label">结果摘要</span>
            <button v-if="store.viewState.lastExportFolderPath" class="ov-result-link" type="button" @click="openExportFolder">{{ exportFileName }}</button>
            <span v-else class="ov-status-value ov-status-muted">尚未导出分配文件</span>
          </div>
        </div>

        <button class="ov-btn-block" :disabled="store.viewState.exporting || !store.viewState.overview.generatedAt" @click="exportBundle">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
          {{ store.viewState.exporting ? "导出中..." : "导出分配结果" }}
        </button>
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
const DISPLAY_SUBJECT_OPTIONS: Subject[] = SUBJECT_OPTIONS.filter(
  (subject) => subject !== Subject.Russian && subject !== Subject.Japanese,
);
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
  return "点击按钮即可开始分配考场 ~~";
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
  if (store.viewState.generationProgress.status === "error") {
    return "失败";
  }
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

const sessionTimeGradeOptions = computed(() =>
  store.viewState.sessionTimeGradeOptions.map((grade) => ({ label: grade, value: grade })),
);
const exportFileName = computed(() => {
  const raw = store.viewState.lastExportFolderPath;
  if (!raw) {
    return "";
  }
  const matched = raw.match(/[^\\/]+$/);
  return matched?.[0] ?? "考场安排";
});

function getStatusClass() {
  if (store.viewState.generationProgress.status === "error") {
    return "status-error";
  }
  if (store.viewState.generating || store.viewState.exporting) {
    return "status-pending";
  }
  if (store.viewState.overview.generatedAt) {
    return "status-success";
  }
  return "status-pending";
}

function examTimeSubjectLabel(subject: Subject): string {
  if (subject === Subject.English || subject === Subject.Russian || subject === Subject.Japanese) {
    return "外语";
  }
  return SUBJECT_LABELS[subject];
}

function addManualSubjectRow() {
  const used = new Set<Subject>([
    ...store.viewState.sessionTimes.map((item) => item.subject),
    ...manualSubjectRows.map((item) => item.subject),
  ]);
  const nextSubject = DISPLAY_SUBJECT_OPTIONS.find((subject) => !used.has(subject)) ?? DISPLAY_SUBJECT_OPTIONS[0];
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

  const extraItems: Array<{ sessionId: number; gradeName: string; subject: Subject; startAt: string; endAt: string }> = [];
  for (const row of manualSubjectRows) {
    if (!row.examMonthDay || !row.startTime || !row.endTime) {
      if (strictManualRows) {
        throw new Error(`请先完整填写 ${examTimeSubjectLabel(row.subject)} 的考试日期（月-日）、开始时间和结束时间`);
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
      gradeName: store.viewState.selectedSessionTimeGradeName,
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

function onSessionTimeGradeChange(value: string | number) {
  if (typeof value !== "string") {
    return;
  }
  void store.setSessionTimeGrade(value);
}

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
  gap: var(--space-xl);
  min-height: calc(100vh - 170px);
  min-width: 900px;
}

.exam-table-scroll {
  flex: 1;
  min-height: 380px;
  max-height: none;
  overflow-y: auto;
  padding-right: var(--space-sm);
}

.exam-table-scroll thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface-table-stripe);
}

.global-hint {
  grid-column: 1 / -1;
  margin: 0;
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-elevated);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.export-link {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--accent-primary);
  font: inherit;
  font-weight: 600;
  text-decoration: underline;
  cursor: pointer;
}

.left-col,
.right-col {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.left-col {
  gap: var(--space-lg);
}

.page-error-note {
  margin: 0 0 var(--space-md);
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

.right-col {
  gap: var(--space-lg);
}

.left-col :deep(.table-card) {
  flex: 1;
  min-height: 0;
}

.left-col :deep(.table-card .content) {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.left-col :deep(.config-card:first-child) {
  padding-bottom: var(--space-lg);
}

.left-col :deep(.config-card:first-child .glass-area) {
  min-height: 120px;
}

.field-stack {
  display: grid;
  gap: var(--space-md);
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.autosave-note {
  margin: 2px 0 0;
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.autosave-note.error {
  color: var(--color-danger);
}

.filled-field::placeholder,
.filled-area::placeholder {
  color: var(--text-secondary);
}

.progress-card {
  padding: var(--space-xl);
  border-radius: var(--radius-lg);
  border-color: rgba(var(--accent-rgb), 0.12);
  background: var(--accent-panel);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  color: var(--text-primary);
}

.progress-card::before,
.progress-card::after {
  display: none;
}

.progress-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-lg);
}

.progress-head h3 {
  margin: 0;
  font-size: var(--font-size-title-md);
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--accent-primary-strong);
}

.progress-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 26px;
  padding: var(--space-xs) var(--space-md);
  border-radius: var(--radius-pill);
  border: 1px solid rgba(var(--accent-rgb), 0.12);
  background: var(--surface-panel);
  color: var(--accent-primary);
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.progress-desc {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.55;
}

.hero-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-sm);
}

.hero-metric {
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.hero-metric span {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.hero-metric strong {
  color: var(--accent-primary);
  font-size: var(--font-size-title-lg);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.cta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-lg);
}

.percent {
  color: var(--accent-primary);
  font-size: var(--font-size-title-md);
  font-family: var(--font-mono);
  font-weight: 600;
  letter-spacing: -0.02em;
}

.progress-track {
  height: 8px;
  border-radius: var(--radius-pill);
  background: rgba(var(--accent-rgb), 0.1);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  background: var(--accent-primary);
  transition: width 0.3s ease;
}

.step-card {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-panel);
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.step-text {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.metric-input {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--text-primary);
  font-size: var(--font-size-xl);
  font-weight: 600;
}

.metric-input:focus {
  outline: none;
}

.table-hint {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.overview-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 32px;
  border-radius: 12px;
  border: 1px solid #edf2f7;
  background: #ffffff;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.03);
}

/* --- 主标题 with 蓝色竖条指示器 --- */
.ov-main-title {
  font-size: 20px;
  font-weight: 600;
  color: #0f172a;
  margin: 0 0 28px 0;
  display: flex;
  align-items: center;
}

.ov-main-title::before {
  content: '';
  display: inline-block;
  width: 4px;
  height: 18px;
  background: #3182ce;
  border-radius: 2px;
  margin-right: 12px;
}

/* --- 子模块标题 --- */
.ov-section-title {
  font-size: 15px;
  font-weight: 600;
  color: #334155;
  margin: 0 0 16px 0;
}

/* --- 容量参数区 --- */
.ov-capacity-grid {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}

.ov-capacity-box {
  flex: 1;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
  transition: all 0.2s ease;
}

.ov-capacity-box:hover {
  background: #ffffff;
  border-color: #cbd5e0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.ov-box-label {
  font-size: 13px;
  color: #64748b;
  margin-bottom: 8px;
  font-weight: 500;
}

.ov-box-value {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.ov-capacity-input {
  border: 0;
  padding: 0;
  background: transparent;
  color: #0f172a;
  font-size: 28px;
  font-weight: 700;
  width: 60px;
  font-family: inherit;
}

.ov-capacity-input:focus {
  outline: none;
}

.ov-capacity-input::-webkit-outer-spin-button,
.ov-capacity-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.ov-capacity-input[type="number"] {
  -moz-appearance: textfield;
  appearance: textfield;
}

.ov-box-unit {
  font-size: 14px;
  font-weight: 500;
  color: #94a3b8;
}

/* --- 分割线 --- */
.ov-divider {
  height: 1px;
  background-color: #e2e8f0;
  border: none;
  margin: 0 0 24px 0;
}

/* --- 结果中心列表 --- */
.ov-status-list {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 28px;
}

.ov-status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background-color: #ffffff;
  border-bottom: 1px solid #f1f5f9;
  font-size: 14px;
}

.ov-status-item:last-child {
  border-bottom: none;
}

.ov-status-label {
  color: #64748b;
}

.ov-status-value {
  color: #0f172a;
  font-weight: 500;
}

.ov-status-muted {
  color: #64748b;
  font-weight: normal;
}

/* --- 状态标签 Pill Badge --- */
.ov-badge {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}

.ov-badge.status-pending {
  background-color: #fffbeb;
  color: #d97706;
  border: 1px solid #fde68a;
}

.ov-badge.status-success {
  background-color: #ecfdf5;
  color: #059669;
  border: 1px solid #a7f3d0;
}

.ov-badge.status-error {
  background-color: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
}

/* --- 结果链接 --- */
.ov-result-link {
  border: 0;
  padding: 0;
  background: transparent;
  color: #3182ce;
  font: inherit;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
  text-align: right;
}

.ov-result-link:hover {
  color: #2563eb;
}

/* --- 底部操作按钮 --- */
.ov-btn-block {
  width: 100%;
  padding: 12px 0;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  text-align: center;
  border: none;
  transition: all 0.2s;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.ov-btn-block:not(:disabled) {
  background-color: #3b82f6;
  color: #ffffff;
}

.ov-btn-block:not(:disabled):hover {
  background-color: #2563eb;
}

.ov-btn-block:disabled {
  background-color: #bfdbfe;
  color: #ffffff;
  cursor: not-allowed;
}

.exam-table thead tr {
  height: 44px;
}

.exam-table tbody tr {
  height: 48px;
}

.exam-table th,
.exam-table td {
  padding-inline: var(--space-lg);
}

.exam-table td {
  font-size: var(--font-size-sm);
}

.exam-table tbody td:first-child {
  font-weight: 600;
  color: var(--text-primary);
}

.time-input {
  width: 88px;
  border: 1px solid transparent;
  background: transparent;
  border-radius: var(--radius-sm);
  padding: 4px var(--space-sm);
  color: var(--text-primary);
  font-family: var(--font-ui);
}

.month-day-input {
  width: 72px;
  border: 0;
  background: transparent;
  font-family: var(--font-ui);
}

.month-day-input.inline-edit {
  width: 84px;
  border: 1px solid rgba(var(--accent-rgb), 0.5);
  border-radius: var(--radius-sm);
  background: var(--surface-input);
  padding: 4px var(--space-sm);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
}

.time-input:focus {
  outline: none;
  border: 1px solid rgba(var(--accent-rgb), 0.5);
  background: var(--surface-input);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
}

.month-day-input:focus {
  outline: none;
}

.date-cell {
  width: 110px;
  border-radius: var(--radius-sm);
  transition: background-color var(--transition-fast);
}

.date-cell.editing {
  background: var(--accent-fill-soft);
}

.date-display-btn {
  border: 0;
  background: transparent;
  color: var(--text-primary);
  font: inherit;
  font-weight: 600;
  cursor: text;
  padding: 0;
}

.time-cell {
  border-radius: var(--radius-sm);
  transition: background-color var(--transition-fast);
}

.time-cell:focus-within {
  background: var(--accent-fill-soft);
}

.manual-subject-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.grade-select {
  width: 150px;
}

.subject-select {
  width: 140px;
}

.icon-btn {
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  transition: background-color var(--transition-base) var(--transition-ease), color var(--transition-base) var(--transition-ease);
}

.icon-btn:hover {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}

.icon-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
}

.icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
