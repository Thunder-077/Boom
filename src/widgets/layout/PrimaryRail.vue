<template>
  <aside class="primary-rail card-shell">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="rail-btn"
      :class="{ active: item.key === activeKey }"
      @click="$emit('select', item.key)"
    >
      <svg v-if="item.icon === 'school'" viewBox="0 0 24 24" class="icon" aria-hidden="true">
        <path d="M12 3 2 8l10 5 8-4v6h2V8L12 3Zm-6.6 8.78V15c0 2.76 4.2 5 6.6 5s6.6-2.24 6.6-5v-3.22L12 15l-6.6-3.22Z" />
      </svg>
      <svg v-else-if="item.icon === 'co_present'" viewBox="0 0 24 24" class="icon" aria-hidden="true">
        <path d="M3 4h18v10H3V4Zm2 2v6h14V6H5Zm7 9 4 5h-8l4-5Zm-4-4a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" />
      </svg>
      <svg v-else-if="item.icon === 'domain'" viewBox="0 0 24 24" class="icon" aria-hidden="true">
        <path d="M3 21h18v-2h-1V9h1V7L12 2 3 7v2h1v10H3v2Zm3-2V9h3v10H6Zm5 0V9h2v10h-2Zm4 0V9h3v10h-3Z" />
      </svg>
      <svg v-else viewBox="0 0 24 24" class="icon" aria-hidden="true">
        <path d="M19 3h-1V1h-2v2H8V1H6v2H5a2 2 0 0 0-2 2v15a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2Zm0 17H5V10h14v10Zm-7-8H7v2h5v-2Zm5 0h-4v2h4v-2Zm-5 4H7v2h5v-2Z" />
      </svg>
      <span class="sr-only">{{ item.label }}</span>
    </button>
  </aside>
</template>

<script setup lang="ts">
import type { RailItem } from "./types";

defineProps<{
  items: RailItem[];
  activeKey: string;
}>();

defineEmits<{
  select: [key: string];
}>();
</script>

<style scoped>
.primary-rail {
  width: 76px;
  height: 924px;
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  border-radius: 22px;
}

.rail-btn {
  width: 52px;
  height: 52px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.4);
  color: var(--color-text-muted);
  cursor: pointer;
}

.icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.rail-btn.active {
  background: var(--color-brand-soft);
  color: var(--color-brand);
  border-color: #b9d6ff;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  border: 0;
}
</style>
