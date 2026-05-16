<template>
  <section class="panel" :class="{ dragging: isDragging }">
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-card">
        <strong>松开鼠标开始导入课表</strong>
        <span>支持 `.xlsx` / `.xls`，每次导入都会保留历史批次</span>
      </div>
    </div>

    <FilterToolbar :items="[]">
      <div class="toolbar-fields">
        <div class="segmented" role="tablist" aria-label="课表查看方式">
          <button
            v-for="option in viewTypeOptions"
            :key="option.value"
            type="button"
            :class="{ active: store.viewState.viewType === option.value }"
            @click="store.setViewType(option.value)"
          >
            <span class="material-symbols-rounded">{{ option.icon }}</span>
            {{ option.label }}
          </button>
        </div>
        <FluentSelect
          :model-value="store.viewState.target"
          :options="targetOptions"
          @update:model-value="store.setTarget($event as string)"
          style="width: 220px;"
        />
      </div>
    </FilterToolbar>

    <div class="import-management">
      <InfoHint
        class="import-status"
        :type="store.viewState.importStatus === 'success' ? 'success' : store.viewState.importStatus === 'error' ? 'error' : store.viewState.importStatus === 'importing' ? 'warning' : 'info'"
        :text="importStatusLabel + '：' + importStatusMessage"
      />
      <div class="import-controls">
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
          <span>生效开始</span>
          <input
            class="glass-input"
            type="date"
            :value="store.viewState.settingsDraft.effectiveStartDate"
            :disabled="!store.viewState.selectedImportId"
            @input="setDraftDate('effectiveStartDate', $event)"
          />
        </label>
        <label class="control-field">
          <span>生效结束</span>
          <input
            class="glass-input"
            type="date"
            :value="store.viewState.settingsDraft.effectiveEndDate"
            :disabled="!store.viewState.selectedImportId"
            @input="setDraftDate('effectiveEndDate', $event)"
          />
        </label>
        <label class="control-field week-field">
          <span>当前从第几周开始</span>
          <input
            class="glass-input"
            type="number"
            min="1"
            step="1"
            :value="store.viewState.settingsDraft.startWeek"
            :disabled="!store.viewState.selectedImportId"
            @input="setDraftWeek"
          />
        </label>
        <button class="action-btn primary" type="button" :disabled="!store.viewState.selectedImportId || isSavingSettings" @click="saveImportSettings">
          <span class="material-symbols-rounded">save</span>
          保存
        </button>
        <button class="action-btn danger" type="button" :disabled="!store.viewState.selectedImportId || isDeletingImport" @click="deleteImportBatch">
          <span class="material-symbols-rounded">delete</span>
          删除
        </button>
      </div>
    </div>

    <TableCard :title="scheduleTitle" :meta="scheduleMeta">
      <div v-if="!store.viewState.schedule || store.viewState.schedule.entries.length === 0" class="empty-state">
        <span class="material-symbols-rounded">event_busy</span>
        <strong>{{ store.viewState.selectedImportId ? "当前条件下暂无课表" : "请先拖拽导入课表 Excel" }}</strong>
      </div>
      <div v-else class="schedule-area">
        <div class="week-switch" role="tablist" aria-label="周次切换">
          <button
            v-for="weekIndex in weekIndexes"
            :key="weekIndex"
            type="button"
            :class="{ active: selectedWeekIndex === weekIndex }"
            @click="selectedWeekIndex = weekIndex"
          >
            第 {{ weekIndex }} 周
          </button>
        </div>
        <div class="schedule-table-scroll">
          <div class="schedule-grid" :style="{ '--period-count': String(periodRows.length) }">
            <div class="corner-cell">节次</div>
            <div v-for="day in days" :key="day.value" class="day-head">{{ day.label }}</div>
            <template v-for="period in periodRows" :key="`p-${selectedWeekIndex}-${period.index}`">
              <div
                v-if="period.isSectionStart"
                class="section-cell"
                :class="sectionToneClass(period.section)"
                :style="{ gridRow: `span ${period.sectionSpan}` }"
              >
                <span>{{ period.section }}</span>
              </div>
              <div class="period-cell" :class="sectionToneClass(period.section)">
                <strong>{{ period.label }}</strong>
              </div>
              <div v-for="day in days" :key="`${selectedWeekIndex}-${period.index}-${day.value}`" class="lesson-cell">
                <div
                  v-for="entry in entriesFor(selectedWeekIndex, day.value, period.index)"
                  :key="`${entry.className}-${entry.subject}-${entry.periodIndex}`"
                  class="lesson"
                  :class="sectionToneClass(period.section)"
                >
                  <strong>{{ entry.subject }}</strong>
                  <span v-if="store.viewState.viewType === 'teacher'">{{ entry.displayClassName }}</span>
                  <span v-else>{{ entry.teacherNames.join(" / ") || "--" }}</span>
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { CoursePeriodSlot, CourseScheduleEntry, CourseViewType } from "../../../entities/course-management/model";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useAppDialog } from "../../../shared/ui/appDialog";
import { useCourseManagementStore } from "../store";

