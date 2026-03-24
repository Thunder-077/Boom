<template>
  <section class="page-shell">
    <PrimaryRail :items="railItems" :active-key="activeRail" @select="$emit('selectRail', $event)" />
    <SecondaryNav
      :title="secondaryTitle"
      :description="secondaryDescription"
      :items="secondaryItems"
      :active-key="activeSecondary"
      @select="$emit('selectSecondary', $event)"
    />
    <main class="content-wrap">
      <slot />
    </main>
  </section>
</template>

<script setup lang="ts">
import PrimaryRail from "./PrimaryRail.vue";
import SecondaryNav from "./SecondaryNav.vue";
import type { RailItem, SecondaryNavItem } from "./types";

defineProps<{
  railItems: RailItem[];
  activeRail: string;
  secondaryTitle: string;
  secondaryDescription: string;
  secondaryItems: SecondaryNavItem[];
  activeSecondary: string;
}>();

defineEmits<{
  selectRail: [key: string];
  selectSecondary: [key: string];
}>();
</script>

<style scoped>
.page-shell {
  width: 1440px;
  height: 980px;
  display: flex;
  gap: 14px;
  padding: 28px;
}

.content-wrap {
  width: 1040px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 22px;
}
</style>
