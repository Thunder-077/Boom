<template>
  <section class="panel" :class="{ dragging: isDragging }">
    <FilterToolbar :items="[]">
      <div class="toolbar-fields">
        <label>
          <select class="glass-field title-select">
            <option>考试标题</option>
          </select>
        </label>
        <label class="search-field">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79L19 20.5 20.5 19 15.5 14ZM9.5 14A4.5 4.5 0 1 1 14 9.5 4.5 4.5 0 0 1 9.5 14Z" />
          </svg>
          <input class="glass-field" :value="store.viewState.filters.nameKeyword" placeholder="按姓名筛选" @input="onNameInput" />
        </label>
      </div>
    </FilterToolbar>

    <InfoHint text="可将 Excel 文件拖拽到页面任意位置导入成绩数据" />

    <TableCard title="考试成绩列表" :meta="`已同步 ${store.viewState.total} 条`">
      <table class="table score-table">
        <thead>
          <tr>
            <th>姓名</th>
            <th>准考证号</th>
            <th>班级</th>
            <th>科目</th>
            <th>分数</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, index) in store.viewState.rows.slice(0, 8)" :key="row.admissionNo" :class="rowClass(index)">
            <td :class="{ emphasis: index === 2 }">{{ row.studentName }}</td>
            <td>{{ row.admissionNo }}</td>
            <td>{{ row.className }}</td>
            <td>总分</td>
            <td class="score-cell" :class="{ highlight: index === 2 }">{{ row.totalScore.toFixed(0) }}</td>
            <td class="link-cell">查看 / 编辑</td>
          </tr>
        </tbody>
      </table>
    </TableCard>
  </section>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import FilterToolbar from "../../../widgets/common/FilterToolbar.vue";
import InfoHint from "../../../widgets/common/InfoHint.vue";
import TableCard from "../../../widgets/common/TableCard.vue";
import { useScoreStore } from "../store";

const store = useScoreStore();
const isDragging = ref(false);
let unlistenDragDrop: (() => void) | null = null;

function rowClass(index: number) {
  if (index === 2) {
    return "row-highlight";
  }
  return index % 2 === 1 ? "row-alt" : "";
}

function onNameInput(event: Event) {
  void store.setFilters({ nameKeyword: (event.target as HTMLInputElement).value });
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
      const excelFilePath = event.payload.paths.find((path) => path.endsWith(".xlsx") || path.endsWith(".xls"));
      if (excelFilePath) {
        void handleImport(excelFilePath);
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

.title-select {
  width: 220px;
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
  width: 220px;
  padding-left: 42px;
}

.score-table tbody tr {
  height: 58px;
}

.score-cell {
  font-size: 18px;
  font-weight: 700;
}

.score-cell.highlight,
.link-cell {
  color: var(--color-brand);
}

.emphasis {
  font-weight: 600;
}

.row-alt {
  background: #f8fbff;
}

.row-highlight {
  background: rgba(234, 243, 255, 0.6);
}
</style>
