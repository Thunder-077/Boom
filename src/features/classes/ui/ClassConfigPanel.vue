<template>
  <section class="panel">
    <div class="class-picker card-shell">
      <div class="class-picker-left">
        <span class="picker-label">当前配置班级</span>
        <div class="class-input-wrap">
          <label class="class-input-shell" @keydown.enter.prevent="onClassNameCommit">
            <input
              class="class-name-input"
              :value="store.viewState.form.className"
              :placeholder="namePlaceholder"
              @input="onClassNameInput"
              @focus="onClassInputFocus"
              @blur="onClassInputBlur"
            />
            <span v-if="statusText" class="status-tag" :class="statusClass">{{ statusText }}</span>
          </label>
          <div v-if="showSuggestionList" class="suggestion-list">
            <button
              v-for="row in suggestionRows"
              :key="row.id"
              type="button"
              class="suggestion-item"
              @mousedown.prevent="onSuggestionMouseDown"
              @click="selectSuggestion(row.id)"
            >
              {{ row.className }}
            </button>
          </div>
        </div>
      </div>

      <div class="action-right">
        <button
          class="secondary-btn delete-btn"
          type="button"
          :disabled="!store.viewState.editingId || store.viewState.saving || store.viewState.deleting"
          @click="deleteCurrent"
        >
          <span class="material-symbols-rounded" aria-hidden="true">delete</span>
          {{ store.viewState.deleting ? "删除中..." : "删除班级" }}
        </button>
        <button class="primary-btn save-btn" type="button" :disabled="store.viewState.saving" @click="saveCurrent">
          {{ store.viewState.saving ? "保存中..." : "保存配置" }}
        </button>
      </div>
    </div>

    <ConfigCard title="所学科目配置" :description="subjectDescription" v-if="store.viewState.form.configType === 'teaching_class'">
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

    <ConfigCard title="楼号与楼层信息" description="维护班级所在教学楼与楼层，切换班级后会自动带入已有配置。">
      <div class="editor-grid">
        <label class="metric-field wide">
          <span class="metric-label">年级</span>
          <input class="metric-input" :value="store.viewState.form.gradeName" @input="onFormInput('gradeName', $event)" />
        </label>
        <label class="metric-field">
          <span class="metric-label">楼号</span>
          <input class="metric-input" :value="store.viewState.form.building" @input="onFormInput('building', $event)" />
        </label>
        <label class="metric-field">
          <span class="metric-label">楼层信息</span>
          <input class="metric-input" :value="store.viewState.form.floor" @input="onFormInput('floor', $event)" />
        </label>
      </div>
      <p v-if="store.viewState.errorMessage" class="error-copy">{{ store.viewState.errorMessage }}</p>
    </ConfigCard>

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
import type { Subject } from "../../../entities/score/model";
import { SUBJECT_OPTIONS } from "../../../entities/class-config/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useClassConfigStore } from "../store";

const store = useClassConfigStore();
const isSuggestionOpen = ref(false);
const isSelectingSuggestion = ref(false);
let dialogResolver: ((value: boolean) => void) | null = null;

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
const namePlaceholder = computed(() => "输入班级名称");
const suggestionRows = computed(() => {
  const keyword = store.viewState.form.className.trim().replace(/\s+/g, "");
  return store.viewState.rows
    .filter((row) => row.configType === "teaching_class")
    .filter((row) => {
      if (!keyword) {
        return true;
      }
      return row.className.replace(/\s+/g, "").includes(keyword);
    })
    .slice(0, 8);
});
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
const statusText = computed(() => {
  if (store.viewState.classNameIntent === "switch") {
    return "切换";
  }
  if (store.viewState.classNameIntent === "create") {
    return "新建";
  }
  if (store.viewState.classNameIntent === "rename") {
    return "重命名";
  }
  return "";
});
const statusClass = computed(() => {
  if (store.viewState.classNameIntent === "rename") {
    return "status-rename";
  }
  if (store.viewState.classNameIntent === "switch" || store.viewState.classNameIntent === "create") {
    return "status-primary";
  }
  return "status-idle";
});
const subjectDescription = computed(
  () => `点击科目标签可启用或取消，当前班级已选择 ${store.viewState.form.subjects.length} 门课程。`,
);

