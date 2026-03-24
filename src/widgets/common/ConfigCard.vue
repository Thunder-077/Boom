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
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(255, 255, 255, 0.88);
}

.config-card h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}

.config-card p {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-muted);
}

.body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
