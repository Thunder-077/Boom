<template>
  <aside class="secondary-nav">
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
  width: 240px;
  padding: var(--space-lg) var(--space-md) var(--space-md);
  background: var(--surface-nav-panel);
  border-right: 1px solid var(--border-default);
  flex-shrink: 0;
}

.nav-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  width: 100%;
  min-height: 100%;
}

.nav-head {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  padding: 2px 4px var(--space-md);
  border-bottom: 1px solid var(--border-default);
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
  color: var(--text-primary);
}

.list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding-top: var(--space-xs);
}

.nav-item {
  position: relative;
  overflow: hidden;
  min-height: 34px;
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
  height: 14px;
  border-radius: var(--radius-pill);
  background: var(--accent-primary);
  opacity: 0;
  transform: translateY(-50%) scaleX(0.5);
  transition: opacity var(--transition-base) var(--transition-ease), transform var(--transition-base) var(--transition-ease);
}

.nav-item:hover {
  background: var(--surface-nav-item-hover);
  color: var(--text-primary);
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
  opacity: 1;
}

.nav-item.active::before {
  opacity: 1;
  transform: translateY(-50%) scaleX(1);
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
  opacity: 0.55;
}

.placeholder {
  width: 18px;
  height: 18px;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}
</style>
