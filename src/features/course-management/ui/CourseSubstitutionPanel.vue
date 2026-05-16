<template>
  <section class="panel">
    <div class="workspace">
      <TableCard title="新建调代课" :meta="candidateMeta">
        <div class="substitution-form">
          <div class="form-grid">
            <label class="control-field batch-field">
              <span>课表批次</span>
              <FluentSelect
                :model-value="store.viewState.selectedImportId ?? ''"
                :options="importOptions"
                placeholder="未导入"
                @update:model-value="store.setSelectedImport(Number($event))"
              />
            </label>
            <label class="control-field">
              <span>换课教师</span>
              <FluentSelect
                v-model="queryTeacher"
                :options="teacherOptions"
                placeholder="选择教师"
                searchable
              />
            </label>
            <label class="control-field">
              <span>开始日期</span>
              <input v-model="startDate" class="glass-input" type="date" />
            </label>
            <label class="control-field">
              <span>结束日期</span>
              <input v-model="endDate" class="glass-input" type="date" />
            </label>
            <label class="control-field">
              <span>原因</span>
              <FluentSelect v-model="reason" :options="reasonOptions" />
            </label>
            <button class="action-btn primary" type="button" :disabled="isSearching" @click="searchCandidates">
              <span class="material-symbols-rounded">search</span>
              查询课次
            </button>
          </div>

          <InfoHint
            :type="feedbackType"
            :text="feedbackMessage"
          />

          <div class="period-picker">
            <div class="period-picker-head">
              <span>涉及节次</span>
              <div class="period-actions">
                <button type="button" :disabled="periodOptions.length === 0" @click="selectAllPeriods">全部</button>
                <button type="button" :disabled="periodOptions.length === 0" @click="selectPeriodGroup('early')">早上</button>
                <button type="button" :disabled="periodOptions.length === 0" @click="selectPeriodGroup('morning')">上午</button>
                <button type="button" :disabled="periodOptions.length === 0" @click="selectPeriodGroup('afternoon')">下午</button>
                <button type="button" :disabled="periodOptions.length === 0" @click="selectPeriodGroup('evening')">晚上</button>
                <button type="button" :disabled="periodOptions.length === 0" @click="clearPeriods">清空</button>
              </div>
            </div>
            <div v-if="periodOptions.length === 0" class="period-empty">
              请选择已导入并设置节次的课表批次
            </div>
            <div v-else class="period-buttons">
              <button
                v-for="period in periodOptions"
                :key="period.value"
                type="button"
                :class="{ active: selectedPeriodIndexes.has(period.value) }"
                @click="togglePeriod(period.value)"
              >
                {{ period.label }}
              </button>
            </div>
          </div>

          <div class="bulk-row">
            <FluentSelect
              v-model="bulkTeacher"
              :options="substituteTeacherOptions"
              placeholder="批量指定代课教师"
              searchable
            />
            <button class="action-btn secondary" type="button" :disabled="!bulkTeacher || selectedKeys.size === 0" @click="applyBulkTeacher">
              <span class="material-symbols-rounded">group_add</span>
              批量指定
            </button>
            <button class="action-btn primary" type="button" :disabled="isSaving || saveableCount === 0" @click="saveSelected">
              <span class="material-symbols-rounded">save</span>
              保存生效
            </button>
          </div>

          <div class="candidate-table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th class="check-col">
                    <input type="checkbox" :checked="allSelected" @change="toggleAll" />
                  </th>
                  <th>日期</th>
                  <th>节次</th>
                  <th>班级</th>
                  <th>科目</th>
                  <th>原任课</th>
                  <th>代课教师</th>
                  <th>状态</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="store.viewState.substitutionCandidates.length === 0">
                  <td colspan="8" class="empty-cell">按教师和日期范围查询需要处理的课次</td>
                </tr>
                <tr v-for="item in store.viewState.substitutionCandidates" :key="candidateKey(item)">
                  <td class="check-col">
                    <input type="checkbox" :checked="selectedKeys.has(candidateKey(item))" @change="toggleCandidate(item)" />
                  </td>
                  <td>{{ formatDate(item.targetDate) }}</td>
                  <td>
                    <strong>{{ item.periodLabel }}</strong>
                    <span>{{ item.sectionLabel }}</span>
                  </td>
                  <td>{{ item.displayClassName }}</td>
                  <td>{{ item.subject }}</td>
                  <td>{{ item.sourceTeacherName }}</td>
                  <td>
                    <FluentSelect
                      :model-value="draftTeachers[candidateKey(item)] ?? item.existingChange?.actualTeacherName ?? ''"
                      :options="substituteOptionsFor(item.sourceTeacherName)"
                      placeholder="选择代课教师"
                      searchable
                      @update:model-value="setDraftTeacher(item, $event as string)"
                    />
                  </td>
                  <td>
                    <span class="status-pill" :class="{ active: item.existingChange }">
                      {{ item.existingChange ? `已由 ${item.existingChange.actualTeacherName} 代课` : "待安排" }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </TableCard>

      <TableCard title="已生效记录" :meta="changeMeta">
        <div class="change-list">
          <div v-if="store.viewState.scheduleChanges.length === 0" class="empty-state">
            <span class="material-symbols-rounded">event_available</span>
            <strong>暂无调代课记录</strong>
          </div>
          <div v-for="change in store.viewState.scheduleChanges" :key="change.id" class="change-row" :class="{ revoked: change.status === 'revoked' }">
            <div class="change-main">
              <strong>{{ formatDate(change.targetDate) }} {{ change.periodLabel }} {{ change.displayClassName }} {{ change.subject }}</strong>
              <span>原任课：{{ change.sourceTeacherName }} · 代课：{{ change.actualTeacherName }}</span>
              <small>{{ change.reason || "未填写原因" }}{{ change.remark ? ` / ${change.remark}` : "" }}</small>
            </div>
            <div class="change-actions">
              <span class="status-pill" :class="{ active: change.status === 'active' }">{{ change.status === "active" ? "已生效" : "已撤销" }}</span>
              <button class="icon-btn danger" type="button" :disabled="change.status !== 'active'" @click="revokeChange(change.id)">
                <span class="material-symbols-rounded">undo</span>
              </button>
            </div>
          </div>
        </div>
      </TableCard>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import type { CourseSubstitutionCandidate } from "../../../entities/course-management/model";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useAppDialog } from "../../../shared/ui/appDialog";