const store = useCourseManagementStore();
const dialog = useAppDialog();
const isDragging = ref(false);
const selectedWeekIndex = ref(1);
const isSavingSettings = ref(false);
const isDeletingImport = ref(false);
let unlistenDragDrop: (() => void) | null = null;

const days = [
  { value: 1, label: "周一" },
  { value: 2, label: "周二" },
  { value: 3, label: "周三" },
  { value: 4, label: "周四" },
  { value: 5, label: "周五" },
  { value: 6, label: "周六" },
  { value: 7, label: "周日" },
];

const viewTypeOptions: Array<{ value: CourseViewType; label: string; icon: string }> = [
  { value: "admin_class", label: "行政班", icon: "domain" },
  { value: "foreign_class", label: "外语教学班", icon: "groups" },
  { value: "teacher", label: "教师", icon: "badge" },
];

const targetOptions = computed(() => {
  if (store.viewState.viewType === "teacher") {
    return store.viewState.teachers.map((teacher) => ({ label: teacher, value: teacher }));
  }
  if (store.viewState.viewType === "foreign_class") {
    return store.viewState.foreignClasses.map((item) => ({ label: item.displayName, value: item.className }));
  }
  return store.viewState.adminClasses.map((item) => ({ label: item.displayName, value: item.className }));
});

const importOptions = computed(() =>
  store.viewState.imports.map((item) => ({
    label: excelFileName(item.sourceFile),
    value: item.id,
  })),
);

const selectedBatch = computed(() => store.viewState.selectedImport);

const scheduleTitle = computed(() => {
  const label = targetOptions.value.find((item) => item.value === store.viewState.target)?.label || store.viewState.target || "课表";
  return label === "课表" ? label : `${label}课表`;
});

const scheduleMeta = computed(() => {
  const count = store.viewState.schedule?.entries.length ?? 0;
  return `双周循环，共 ${count} 条记录`;
});

const weekIndexes = computed(() => {
  const periodWeeks = store.viewState.schedule?.periods ?? [];
  const source = periodWeeks.length > 0 ? periodWeeks : (store.viewState.schedule?.entries ?? []);
  const values = new Set(source.map((entry) => entry.weekIndex));
  return Array.from(values).sort((a, b) => a - b);
});

const currentWeekEntries = computed(() =>
  (store.viewState.schedule?.entries ?? []).filter((entry) => entry.weekIndex === selectedWeekIndex.value),
);

const periodLabels = computed(() => {
  const map = new Map<number, { index: number; label: string; section: string }>();
  for (const period of currentWeekPeriods.value) {
    map.set(period.periodIndex, {
      index: period.periodIndex,
      label: period.periodLabel,
      section: period.sectionLabel,
    });
  }
  if (map.size > 0) {
    return Array.from(map.values()).sort((a, b) => a.index - b.index);
  }
  for (const entry of currentWeekEntries.value) {
    map.set(entry.periodIndex, {
      index: entry.periodIndex,
      label: entry.periodLabel,
      section: entry.sectionLabel,
    });
  }
  return Array.from(map.values()).sort((a, b) => a.index - b.index);
});

const periodRows = computed(() => {
  const rows = periodLabels.value.map((period) => ({
    ...period,
    isSectionStart: false,
    sectionSpan: 1,
  }));
  let index = 0;
  while (index < rows.length) {
    const section = rows[index].section;
    let span = 1;
    while (index + span < rows.length && rows[index + span].section === section) {
      span += 1;
    }
    rows[index].isSectionStart = true;
    rows[index].sectionSpan = span;
    index += span;
  }
  return rows;
});

const currentWeekPeriods = computed<readonly CoursePeriodSlot[]>(() =>
  (store.viewState.schedule?.periods ?? []).filter((period) => period.weekIndex === selectedWeekIndex.value),
);