function onFormInput(field: "gradeName" | "building" | "floor", event: Event) {
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

function onClassNameInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField("className", value);
  isSuggestionOpen.value = true;
}

function onClassInputFocus() {
  isSuggestionOpen.value = true;
}

function onClassInputBlur() {
  window.setTimeout(async () => {
    if (isSelectingSuggestion.value) {
      isSelectingSuggestion.value = false;
      return;
    }
    isSuggestionOpen.value = false;
    await onClassNameCommit();
  }, 0);
}

function onSuggestionMouseDown() {
  isSelectingSuggestion.value = true;
}

async function selectSuggestion(id: number) {
  const row = store.viewState.rows.find((item) => item.id === id);
  if (!row) {
    return;
  }
  store.setFormField("className", row.className);
  isSuggestionOpen.value = false;
  await onClassNameCommit();
}

async function onClassNameCommit() {
  if (store.viewState.classNameIntent === "switch" && store.viewState.targetMatchId) {
    if (store.viewState.isDirtyExceptClassName) {
      const abandon = await openDialog({
        kind: "confirm",
        tone: "danger",
        icon: "warning",
        title: "检测到未保存修改",
        summary: "切换班级前需要先放弃当前页面中尚未保存的配置变更。",
        details: [
          `当前编辑：${store.viewState.originalClassName || store.viewState.form.className || "未命名班级"}`,
          "未保存内容可能包含楼号、楼层和科目选择。",
          "若继续切换，这些修改将不会保留。",
        ],
        confirmText: "放弃并继续",
        cancelText: "返回继续编辑",
      });
      if (!abandon) {
        return;
      }
    }
    const nextName = store.viewState.form.className.trim();
    const currentName = store.viewState.originalClassName || store.viewState.form.className.trim() || "当前班级";
    const confirmSwitch = await openDialog({
      kind: "confirm",
      tone: "neutral",
      icon: "swap_horiz",
      title: "确认切换班级配置",
      summary: "系统将加载目标班级的楼号、楼层和科目配置并覆盖当前编辑区显示。",
      details: [`当前：${currentName}`, `目标：${nextName}`],
      confirmText: "确认切换",
      cancelText: "暂不切换",
    });
    if (!confirmSwitch) {
      return;
    }
    await store.loadDetail(store.viewState.targetMatchId);
    return;
  }

  if (store.viewState.classNameIntent === "create") {
    store.startCreateFromClassName(store.viewState.form.className);
  }
}

async function deleteCurrent() {
  if (!store.viewState.editingId) {
    return;
  }
  const className = store.viewState.originalClassName || store.viewState.form.className.trim();
  const approved = await openDialog({
    kind: "confirm",
    tone: "danger",
    icon: "delete_forever",
    title: "确认删除班级配置",
    summary: "删除后将无法恢复，请确认是否继续。",
    details: [`班级：${className}`, "将同时删除该班级的楼号、楼层、科目配置。"],
    confirmText: "确认删除",
    cancelText: "取消",
  });
  if (!approved) {
    return;
  }
  try {
    await store.remove(store.viewState.editingId);
    await openDialog({
      kind: "alert",
      tone: "success",
      icon: "check_circle",
      title: "删除成功",
      summary: "班级配置已移除，页面已自动切换到下一条可用班级。",
      details: [`已删除：${className}`],
      confirmText: "知道了",
    });
  } catch {
    // Error message is provided by store.viewState.errorMessage.
  }
}

