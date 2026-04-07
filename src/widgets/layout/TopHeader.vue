<template>
  <header class="header" :class="{ compact }">
    <div class="left">
      <p class="crumb">{{ breadcrumb }}</p>
      <h1 class="title">{{ title }}</h1>
      <p v-if="summary" class="summary">{{ summary }}</p>
    </div>
    <div class="right">
      <slot name="actions" />
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  breadcrumb: string;
  title: string;
  summary?: string;
  compact?: boolean;
}>();
</script>

<style scoped>
.header {
  min-height: 96px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 8px 6px 0;
}

.header.compact {
  min-height: 82px;
}

.left {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.crumb {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.title {
  margin: 0;
  font-size: clamp(30px, 3vw, 40px);
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--text-primary);
}

.summary {
  max-width: 720px;
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.5;
}

.right {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 44px;
  padding-top: 10px;
}

@media (max-width: 900px) {
  .header {
    flex-direction: column;
  }

  .right {
    padding-top: 0;
  }
}
</style>
