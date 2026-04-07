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
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  border-radius: var(--radius-card-large);
  background: var(--surface-card-gradient);
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.copy {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.meta-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.head h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.head p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.content {
  overflow: hidden;
  border-radius: 18px;
  border: 1px solid var(--color-border-soft);
  background: var(--surface-table-content);
}

@media (max-width: 900px) {
  .head {
    flex-direction: column;
  }

  .meta-wrap {
    justify-content: flex-start;
  }
}
</style>