import { useCourseManagementStore } from "../store";

const store = useCourseManagementStore();
const dialog = useAppDialog();
const today = new Date().toISOString().slice(0, 10);

const queryTeacher = ref("");
const startDate = ref(today);
const endDate = ref(today);
const reason = ref("请假");
const bulkTeacher = ref("");
const selectedKeys = ref(new Set<string>());
const selectedPeriodIndexes = ref(new Set<number>());
const draftTeachers = ref<Record<string, string>>({});
const isSearching = ref(false);
const isSaving = ref(false);
const feedbackType = ref<"info" | "success" | "warning" | "error">("info");
const feedbackMessage = ref("选择教师和日期范围后，查询该教师涉及的课次并逐节指定代课教师。");

const reasonOptions = [
  { label: "请假", value: "请假" },
  { label: "公出", value: "公出" },
  { label: "培训", value: "培训" },
  { label: "临时换课", value: "临时换课" },
  { label: "其他", value: "其他" },
];

const importOptions = computed(() =>
  store.viewState.imports.map((item) => ({
    label: excelFileName(item.sourceFile),
    value: item.id,
  })),
);

const teacherOptions = computed(() =>
  store.viewState.teachers.map((teacher) => ({ label: teacher, value: teacher })),
);

const substituteTeacherOptions = computed(() =>
  store.viewState.teachers.map((teacher) => ({ label: teacher, value: teacher })),
);

