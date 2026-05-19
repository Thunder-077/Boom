<template>
  <section class="class-config-page">
    <section class="workspace">
      <aside class="sidebar card-shell">
        <div class="sidebar-toolbar">
          <div class="type-switch">
            <button
              v-for="option in configTypeOptions"
              :key="option.value"
              type="button"
              class="type-btn"
              :class="{ active: store.viewState.filters.configType === option.value }"
              @click="switchConfigType(option.value)"
            >
              {{ option.label }}
            </button>
          </div>

          <div class="search-anchor">
            <div class="search-wrap">
            <span class="material-symbols-rounded search-icon" aria-hidden="true">search</span>
            <input
              ref="searchInputRef"
              v-model="searchKeyword"
              class="search-input"
              :placeholder="searchPlaceholder"
              @focus="openSearchPanel"
              @click="openSearchPanel"
              @blur="onSearchBlur"
            />
            </div>

            <div v-if="isSearchPanelOpen" class="search-panel" @mousedown.prevent="onSearchPanelMouseDown">
              <div class="list-head">
                <strong>{{ listTitle }}</strong>
                <span>{{ filteredRows.length }} / {{ store.viewState.rows.length }}</span>
              </div>

              <div class="config-list">
                <button
                  v-for="row in filteredRows"
                  :key="row.id"
                  type="button"
                  class="config-item"
                  :class="{ active: row.id === store.viewState.editingId && store.viewState.mode === 'existing' }"
                  @click="selectRow(row)"
                >
                  <div class="config-item-top">
                    <strong>{{ row.className }}</strong>
                    <Tag variant="primary" size="sm">{{ row.gradeName || currentTypeLabel }}</Tag>
                  </div>
                  <p class="config-item-subtitle">{{ compactLocation(row) }}</p>
                </button>

                <EmptyState
                  v-if="filteredRows.length === 0"
                  :title="emptyListTitle"
                  :description="emptyListSummary"
                />
              </div>
            </div>
          </div>

          <Button variant="primary" class="create-btn" @click="createNewConfig">
            <span class="material-symbols-rounded" aria-hidden="true">add</span>
            {{ createButtonLabel }}
          </Button>
        </div>
      </aside>

      <div class="editor">
        <ConfigCard title="基础信息" :description="editorDescription">
          <div class="form-grid">
            <label class="field">
              <span class="field-label">年级</span>
              <input
                class="field-input"
                :value="store.viewState.form.gradeName"
                placeholder="例如：高一"
                list="class-grade-options"
                @input="onFormInput('gradeName', $event)"
              />
            </label>

            <label class="field field-wide">
              <span class="field-label">{{ nameLabel }}</span>
              <input
                class="field-input field-input-strong"
                :value="store.viewState.form.className"
                :placeholder="namePlaceholder"
                @input="onFormInput('className', $event)"
              />
            </label>

            <label class="field">
              <span class="field-label">教室标签</span>
              <input
                class="field-input"
                :value="store.viewState.form.roomLabel || ''"
                placeholder="例如：X-505"
                @input="onRoomLabelInput"
              />
            </label>
          </div>
          <datalist id="class-grade-options">
            <option v-for="grade in store.viewState.gradeOptions" :key="grade" :value="grade" />
          </datalist>
        </ConfigCard>

        <ConfigCard
          v-if="store.viewState.form.configType === 'teaching_class'"
          title="所学科目"
          :description="`当前已选择 ${store.viewState.form.subjects.length} 门课程。`"
        >
          <div class="subject-grid">
            <button
              v-for="subject in leadingSubjects"
              :key="subject.value"
              type="button"
              class="subject-chip"
              :class="{ active: store.viewState.form.subjects.includes(subject.value) }"
              @click="toggleSubject(subject.value)"
            >
              {{ subject.label }}
            </button>

            <div class="subject-curve-group">
              <button
                v-for="subject in foreignLanguageSubjects"
                :key="subject.value"
                type="button"
                class="subject-chip"
                :class="{ active: store.viewState.form.subjects.includes(subject.value) }"
                @click="toggleSubject(subject.value)"
              >
                {{ subject.label }}
              </button>
            </div>

            <button
              v-for="subject in remainingSubjects"
              :key="subject.value"
              type="button"
              class="subject-chip"
              :class="{ active: store.viewState.form.subjects.includes(subject.value) }"
              @click="toggleSubject(subject.value)"
            >
              {{ subject.label }}
            </button>
          </div>
        </ConfigCard>

        <ConfigCard title="教室位置">
          <div class="form-grid form-grid-compact">
            <label class="field">
              <span class="field-label">楼号</span>
              <input
                class="field-input"
                :value="store.viewState.form.building"
                placeholder="例如：向远楼"
                @input="onFormInput('building', $event)"
              />
            </label>

            <label class="field">
              <span class="field-label">楼层</span>
              <input
                class="field-input"
                :value="store.viewState.form.floor"
                placeholder="例如：3层"
                @input="onFormInput('floor', $event)"
              />
            </label>
          </div>
          <div class="config-footer">
            <Button
              variant="danger"
              :disabled="store.viewState.mode !== 'existing' || store.viewState.deleting"
              @click="deleteCurrent"
            >
              {{ store.viewState.deleting ? "删除中..." : "删除教室" }}
            </Button>
            <Button
              variant="primary"
              :disabled="store.viewState.saving"
              @click="saveCurrent"
            >
              {{ store.viewState.saving ? "保存中..." : saveButtonLabel }}
            </Button>
          </div>
          <p v-if="store.viewState.errorMessage" class="error-text">{{ store.viewState.errorMessage }}</p>
        </ConfigCard>
      </div>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { CLASS_CONFIG_TYPE_OPTIONS, SUBJECT_OPTIONS } from "../../../entities/class-config/model";