function entriesFor(weekIndex: number, dayOfWeek: number, periodIndex: number): readonly CourseScheduleEntry[] {
  return currentWeekEntries.value.filter(
    (entry) => entry.weekIndex === weekIndex && entry.dayOfWeek === dayOfWeek && entry.periodIndex === periodIndex,
  );
}

function sectionToneClass(section: string) {
  const normalized = section.replace(/\s+/g, "");
  if (normalized.includes("早")) return "tone-early";
  if (normalized.includes("上午")) return "tone-morning";
  if (normalized.includes("下午")) return "tone-afternoon";
  if (normalized.includes("晚")) return "tone-evening";
  return "tone-default";
}

watch(
  weekIndexes,
  (weeks) => {
    if (weeks.length > 0 && !weeks.includes(selectedWeekIndex.value)) {
      selectedWeekIndex.value = weeks[0];
    }
  },
  { immediate: true },
);

const importStatusLabel = computed(() => {
  if (store.viewState.importStatus === "idle") return "待导入";
  if (store.viewState.importStatus === "importing") return "导入中";
  if (store.viewState.importStatus === "success") return "导入成功";
  return "导入失败";
});

const importStatusMessage = computed(() => {
  if (store.viewState.importStatus === "idle") {
    return "拖拽 Excel 导入，历史保留";
  }
  return store.viewState.importMessage;
});

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function excelFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || formatDate(selectedBatch.value?.importedAt ?? new Date().toISOString());
}

function setDraftDate(field: "effectiveStartDate" | "effectiveEndDate", event: Event) {
  store.setSettingsDraft({ [field]: (event.target as HTMLInputElement).value });
}

function setDraftWeek(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  store.setSettingsDraft({ startWeek: Number.isFinite(value) ? Math.max(1, Math.floor(value)) : 1 });
}

async function saveImportSettings() {
  isSavingSettings.value = true;
  try {
    await store.saveSelectedImportSettings();
    store.setImportFeedback("success", "课表批次设置已保存");
  } catch (error) {
    store.setImportFeedback("error", error instanceof Error ? error.message : String(error));
  } finally {
    isSavingSettings.value = false;
  }
}

async function deleteImportBatch() {
  const batch = selectedBatch.value;
  if (!batch) return;
  const confirmed = await dialog.confirm({
    tone: "danger",
    title: "删除课表批次",
    summary: `确定删除 ${formatDate(batch.importedAt)} 导入的全部课表数据吗？删除后该批次的课表、节次与调代课引用都将不可恢复。`,
    details: [excelFileName(batch.sourceFile), `导入时间：${formatDate(batch.importedAt)}`],
    confirmText: "确认删除",
    cancelText: "取消",
  });
  if (!confirmed) return;
  isDeletingImport.value = true;
  try {
    await store.deleteSelectedImport();
    store.setImportFeedback("success", "已删除该导入批次的课表数据");
  } catch (error) {
    store.setImportFeedback("error", error instanceof Error ? error.message : String(error));
  } finally {
    isDeletingImport.value = false;
  }
}

function normalizeDroppedPath(rawPath: string): string {
  const trimmed = rawPath.trim();
  if (!trimmed.startsWith("file://")) return trimmed;
  try {
    const url = new URL(trimmed);
    return decodeURIComponent(url.pathname)
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  } catch {
    return decodeURIComponent(trimmed.replace(/^file:\/\//i, ""))
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

async function handleImport(filePath: string) {
  try {
    await store.importExcel(filePath);
  } catch {
    // Import status is already visible in the panel.
  }
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
      store.setImportFeedback("error", "已收到拖拽，但未识别到 Excel 文件");
    }
  });
  await store.loadOptions();
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
  gap: 18px;
  min-width: 1160px;
  min-height: calc(100vh - 118px);
  position: relative;
}

.panel :deep(.table-card) {
  min-height: 0;
}

.panel :deep(.table-card .content) {
  min-height: 0;
  overflow: visible;
}

.toolbar-fields {
  display: flex;
  gap: 14px;
  align-items: center;
}

.segmented {
  display: inline-grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--color-border-soft);
  border-radius: 12px;
  background: var(--surface-panel);
}

.segmented button {
  height: 34px;
  min-width: 112px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 13px;
}

.segmented .material-symbols-rounded {
  font-size: 18px;
}

.segmented button.active {
  background: var(--accent-primary);
  color: #fff;
}

.import-management {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  align-items: stretch;
  min-width: 0;
  padding: 14px;
  border: 1px solid var(--border-default);
  border-radius: 18px;
  background: var(--surface-panel-strong);
}

