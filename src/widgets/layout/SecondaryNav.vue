<template>
  <aside class="secondary-nav card-shell">
    <h2 class="title">{{ title }}</h2>
    <p class="desc">{{ description }}</p>
    <div class="list">
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        class="nav-item"
        :class="{ active: item.key === activeKey }"
        @click="$emit('select', item.key)"
      >
        <svg v-if="item.icon === 'assignment'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="M19 3h-3.18A3 3 0 0 0 13 2h-2a3 3 0 0 0-2.82 1H5v18h14V3Zm-7 1a1 1 0 0 1 1 1h-2a1 1 0 0 1 1-1Zm5 15H7V5h1v2h8V5h1v14ZM9 9h6v2H9V9Zm0 4h6v2H9v-2Z" />
        </svg>
        <svg v-else-if="item.icon === 'badge'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="M17 11V3H7v8H3l2 11h14l2-11h-4Zm-8-6h6v6H9V5Zm3 15a3 3 0 1 1 0-6 3 3 0 0 1 0 6Z" />
        </svg>
        <svg v-else-if="item.icon === 'settings'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="m19.14 12.94.04-.94-.04-.94 2.03-1.58-1.92-3.32-2.39.96a7.25 7.25 0 0 0-1.63-.94L14.96 3h-3.92l-.27 2.18c-.58.22-1.13.53-1.63.94l-2.39-.96-1.92 3.32 2.03 1.58-.04.94.04.94-2.03 1.58 1.92 3.32 2.39-.96c.5.41 1.05.72 1.63.94l.27 2.18h3.92l.27-2.18c.58-.22 1.13-.53 1.63-.94l2.39.96 1.92-3.32-2.03-1.58ZM13 15.5A3.5 3.5 0 1 1 13 8.5a3.5 3.5 0 0 1 0 7Z" />
        </svg>
        <svg v-else-if="item.icon === 'inventory_2'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="M20.54 5.23 19.15 3H4.85L3.46 5.23A2 2 0 0 0 3 6.29V19a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6.29a2 2 0 0 0-.46-1.06ZM5.96 5h12.08l.62 1H5.34l.62-1ZM5 8h14v11H5V8Zm5 2v5l4-2.5L10 10Z" />
        </svg>
        <svg v-else-if="item.icon === 'shuffle'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="M17 3h4v4h-2V6.41l-3.29 3.3-1.42-1.42L17.59 5H17V3ZM3 7h4.59l9 9H19v-1.59l2 2V20h-4.59l-9-9H3V7Zm0 10h4.59l2.7-2.71 1.42 1.42L7.59 19H3v-2Zm11.29-8.29L16.41 11 19 8.41V10h2V4h-6v2h1.59l-2.3 2.29Z" />
        </svg>
        <svg v-else-if="item.icon === 'tune'" viewBox="0 0 24 24" class="nav-icon" aria-hidden="true">
          <path d="M3 17v2h6v-2H3Zm0-6v2h10v-2H3Zm0-8v2h14V3H3Zm18 14v-2h-8v2h8Zm-4-10v2h4V7h-4Zm-6 12v-2h10v2H11Z" />
        </svg>
        <span v-else class="nav-icon placeholder" aria-hidden="true" />
        {{ item.label }}
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { SecondaryNavItem } from "./types";

defineProps<{
  title: string;
  description: string;
  items: SecondaryNavItem[];
  activeKey: string;
}>();

defineEmits<{
  select: [key: string];
}>();
</script>

<style scoped>
.secondary-nav {
  width: 232px;
  height: 924px;
  padding: 24px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-radius: 22px;
}

.title {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}

.desc {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 13px;
  line-height: 1.35;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 8px;
}

.nav-item {
  height: 52px;
  padding: 0 16px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.45);
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
  font-size: 15px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 12px;
}

.nav-item.active {
  color: var(--color-brand);
  border-color: #c5dcff;
  background: rgba(234, 243, 255, 0.8);
  font-weight: 600;
}

.nav-icon {
  width: 20px;
  height: 20px;
  fill: currentColor;
}

.placeholder {
  width: 20px;
  height: 20px;
}
</style>
