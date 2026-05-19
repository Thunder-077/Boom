<template>
  <aside class="primary-rail card-shell">
    <div class="rail-top">
      <button
        type="button"
        class="toggle-btn"
        :aria-label="isSecondaryNavVisible ? '收起二级菜单' : '展开二级菜单'"
        :aria-pressed="isSecondaryNavVisible"
        :data-tooltip="isSecondaryNavVisible ? '收起二级菜单' : '展开二级菜单'"
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
        :aria-label="item.label"
        :data-tooltip="item.label"
        @click="$emit('select', item.key)"
      >
        <span class="hover-glow" aria-hidden="true" />
        <span class="icon material-symbols-rounded" aria-hidden="true">{{ item.icon }}</span>
        <span class="sr-only">{{ item.label }}</span>
      </button>
    </div>
    <div class="rail-footer">
      <button
        type="button"
        class="rail-btn utility-btn"
        :class="{ active: isSettingsActive }"
        aria-label="打开系统设置"
        data-tooltip="系统设置"
        @click="$emit('openSettings')"
      >
        <span class="hover-glow" aria-hidden="true" />
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
  position: relative;
  z-index: 3;
  overflow: visible;
  width: 60px;
  padding: var(--space-4) var(--space-2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-6);
  border-radius: 0;
  justify-content: flex-start;
  background: var(--surface-nav-panel);
  border-right: 1px solid var(--border-default);
  box-shadow: none;
}

.rail-top {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px 0 var(--space-md);
}

.rail-nav {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding-top: var(--space-2);
}

.rail-footer {
  width: 100%;
  margin-top: auto;
  padding-top: var(--space-4);
  display: flex;
  justify-content: center;
}

.toggle-btn {
  position: relative;
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.toggle-btn:hover {
  background: var(--surface-nav-item-hover);
  color: var(--text-primary);
}

.toggle-btn:focus-visible {
  outline: none;
  background: var(--surface-nav-item-hover);
}

.toggle-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
}

.toggle-btn::before,
.rail-btn::before {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  transition:
    opacity var(--transition-base) var(--transition-ease),
    transform var(--transition-base) var(--transition-ease);
}

.toggle-btn::before,
.rail-btn::before {
  content: attr(data-tooltip);
  left: calc(100% + 8px);
  top: 50%;
  transform: translate(2px, -50%);
  padding: 6px 9px;
  border-radius: var(--radius-sm);
  background: var(--surface-panel-strong);
  color: var(--text-primary);
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-medium);
  font-size: var(--font-size-caption);
  font-weight: 500;
  letter-spacing: 0;
  line-height: 1;
  white-space: nowrap;
  z-index: 18;
}

.toggle-btn:hover::before,
.toggle-btn:focus-visible::before,
.rail-btn:hover::before,
.rail-btn:focus-visible::before {
  opacity: 1;
  transform: translate(0, -50%);
}

.rail-btn {
  position: relative;
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  transition:
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.hover-glow {
  position: absolute;
  inset: 4px;
  border-radius: var(--radius-sm);
  background: var(--accent-radial-soft);
  opacity: 0;
  transform: scale(0.88);
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.rail-btn:hover {
  background: var(--surface-nav-item-hover);
  color: var(--text-secondary);
}

.rail-btn:hover .hover-glow {
  opacity: 0;
}

.rail-btn:active {
  transform: none;
}

.rail-btn:focus-visible {
  outline: none;
  background: var(--surface-nav-item-hover);
}

.icon {
  position: relative;
  z-index: 1;
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  line-height: 1;
  font-variation-settings:
    "FILL" 0,
    "wght" 400,
    "GRAD" 0,
    "opsz" 20;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}

.rail-btn.active {
  background: var(--surface-nav-item-active);
  color: var(--text-primary);
  box-shadow: none;
}

.utility-btn {
  margin-bottom: var(--space-2);
}

.rail-btn.active .hover-glow {
  opacity: 0;
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
