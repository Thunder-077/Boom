<template>
  <section class="panel">
    <div class="search-card card-shell">
      <div class="search-block">
        <span class="picker-label">查询班级</span>
        <div class="class-input-wrap">
          <label class="class-input-shell" @keydown.enter.prevent="onSearchCommit">
            <input
              v-model="searchKeyword"
              class="search-input"
              placeholder="请输入班级名称"
              @input="onSearchInput"
              @focus="onSearchFocus"
              @click="onSearchFocus"
              @blur="onSearchBlur"
            />
            <button
              v-if="searchKeyword"
              type="button"
              class="clear-btn"
              @mousedown.prevent
              @click="clearSearchKeyword"
            >
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
            </button>
          </label>
          <div v-if="showSuggestionList" ref="suggestionListRef" class="suggestion-list" @scroll.passive="onSuggestionScroll">
            <button
              v-for="row in suggestionRows"
              :key="row.id"
              type="button"
              class="suggestion-item"
              @mousedown.prevent="onSuggestionMouseDown"
              @click="selectSuggestion(row.id)"
            >
              <span>{{ row.className }}</span>
              <span class="suggestion-meta">{{ row.gradeName || "未设置年级" }}</span>
            </button>
            <div v-if="hasMoreSuggestions" class="suggestion-load-more">
              向下滚动加载更多班级...
            </div>
          </div>
        </div>
      </div>
    </div>

    <section class="context-card card-shell" aria-label="当前配置上下文">
      <div class="context-copy">
        <span class="section-kicker">当前上下文</span>
        <strong>{{ currentModeLabel }}</strong>
        <p>{{ stateDescription }}</p>
      </div>
      <div class="context-badges">
        <div class="context-badge">
          <span>当前模式</span>
          <strong>{{ currentModeLabel }}</strong>
        </div>
        <div class="context-badge">
          <span>当前类型</span>
          <strong>{{ currentTypeLabel }}</strong>
        </div>
      </div>
    </section>

    <ConfigCard class="current-card" title="当前配置班级">
      <div class="current-head">
        <div class="current-inputs-wrap">
          <div class="current-input-wrap">
            <p class="picker-label">教室名</p>
            <label class="class-input-shell current-input-shell">
              <input
                class="current-input"
                :value="store.viewState.form.className"
                placeholder="请输入班级名称"
                @input="onCurrentClassInput"
              />
            </label>
            <p class="status-copy" :class="{ 'status-copy-new': store.viewState.mode === 'new' }">
              {{ stateDescription }}
            </p>
          </div>
          <div class="current-input-wrap room-label-wrap">
            <p class="picker-label">教室标签</p>
            <label class="class-input-shell current-input-shell room-label-input-shell">
              <input
                class="room-label-input"
                :value="store.viewState.form.roomLabel || ''"
                placeholder="请输入教室标签"
                @input="onRoomLabelInput"
              />
            </label>
          </div>
        </div>

        <button
          class="secondary-btn delete-btn"
          type="button"
          :disabled="store.viewState.mode !== 'existing' || store.viewState.saving || store.viewState.deleting"
          @click="deleteCurrent"
        >
          <span class="material-symbols-rounded" aria-hidden="true">delete</span>
          {{ store.viewState.deleting ? "删除中..." : "删除" }}
        </button>
      </div>
    </ConfigCard>

    <ConfigCard title="教室类型" description="选择当前维护的是教学教室还是考试教室，系统会按类型校验科目配置。">
      <div class="type-toggle-row">
        <button
          v-for="option in configTypeOptions"
          :key="option.value"
          type="button"
          class="type-toggle-btn"
          :class="{ active: store.viewState.filters.configType === option.value }"
          @click="switchConfigType(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </ConfigCard>

    <ConfigCard
      v-if="store.viewState.form.configType === 'teaching_class'"
      title="所学科目配置"
      :description="subjectDescription"
    >
      <div class="subject-row">
        <button
          v-for="subject in visibleSubjects"
          :key="subject.value"
          type="button"
          class="subject-pill"
          :class="{ active: store.viewState.form.subjects.includes(subject.value) }"
          @click="toggleSubject(subject.value)"
        >
          {{ subject.label }}
        </button>
      </div>
    </ConfigCard>

    <ConfigCard title="楼号与楼层信息" description="维护班级所在教学楼与楼层，保存后可用于后续考试与排班配置。">
      <div class="editor-grid">
        <label class="metric-field">
          <span class="metric-label">班级楼号</span>
          <input class="metric-input" :value="store.viewState.form.building" @input="onFormInput('building', $event)" />
        </label>
        <label class="metric-field">
          <span class="metric-label">楼层信息</span>
          <input class="metric-input" :value="store.viewState.form.floor" @input="onFormInput('floor', $event)" />
        </label>
      </div>
      <p v-if="store.viewState.errorMessage" class="error-copy">{{ store.viewState.errorMessage }}</p>
    </ConfigCard>

    <div class="footer-actions">
      <button class="primary-btn save-btn" type="button" :disabled="store.viewState.saving" @click="saveCurrent">
        {{ store.viewState.saving ? "保存中..." : "保存配置" }}
      </button>
    </div>

    <div v-if="dialogState.visible" class="fluent-mask" @click.self="closeDialog(false)">
      <section class="fluent-dialog card-shell" :class="dialogToneClass">
        <header class="dialog-head">
          <div class="dialog-title-wrap">
            <span class="dialog-icon material-symbols-rounded" aria-hidden="true">{{ dialogState.icon }}</span>
            <h3>{{ dialogState.title }}</h3>
          </div>
          <button class="dialog-close" type="button" @click="closeDialog(false)">×</button>
        </header>
        <p class="dialog-summary">{{ dialogState.summary }}</p>
        <ul v-if="dialogState.details.length > 0" class="dialog-details">
          <li v-for="(line, index) in dialogState.details" :key="index">{{ line }}</li>
        </ul>
        <footer class="dialog-actions">
          <button v-if="dialogState.kind === 'confirm'" class="secondary-btn" type="button" @click="closeDialog(false)">
            {{ dialogState.cancelText }}
          </button>
          <button :class="dialogState.kind === 'confirm' ? 'primary-btn' : 'secondary-btn'" type="button" @click="closeDialog(true)">
            {{ dialogState.confirmText }}
          </button>
        </footer>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { CLASS_CONFIG_TYPE_OPTIONS } from "../../../entities/class-config/model";