const periodOptions = computed(() => {
  const periods = store.viewState.periods;
  return periods.map((period) => ({
    label: period.sectionLabel ? `${period.sectionLabel} ${period.periodLabel}` : period.periodLabel,
    value: period.periodIndex,
    sectionLabel: period.sectionLabel,
    periodLabel: period.periodLabel,
  }));
});

const candidateMeta = computed(() => `已查询 ${store.viewState.substitutionCandidates.length} 节课`);
const changeMeta = computed(() => `${activeChanges.value.length} 条生效，${revokedChanges.value.length} 条撤销`);
const activeChanges = computed(() => store.viewState.scheduleChanges.filter((item) => item.status === "active"));
const revokedChanges = computed(() => store.viewState.scheduleChanges.filter((item) => item.status === "revoked"));

const allSelected = computed(() => {
  const candidates = store.viewState.substitutionCandidates;
  return candidates.length > 0 && candidates.every((item) => selectedKeys.value.has(candidateKey(item)));
});

const saveableCount = computed(() =>
  store.viewState.substitutionCandidates.filter((item) => {
    const key = candidateKey(item);
    const teacher = draftTeachers.value[key] ?? item.existingChange?.actualTeacherName ?? "";
    return selectedKeys.value.has(key) && teacher && teacher !== item.sourceTeacherName;
  }).length,
);

function candidateKey(item: CourseSubstitutionCandidate) {
  return `${item.targetDate}:${item.sourceEntryId}:${item.sourceTeacherName}`;
}

function excelFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || path;
}

function formatDate(value: string) {
  return new Date(`${value}T00:00:00`).toLocaleDateString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    weekday: "short",
  });
}

function substituteOptionsFor(sourceTeacher: string) {
  return substituteTeacherOptions.value.filter((item) => item.value !== sourceTeacher);
}

function togglePeriod(periodIndex: number) {
  const next = new Set(selectedPeriodIndexes.value);
  if (next.has(periodIndex)) {
    next.delete(periodIndex);
  } else {
    next.add(periodIndex);
  }
  selectedPeriodIndexes.value = next;
}

function selectAllPeriods() {
  selectedPeriodIndexes.value = new Set(periodOptions.value.map((item) => item.value));
}

function clearPeriods() {
  selectedPeriodIndexes.value = new Set();
}

function selectPeriodGroup(group: "early" | "morning" | "afternoon" | "evening") {
  const values = periodOptions.value
    .filter((item) => periodMatchesGroup(item, group))
    .map((item) => item.value);
  selectedPeriodIndexes.value = new Set(values);
}

function periodMatchesGroup(
  period: { value: number; label: string; sectionLabel: string; periodLabel: string },
  group: "early" | "morning" | "afternoon" | "evening",
) {
  const section = normalizePeriodText(period.sectionLabel);
  const labelText = normalizePeriodText(`${period.periodLabel}${period.label}`);
  if (group === "afternoon" && labelText.includes("午练")) {
    return true;
  }
  if (section) {
    if (group === "early") return section.includes("早");
    if (group === "morning") return section.includes("上午");
    if (group === "afternoon") return section.includes("下午");
    return section.includes("晚");
  }
  const text = labelText;
  if (group === "evening") {
    return text.includes("晚");
  }
  if (group === "afternoon") {
    return text.includes("下午") || text.includes("午练") || text.includes("午间");
  }
  if (group === "early") {
    return text.includes("早上") || text.includes("晨读") || text.includes("早读");
  }
  return text.includes("上午") || text.includes("大课间");
}

function normalizePeriodText(value: string) {
  return value.replace(/\s+/g, "");
}

function setDraftTeacher(item: CourseSubstitutionCandidate, teacher: string) {
  draftTeachers.value = {
    ...draftTeachers.value,
    [candidateKey(item)]: teacher,
  };
  if (teacher) {
    const next = new Set(selectedKeys.value);
    next.add(candidateKey(item));
    selectedKeys.value = next;
  }
}

