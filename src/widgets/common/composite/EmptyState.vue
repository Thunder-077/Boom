<template>
  <div class="empty-state">
    <span class="empty-icon material-symbols-rounded" aria-hidden="true">{{ icon }}</span>
    <h4 class="empty-title">{{ title }}</h4>
    <p v-if="description" class="empty-description">{{ description }}</p>
    <div v-if="$slots.default" class="empty-actions">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    icon?: string
    title: string
    description?: string
  }>(),
  {
    icon: "inventory_2",
    description: "",
  }
);
</script>

<style scoped>
.empty-state {
  min-height: 200px;
  border: 1px dashed var(--border-default);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  padding: var(--space-2xl);
  text-align: center;
}

.empty-icon {
  font-size: 32px;
  color: var(--text-tertiary);
  opacity: 0.6;
}

.empty-title {
  margin: 0;
  color: var(--text-primary);
  font-size: var(--font-size-base);
  font-weight: 600;
}

.empty-description {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.55;
  max-width: 320px;
}

.empty-actions {
  display: flex;
  gap: var(--space-md);
  margin-top: var(--space-md);
}
</style>
