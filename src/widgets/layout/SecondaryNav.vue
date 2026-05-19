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
  width: 260px;
  height: 100%;
  padding: 0;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border-left: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.nav-content {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-height: 100%;
}

.nav-head {
  padding: 24px 20px 16px;
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
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 15px;
  font-weight: 600;
  color: #6b7280;
  line-height: 1.3;
  letter-spacing: 0.01em;
}

.list {
  padding: 8px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  width: 100%;
  min-height: 36px;
  padding: 8px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #4b5563;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: background 0.15s ease, color 0.15s ease;
}

.nav-item:hover {
  background: #f3f4f6;
  color: #111827;
}

.nav-item:hover .nav-icon {
  color: #6b7280;
}

.nav-item:active {
  transform: none;
}

.nav-item:focus-visible {
  outline: none;
  background: #f3f4f6;
}

.nav-item.active {
  background: #f3f4f6;
  color: #111827;
  font-weight: 600;
}

.nav-item.active .nav-icon {
  color: #374151;
  font-variation-settings:
    "FILL" 1,
    "wght" 500,
    "GRAD" 0,
    "opsz" 18;
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
  color: #9ca3af;
  transition: color 0.15s ease;
}

.placeholder {
  width: 18px;
  height: 18px;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}
</style>