function toggleCandidate(item: CourseSubstitutionCandidate) {
  const key = candidateKey(item);
  const next = new Set(selectedKeys.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  selectedKeys.value = next;
}

function toggleAll() {
  if (allSelected.value) {
    selectedKeys.value = new Set();
    return;
  }
  selectedKeys.value = new Set(store.viewState.substitutionCandidates.map(candidateKey));
}

function applyBulkTeacher() {
  if (!bulkTeacher.value) return;
  const next = { ...draftTeachers.value };
  for (const item of store.viewState.substitutionCandidates) {
    const key = candidateKey(item);
    if (selectedKeys.value.has(key) && item.sourceTeacherName !== bulkTeacher.value) {
      next[key] = bulkTeacher.value;
    }
  }
  draftTeachers.value = next;
}

async function searchCandidates() {
  if (!queryTeacher.value || !startDate.value || !endDate.value) {
    feedbackType.value = "error";
    feedbackMessage.value = "请选择换课教师和日期范围。";
    return;
  }
  if (selectedPeriodIndexes.value.size === 0) {
    feedbackType.value = "error";
    feedbackMessage.value = "请至少选择一个涉及节次。";
    return;
  }
  isSearching.value = true;
  try {
    const candidates = await store.findSubstitutionCandidates({
      teacherName: queryTeacher.value,
      startDate: startDate.value,
      endDate: endDate.value,
      periodIndexes: Array.from(selectedPeriodIndexes.value).sort((a, b) => a - b),
    });
    selectedKeys.value = new Set();
    const drafts: Record<string, string> = {};
    for (const item of candidates) {
      if (item.existingChange) {
        drafts[candidateKey(item)] = item.existingChange.actualTeacherName;
      }
    }
    draftTeachers.value = drafts;
    feedbackType.value = candidates.length > 0 ? "success" : "warning";
    feedbackMessage.value = candidates.length > 0 ? `找到 ${candidates.length} 节相关课程。` : "该时间范围内没有找到该教师的课程。";
  } catch (error) {
    feedbackType.value = "error";
    feedbackMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSearching.value = false;
  }
}

async function saveSelected() {
  const items = store.viewState.substitutionCandidates
    .filter((item) => selectedKeys.value.has(candidateKey(item)))
    .map((item) => ({
      sourceEntryId: item.sourceEntryId,
      targetDate: item.targetDate,
      sourceTeacherName: item.sourceTeacherName,
      actualTeacherName: draftTeachers.value[candidateKey(item)] ?? item.existingChange?.actualTeacherName ?? "",
    }))
    .filter((item) => item.actualTeacherName && item.actualTeacherName !== item.sourceTeacherName);
  if (items.length === 0) {
    feedbackType.value = "error";
    feedbackMessage.value = "请至少为一节课指定有效的代课教师。";
    return;
  }
  isSaving.value = true;
  try {
    await store.saveSubstitutions({
      reason: reason.value,
      remark: "",
      items,
    });
    feedbackType.value = "success";
    feedbackMessage.value = `已保存 ${items.length} 条调代课记录。`;
    await searchCandidates();
  } catch (error) {
    feedbackType.value = "error";
    feedbackMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSaving.value = false;
  }
}

async function revokeChange(changeId: number) {
  const confirmed = await dialog.confirm({
    tone: "warning",
    icon: "undo",
    title: "撤销调代课记录",
    summary: "确定撤销这条调代课记录吗？撤销后该记录会保留在历史列表中，但不再参与课表执行。",
    confirmText: "确认撤销",
    cancelText: "取消",
  });
  if (!confirmed) return;
  try {
    await store.revokeScheduleChange(changeId);
    feedbackType.value = "success";
    feedbackMessage.value = "调代课记录已撤销。";
  } catch (error) {
    feedbackType.value = "error";
    feedbackMessage.value = error instanceof Error ? error.message : String(error);
  }
}

onMounted(async () => {
  await store.loadOptions();
  if (selectedPeriodIndexes.value.size === 0 && periodOptions.value.length > 0) {
    selectAllPeriods();
  }
});

watch(
  () => store.viewState.selectedImportId,
  () => {
    selectedPeriodIndexes.value = new Set();
  },
);

watch(
  () => periodOptions.value.map((item) => item.value).join(","),
  () => {
    if (selectedPeriodIndexes.value.size === 0 && periodOptions.value.length > 0) {
      selectAllPeriods();
    }
  },
);
</script>

<style scoped>
.panel {
  min-width: 1160px;
  min-height: calc(100vh - 118px);
}

.workspace {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 0;
}

.workspace :deep(.table-card) {
  min-height: 0;
}

.workspace :deep(.table-card .content) {
  min-height: 0;
  overflow: visible;
}

.substitution-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  height: 100%;
  padding: 14px;
}

