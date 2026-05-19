<template>
  <section class="config-card card-shell">
    <h3 v-if="title">{{ title }}</h3>
    <p v-if="description">{{ description }}</p>
    <div class="body">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title?: string;
  description?: string;
}>();
</script>

<style scoped>
.config-card {
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  border-radius: var(--radius-card-large);
}

.config-card h3 {
  margin: 0;
  font-size: var(--font-size-title-md);
  font-weight: 600;
  letter-spacing: -0.01em;
}

.config-card p {
  margin: 0;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.body {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}
</style>
