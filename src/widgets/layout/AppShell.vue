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
const SECONDARY_NAV_WIDTH = 248;
const SECONDARY_NAV_DURATION_MS = 260;

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
  el.style.transform = "translateX(-10px) scaleX(0.98)";
}

function onSecondaryEnter(element: Element, done: () => void) {
  const el = element as HTMLElement;
  const animation = el.animate(
    [
      { width: "0px", opacity: 0, transform: "translateX(-10px) scaleX(0.98)" },
      { width: `${SECONDARY_NAV_WIDTH}px`, opacity: 1, transform: "translateX(0) scaleX(1)" },
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
  el.style.transform = "";
}

function onBeforeSecondaryLeave(element: Element) {
  const el = element as HTMLElement;
  el.style.width = `${el.offsetWidth || SECONDARY_NAV_WIDTH}px`;
  el.style.opacity = "1";
  el.style.transform = "translateX(0) scaleX(1)";
}

function onSecondaryLeave(element: Element, done: () => void) {
  const el = element as HTMLElement;
  const fromWidth = el.offsetWidth || SECONDARY_NAV_WIDTH;
  // Use WAAPI so the sidebar and content area reflow smoothly together during collapse.
  const animation = el.animate(
    [
      { width: `${fromWidth}px`, opacity: 1, transform: "translateX(0) scaleX(1)" },
      { width: "0px", opacity: 0, transform: "translateX(-10px) scaleX(0.98)" },
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
  el.style.transform = "";
}
</script>

<style scoped>
.page-shell {
  width: min(100%, 1560px);
  margin: 0 auto;
  min-height: 100%;
  display: flex;
  gap: 18px;
  padding: 30px 28px 36px;
  align-items: stretch;
}

.nav-stack {
  display: flex;
  align-items: stretch;
  gap: 10px;
  flex-shrink: 0;
  transition: gap 0.22s ease;
}

.nav-stack.collapsed {
  gap: 0;
}

.secondary-nav-wrapper {
  overflow: hidden;
  display: flex;
  min-width: 0;
  width: 248px;
  will-change: width, opacity, transform;
}

.content-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

@media (max-width: 1280px) {
  .page-shell {
    padding-inline: 18px;
  }
}

@media (max-width: 1100px) {
  .page-shell {
    gap: 14px;
    padding: 18px 14px 24px;
  }

  .nav-stack {
    gap: 8px;
  }
}
</style>
