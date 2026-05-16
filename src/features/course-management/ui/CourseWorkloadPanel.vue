<template>
  <section class="panel">
    <TableCard title="课时统计" :meta="summaryMeta">
      <div class="workload">
        <div class="query-grid">
          <label class="control-field">
            <span>课表批次</span>
            <FluentSelect
              :model-value="store.viewState.selectedImportId ?? ''"
              :options="importOptions"
              placeholder="未导入"
              @update:model-value="store.setSelectedImport(Number($event))"
            />
          </label>
          <label class="control-field">
            <span>开始日期</span>
            <input v-model="startDate" class="glass-input" type="date" />
          </label>
          <label class="control-field">
            <span>开始节次</span>
            <FluentSelect v-model="startPeriodIndex" :options="periodOptions" />
          </label>
          <label class="control-field">
            <span>结束日期</span>
            <input v-model="endDate" class="glass-input" type="date" />
          </label>
          <label class="control-field">
            <span>结束节次</span>
            <FluentSelect v-model="endPeriodIndex" :options="periodOptions" />
          </label>
          <button class="action-btn primary" type="button" :disabled="isLoading" @click="loadReport">
            <span class="material-symbols-rounded">query_stats</span>
            查看统计
          </button>
          <button class="action-btn secondary" type="button" :disabled="store.viewState.exportingWorkload" @click="exportReport">
            <span class="material-symbols-rounded">download</span>
            导出 Excel
          </button>
        </div>

        <InfoHint :type="feedbackType" :text="feedbackMessage" />

        <div class="stats-strip">
          <div class="stat-cell">
            <span>教师数</span>
            <strong>{{ store.viewState.workloadReport?.summaries.length ?? 0 }}</strong>
          </div>
          <div class="stat-cell">
            <span>总课时</span>
            <strong>{{ totalLessons }}</strong>
          </div>
          <div class="stat-cell">
            <span>代课节数</span>
            <strong>{{ substitutionLessons }}</strong>
          </div>
          <button v-if="store.viewState.lastWorkloadExport" class="export-link" type="button" @click="openLastExport">
            {{ exportFileName(store.viewState.lastWorkloadExport.filePath) }}
          </button>
        </div>

        <div class="tables">
          <div class="summary-table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>教师</th>
                  <th>早上</th>
                  <th>上午</th>
                  <th>下午</th>
                  <th>晚上</th>
                  <th>代课</th>
                  <th>合计</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="summaries.length === 0">
                  <td colspan="7" class="empty-cell">暂无统计结果</td>
                </tr>
                <tr v-for="row in summaries" :key="row.teacherName" :class="{ active: row.teacherName === selectedTeacher }" @click="selectedTeacher = row.teacherName">
                  <td>{{ row.teacherName }}</td>
                  <td>{{ row.morningReadingCount }}</td>
                  <td>{{ row.morningCount }}</td>
                  <td>{{ row.afternoonCount }}</td>
                  <td>{{ row.eveningCount }}</td>
                  <td>{{ row.substitutionCount }}</td>
                  <td>{{ row.totalCount }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="detail-table-wrap">
            <table class="data-table detail-table">
              <thead>
                <tr>
                  <th>教师</th>
                  <th>日期</th>
                  <th>节次</th>
                  <th>类别</th>
                  <th>班级</th>
                  <th>科目</th>
                  <th>备注</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="filteredDetails.length === 0">
                  <td colspan="7" class="empty-cell">选择左侧教师查看课时明细</td>
                </tr>
                <tr v-for="detail in filteredDetails" :key="`${detail.teacherName}-${detail.targetDate}-${detail.periodIndex}-${detail.className}-${detail.subject}`">
                  <td>{{ detail.teacherName }}</td>
                  <td>{{ formatDate(detail.targetDate) }}</td>
                  <td>{{ detail.periodLabel }}</td>
                  <td>{{ detail.category }}</td>
                  <td>{{ detail.displayClassName }}</td>
                  <td>{{ detail.subject }}</td>
                  <td>{{ detail.remark || "--" }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { revealInExplorer } from "../../../shared/utils/appLog";
import { useCourseManagementStore } from "../store";

const store = useCourseManagementStore();
const today = new Date().toISOString().slice(0, 10);
const startDate = ref(today);
const endDate = ref(today);
const startPeriodIndex = ref(1);
const endPeriodIndex = ref(12);
const selectedTeacher = ref("");
const isLoading = ref(false);
const feedbackType = ref<"info" | "success" | "warning" | "error">("info");
const feedbackMessage = ref("选择真实日期和节次范围后查看课时统计；导出文件包含明细和分类汇总两个 Sheet。");

const importOptions = computed(() =>
  store.viewState.imports.map((item) => ({
    label: excelFileName(item.sourceFile),
    value: item.id,
  })),
);

const summaries = computed(() => store.viewState.workloadReport?.summaries ?? []);
const details = computed(() => store.viewState.workloadReport?.details ?? []);
const periodOptions = computed(() => {
  const periods = store.viewState.periods;
  return periods.map((period) => ({
    label: period.sectionLabel ? `${period.sectionLabel} ${period.periodLabel}` : period.periodLabel,
    value: period.periodIndex,
  }));
});
const totalLessons = computed(() => summaries.value.reduce((sum, row) => sum + row.totalCount, 0));
const substitutionLessons = computed(() => summaries.value.reduce((sum, row) => sum + row.substitutionCount, 0));
const summaryMeta = computed(() => `${summaries.value.length} 位教师，${totalLessons.value} 节课`);
const filteredDetails = computed(() => {
  if (!selectedTeacher.value) return details.value;
  return details.value.filter((item) => item.teacherName === selectedTeacher.value);
});

watch(summaries, (rows) => {
  if (rows.length > 0 && !rows.some((row) => row.teacherName === selectedTeacher.value)) {
    selectedTeacher.value = rows[0].teacherName;
  }
});

function buildQuery() {
  return {
    startDate: startDate.value,
    endDate: endDate.value,
    startPeriodIndex: Math.max(1, Number(startPeriodIndex.value) || 1),
    endPeriodIndex: Math.max(1, Number(endPeriodIndex.value) || 99),
  };
}

function excelFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || path;
}

function exportFileName(path: string) {
  return excelFileName(path);
}

function formatDate(value: string) {
  return new Date(`${value}T00:00:00`).toLocaleDateString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    weekday: "short",
  });
}