import type { ClassConfigRow, ClassConfigType } from "../../../entities/class-config/model";
import { Subject } from "../../../entities/score/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { Button, Tag, EmptyState } from "@/widgets/common";
import { useClassConfigStore } from "../store";
import { useAppDialog } from "@/shared/ui/appDialog";

const store = useClassConfigStore();
const dialog = useAppDialog();
const searchKeyword = ref("");
const isSearchPanelOpen = ref(false);
const isInteractingWithSearchPanel = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);

const configTypeOptions = CLASS_CONFIG_TYPE_OPTIONS;
const subjectMap = new Map(SUBJECT_OPTIONS.map((subject) => [subject.value, subject]));
// Teaching subjects are split into a fixed editor-style order so foreign languages stay grouped visually.
const leadingSubjects = computed(() => [Subject.Chinese, Subject.Math].map((value) => subjectMap.get(value)!));
const foreignLanguageSubjects = computed(() =>
  [Subject.English, Subject.Russian, Subject.Japanese].map((value) => subjectMap.get(value)!),
);
const remainingSubjects = computed(() =>
  [Subject.Physics, Subject.Chemistry, Subject.Biology, Subject.Politics, Subject.History, Subject.Geography].map(
    (value) => subjectMap.get(value)!,
  ),
);
const normalizedSearchKeyword = computed(() => searchKeyword.value.trim().replace(/\s+/g, "").toLowerCase());
const filteredRows = computed(() => {
  if (!normalizedSearchKeyword.value) {
    return store.viewState.rows;
  }
  return store.viewState.rows.filter((row) => {
    const text = [row.className, row.gradeName, row.roomLabel ?? "", row.building, row.floor]
      .join(" ")
      .replace(/\s+/g, "")
      .toLowerCase();
    return text.includes(normalizedSearchKeyword.value);
  });
});
const currentTypeLabel = computed(() =>
  store.viewState.filters.configType === "teaching_class" ? "教学班" : "考试教室",
);
const searchPlaceholder = computed(() =>
  store.viewState.filters.configType === "teaching_class" ? "搜索班级名称、年级、楼层" : "搜索教室名称、标签、楼层",
);
const listTitle = computed(() => `${currentTypeLabel.value}列表`);
const createButtonLabel = computed(() => `新建${currentTypeLabel.value}`);
const saveButtonLabel = computed(() => (store.viewState.mode === "new" ? "创建配置" : "保存修改"));
const nameLabel = computed(() => (store.viewState.filters.configType === "teaching_class" ? "班级名称" : "教室名称"));
const namePlaceholder = computed(() => (store.viewState.filters.configType === "teaching_class" ? "请输入班级名称" : "请输入教室名称"));
const editorDescription = computed(() =>
  store.viewState.mode === "new"
    ? `当前正在创建新的${currentTypeLabel.value}配置。`
    : `当前正在编辑 ${store.viewState.loadedClassName || "已选记录"}。`,
);
const emptyListTitle = computed(() => (normalizedSearchKeyword.value ? "没有匹配结果" : `暂无${currentTypeLabel.value}配置`));
const emptyListSummary = computed(() =>
  normalizedSearchKeyword.value ? "试试换个关键词，或者点击上方按钮创建新配置。" : "点击“新建”后即可开始录入。",
);

