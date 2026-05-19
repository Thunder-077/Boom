<template>
  <section class="page-shell">
    <aside class="nav-stack" :class="{ collapsed: !isSecondaryNavVisible }">
      <PrimaryRail
        :items="railItems"
        :active-key="activeRail"
        :is-secondary-nav-visible="isSecondaryNavVisible"
        :is-settings-active="isSettingsActive"
        @select="handleRailSelect"
        @toggle-secondary-nav="toggleSecondaryNav"
        @open-settings="$emit('openSettings')"
      />
      <Transition
        :css="false"
        @before-enter="onBeforeSecondaryEnter"
        @enter="onSecondaryEnter"
        @after-enter="onAfterSecondaryEnter"
        @before-leave="onBeforeSecondaryLeave"
        @leave="onSecondaryLeave"
        @after-leave="onAfterSecondaryLeave"
      >
        <div v-if="isSecondaryNavVisible" class="secondary-nav-wrapper">
          <SecondaryNav
            :title="secondaryTitle"
            :description="secondaryDescription"
            :items="secondaryItems"
            :active-key="activeSecondary"
            @select="$emit('selectSecondary', $event)"
          />
        </div>
      </Transition>
    </aside>
    <main class="content-wrap">
      <slot />
    </main>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import PrimaryRail from "./PrimaryRail.vue";
import SecondaryNav from "./SecondaryNav.vue";
import type { RailItem, SecondaryNavItem } from "./types";

const props = defineProps<{
  railItems: RailItem[];
  activeRail: string;
  secondaryTitle: string;
  secondaryDescription: string;
  secondaryItems: SecondaryNavItem[];
  activeSecondary: string;
  isSettingsActive?: boolean;
}>();

const emit = defineEmits<{
  selectRail: [key: string];
  selectSecondary: [key: string];
  openSettings: [];
}>();

const isSecondaryNavVisible = ref(true);
const SECONDARY_NAV_WIDTH = 260;
const SECONDARY_NAV_DURATION_MS = 220;

function handleRailSelect(key: string) {
  if (key === props.activeRail) {
    isSecondaryNavVisible.value = !isSecondaryNavVisible.value;
  } else {
    isSecondaryNavVisible.value = true;
    emit("selectRail", key);
  }
}

function toggleSecondaryNav() {
  isSecondaryNavVisible.value = !isSecondaryNavVisible.value;
}

function onBeforeSecondaryEnter(element: Element) {
  const el = element as HTMLElement;
  el.style.width = "0px";
  el.style.opacity = "0";
}

function onSecondaryEnter(element: Element, done: () => void) {
  const el = element as HTMLElement;
  const animation = el.animate(
    [
      { width: "0px", opacity: 0 },
      { width: `${SECONDARY_NAV_WIDTH}px`, opacity: 1 },
    ],
    {
      duration: SECONDARY_NAV_DURATION_MS,
      easing: "cubic-bezier(0.22, 1, 0.36, 1)",
      fill: "forwards",
    },
  );
  animation.finished.then(done).catch(done);
}

function onAfterSecondaryEnter(element: Element) {
  const el = element as HTMLElement;
  el.style.width = "";
  el.style.opacity = "";
}

function onBeforeSecondaryLeave(element: Element) {
  const el = element as HTMLElement;
  el.style.width = `${el.offsetWidth || SECONDARY_NAV_WIDTH}px`;
  el.style.opacity = "1";
}

function onSecondaryLeave(element: Element, done: () => void) {
  const el = element as HTMLElement;
  const fromWidth = el.offsetWidth || SECONDARY_NAV_WIDTH;
  const animation = el.animate(
    [
      { width: `${fromWidth}px`, opacity: 1 },
      { width: "0px", opacity: 0 },
    ],
    {
      duration: SECONDARY_NAV_DURATION_MS,
      easing: "cubic-bezier(0.4, 0, 0.2, 1)",
      fill: "forwards",
    },
  );
  animation.finished.then(done).catch(done);
}

function onAfterSecondaryLeave(element: Element) {
  const el = element as HTMLElement;
  el.style.width = "";
  el.style.opacity = "";
}
</script>

<style scoped>
.page-shell {
  flex: 1;
  width: 100%;
  margin: 0;
  min-height: 0;
  display: flex;
  padding: 0 24px 0 0;
  align-items: stretch;
  box-sizing: border-box;
}

.nav-stack {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
  align-self: stretch;
  min-height: 0;
  border-radius: 0;
  box-shadow: none;
}

.nav-stack.collapsed {
  border-right: 1px solid #e5e7eb;
}

.secondary-nav-wrapper {
  overflow: hidden;
  display: flex;
  height: 100%;
  min-width: 0;
  width: 260px;
  will-change: width, opacity;
  border-right: 1px solid #e5e7eb;
}

.content-wrap {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px 0 32px 24px;
  overflow-x: hidden;
  overflow-y: auto;
}

.content-wrap > :deep(*) {
  min-width: 0;
  max-width: 100%;
}

@media (max-width: 1280px) {
  .page-shell {
    padding: 0 16px 0 0;
  }

  .content-wrap {
    padding: 20px 0 28px 16px;
  }
}

@media (max-width: 1100px) {
  .page-shell {
    padding: 0 12px 0 0;
  }

  .content-wrap {
    padding: 16px 0 24px 12px;
  }
}
</style>