import type { ClassConfigRow, ClassConfigType } from "../../../entities/class-config/model";
import type { Subject } from "../../../entities/score/model";
import { SUBJECT_OPTIONS } from "../../../entities/class-config/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useClassConfigStore } from "../store";

const store = useClassConfigStore();
const searchKeyword = ref("");
const isSuggestionOpen = ref(false);
const isSelectingSuggestion = ref(false);
const suggestionPage = ref(1);
const suggestionListRef = ref<HTMLElement | null>(null);
let dialogResolver: ((value: boolean) => void) | null = null;
const SUGGESTION_PAGE_SIZE = 30;
const gradeRankMap: Record<string, number> = { 高一: 1, 高二: 2, 高三: 3 };

const dialogState = reactive({
  visible: false,
  kind: "confirm" as "confirm" | "alert",
  title: "",
  summary: "",
  details: [] as string[],
  confirmText: "确认",
  cancelText: "取消",
  tone: "neutral" as "neutral" | "danger" | "success",
  icon: "info",
});

const visibleSubjects = computed(() => SUBJECT_OPTIONS);
const configTypeOptions = CLASS_CONFIG_TYPE_OPTIONS;
const normalizedSearchKeyword = computed(() => searchKeyword.value.trim().replace(/\s+/g, ""));
const sortedRows = computed(() => [...store.viewState.rows].sort(compareClassRows));
const filteredSuggestionRows = computed(() => {
  if (!normalizedSearchKeyword.value) {
    return sortedRows.value;
  }
  return sortedRows.value.filter((row) =>
    normalizeClassName(row.className).includes(normalizedSearchKeyword.value),
  );
});
const suggestionRows = computed(() => {
  return filteredSuggestionRows.value.slice(0, suggestionPage.value * SUGGESTION_PAGE_SIZE);
});
const hasMoreSuggestions = computed(() => suggestionRows.value.length < filteredSuggestionRows.value.length);
const showSuggestionList = computed(() => isSuggestionOpen.value && suggestionRows.value.length > 0);
const dialogToneClass = computed(() => {
  if (dialogState.tone === "danger") {
    return "dialog-danger";
  }
  if (dialogState.tone === "success") {
    return "dialog-success";
  }
  return "dialog-neutral";
});
const subjectDescription = computed(
  () => `点击科目标签可启用或取消，当前班级已选择 ${store.viewState.form.subjects.length} 门课程。`,
);
const currentModeLabel = computed(() => (store.viewState.mode === "new" ? "新建配置" : "编辑已有班级"));
const currentTypeLabel = computed(() =>
  store.viewState.form.configType === "teaching_class" ? "教学班" : "考试教室",
);
const stateDescription = computed(() => {
  if (store.viewState.mode === "new") {
    return "将作为新班级配置保存，当前楼号、楼层和科目内容均为空。";
  }
  return "正在编辑已有班级，保存后将更新至当前班级。";
});

