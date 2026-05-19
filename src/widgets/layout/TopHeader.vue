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
  min-height: 88px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
  padding: var(--space-sm) var(--space-xs) 0;
}

.header.compact {
  min-height: 72px;
}

.left {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.crumb {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.title {
  margin: 0;
  font-size: clamp(26px, 2.8vw, 36px);
  font-weight: 700;
  letter-spacing: -0.025em;
  color: var(--text-primary);
  line-height: 1.2;
}

.summary {
  max-width: 640px;
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.55;
}

.right {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  min-height: 40px;
  padding-top: var(--space-sm);
  flex-shrink: 0;
}
</style>
