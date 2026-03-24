<template>
  <section class="panel" :class="{ dragging: isDragging }">
    <FilterToolbar :items="[]">
      <div class="toolbar-fields">
        <label class="search-field">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79L19 20.5 20.5 19 15.5 14ZM9.5 14A4.5 4.5 0 1 1 14 9.5 4.5 4.5 0 0 1 9.5 14Z" />
          </svg>
          <input class="glass-field" :value="store.viewState.filters.nameKeyword" placeholder="按教师姓名查询" @input="onNameInput" />
        </label>
        <label class="select-field">
          <select class="glass-field" :value="store.viewState.filters.className" @change="onClassChange">
            <option value="">班级</option>
            <option v-for="className in classOptions" :key="className" :value="className">{{ className }}</option>
          </select>
          <span class="material-symbols-rounded select-icon" aria-hidden="true">keyboard_arrow_down</span>
        </label>
        <label class="select-field">
          <select class="glass-field" :value="store.viewState.filters.subject" @change="onSubjectChange">
            <option v-for="option in TEACHER_SUBJECT_OPTIONS" :key="option.value || 'all'" :value="option.value">{{ option.label }}</option>
          </select>
          <span class="material-symbols-rounded select-icon" aria-hidden="true">keyboard_arrow_down</span>
        </label>
      </div>
    </FilterToolbar>

    <InfoHint text="教师可关联多个班级，拖拽 Excel 到页面任意位置可导入教师数据并按班级筛选" />

    <TableCard title="教师列表" :meta="`共 ${store.viewState.total} 位`">
      <div class="table-scroll">
        <div class="teacher-grid teacher-grid-head">
          <div class="cell head name-col">姓名</div>
          <div class="cell head class-col">班级</div>
          <div class="cell head subject-col">教学科目</div>
        </div>
        <div
          v-for="(row, index) in store.viewState.rows"
          :key="row.id"
          class="teacher-grid teacher-grid-row"
          :class="{ 'row-alt': index % 2 === 1 }"
        >
          <div class="cell name-col teacher-name">{{ row.teacherName }}</div>
          <div class="cell class-col">
            <div class="tag-row">
              <span v-for="className in row.classNames" :key="className" class="tag-pill">{{ className }}</span>
            </div>
          </div>
          <div class="cell subject-col">{{ row.subjects.map((subject) => TEACHER_SUBJECT_LABELS[subject]).join(" / ") }}</div>
        </div>
      </div>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TEACHER_SUBJECT_LABELS, type TeacherSubject } from "../../../entities/teacher/model";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { TEACHER_SUBJECT_OPTIONS, useTeacherStore } from "../store";

const store = useTeacherStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;

const classOptions = computed(() =>
  Array.from(new Set(store.viewState.rows.flatMap((row) => row.classNames))).sort((a, b) => a.localeCompare(b, "zh-CN")),
);

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
}

function onClassChange(event: Event) {
  void store.setFilters({ className: (event.target as HTMLSelectElement).value });
}

function onSubjectChange(event: Event) {
  const subject = (event.target as HTMLSelectElement).value as TeacherSubject | "";
  void store.setFilters({ subject });
}

async function handleImport(filePath: string) {
  if (!filePath) {
    return;
  }
  try {
    await store.importExcel(filePath);
  } catch {
    // Import status is already persisted in store.
  }
}

onMounted(async () => {
  await store.load();
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
      const excelPath = event.payload.paths.find((path) => path.endsWith(".xlsx") || path.endsWith(".xls"));
      if (excelPath) {
        void handleImport(excelPath);
      }
    }
  });
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
  gap: 22px;
}

.panel.dragging :deep(.toolbar) {
  border-color: #b9d6ff;
  background: rgba(232, 242, 255, 0.92);
}

.toolbar-fields {
  display: flex;
  gap: 12px;
}

.toolbar-fields label {
  display: block;
}

.search-field {
  position: relative;
}

.search-field svg {
  position: absolute;
  left: 14px;
  top: 12px;
  width: 18px;
  height: 18px;
  fill: var(--color-text-muted);
}

.search-field input {
  width: 260px;
  padding-left: 42px;
}

.select-field {
  position: relative;
}

.select-field .glass-field {
  width: 150px;
  appearance: none;
  padding-right: 36px;
}

.select-icon {
  position: absolute;
  right: 12px;
  top: 12px;
  color: var(--color-text-muted);
  font-size: 18px;
  pointer-events: none;
  font-family: "Material Symbols Rounded";
}

.table-scroll {
  max-height: 264px;
  overflow: auto;
}

.teacher-grid {
  display: grid;
  grid-template-columns: 220px 1fr 180px;
  align-items: center;
}

.teacher-grid-head {
  min-height: 48px;
  background: #fafcff;
  border-bottom: 1px solid var(--color-border-soft);
}

.teacher-grid-row {
  min-height: 72px;
  background: #fff;
  border-bottom: 1px solid var(--color-border-soft);
}

.cell {
  min-height: inherit;
  display: flex;
  align-items: center;
  padding: 0 18px;
  font-size: 14px;
  color: var(--color-text);
}

.head {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.teacher-name {
  font-weight: 600;
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 10px;
  overflow: hidden;
  white-space: nowrap;
}

.row-alt {
  background: #f8fbff;
}
</style>
