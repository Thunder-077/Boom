<template>
  <header class="header" :class="{ compact }">
    <div class="left">
      <p class="crumb">{{ breadcrumb }}</p>
      <h1 class="title">{{ title }}</h1>
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
  compact?: boolean;
}>();
</script>

<style scoped>
.header {
  min-height: 96px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
}

.header.compact {
  min-height: 84px;
}

.left {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.crumb {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 500;
}

.title {
  margin: 0;
  font-size: 34px;
  font-weight: 700;
}

.right {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>
