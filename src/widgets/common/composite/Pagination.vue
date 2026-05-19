<template>
  <div v-if="totalPages > 1" class="pagination">
    <span class="pagination-info">{{ infoText }}</span>
    <div class="pagination-actions">
      <Button
        variant="ghost"
        size="sm"
        :disabled="currentPage === 1"
        @click="goToPrevPage"
      >
        <span class="material-symbols-rounded">chevron_left</span>
        上一页
      </Button>

      <button
        v-for="page in visiblePages"
        :key="page"
        class="page-btn"
        :class="{ active: page === currentPage }"
        @click="goToPage(page)"
      >
        {{ page }}
      </button>

      <Button
        variant="ghost"
        size="sm"
        :disabled="currentPage === totalPages"
        @click="goToNextPage"
      >
        下一页
        <span class="material-symbols-rounded">chevron_right</span>
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Button from "../base/Button.vue";

const props = withDefaults(
  defineProps<{
    currentPage: number
    pageSize: number
    total: number
  }>(),
  {
    currentPage: 1,
    pageSize: 10,
    total: 0,
  }
);

const emit = defineEmits<{
  "update:currentPage": [page: number]
  "change": [page: number]
}>();

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)));

const infoText = computed(() => {
  if (props.total === 0) return "共 0 条";
  const start = (props.currentPage - 1) * props.pageSize + 1;
  const end = Math.min(props.currentPage * props.pageSize, props.total);
  return `共 ${props.total} 条，本页 ${start} - ${end}`;
});

const visiblePages = computed(() => {
  const current = props.currentPage;
  const total = totalPages.value;
  const pages = new Set([1, Math.max(1, current - 1), current, Math.min(total, current + 1), total]);
  return Array.from(pages).filter((p) => p >= 1 && p <= total).sort((a, b) => a - b);
});

function goToPage(page: number) {
  emit("update:currentPage", page);
  emit("change", page);
}

function goToPrevPage() {
  if (props.currentPage > 1) {
    goToPage(props.currentPage - 1);
  }
}

function goToNextPage() {
  if (props.currentPage < totalPages.value) {
    goToPage(props.currentPage + 1);
  }
}
</script>

<style scoped>
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-default);
  background: var(--surface-panel);
}

.pagination-info {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.pagination-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.page-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-xs);
  border: 1px solid var(--border-default);
  background: var(--surface-panel-strong);
  cursor: pointer;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-family: var(--font-ui);
  transition: all var(--transition-base) var(--transition-ease);
}

.page-btn:hover:not(.active) {
  background: var(--accent-fill-soft);
  border-color: var(--accent-border-strong);
  color: var(--accent-primary);
}

.page-btn.active {
  background: var(--accent-primary);
  color: var(--color-on-primary);
  border-color: var(--accent-primary);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
