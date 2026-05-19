<template>
  <span
    class="tag"
    :class="[variantClass, sizeClass, { clickable, active }]"
    :tabindex="clickable ? 0 : undefined"
    @click="clickable && emit('click')"
    @keydown.enter="clickable && emit('click')"
  >
    <slot />
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "default" | "primary" | "success" | "warning" | "danger" | "info"
    size?: "sm" | "md" | "lg"
    clickable?: boolean
    active?: boolean
  }>(),
  {
    variant: "default",
    size: "md",
    clickable: false,
    active: false,
  }
);

const emit = defineEmits<{
  click: []
}>();

const variantClass = computed(() => `tag-${props.variant}`);
const sizeClass = computed(() => `tag-${props.size}`);
</script>

<style scoped>
.tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-pill);
  font-weight: 600;
  white-space: nowrap;
  transition:
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.tag-sm {
  min-height: 22px;
  padding: 2px 8px;
  font-size: var(--font-size-xs);
}

.tag-md {
  min-height: 26px;
  padding: 4px 10px;
  font-size: var(--font-size-xs);
}

.tag-lg {
  min-height: 30px;
  padding: 4px 12px;
  font-size: var(--font-size-sm);
}

.tag-default {
  background: var(--accent-fill-soft);
  color: var(--text-secondary);
}

.tag-primary {
  background: var(--accent-soft);
  color: var(--accent-primary);
}

.tag-success {
  background: var(--color-success-soft);
  color: var(--color-success);
}

.tag-warning {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.tag-danger {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}

.tag-info {
  background: var(--accent-softer);
  color: var(--accent-primary);
}

.tag.clickable {
  cursor: pointer;
}

.tag.clickable:hover {
  filter: brightness(0.96);
}

.tag.clickable.active {
  background: var(--accent-soft);
  color: var(--accent-primary-strong);
}
</style>