function syncSearchToCurrentClass() {
  searchKeyword.value = store.viewState.form.className;
}

function normalizeClassName(name: string) {
  return name.trim().replace(/\s+/g, "");
}

function extractClassSortNumber(className: string) {
  const match = className.match(/(\d+)/g);
  return match && match.length > 0 ? Number(match[match.length - 1]) : Number.POSITIVE_INFINITY;
}

function extractGradeName(value: string) {
  const source = value.trim();
  if (source) {
    const match = source.match(/^(高[一二三]|高中[一二三]|初[一二三]|初中[一二三])/);
    if (match?.[0]) {
      return match[0];
    }
  }
  return "";
}

function gradeSortRank(gradeName: string, className: string) {
  const normalizedGrade = gradeName.trim() || extractGradeName(className);
  return gradeRankMap[normalizedGrade] ?? 99;
}

function compareClassRows(a: ClassConfigRow, b: ClassConfigRow) {
  const gradeDiff = gradeSortRank(a.gradeName, a.className) - gradeSortRank(b.gradeName, b.className);
  if (gradeDiff !== 0) {
    return gradeDiff;
  }
  const classDiff = extractClassSortNumber(a.className) - extractClassSortNumber(b.className);
  if (classDiff !== 0) {
    return classDiff;
  }
  return a.className.localeCompare(b.className, "zh-CN", { numeric: true });
}

function resetSuggestionPaging() {
  suggestionPage.value = 1;
  if (suggestionListRef.value) {
    suggestionListRef.value.scrollTop = 0;
  }
}

function loadMoreSuggestions() {
  if (hasMoreSuggestions.value) {
    suggestionPage.value += 1;
  }
}

function onSuggestionScroll(event: Event) {
  // 在接近列表底部时增量加载下一页，避免一次渲染过多班级项。
  const target = event.target as HTMLElement;
  if (target.scrollTop + target.clientHeight >= target.scrollHeight - 24) {
    loadMoreSuggestions();
  }
}

function findExactRowByName(name: string): ClassConfigRow | undefined {
  const normalizedName = normalizeClassName(name);
  if (!normalizedName) {
    return undefined;
  }
  return sortedRows.value.find((row) => normalizeClassName(row.className) === normalizedName);
}

function inferGradeName(className: string) {
  const trimmed = className.trim();
  const match = trimmed.match(/^(高[一二三四五六七八九十]+|初[一二三四五六七八九]|初中[一二三]|高中[一二三])/);
  return match?.[1] ?? "";
}

function syncInferredGradeName(className: string) {
  if (store.viewState.mode !== "new") {
    return;
  }
  store.setFormField("gradeName", inferGradeName(className));
}

function onFormInput(field: "building" | "floor", event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField(field, value);
}

function openDialog(options: {
  kind: "confirm" | "alert";
  title: string;
  summary: string;
  details?: string[];
  confirmText?: string;
  cancelText?: string;
  tone?: "neutral" | "danger" | "success";
  icon?: string;
}) {
  dialogState.visible = true;
  dialogState.kind = options.kind;
  dialogState.title = options.title;
  dialogState.summary = options.summary;
  dialogState.details = options.details ?? [];
  dialogState.confirmText = options.confirmText ?? (options.kind === "confirm" ? "确认" : "知道了");
  dialogState.cancelText = options.cancelText ?? "取消";
  dialogState.tone = options.tone ?? "neutral";
  dialogState.icon = options.icon ?? "info";
  return new Promise<boolean>((resolve) => {
    dialogResolver = resolve;
  });
}

function closeDialog(result: boolean) {
  if (dialogResolver) {
    dialogResolver(result);
    dialogResolver = null;
  }
  dialogState.visible = false;
}

function onSearchFocus() {
  isSuggestionOpen.value = true;
  resetSuggestionPaging();
}

function onSearchInput() {
  isSuggestionOpen.value = true;
  resetSuggestionPaging();
}

function onSuggestionMouseDown() {
  isSelectingSuggestion.value = true;
}