function compactLocation(row: ClassConfigRow) {
  const building = row.building?.trim() || "未填楼号";
  const floor = row.floor?.trim() || "未填楼层";
  return `${building} / ${floor}`;
}

function openSearchPanel() {
  isSearchPanelOpen.value = true;
}

function onSearchPanelMouseDown() {
  isInteractingWithSearchPanel.value = true;
}

function onSearchBlur() {
  window.setTimeout(() => {
    if (isInteractingWithSearchPanel.value) {
      isInteractingWithSearchPanel.value = false;
      searchInputRef.value?.focus();
      return;
    }
    isSearchPanelOpen.value = false;
  }, 0);
}

async function confirmDiscardIfNeeded(nextAction: string) {
  if (!store.viewState.isDirty) {
    return true;
  }
  return dialog.confirm({
    title: "检测到未保存修改",
    summary: "继续操作前需要先放弃当前未保存的内容。",
    details: [
      `当前内容：${store.viewState.form.className || `未命名${currentTypeLabel.value}`}`,
      `后续操作：${nextAction}`,
    ],
    confirmText: "放弃修改",
    cancelText: "继续编辑",
  });
}

async function switchConfigType(configType: ClassConfigType) {
  if (store.viewState.filters.configType === configType) {
    return;
  }
  const allowSwitch = await confirmDiscardIfNeeded(`切换到${configType === "teaching_class" ? "教学班" : "考试教室"}`);
  if (!allowSwitch) {
    return;
  }
  searchKeyword.value = "";
  isSearchPanelOpen.value = false;
  await store.setFilters({ configType, gradeName: "", keyword: "" });
}

async function createNewConfig() {
  const allowCreate = await confirmDiscardIfNeeded(`新建${currentTypeLabel.value}`);
  if (!allowCreate) {
    return;
  }
  isSearchPanelOpen.value = false;
  store.startCreate("");
}

async function selectRow(row: ClassConfigRow) {
  if (store.viewState.mode === "existing" && store.viewState.editingId === row.id) {
    isSearchPanelOpen.value = false;
    return;
  }
  const allowSwitch = await confirmDiscardIfNeeded(`切换到 ${row.className}`);
  if (!allowSwitch) {
    return;
  }
  await store.loadDetail(row.id);
  isSearchPanelOpen.value = false;
}

function onFormInput(field: "gradeName" | "className" | "building" | "floor", event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField(field, value);
}

function onRoomLabelInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField("roomLabel", value || null);
}

function toggleSubject(subject: Subject) {
  const checked = !store.viewState.form.subjects.includes(subject);
  store.toggleSubject(subject, checked);
}

async function saveCurrent() {
  const gradeName = store.viewState.form.gradeName.trim();
  const className = store.viewState.form.className.trim();
  if (!gradeName || !className) {
    const details: string[] = [];
    if (!gradeName) {
      details.push("请先填写年级。");
    }
    if (!className) {
      details.push(`请先填写${nameLabel.value}。`);
    }
    await dialog.alert({
      title: "缺少必填信息",
      summary: "当前配置还不能保存。",
      details,
    });
    return;
  }

  const wasCreating = store.viewState.mode === "new";
  try {
    if (wasCreating) {
      await store.create();
    } else {
      await store.update();
    }
    await dialog.alert({
      title: "保存成功",
      summary: wasCreating ? "配置已创建。" : "配置已更新。",
      details: [`名称：${store.viewState.form.className || `未命名${currentTypeLabel.value}`}`],
    });
  } catch {
    // Errors are surfaced through store.viewState.errorMessage.
  }
}

