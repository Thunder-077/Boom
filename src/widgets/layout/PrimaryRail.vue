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
      <span class="icon material-symbols-rounded" aria-hidden="true">{{ item.icon }}</span>
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
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  border-radius: 22px;
}

.rail-btn {
  position: relative;
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
  transition:
    background-color 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease,
    transform 0.18s ease,
    box-shadow 0.18s ease;
}

.rail-btn::after {
  content: "";
  position: absolute;
  inset: 8px;
  border-radius: 12px;
  background: radial-gradient(circle, rgba(15, 108, 189, 0.18), rgba(15, 108, 189, 0));
  opacity: 0;
  transform: scale(0.88);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.rail-btn:hover {
  background: rgba(255, 255, 255, 0.58);
  color: #49566a;
  transform: translateY(-1px);
}

.rail-btn:hover::after {
  opacity: 0.55;
  transform: scale(1);
}

.rail-btn:active {
  transform: scale(0.97);
}

.rail-btn:focus-visible {
  outline: none;
  border-color: #b9d6ff;
  box-shadow: 0 0 0 4px rgba(185, 214, 255, 0.32);
}

.icon {
  position: relative;
  z-index: 1;
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  line-height: 1;
  font-variation-settings:
    "FILL" 0,
    "wght" 500,
    "GRAD" 0,
    "opsz" 24;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}

.rail-btn.active {
  background: var(--color-brand-soft);
  color: var(--color-brand);
  border-color: #b9d6ff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.45), 0 10px 22px rgba(15, 108, 189, 0.12);
  transform: translateY(-1px);
}

.rail-btn.active::after {
  opacity: 0.8;
  transform: scale(1);
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
