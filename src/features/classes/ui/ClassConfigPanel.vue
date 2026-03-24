<template>
  <section class="panel">
    <div class="class-picker card-shell">
      <div class="picker-copy">
        <span class="picker-label">当前配置对象</span>
        <strong class="picker-value">{{ currentDisplayName }}</strong>
      </div>
      <div class="picker-actions">
        <button
          v-for="option in CLASS_CONFIG_TYPE_OPTIONS"
          :key="option.value"
          type="button"
          class="mode-pill"
          :class="{ active: store.viewState.form.configType === option.value }"
          @click="switchConfigType(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <ConfigCard title="基础信息" :description="baseDescription">
      <div class="editor-grid">
        <label class="metric-field wide">
          <span class="metric-label">年级</span>
          <input class="metric-input" :value="store.viewState.form.gradeName" @input="onFormInput('gradeName', $event)" />
        </label>
        <label class="metric-field wide">
          <span class="metric-label">{{ nameLabel }}</span>
          <input class="metric-input" :value="store.viewState.form.className" @input="onFormInput('className', $event)" />
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
      <div class="editor-actions">
        <label class="picker-select">
          <input
            v-model.trim="lookupKeyword"
            class="glass-field"
            type="text"
            :placeholder="store.viewState.form.configType === 'exam_room' ? '输入教室名称查询已有配置' : '输入班级名称查询已有配置'"
          />
        </label>
        <div v-if="matchedRows.length > 0" class="lookup-list">
          <button
            v-for="row in matchedRows"
            :key="row.id"
            type="button"
            class="lookup-item"
            @click="loadConfig(row.id)"
          >
            {{ row.className }}
          </button>
        </div>
        <button class="secondary-btn" type="button" @click="createNew">新建配置</button>
        <button class="primary-btn" type="button" :disabled="store.viewState.saving" @click="saveCurrent">
          {{ store.viewState.saving ? "保存中..." : store.viewState.editingId ? "保存修改" : "新增并保存" }}
        </button>
      </div>
    </ConfigCard>

    <ConfigCard v-if="store.viewState.form.configType === 'teaching_class'" :title="'所学科目配置'" :description="subjectDescription">
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

    <ConfigCard v-else title="考试教室说明" description="备用考场和空置教室可直接在这里新增，用于保存楼号与楼层信息，便于后续分配监考老师。">
      <p class="exam-room-copy">考试教室不需要配置所学科目，只需维护年级、教室名称、楼号和楼层信息。</p>
    </ConfigCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { ClassConfigType } from "../../../entities/class-config/model";
import type { Subject } from "../../../entities/score/model";
import { CLASS_CONFIG_TYPE_OPTIONS, SUBJECT_OPTIONS } from "../../../entities/class-config/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useClassConfigStore } from "../store";

const store = useClassConfigStore();
const lookupKeyword = ref("");

const visibleRows = computed(() => store.viewState.rows.filter((row) => row.configType === store.viewState.form.configType));
const matchedRows = computed(() => {
  const keyword = lookupKeyword.value.trim();
  if (!keyword) {
    return visibleRows.value.slice(0, 8);
  }
  return visibleRows.value.filter((row) => row.className.includes(keyword)).slice(0, 8);
});
const visibleSubjects = computed(() => SUBJECT_OPTIONS);
const currentDisplayName = computed(() => {
  const name = store.viewState.form.className.trim();
  if (name) {
    return name;
  }
  const typeLabel = store.viewState.form.configType === "exam_room" ? "考试教室" : "班级";
  return `未命名${typeLabel}`;
});
const nameLabel = computed(() => (store.viewState.form.configType === "exam_room" ? "教室名称" : "班级名称"));
const baseDescription = computed(() =>
  store.viewState.form.configType === "exam_room"
    ? "可直接新增备用考场或空置教室，并维护其楼号与楼层。"
    : "可直接输入新增班级，也可以从已有配置中加载后继续修改。",
);
const subjectDescription = computed(
  () => `点击科目标签可启用或取消，当前班级已选择 ${store.viewState.form.subjects.length} 门课程。`,
);

function onFormInput(field: "gradeName" | "className" | "building" | "floor", event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField(field, value);
}

async function switchConfigType(configType: ClassConfigType) {
  if (store.viewState.form.configType === configType) {
    return;
  }
  lookupKeyword.value = "";
  await store.setFilters({ configType, gradeName: "", keyword: "" });
}

function createNew() {
  lookupKeyword.value = "";
  store.resetForm(store.viewState.form.configType);
}

async function loadConfig(id: number) {
  await store.loadDetail(id);
  lookupKeyword.value = "";
}

async function saveCurrent() {
  if (store.viewState.editingId) {
    await store.update();
    return;
  }
  await store.create();
}

async function toggleSubject(subject: Subject) {
  const checked = !store.viewState.form.subjects.includes(subject);
  store.toggleSubject(subject, checked);
  if (store.viewState.editingId) {
    await saveCurrent();
  }
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
}

.class-picker {
  min-height: 92px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px;
  border-radius: 18px;
  gap: 16px;
}

.picker-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.picker-label {
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 600;
}

.picker-value {
  font-size: 22px;
  font-weight: 700;
}

.picker-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 10px;
}

.mode-pill {
  min-width: 112px;
  height: 40px;
  padding: 0 16px;
  border-radius: 14px;
  border: 1px solid var(--color-border-soft);
  background: rgba(255, 255, 255, 0.6);
  color: var(--color-text-muted);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.mode-pill.active {
  border-color: #b9d6ff;
  background: var(--color-brand-soft);
  color: var(--color-brand);
  font-weight: 700;
}

.subject-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.editor-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.metric-field.wide {
  grid-column: span 2;
}

.editor-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}

.picker-select {
  flex: 1 1 220px;
}

.picker-select :deep(.glass-field),
.picker-select input {
  width: 100%;
  min-height: 40px;
}

.lookup-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  width: 100%;
}

.lookup-item {
  padding: 8px 12px;
  border: 1px solid var(--color-border-soft);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.72);
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
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

.exam-room-copy {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 14px;
  line-height: 1.7;
}
</style>
