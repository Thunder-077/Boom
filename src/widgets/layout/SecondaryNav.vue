<template>
  <aside class="secondary-nav card-shell">
    <Transition name="nav-switch" appear mode="out-in">
      <div class="nav-content" :key="title">
        <div class="nav-head">
          <h2 class="title">{{ title }}</h2>
          <p class="desc">{{ description }}</p>
          <div class="meta-row">
            <span class="meta-pill">{{ items.length }} 个工作区</span>
          </div>
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
  padding: 18px 14px 14px;
  border-radius: var(--radius-card-large);
  background: var(--surface-nav-panel);
  border: 1px solid var(--border-default);
  box-shadow: 0 12px 28px rgba(31, 60, 103, 0.06);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  position: relative;
}

.secondary-nav::before,
.secondary-nav::after {
  display: none;
}

.nav-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: 100%;
  min-height: 100%;
}

.nav-head {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 2px 4px 0;
  position: relative;
}

.nav-head::after {
  content: "";
  width: 100%;
  height: 1px;
  background: var(--accent-divider);
  margin-top: 4px;
}

.eyebrow {
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.nav-switch-enter-active,
.nav-switch-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.nav-switch-enter-from {
  opacity: 0;
  transform: translateX(-8px);
}

.nav-switch-leave-to {
  opacity: 0;
  transform: translateX(8px);
}

.title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.desc {
  margin: 0;
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.meta-row {
  display: flex;
  align-items: center;
  padding-top: 2px;
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(var(--accent-rgb), 0.1);
  color: var(--accent-primary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  position: relative;
  overflow: hidden;
  min-height: 50px;
  padding: 0 14px 0 16px;
  border-radius: 16px;
  border: 1px solid var(--border-default);
  background: var(--surface-nav-item);
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 12px;
  transition:
    background-color 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease,
    transform 0.18s ease,
    box-shadow 0.18s ease;
}

.nav-item::before {
  content: "";
  position: absolute;
  left: 8px;
  top: 50%;
  width: 4px;
  height: 24px;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--accent-primary-strong), var(--accent-primary));
  opacity: 0;
  transform: translateY(-50%) scaleY(0.4);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.nav-item::after {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--accent-sheen);
  opacity: 0;
  transition: opacity 0.18s ease;
}

.nav-item:hover {
  background: var(--surface-nav-item-hover);
  color: var(--text-primary);
  transform: none;
}

.nav-item:hover::after {
  opacity: 1;
}

.nav-item:active {
  transform: scale(0.99);
}

.nav-item:focus-visible {
  outline: none;
  border-color: var(--accent-border-strong);
  box-shadow: 0 0 0 4px var(--accent-focus-ring);
}

.nav-item.active {
  color: var(--accent-primary-strong);
  border-color: var(--accent-border-soft);
  background: var(--surface-nav-item-active);
  transform: none;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.58),
    0 8px 18px rgba(var(--accent-rgb), 0.06);
}

.nav-item.active::before {
  opacity: 1;
  transform: translateY(-50%) scaleY(1);
}

.nav-item.active::after {
  opacity: 1;
}

.nav-icon {
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
    "opsz" 20;
}

.placeholder {
  width: 20px;
  height: 20px;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}
</style>
