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
  min-height: 78px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.72);
  border-color: rgba(255, 255, 255, 0.85);
}

.chip {
  height: 42px;
  min-width: 150px;
  padding: 0 14px;
  border: 1px solid var(--color-border-soft);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.58);
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
  color: var(--color-text-muted);
}

.value {
  color: var(--color-text);
  font-weight: 600;
}
</style>
