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
        <label>
          <select class="glass-field" :value="store.viewState.filters.className" @change="onClassChange">
            <option value="">班级</option>
            <option v-for="className in classOptions" :key="className" :value="className">{{ className }}</option>
          </select>
        </label>
        <label>
          <input class="glass-field" value="班级" disabled />
        </label>
      </div>
    </FilterToolbar>

    <InfoHint text="教师可关联多个班级，拖拽 Excel 到页面任意位置可导入教师数据并按班级筛选" />

    <TableCard title="教师列表" :meta="`共 ${store.viewState.total} 位`">
      <table class="table teacher-table">
        <thead>
          <tr>
            <th>姓名</th>
            <th>班级</th>
            <th>教学科目</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, index) in store.viewState.rows.slice(0, 8)" :key="row.id" :class="rowClass(index)">
            <td class="teacher-name">{{ row.teacherName }}</td>
            <td>
              <div class="tag-row">
                <span v-for="className in row.classNames" :key="className" class="tag-pill">{{ className }}</span>
              </div>
            </td>
            <td>{{ row.subjects.map((subject) => TEACHER_SUBJECT_LABELS[subject]).join(" / ") }}</td>
          </tr>
        </tbody>
      </table>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { TEACHER_SUBJECT_LABELS } from "../../../entities/teacher/model";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useTeacherStore } from "../store";

const store = useTeacherStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;

const classOptions = computed(() =>
  Array.from(new Set(store.viewState.rows.flatMap((row) => row.classNames))).sort((a, b) => a.localeCompare(b, "zh-CN")),
);

function rowClass(index: number) {
  if (index === 2) {
    return "row-highlight";
  }
  return index % 2 === 1 ? "row-alt" : "";
}

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
}

function onClassChange(event: Event) {
  void store.setFilters({ className: (event.target as HTMLSelectElement).value });
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

.toolbar-fields select,
.toolbar-fields input {
  width: 150px;
}

.teacher-table tbody tr {
  height: 72px;
}

.teacher-table td:first-child,
.teacher-table th:first-child {
  padding-left: 18px;
}

.teacher-name {
  font-weight: 600;
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 10px;
  overflow: hidden;
}

.row-alt {
  background: #f8fbff;
}

.row-highlight {
  background: rgba(234, 243, 255, 0.6);
}
</style>
