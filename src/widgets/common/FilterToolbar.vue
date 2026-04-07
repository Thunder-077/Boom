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
  min-height: 84px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px 20px;
  border-radius: var(--radius-card-large);
  background: var(--surface-toolbar-gradient);
}

.chip {
  height: 42px;
  min-width: 150px;
  padding: 0 14px;
  border: 1px solid var(--color-border-soft);
  border-radius: 16px;
  background: var(--surface-chip);
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
  color: var(--text-secondary);
}

.value {
  color: var(--text-primary);
  font-weight: 700;
}
</style>