async function loadReport() {
  isLoading.value = true;
  try {
    const report = await store.loadWorkloadReport(buildQuery());
    const count = report?.details.length ?? 0;
    feedbackType.value = count > 0 ? "success" : "warning";
    feedbackMessage.value = count > 0 ? `已生成 ${count} 条课时明细。` : "该范围内暂无课时数据。";
  } catch (error) {
    feedbackType.value = "error";
    feedbackMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

async function exportReport() {
  try {
    const result = await store.exportWorkloadReport(buildQuery());
    if (!result) return;
    feedbackType.value = "success";
    feedbackMessage.value = `已导出：${exportFileName(result.filePath)}`;
  } catch (error) {
    feedbackType.value = "error";
    feedbackMessage.value = error instanceof Error ? error.message : String(error);
  }
}

async function openLastExport() {
  const target = store.viewState.lastWorkloadExport?.filePath;
  if (!target) return;
  await revealInExplorer(target);
}

onMounted(async () => {
  await store.loadOptions();
});
</script>

<style scoped>
.panel {
  min-width: 1160px;
  min-height: 0;
  height: calc(100vh - 118px);
}

.panel :deep(.table-card) {
  height: 100%;
  min-height: 0;
}

.panel :deep(.table-card .content) {
  min-height: 0;
  overflow: hidden;
}

.workload {
  display: flex;
  flex-direction: column;
  gap: 14px;
  height: 100%;
  min-height: 0;
  padding: 14px;
}

.query-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1.3fr) repeat(4, minmax(116px, 0.7fr)) minmax(112px, 0.5fr) minmax(118px, 0.5fr);
  gap: 10px;
  align-items: end;
}

.control-field {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.control-field > span {
  color: var(--color-text-muted);
  font-size: 12px;
}

.glass-input {
  width: 100%;
  height: 42px;
  box-sizing: border-box;
  border: 1px solid var(--color-border-soft);
  border-radius: 14px;
  background: var(--surface-panel);
  color: var(--color-text);
  padding: 0 12px;
  font-size: 13px;
  outline: none;
}

.action-btn {
  height: 42px;
  min-width: 0;
  padding: 0 12px;
  border: 0;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: #fff;
  font-weight: 700;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.action-btn.primary {
  background: var(--accent-primary);
}

.action-btn.secondary {
  background: #3a6ea5;
}

.action-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.action-btn .material-symbols-rounded {
  font-size: 18px;
}

.stats-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(120px, 150px)) minmax(220px, 1fr);
  gap: 10px;
  align-items: stretch;
}

.stat-cell {
  padding: 10px 12px;
  border: 1px solid var(--border-default);
  border-radius: 12px;
  background: var(--surface-panel);
}

.stat-cell span {
  display: block;
  color: var(--color-text-muted);
  font-size: 12px;
}

.stat-cell strong {
  display: block;
  margin-top: 4px;
  color: var(--color-text);
  font-size: 20px;
}

.export-link {
  justify-self: start;
  align-self: center;
  border: 0;
  background: transparent;
  color: var(--accent-primary);
  font-weight: 700;
  cursor: pointer;
}

.tables {
  display: grid;
  grid-template-columns: minmax(420px, 0.7fr) minmax(560px, 1fr);
  gap: 12px;
  min-height: 0;
  flex: 1;
}

.summary-table-wrap,
.detail-table-wrap {
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--border-default);
  border-radius: 12px;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
}

.detail-table {
  min-width: 780px;
}

.data-table th,
.data-table td {
  border-bottom: 1px solid var(--border-default);
  padding: 10px 12px;
  text-align: left;
  font-size: 13px;
  vertical-align: middle;
}

.data-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface-table-stripe);
  color: var(--text-secondary);
  font-weight: 700;
}

.data-table tbody tr {
  cursor: pointer;
}

.data-table tbody tr.active {
  background: rgba(var(--accent-rgb), 0.08);
}

.empty-cell {
  height: 160px;
  text-align: center !important;
  color: var(--color-text-muted);
  cursor: default;
}
</style>