function onSearchBlur() {
  window.setTimeout(async () => {
    if (isSelectingSuggestion.value) {
      isSelectingSuggestion.value = false;
      return;
    }
    isSuggestionOpen.value = false;
    await onSearchCommit();
  }, 0);
}

async function handleSwitchToRow(row: ClassConfigRow) {
  const currentName = store.viewState.loadedClassName || store.viewState.form.className || "当前班级";
  if (
    store.viewState.isDirty &&
    (store.viewState.mode === "new" || row.id !== store.viewState.editingId)
  ) {
    const abandon = await openDialog({
      kind: "confirm",
      tone: "danger",
      icon: "warning",
      title: "检测到未保存修改",
      summary: "切换班级前需要先放弃当前页面中尚未保存的配置变更。",
      details: [`当前编辑：${currentName}`, "若继续切换，当前页面里的修改将不会保留。"],
      confirmText: "放弃并切换",
      cancelText: "继续编辑",
    });
    if (!abandon) {
      if (store.viewState.mode === "existing") {
        syncSearchToCurrentClass();
      }
      return;
    }
  }

  await store.loadDetail(row.id);
  syncSearchToCurrentClass();
}

async function switchConfigType(configType: ClassConfigType) {
  if (store.viewState.filters.configType === configType) {
    return;
  }
  if (store.viewState.isDirty) {
    const abandon = await openDialog({
      kind: "confirm",
      tone: "danger",
      icon: "warning",
      title: "检测到未保存修改",
      summary: "切换教室类型前需要先放弃当前页面中尚未保存的配置变更。",
      details: [
        `当前编辑：${store.viewState.form.className || "当前配置"}`,
        `切换到：${configType === "teaching_class" ? "教学教室" : "考试教室"}`,
      ],
      confirmText: "放弃并切换",
      cancelText: "继续编辑",
    });
    if (!abandon) {
      return;
    }
  }
  await store.setFilters({ configType, gradeName: "", keyword: "" });
  syncSearchToCurrentClass();
}

async function selectSuggestion(id: number) {
  const row = store.viewState.rows.find((item) => item.id === id);
  if (!row) {
    return;
  }
  isSuggestionOpen.value = false;
  searchKeyword.value = row.className;
  await handleSwitchToRow(row);
}

async function onSearchCommit() {
  const keyword = searchKeyword.value.trim();
  if (!keyword) {
    if (store.viewState.mode === "existing") {
      syncSearchToCurrentClass();
    }
    return;
  }

  const exactRow = findExactRowByName(keyword);
  const normalizedKeyword = keyword.replace(/\s+/g, "");
  const normalizedCurrentClass = store.viewState.form.className.trim().replace(/\s+/g, "");
  if (exactRow) {
    if (exactRow.id === store.viewState.editingId && store.viewState.mode === "existing") {
      syncSearchToCurrentClass();
      return;
    }
    await handleSwitchToRow(exactRow);
    return;
  }

  if (store.viewState.mode === "new" && normalizedKeyword && normalizedKeyword === normalizedCurrentClass) {
    syncSearchToCurrentClass();
    return;
  }

  if (store.viewState.isDirty) {
    const abandon = await openDialog({
      kind: "confirm",
      tone: "danger",
      icon: "warning",
      title: "检测到未保存修改",
      summary: "创建新班级前需要先放弃当前页面中尚未保存的配置变更。",
      details: [`当前编辑：${store.viewState.form.className || "当前班级"}`, `新班级：${keyword}`],
      confirmText: "放弃并新建",
      cancelText: "继续编辑",
    });
    if (!abandon) {
      syncSearchToCurrentClass();
      return;
    }
  }

  store.startCreate(keyword);
  syncInferredGradeName(keyword);
  syncSearchToCurrentClass();
}

function onCurrentClassInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField("className", value);
  if (store.viewState.mode === "new") {
    syncInferredGradeName(value);
    searchKeyword.value = value;
  }
}

function onRoomLabelInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField("roomLabel", value || null);
}

function clearSearchKeyword() {
  searchKeyword.value = "";
  isSuggestionOpen.value = true;
  resetSuggestionPaging();
}

async function deleteCurrent() {
  if (!store.viewState.editingId) {
    return;
  }
  const className = store.viewState.loadedClassName || store.viewState.form.className.trim();
  try {
    await store.remove(store.viewState.editingId);
    syncSearchToCurrentClass();
    await openDialog({
      kind: "alert",
      tone: "success",
      icon: "check_circle",
      title: "删除成功",
      summary: "班级配置已删除。",
      details: className ? [`已删除：${className}`] : [],
      confirmText: "知道了",
    });
  } catch {
    // Error message is provided by store.viewState.errorMessage.
  }
}