async function saveCurrent() {
  try {
    if (store.viewState.classNameIntent === "rename" && store.viewState.editingId) {
      const nextName = store.viewState.form.className.trim();
      const prevName = store.viewState.originalClassName;
      const approved = await openDialog({
        kind: "confirm",
        tone: "neutral",
        icon: "edit",
        title: "确认重命名班级",
        summary: "将保留当前班级配置内容，仅更新班级名称。",
        details: [`原名称：${prevName}`, `新名称：${nextName}`],
        confirmText: "确认重命名",
        cancelText: "取消",
      });
      if (!approved) {
        return;
      }
    }

    if (store.viewState.editingId) {
      await store.update();
    } else {
      await store.create();
    }
    await openDialog({
      kind: "alert",
      tone: "success",
      icon: "check_circle",
      title: "保存成功",
      summary: "班级配置已成功写入。",
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

async function toggleSubject(subject: Subject) {
  const checked = !store.viewState.form.subjects.includes(subject);
  store.toggleSubject(subject, checked);
}

onMounted(async () => {
  await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
});
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 18px;
  overflow: visible;
}

.class-picker {
  height: 74px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  border-radius: 18px;
  gap: 16px;
  position: relative;
  z-index: 30;
  overflow: visible;
}

.class-picker-left {
  width: 360px;
  flex: 0 0 360px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.class-input-wrap {
  width: 360px;
  position: relative;
  z-index: 40;
}

.picker-label {
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 600;
}

.class-input-shell {
  width: 360px;
  height: 44px;
  border: 1px solid var(--color-border-soft);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.59);
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.class-input-shell:focus-within {
  border-color: #b9d6ff;
  box-shadow: 0 0 0 3px rgba(185, 214, 255, 0.35);
}

.class-name-input {
  min-width: 0;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 700;
}

.class-name-input:focus {
  outline: none;
}

.status-tag {
  min-height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.status-idle {
  background: rgba(255, 255, 255, 0.7);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border-soft);
}

.status-primary {
  background: var(--color-brand-soft);
  color: var(--color-brand);
}

.status-rename {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.suggestion-list {
  position: absolute;
  top: 48px;
  left: 0;
  width: 360px;
  max-height: 256px;
  overflow: auto;
  border: 1px solid #ffffffd8;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow:
    0 12px 30px rgba(151, 169, 194, 0.18),
    0 0 0 1px rgba(255, 255, 255, 0.22) inset;
  backdrop-filter: blur(18px);
  z-index: 60;
  padding: 6px;
}

.suggestion-item {
  width: 100%;
  min-height: 36px;
  border: 1px solid transparent;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.35);
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
  text-align: left;
  padding: 0 10px;
  cursor: pointer;
  transition: background-color 0.16s ease, border-color 0.16s ease, color 0.16s ease;
}

.suggestion-item:hover {
  background: #eaf3ffcc;
  border-color: #c5dcff;
  color: var(--color-brand);
}

.suggestion-item + .suggestion-item {
  margin-top: 4px;
}

.suggestion-list::-webkit-scrollbar {
  width: 8px;
}

.suggestion-list::-webkit-scrollbar-thumb {
  background: rgba(15, 108, 189, 0.28);
  border-radius: 999px;
}

.panel :deep(.config-card) {
  position: relative;
  z-index: 1;
}

.action-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.delete-btn {
  min-width: 96px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: #b42318;
  border-color: #f4c7c9;
  background: linear-gradient(180deg, #fff8f8 0%, #fff2f3 100%);
  box-shadow: 0 6px 14px rgba(180, 35, 24, 0.08);
}

.delete-btn:hover:not(:disabled) {
  border-color: #eba7ab;
  background: linear-gradient(180deg, #fff2f2 0%, #ffe6e8 100%);
}

.save-btn {
  width: 132px;
  flex-shrink: 0;
}

.editor-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.metric-field.wide {
  grid-column: span 2;
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

.subject-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.subject-pill {
  width: 124px;
  height: 40px;
  border-radius: 14px;
  border: 1px solid var(--color-border-soft);
  background: rgba(255, 255, 255, 0.6);
  color: var(--color-text-muted);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.subject-pill.active {
  border-color: #b9d6ff;
  background: var(--color-brand-soft);
  color: var(--color-brand);
  font-weight: 700;
}

.error-copy {
  margin: 0;
  color: var(--color-danger);
  font-size: 13px;
}

.fluent-mask {
  position: fixed;
  inset: 0;
  background: rgba(17, 21, 26, 0.34);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 80;
}

.fluent-dialog {
  width: 520px;
  max-width: calc(100vw - 32px);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
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
  background: rgba(255, 255, 255, 0.66);
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
  color: #0f6cbd;
}

.dialog-danger .dialog-icon {
  color: #b42318;
}

.dialog-success .dialog-icon {
  color: #0f8a5f;
}
</style>
