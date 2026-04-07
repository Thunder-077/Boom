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
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-radius: var(--radius-card-large);
  background: var(--surface-card-gradient);
}

.config-card h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.config-card p {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
</style>
