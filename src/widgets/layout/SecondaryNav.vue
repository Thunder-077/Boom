<template>
  <aside class="secondary-nav card-shell">
    <Transition name="nav-switch" appear mode="out-in">
      <div class="nav-content" :key="title">
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
  width: 232px;
  padding: 24px 18px;
  border-radius: 22px;
  box-shadow: 
    0 1px 0 rgba(255, 255, 255, 0.82) inset,
    0 0 0 1px rgba(255, 255, 255, 0.16);
  transition: all 0.25s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.secondary-nav:hover {
  box-shadow: 
    0 1px 0 rgba(255, 255, 255, 0.88) inset,
    0 0 0 1px rgba(197, 220, 255, 0.3);
}

.nav-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
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
  position: relative;
  overflow: hidden;
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
  background: linear-gradient(180deg, #0f6cbd, #2e86de);
  opacity: 0;
  transform: translateY(-50%) scaleY(0.4);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.nav-item::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, rgba(15, 108, 189, 0.08), rgba(255, 255, 255, 0));
  opacity: 0;
  transition: opacity 0.18s ease;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.58);
  color: #49566a;
  transform: translateX(2px);
}

.nav-item:hover::after {
  opacity: 1;
}

.nav-item:active {
  transform: translateX(1px) scale(0.99);
}

.nav-item:focus-visible {
  outline: none;
  border-color: #b9d6ff;
  box-shadow: 0 0 0 4px rgba(185, 214, 255, 0.3);
}

.nav-item.active {
  color: var(--color-brand);
  border-color: #b9d6ff;
  background: var(--color-brand-soft);
  font-weight: 600;
  transform: translateX(3px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.45),
    0 10px 22px rgba(15, 108, 189, 0.12);
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
