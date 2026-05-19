<template>
  <aside class="primary-rail">
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
    <div class="nav-group">
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
        <span class="icon material-symbols-rounded" aria-hidden="true">{{ item.icon }}</span>
        <span class="sr-only">{{ item.label }}</span>
      </button>
    </div>
    <div class="nav-bottom">
      <button
        type="button"
        class="rail-btn"
        :class="{ active: isSettingsActive }"
        aria-label="打开系统设置"
        data-tooltip="系统设置"
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
  position: relative;
  z-index: 2;
  width: 72px;
  height: 100%;
  padding: 24px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  background: #ffffff;
  border-right: 1px solid #f0f0f0;
  flex-shrink: 0;
}

.rail-top {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-group {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.nav-bottom {
  width: 100%;
  margin-top: auto;
  display: flex;
  justify-content: center;
}

.toggle-btn {
  position: relative;
  width: 44px;
  height: 44px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease;
  margin-bottom: 20px;
}

.toggle-btn:hover {
  background-color: #f0f0f1;
  color: #374151;
}

.toggle-btn:focus-visible {
  outline: none;
  background-color: #f0f0f1;
  color: #374151;
}

.toggle-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 22px;
}

.toggle-btn::before {
  content: attr(data-tooltip);
  position: absolute;
  left: calc(100% + 8px);
  top: 50%;
  transform: translate(4px, -50%);
  opacity: 0;
  pointer-events: none;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-panel);
  color: var(--text-primary);
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-medium);
  font-size: var(--font-size-caption);
  font-weight: 500;
  line-height: 1;
  white-space: nowrap;
  z-index: 18;
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.toggle-btn:hover::before,
.toggle-btn:focus-visible::before {
  opacity: 1;
  transform: translate(0, -50%);
}

.rail-btn {
  position: relative;
  width: 44px;
  height: 44px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease;
}

.rail-btn::before {
  content: attr(data-tooltip);
  position: absolute;
  left: calc(100% + 8px);
  top: 50%;
  transform: translate(4px, -50%);
  opacity: 0;
  pointer-events: none;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-panel);
  color: var(--text-primary);
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-medium);
  font-size: var(--font-size-caption);
  font-weight: 500;
  line-height: 1;
  white-space: nowrap;
  z-index: 18;
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.rail-btn:hover::before,
.rail-btn:focus-visible::before {
  opacity: 1;
  transform: translate(0, -50%);
}

.rail-btn:hover {
  background: #f0f0f1;
  color: #374151;
}

.rail-btn:active {
  transform: none;
}

.rail-btn:focus-visible {
  outline: none;
  background: #f0f0f1;
  color: #374151;
}

.rail-btn.active {
  background: #f0f0f1;
  color: #111827;
  box-shadow: none;
}

.rail-btn.active::after {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  border-radius: 0 3px 3px 0;
  background: #3b82f6;
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
    "wght" 400,
    "GRAD" 0,
    "opsz" 20;
}

.rail-btn.active .icon {
  color: #111827;
  font-variation-settings:
    "FILL" 1,
    "wght" 500,
    "GRAD" 0,
    "opsz" 20;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
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
