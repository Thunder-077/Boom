<template>
  <aside class="secondary-nav card-shell">
    <Transition name="nav-switch" appear mode="out-in">
      <div class="nav-content" :key="title">
        <div class="nav-head">
          <h2 class="title">{{ title }}</h2>
        </div>
        <div class="list">
          <button
            v-for="item in items"
            :key="item.key"
            type="button"
            class="nav-item"
            :class="{ active: item.key === activeKey }"
            @click="$emit('select', item.key)"
          >
            <span v-if="item.icon" class="nav-icon material-symbols-rounded" aria-hidden="true">{{ item.icon }}</span>
            <span v-else class="nav-icon placeholder" aria-hidden="true" />
            {{ item.label }}
          </button>
        </div>
      </div>
    </Transition>
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
  width: 248px;
  padding: var(--space-6) var(--space-md) var(--space-md);
  border-radius: 0;
  background: var(--surface-nav-panel);
  border-right: 1px solid var(--border-default);
  box-shadow: none;
  position: relative;
}

.secondary-nav::before,
.secondary-nav::after {
  display: none;
}

.nav-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  width: 100%;
  min-height: 100%;
}

.nav-head {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  padding: 2px 4px 0;
  position: relative;
}

.nav-head::after {
  content: "";
  width: 100%;
  height: 1px;
  background: var(--accent-divider);
  margin-top: var(--space-sm);
}

.eyebrow {
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 500;
  letter-spacing: 0.08em;
  text-transform: none;
}

.nav-switch-enter-active,
.nav-switch-leave-active {
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.nav-switch-enter-from {
  opacity: 0;
  transform: translateX(-4px);
}

.nav-switch-leave-to {
  opacity: 0;
  transform: translateX(4px);
}

.title {
  margin: 0;
  font-size: var(--font-size-xl);
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1.3;
}

.desc {
  margin: 0;
  color: color-mix(in srgb, var(--text-secondary) 72%, var(--text-primary));
  font-size: var(--font-size-xs);
  line-height: 1.45;
}

.meta-row {
  display: flex;
  align-items: center;
  padding-top: var(--space-xs);
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 4px 10px;
  border-radius: var(--radius-pill);
  background: rgba(var(--accent-rgb), 0.2);
  color: var(--accent-primary);
  font-size: var(--font-size-xs);
  font-weight: 700;
  letter-spacing: 0.04em;
}

.list {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.nav-item {
  position: relative;
  overflow: hidden;
  min-height: 36px;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  text-align: left;
  cursor: pointer;
  font-size: var(--font-size-sm);
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  transition:
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.nav-item::before {
  content: "";
  position: absolute;
  left: 0;
  top: 50%;
  width: 3px;
  height: 16px;
  border-radius: var(--radius-pill);
  background: var(--accent-primary);
  opacity: 0;
  transform: translateY(-50%) scaleX(0.5);
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.nav-item::after {
  content: "";
  position: absolute;
  inset: 0;
  background: transparent;
  opacity: 0;
}

.nav-item:hover {
  background: var(--surface-nav-item-hover);
  color: var(--text-primary);
}

.nav-item:hover::after {
  opacity: 1;
}

.nav-item:active {
  transform: none;
}

.nav-item:focus-visible {
  outline: none;
  background: var(--surface-nav-item-hover);
}

.nav-item.active {
  color: var(--text-primary);
  background: var(--surface-nav-item-active);
  font-weight: 600;
}

.nav-item.active .nav-icon {
  color: var(--accent-primary);
}

.nav-item.active::before {
  opacity: 1;
  transform: translateY(-50%) scaleX(1);
}

.nav-item.active::after {
  opacity: 1;
}

.nav-icon {
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
    "opsz" 18;
  opacity: 0.7;
}

.nav-item.active .nav-icon {
  opacity: 1;
}

.placeholder {
  width: 18px;
  height: 18px;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}
</style>
