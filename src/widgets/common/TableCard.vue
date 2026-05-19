<template>
  <section class="table-card card-shell">
    <header class="head">
      <div class="copy">
        <h3>{{ title }}</h3>
        <slot name="description" />
      </div>
      <div class="meta-wrap">
        <slot name="actions" />
        <p v-if="meta">{{ meta }}</p>
      </div>
    </header>
    <div class="content">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  meta?: string;
}>();
</script>

<style scoped>
.table-card {
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  border-radius: var(--radius-card-large);
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
}

.copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.meta-wrap {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  flex-wrap: wrap;
  justify-content: flex-end;
}

.head h3 {
  margin: 0;
  font-size: var(--font-size-title-md);
  font-weight: 600;
  letter-spacing: -0.01em;
}

.head p {
  margin: 0;
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.content {
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
  background: var(--surface-table-content);
}
</style>
