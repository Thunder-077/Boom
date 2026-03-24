<template>
  <section class="panel">
    <div class="class-picker card-shell">
      <div class="picker-copy">
        <span class="picker-label">当前配置班级</span>
        <strong class="picker-value">{{ currentRow?.className || "暂无班级" }}</strong>
      </div>
      <label class="picker-select">
        <select class="glass-field" :value="String(store.viewState.selectedId ?? '')" @change="onSelectClass">
          <option value="">切换班级</option>
          <option v-for="row in visibleRows" :key="row.id" :value="row.id">{{ row.className }}</option>
        </select>
      </label>
    </div>

    <ConfigCard :title="'所学科目配置'" :description="subjectDescription">
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

    <ConfigCard title="楼号与楼层信息">
      <div class="metric-grid">
        <label class="metric-field">
          <span class="metric-label">班级楼号</span>
          <input class="metric-input" :value="store.viewState.form.building" @input="onFormInput('building', $event)" @blur="saveCurrent" />
        </label>
        <label class="metric-field">
          <span class="metric-label">楼层信息</span>
          <input class="metric-input" :value="store.viewState.form.floor" @input="onFormInput('floor', $event)" @blur="saveCurrent" />
        </label>
      </div>
    </ConfigCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import type { Subject } from "../../../entities/score/model";
import { SUBJECT_OPTIONS } from "../../../entities/class-config/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useClassConfigStore } from "../store";

const store = useClassConfigStore();

const visibleRows = computed(() => store.viewState.rows.filter((row) => row.configType === "teaching_class"));
const currentRow = computed(() => visibleRows.value.find((row) => row.id === store.viewState.selectedId) ?? visibleRows.value[0] ?? null);
const visibleSubjects = computed(() => SUBJECT_OPTIONS.slice(0, 6));
const subjectDescription = computed(
  () => `点击科目标签可启用或取消，当前班级已选择 ${store.viewState.form.subjects.length} 门课程。`,
);

function onFormInput(field: "building" | "floor", event: Event) {
  const value = (event.target as HTMLInputElement).value;
  store.setFormField(field, value);
}

function onSelectClass(event: Event) {
  const value = Number((event.target as HTMLSelectElement).value);
  if (!Number.isNaN(value) && value > 0) {
    void store.loadDetail(value);
  }
}

async function saveCurrent() {
  if (store.viewState.editingId) {
    await store.update();
  }
}

async function toggleSubject(subject: Subject) {
  const checked = !store.viewState.form.subjects.includes(subject);
  store.toggleSubject(subject, checked);
  await saveCurrent();
}

onMounted(async () => {
  await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
  if (visibleRows.value.length > 0) {
    await store.loadDetail(visibleRows.value[0].id);
  }
});
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.class-picker {
  height: 74px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  border-radius: 18px;
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

.picker-select select {
  width: 154px;
  min-height: 40px;
}

.subject-row {
  display: flex;
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

.metric-grid {
  display: flex;
  gap: 14px;
}

.metric-grid label {
  width: 220px;
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
</style>