.form-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1.4fr) minmax(180px, 1fr) repeat(2, minmax(130px, 0.8fr)) minmax(130px, 0.8fr) minmax(110px, 0.5fr);
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

.bulk-row {
  display: grid;
  grid-template-columns: minmax(220px, 280px) 120px 120px 1fr;
  gap: 10px;
  align-items: center;
}

.period-picker {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-default);
  border-radius: 12px;
  background: var(--surface-panel);
}

.period-picker-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.period-picker-head > span {
  color: var(--color-text-muted);
  font-size: 12px;
}

.period-actions {
  display: inline-flex;
  gap: 6px;
  flex-wrap: wrap;
}

.period-actions button,
.period-buttons button {
  height: 30px;
  border: 1px solid var(--color-border-soft);
  border-radius: 9px;
  background: var(--surface-table-content);
  color: var(--text-secondary);
  padding: 0 10px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
}

.period-actions button:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

.period-empty {
  min-height: 36px;
  display: flex;
  align-items: center;
  color: var(--color-text-muted);
  font-size: 13px;
}

.period-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  max-height: 112px;
  overflow-y: auto;
  padding-right: 4px;
  scrollbar-gutter: stable;
}

.period-buttons button.active {
  border-color: var(--accent-primary);
  background: var(--accent-primary);
  color: #fff;
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

.action-btn .material-symbols-rounded {
  font-size: 18px;
}

.action-btn.primary {
  background: var(--accent-primary);
}

.action-btn.secondary {
  background: #3a6ea5;
}

.action-btn:disabled,
.icon-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.candidate-table-wrap {
  min-height: 0;
  max-height: 360px;
  overflow: auto;
  border: 1px solid var(--border-default);
  border-radius: 12px;
}

.data-table {
  width: 100%;
  min-width: 1040px;
  border-collapse: collapse;
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

.data-table td > span,
.change-main span,
.change-main small {
  display: block;
  color: var(--color-text-muted);
  font-size: 12px;
  margin-top: 3px;
}

.check-col {
  width: 42px;
  text-align: center;
}

.empty-cell {
  height: 120px;
  text-align: center !important;
  color: var(--color-text-muted);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  padding: 0 9px;
  border-radius: 999px;
  background: var(--surface-table-stripe);
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
}

.status-pill.active {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.change-list {
  max-height: 360px;
  min-height: 0;
  overflow: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.change-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 12px 14px;
  border: 1px solid var(--border-default);
  border-radius: 12px;
  background: var(--surface-panel);
}

.change-row.revoked {
  opacity: 0.62;
}

.change-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.change-main strong {
  color: var(--color-text);
  font-size: 14px;
}

.change-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.icon-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 10px;
  color: #fff;
  background: var(--accent-primary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.icon-btn.danger {
  background: #d14343;
}

.icon-btn .material-symbols-rounded {
  font-size: 18px;
}

.empty-state {
  min-height: 160px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
}

.empty-state .material-symbols-rounded {
  font-size: 38px;
}
</style>
