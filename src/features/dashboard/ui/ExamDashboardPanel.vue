<template>
  <section class="dashboard-grid">
    <div class="left-col">
      <ConfigCard title="当前考试配置" description="统一设置考试标题、须知与各科目时间，保持编排逻辑集中可见。">
        <div class="field-stack">
          <label class="field-block">
            <span class="metric-label">考试标题</span>
            <input v-model.trim="capacityForm.examTitle" class="glass-field" type="text" />
          </label>
          <label class="field-block">
            <span class="metric-label">考试须知</span>
            <textarea v-model="capacityForm.examNoticesText" class="glass-area" />
          </label>
        </div>
      </ConfigCard>

      <TableCard title="考试时间" meta="支持新增、双击单元格修改、删除">
        <template #actions>
          <button class="secondary-btn" disabled>新增科目</button>
        </template>
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
              <td>{{ formatDate(item.startAt) }}</td>
              <td>
                <input class="time-input" type="time" :value="formatTimeInput(store.viewState.sessionTimeDrafts[item.sessionId]?.startAt)" @input="onTimeInput(item.sessionId, 'startAt', $event)" />
              </td>
              <td>
                <input class="time-input" type="time" :value="formatTimeInput(store.viewState.sessionTimeDrafts[item.sessionId]?.endAt)" @input="onTimeInput(item.sessionId, 'endAt', $event)" />
              </td>
              <td><button class="icon-btn" type="button" disabled>🗑</button></td>
            </tr>
          </tbody>
        </table>
        <div class="table-actions">
          <button class="secondary-btn" :disabled="store.viewState.savingTimes" @click="saveSessionTimes">
            {{ store.viewState.savingTimes ? "保存中..." : "保存考试时间" }}
          </button>
        </div>
      </TableCard>
    </div>

    <div class="right-col">
      <section class="progress-card card-shell">
        <div class="progress-head">
          <h3>开始分配考场</h3>
          <span class="tag-pill">AUTO</span>
        </div>
        <p class="progress-desc">系统将先按班级聚合，再根据科目与监考需求进行自动编排。</p>
        <div class="cta-row">
          <button class="primary-btn" :disabled="store.viewState.generating" @click="generateExamPlan">
            {{ store.viewState.generating ? "分配中..." : "开始分配考场" }}
          </button>
          <strong class="percent">{{ progressPercent }}%</strong>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: `${progressPercent}%` }" />
        </div>
        <div class="step-card">
          <span class="metric-label">当前步骤</span>
          <strong class="step-text">{{ store.viewState.generating ? "正在生成考场快照" : "进度已就绪" }}</strong>
        </div>
      </section>

      <ConfigCard title="考场容量配置">
        <div class="field-stack compact">
          <label class="metric-field">
            <span class="metric-label">考场默认容量</span>
            <input v-model.number="capacityForm.defaultCapacity" class="metric-input" type="number" min="1" />
          </label>
          <label class="metric-field">
            <span class="metric-label">考场最大容量</span>
            <input v-model.number="capacityForm.maxCapacity" class="metric-input" type="number" min="1" />
          </label>
        </div>
        <button class="secondary-btn fit" :disabled="store.viewState.saving" @click="saveCapacity">
          {{ store.viewState.saving ? "保存中..." : "保存配置" }}
        </button>
      </ConfigCard>

      <ConfigCard title="分配已完成">
        <p class="complete-desc">完成 {{ store.viewState.overview.examRoomCount }} 个考场、{{ store.viewState.staffOverview.assignedCount }} 位监考老师与 {{ store.viewState.overview.studentAllocationCount }} 名考生自动分配。</p>
        <div class="complete-stack">
          <div class="complete-item">
            <span class="metric-label">生成时间</span>
            <span>{{ store.viewState.overview.generatedAt ?? "暂无" }}</span>
          </div>
          <div class="complete-item">
            <span class="metric-label">场次</span>
            <span>{{ store.viewState.overview.sessionCount }} 场</span>
          </div>
          <div class="complete-item">
            <span class="metric-label">告警</span>
            <span>{{ store.viewState.overview.warningCount }} 条</span>
          </div>
        </div>
        <button class="primary-btn fit" :disabled="store.viewState.exporting" @click="exportBundle">
          {{ store.viewState.exporting ? "导出中..." : "导出分配结果" }}
        </button>
      </ConfigCard>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive } from "vue";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useExamAllocationStore } from "../store";

const store = useExamAllocationStore();
const capacityForm = reactive({
  defaultCapacity: 40,
  maxCapacity: 41,
  examTitle: "",
  examNoticesText: "",
});

const progressPercent = computed(() => {
  if (store.viewState.generating) {
    return 68;
  }
  if (!store.viewState.overview.generatedAt) {
    return 0;
  }
  return 100;
});

function formatDate(value?: string | null) {
  if (!value) {
    return "--";
  }
  return value.replace("T", " ").slice(0, 10);
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

async function saveSessionTimes() {
  await store.saveSessionTimes();
}

async function saveCapacity() {
  const examNotices = capacityForm.examNoticesText
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  await store.saveSettings(capacityForm.defaultCapacity, capacityForm.maxCapacity, capacityForm.examTitle, examNotices);
}

async function generateExamPlan() {
  await store.generate();
}

async function exportBundle() {
  await store.exportLatestBundle();
}

onMounted(async () => {
  await store.loadAll();
  capacityForm.defaultCapacity = store.viewState.settings.defaultCapacity;
  capacityForm.maxCapacity = store.viewState.settings.maxCapacity;
  capacityForm.examTitle = store.viewState.settings.examTitle ?? "";
  capacityForm.examNoticesText = (store.viewState.settings.examNotices ?? []).join("\n");
});
</script>

<style scoped>
.dashboard-grid {
  display: grid;
  grid-template-columns: 672px 346px;
  gap: 22px;
}

.left-col,
.right-col {
  display: flex;
  flex-direction: column;
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

.fit {
  width: fit-content;
}

.complete-stack {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.complete-item {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}

.exam-table tbody tr {
  height: 58px;
}

.time-input {
  width: 88px;
  border: 0;
  background: transparent;
}

.time-input:focus {
  outline: none;
}

.icon-btn {
  border: 0;
  background: transparent;
  color: #ff8b8b;
}

.table-actions {
  margin-top: 14px;
}
</style>