.import-status {
  min-height: 46px;
}

.import-status :deep(p) {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.import-controls {
  display: grid;
  grid-template-columns:
    minmax(220px, 1.4fr)
    minmax(120px, 0.8fr)
    minmax(120px, 0.8fr)
    minmax(120px, 0.8fr)
    minmax(68px, 0.42fr)
    minmax(68px, 0.42fr);
  gap: 10px;
  align-items: end;
  min-width: 0;
  overflow: hidden;
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

.glass-input:focus {
  border-color: var(--accent-border-strong);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
  background: var(--surface-input-strong);
}

.glass-input:disabled {
  opacity: 0.58;
  cursor: not-allowed;
}

.action-btn {
  height: 42px;
  width: 100%;
  min-width: 0;
  padding: 0 8px;
  border: 0;
  border-radius: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
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

.action-btn.danger {
  background: #d14343;
}

.action-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}


.schedule-area {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
  padding: 4px;
}

.week-switch {
  display: inline-flex;
  align-items: center;
  width: fit-content;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--color-border-soft);
  border-radius: 12px;
  background: var(--surface-panel);
}

.week-switch button {
  height: 32px;
  min-width: 86px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 700;
}

.week-switch button.active {
  background: var(--accent-primary);
  color: #fff;
}

.schedule-table-scroll {
  min-height: 0;
  max-height: calc(100vh - 360px);
  overflow: auto;
  border: 1px solid var(--border-default);
  border-radius: 12px;
  background: var(--surface-table-content);
  padding-bottom: 10px;
  scrollbar-gutter: stable;
}

.schedule-grid {
  display: grid;
  grid-template-columns: 34px 72px repeat(7, minmax(120px, 1fr));
  grid-auto-rows: minmax(78px, auto);
  min-width: 1040px;
  background: var(--surface-table-content);
}

.corner-cell,
.day-head,
.section-cell,
.period-cell,
.lesson-cell {
  border-right: 1px solid var(--border-default);
  border-bottom: 1px solid var(--border-default);
}

.corner-cell {
  grid-column: span 2;
}

.corner-cell,
.day-head {
  min-height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  color: var(--text-secondary);
  background: rgba(var(--accent-rgb), 0.08);
}

.section-cell {
  grid-column: 1;
  padding: 8px 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--period-section-bg, var(--surface-table-stripe));
}

.section-cell span {
  color: var(--color-text);
  font-size: 15px;
  font-weight: 700;
  writing-mode: vertical-rl;
  text-orientation: upright;
  letter-spacing: 0.16em;
}

.period-cell {
  grid-column: 2;
  padding: 10px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  background: var(--period-cell-bg, var(--surface-table-stripe));
}

.period-cell strong {
  color: var(--color-text);
  font-size: 14px;
}

.lesson-cell {
  min-height: 78px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.lesson {
  min-height: 54px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--period-lesson-bg, rgba(var(--accent-rgb), 0.08));
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  border-left: 3px solid var(--period-accent, var(--accent-primary));
}

.tone-early {
  --period-section-bg: rgba(35, 118, 196, 0.14);
  --period-cell-bg: rgba(35, 118, 196, 0.08);
  --period-lesson-bg: rgba(35, 118, 196, 0.10);
  --period-accent: #2376c4;
}

.tone-morning {
  --period-section-bg: rgba(24, 141, 102, 0.14);
  --period-cell-bg: rgba(24, 141, 102, 0.08);
  --period-lesson-bg: rgba(24, 141, 102, 0.10);
  --period-accent: #188d66;
}

.tone-afternoon {
  --period-section-bg: rgba(191, 119, 31, 0.16);
  --period-cell-bg: rgba(191, 119, 31, 0.09);
  --period-lesson-bg: rgba(191, 119, 31, 0.11);
  --period-accent: #bf771f;
}

.tone-evening {
  --period-section-bg: rgba(108, 91, 181, 0.16);
  --period-cell-bg: rgba(108, 91, 181, 0.09);
  --period-lesson-bg: rgba(108, 91, 181, 0.11);
  --period-accent: #6c5bb5;
}

.lesson strong {
  font-size: 15px;
  color: var(--color-text);
}

.lesson span {
  font-size: 12px;
  color: var(--color-text-muted);
  word-break: break-word;
}

.empty-state {
  min-height: 320px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
}

.empty-state .material-symbols-rounded {
  font-size: 42px;
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
  min-width: 320px;
  padding: 20px 24px;
  border-radius: 18px;
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
</style>
