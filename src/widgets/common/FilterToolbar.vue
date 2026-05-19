<template>
  <section class="toolbar card-shell">
    <slot>
      <div v-for="item in items" :key="item.label" class="chip">
        <span>{{ item.label }}</span>
        <span class="value">{{ item.value }}</span>
      </div>
    </slot>
  </section>
</template>

<script setup lang="ts">
export interface ToolbarItem {
  label: string;
  value: string;
}

defineProps<{
  items: ToolbarItem[];
}>();
</script>

<style scoped>
.toolbar {
  min-height: 56px;
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-card-large);
}

.chip {
  height: 36px;
  min-width: 140px;
  padding: 0 var(--space-md);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-chip);
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.value {
  color: var(--text-primary);
  font-weight: 600;
}
</style>
