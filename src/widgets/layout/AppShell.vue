<template>
  <section class="page-shell">
    <PrimaryRail :items="railItems" :active-key="activeRail" @select="handleRailSelect" />
    <Transition name="rail-slide">
      <div v-show="isSecondaryNavVisible" class="secondary-nav-wrapper">
        <SecondaryNav
          :title="secondaryTitle"
          :description="secondaryDescription"
          :items="secondaryItems"
          :active-key="activeSecondary"
          @select="$emit('selectSecondary', $event)"
        />
      </div>
    </Transition>
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
}>();

const emit = defineEmits<{
  selectRail: [key: string];
  selectSecondary: [key: string];
}>();

const isSecondaryNavVisible = ref(true);

function handleRailSelect(key: string) {
  if (key === props.activeRail) {
    isSecondaryNavVisible.value = !isSecondaryNavVisible.value;
  } else {
    isSecondaryNavVisible.value = true;
    emit("selectRail", key);
  }
}
</script>

<style scoped>
.page-shell {
  width: 1440px;
  margin: 0 auto;
  min-height: 100%;
  display: flex;
  gap: 14px;
  padding: 28px;
}

.secondary-nav-wrapper {
  overflow: hidden;
  display: flex;
}

.rail-slide-enter-active,
.rail-slide-leave-active {
  transition: width 0.3s cubic-bezier(0.25, 0.8, 0.25, 1),
              margin-right 0.3s cubic-bezier(0.25, 0.8, 0.25, 1),
              opacity 0.25s ease;
}

.rail-slide-enter-from,
.rail-slide-leave-to {
  width: 0 !important;
  opacity: 0;
  margin-right: -14px;
}

.rail-slide-enter-to,
.rail-slide-leave-from {
  width: 232px;
  opacity: 1;
  margin-right: 0;
}

.content-wrap {
  width: 1040px;
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 22px;
}
</style>