async function deleteCurrent() {
  if (!store.viewState.editingId) {
    return;
  }
  const confirmed = await dialog.confirm({
    title: "确认删除",
    summary: "删除后当前配置无法恢复，请确认是否继续。",
    details: [`当前记录：${store.viewState.form.className || `未命名${currentTypeLabel.value}`}`],
    confirmText: "确认删除",
    cancelText: "取消",
  });
  if (!confirmed) {
    return;
  }
  try {
    await store.remove(store.viewState.editingId);
    await dialog.alert({
      title: "删除成功",
      summary: "当前配置已经删除。",
    });
  } catch {
    // Errors are surfaced through store.viewState.errorMessage.
  }
}

onMounted(async () => {
  // Initialize with teaching class records so the page always opens into a valid CRUD context.
  await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
});
</script>

<style scoped>
.class-config-page {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  min-width: 0;
}

.workspace {
  display: grid;
  grid-template-columns: 360px minmax(0, 1fr);
  gap: var(--space-lg);
  align-items: stretch;
  min-width: 1280px;
}

.sidebar {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  min-height: 100%;
  overflow: visible;
}

.sidebar-toolbar {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.type-switch {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-sm);
}

.type-btn {
  min-height: 42px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  color: var(--text-secondary);
  font-size: var(--font-size-base);
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.type-btn.active {
  border-color: var(--accent-primary);
  background: var(--accent-soft);
  color: var(--accent-primary-strong);
}

.search-anchor {
  position: relative;
}

.search-wrap {
  min-height: 44px;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: 0 var(--space-lg);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-input);
}

.search-panel {
  position: absolute;
  top: calc(100% + var(--space-sm));
  left: 0;
  right: 0;
  z-index: 40;
  padding: var(--space-sm);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  box-shadow: var(--shadow-medium);
}

.search-icon {
  color: var(--text-tertiary);
  font-size: var(--font-size-lg);
}

.search-input,
.field-input {
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--text-primary);
}

.search-input {
  font-size: var(--font-size-base);
}

.field-input {
  font-size: var(--font-size-base);
  font-weight: 600;
}

.search-input:focus,
.field-input:focus {
  outline: none;
}

.list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.list-head strong {
  color: var(--text-primary);
  font-size: var(--font-size-base);
}

.config-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  max-height: 520px;
  overflow: auto;
  padding-right: var(--space-sm);
  margin-top: var(--space-sm);
}

.config-item {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  padding: var(--space-sm) var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  text-align: left;
  cursor: pointer;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease);
}

.config-item:hover {
  border-color: var(--accent-border-soft);
}

.config-item.active {
  border-color: var(--accent-primary);
  background: var(--accent-soft);
}

.config-item-top strong {
  color: var(--text-primary);
  font-size: var(--font-size-base);
  line-height: 1.3;
}

.config-item p {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}

.config-item-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
}

.config-item-subtitle {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.editor {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  min-width: 0;
}

.form-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(280px, 1fr);
  gap: var(--space-sm);
}

.form-grid-compact {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.field {
  min-height: 82px;
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-lg);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
}

.field-wide {
  min-width: 0;
}

.field-label {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.field-input-strong {
  font-size: var(--font-size-title-md);
  font-weight: 600;
}

.subject-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
  align-items: center;
}

.subject-curve-group {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-xs);
  border: 1px dashed var(--accent-primary);
  border-radius: var(--radius-pill);
  background: var(--surface-panel);
}

.subject-chip {
  min-width: 96px;
  min-height: 40px;
  padding: 0 var(--space-lg);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  color: var(--text-secondary);
  font-size: var(--font-size-base);
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.subject-chip.active {
  border-color: var(--accent-primary);
  background: var(--accent-soft);
  color: var(--accent-primary-strong);
}

.config-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-sm);
  margin-top: var(--space-sm);
  padding-top: var(--space-sm);
}

.error-text {
  margin: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

@media (max-width: 1100px) {
  .page-head {
    align-items: flex-start;
  }

  .form-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .field-wide {
    grid-column: 1 / -1;
  }
}
</style>
