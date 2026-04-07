<template>
  <aside class="primary-rail card-shell">
    <div class="rail-top">
      <button
        type="button"
        class="toggle-btn"
        :aria-label="isSecondaryNavVisible ? '收起二级菜单' : '展开二级菜单'"
        :aria-pressed="isSecondaryNavVisible"
        :title="isSecondaryNavVisible ? '收起二级菜单' : '展开二级菜单'"
        @click="$emit('toggleSecondaryNav')"
      >
        <span class="material-symbols-rounded" aria-hidden="true">menu</span>
      </button>
    </div>
    <div class="rail-nav">
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        class="rail-btn"
        :class="{ active: item.key === activeKey }"
        :title="item.label"
        @click="$emit('select', item.key)"
      >
        <span class="icon material-symbols-rounded" aria-hidden="true">{{ item.icon }}</span>
        <span class="sr-only">{{ item.label }}</span>
      </button>
    </div>
    <div class="rail-footer">
      <button
        type="button"
        class="rail-btn utility-btn"
        :class="{ active: isSettingsActive }"
        title="系统设置"
        aria-label="打开系统设置"
        @click="$emit('openSettings')"
      >
        <span class="icon material-symbols-rounded" aria-hidden="true">settings</span>
        <span class="sr-only">系统设置</span>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { RailItem } from "./types";

defineProps<{
  items: RailItem[];
  activeKey: string;
  isSecondaryNavVisible: boolean;
  isSettingsActive?: boolean;
}>();

defineEmits<{
  select: [key: string];
  toggleSecondaryNav: [];
  openSettings: [];
}>();
</script>

<style scoped>
.primary-rail {
  width: 64px;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  border-radius: calc(var(--radius-card-large) - 6px);
  justify-content: flex-start;
  background: var(--surface-nav-gradient);
  border: 1px solid var(--border-default);
  box-shadow: 0 16px 36px rgba(31, 60, 103, 0.08);
  backdrop-filter: blur(18px);
  -webkit-backdrop-filter: blur(18px);
}

.rail-top {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px 0 8px;
}

.rail-nav {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding-top: 6px;
}

.rail-footer {
  width: 100%;
  margin-top: auto;
  padding-top: 8px;
  display: flex;
  justify-content: center;
}

.toggle-btn {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-default);
  border-radius: 12px;
  background: var(--surface-nav-panel);
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    background-color 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease,
    box-shadow 0.18s ease;
}

.toggle-btn:hover {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary-strong);
  border-color: rgba(var(--accent-rgb), 0.22);
}

.toggle-btn:focus-visible {
  outline: none;
  border-color: rgba(var(--accent-rgb), 0.3);
  box-shadow: 0 0 0 4px var(--accent-focus-ring);
}

.toggle-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
}

.rail-btn {
  position: relative;
  width: 40px;
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 14px;
  background: transparent;
  color: var(--text-secondary);
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
  inset: 7px;
  border-radius: 10px;
  background: var(--accent-radial-soft);
  opacity: 0;
  transform: scale(0.88);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.rail-btn:hover {
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--text-primary);
  transform: none;
}

.rail-btn:hover::after {
  opacity: 0.55;
  transform: scale(1);
}

.rail-btn:active {
  transform: scale(0.98);
}

.rail-btn:focus-visible {
  outline: none;
  border-color: rgba(var(--accent-rgb), 0.32);
  box-shadow: 0 0 0 4px var(--accent-focus-ring);
}

.icon {
  position: relative;
  z-index: 1;
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
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
  background: var(--accent-panel-strong);
  color: var(--accent-primary-strong);
  border-color: rgba(var(--accent-rgb), 0.26);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.7), 0 8px 18px rgba(var(--accent-rgb), 0.08);
  transform: none;
}

.utility-btn {
  margin-bottom: 2px;
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

.primary-rail::before,
.primary-rail::after {
  display: none;
}
</style>