async function saveCurrent() {
  const wasExisting = store.viewState.mode === "existing";
  try {
    if (wasExisting) {
      await store.update();
    } else {
      await store.create();
    }
    syncSearchToCurrentClass();
    await openDialog({
      kind: "alert",
      tone: "success",
      icon: "check_circle",
      title: "保存成功",
      summary: wasExisting ? "班级配置已更新。" : "新班级配置已创建。",
      details: [
        `班级：${store.viewState.form.className.trim() || "未命名班级"}`,
        `楼号：${store.viewState.form.building || "未填写"}，楼层：${store.viewState.form.floor || "未填写"}`,
      ],
      confirmText: "知道了",
    });
  } catch {
    // Error message is provided by store.viewState.errorMessage.
  }
}

function toggleSubject(subject: Subject) {
  const checked = !store.viewState.form.subjects.includes(subject);
  store.toggleSubject(subject, checked);
}

onMounted(async () => {
  await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
  syncSearchToCurrentClass();
});
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
  overflow: visible;
  position: relative;
  z-index: 1;
}

.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.search-card {
  min-height: 92px;
  padding: 16px 18px;
  display: flex;
  align-items: center;
  position: relative;
  z-index: 20;
  overflow: visible;
  border-radius: 24px;
}

.context-card {
  padding: 16px 18px;
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) auto;
  gap: 16px;
  align-items: center;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--surface-panel) 86%, white), color-mix(in srgb, var(--surface-elevated) 90%, white)),
    radial-gradient(circle at top right, rgba(var(--accent-rgb), 0.08), rgba(var(--accent-rgb), 0));
}

.context-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.context-copy strong {
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 700;
  line-height: 1.2;
}

.context-copy p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.context-badges {
  display: flex;
  align-items: stretch;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.context-badge {
  min-width: 132px;
  padding: 12px 14px;
  border-radius: 18px;
  border: 1px solid var(--color-border-soft);
  background: color-mix(in srgb, var(--surface-panel) 84%, white);
  box-shadow: var(--shadow-soft);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.context-badge span {
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.context-badge strong {
  color: var(--accent-primary-strong);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.3;
}

.search-block {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.picker-label {
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
  min-height: 18px;
}

.class-input-wrap {
  width: 380px;
  position: relative;
  z-index: 24;
}

.class-input-shell {
  width: 100%;
  min-height: 44px;
  border: 1px solid var(--color-border-soft);
  border-radius: 16px;
  background: var(--surface-input);
  padding: 0 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  position: relative;
  box-shadow: 0 10px 22px rgba(var(--accent-rgb), 0.06);
}

.clear-btn {
  border: 0;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  margin-left: auto;
  flex-shrink: 0;
}

.clear-btn .material-symbols-rounded {
  font-size: 20px;
}

.clear-btn:hover {
  color: var(--color-text);
}

.class-input-shell:focus-within {
  border-color: var(--accent-border-strong);
  box-shadow: 0 0 0 3px var(--accent-focus-ring), 0 10px 24px rgba(var(--accent-rgb), 0.08);
}

.search-input,
.current-input {
  min-width: 0;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text);
}

.search-input {
  font-size: 16px;
}

.current-input {
  font-size: 18px;
  font-weight: 700;
}

.search-input:focus,
.current-input:focus,
.metric-input:focus {
  outline: none;
}

.type-toggle-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.type-toggle-btn {
  min-height: 44px;
  border: 1px solid var(--color-border-soft);
  border-radius: 16px;
  background: var(--surface-panel);
  color: var(--color-text-muted);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition:
    border-color 0.18s ease,
    background-color 0.18s ease,
    color 0.18s ease,
    box-shadow 0.18s ease;
}

.type-toggle-btn.active {
  border-color: var(--accent-border-strong);
  background: color-mix(in srgb, rgba(var(--accent-rgb), 0.12) 72%, var(--surface-panel-strong));
  color: var(--accent-primary-strong);
  box-shadow: 0 10px 22px rgba(var(--accent-rgb), 0.1);
}

.suggestion-list {
  position: absolute;
  top: 48px;
  left: 0;
  width: 100%;
  max-height: 256px;
  overflow: auto;
  border: 1px solid var(--color-border-soft);
  border-radius: 18px;
  background: var(--surface-input-strong);
  box-shadow: var(--shadow-strong);
  backdrop-filter: blur(18px);
  z-index: 40;
  padding: 8px;
}

.suggestion-item {
  width: 100%;
  min-height: 42px;
  border: 1px solid transparent;
  border-radius: 14px;
  background: var(--surface-elevated);
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
  text-align: left;
  padding: 0 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition: background-color 0.16s ease, border-color 0.16s ease, color 0.16s ease;
}

.suggestion-item:hover {
  background: rgba(var(--accent-rgb), 0.12);
  border-color: var(--accent-border-strong);
  color: var(--accent-primary);
}

.suggestion-item + .suggestion-item {
  margin-top: 4px;
}

.suggestion-load-more {
  margin-top: 6px;
  min-height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  font-size: 12px;
}

.suggestion-meta {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 500;
}

.current-card :deep(.body) {
  gap: 0;
}

.current-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.current-inputs-wrap {
  display: flex;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.current-input-wrap {
  width: 280px;
  max-width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-start;
  flex-shrink: 0;
}

.room-label-wrap {
  width: 280px;
  align-items: flex-start;
  margin-left: auto;
  margin-right: 32px;
}

.room-label-input-shell {
  padding-right: 18px;
}

.room-label-input {
  min-width: 0;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
}

.room-label-input:focus {
  outline: none;
}

.current-input-shell {
  padding-right: 18px;
}

.status-copy {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 13px;
  line-height: 1.5;
  white-space: normal;
}

.status-copy-new {
  color: var(--color-brand);
}

.delete-btn {
  min-width: 100px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--color-danger);
  border-color: rgba(209, 52, 56, 0.2);
  background: linear-gradient(180deg, color-mix(in srgb, var(--color-danger-soft) 84%, white), var(--color-danger-soft) 100%);
}

.editor-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.metric-input {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
}

.metric-field {
  min-height: 84px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid var(--color-border-soft);
  background: color-mix(in srgb, var(--surface-panel) 82%, white);
  box-shadow: var(--shadow-soft);
}

.subject-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.subject-pill {
  width: 124px;
  height: 40px;
  border-radius: 16px;
  border: 1px solid var(--color-border-soft);
  background: var(--surface-panel);
  color: var(--color-text-muted);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color 0.18s ease,
    background-color 0.18s ease,
    color 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.subject-pill:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--accent-rgb), 0.24);
  box-shadow: 0 10px 22px rgba(var(--accent-rgb), 0.1);
}

.subject-pill.active {
  border-color: var(--accent-border-strong);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary-strong);
  font-weight: 700;
}

.footer-actions {
  display: flex;
  justify-content: flex-start;
}

.save-btn {
  width: 132px;
}

.error-copy {
  margin: 0;
  color: var(--color-danger);
  font-size: 13px;
}

.fluent-mask {
  position: fixed;
  inset: 0;
  background: var(--surface-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 600;
}

.fluent-dialog {
  width: 520px;
  max-width: calc(100vw - 32px);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-radius: 24px;
  position: relative;
  z-index: 610;
}

.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.dialog-title-wrap {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

.dialog-icon {
  font-size: 20px;
}

.dialog-head h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
}

.dialog-close {
  border: 0;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 20px;
  line-height: 1;
}

.dialog-close:hover {
  background: var(--surface-panel);
}

.dialog-summary {
  margin: 0;
  color: var(--color-text);
  font-size: 14px;
  line-height: 1.55;
}

.dialog-details {
  margin: 0;
  padding-left: 18px;
  color: var(--color-text-muted);
  font-size: 13px;
  display: grid;
  gap: 4px;
}

.dialog-actions {
  margin-top: 4px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.dialog-neutral .dialog-icon {
  color: var(--accent-primary);
}

.dialog-danger .dialog-icon {
  color: var(--color-danger);
}

.dialog-success .dialog-icon {
  color: var(--color-success);
}

@media (max-width: 900px) {
  .context-card {
    grid-template-columns: 1fr;
  }

  .context-badges {
    justify-content: flex-start;
  }

  .class-input-wrap,
  .current-input-wrap {
    width: 100%;
  }

  .current-head,
  .current-inputs-wrap {
    flex-direction: column;
  }

  .room-label-wrap {
    width: 100%;
  }

  .editor-grid {
    grid-template-columns: 1fr;
  }
}
</style>
