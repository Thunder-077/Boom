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
  font-weight: 700;
  white-space: nowrap;
  transition:
    background-color var(--transition-base) var(--transition-ease),
    border-color var(--transition-base) var(--transition-ease),
    transform var(--transition-base) var(--transition-ease);
}

.tag-sm {
  min-height: 24px;
  padding: 4px 10px;
  font-size: var(--font-size-xs);
}

.tag-md {
  min-height: 28px;
  padding: 6px 12px;
  font-size: var(--font-size-xs);
}

.tag-lg {
  min-height: 32px;
  padding: 6px 14px;
  font-size: var(--font-size-sm);
}

.tag-default {
  background: var(--accent-soft);
  color: var(--accent-primary);
  border: 1px solid rgba(var(--accent-rgb), 0.16);
}

.tag-primary {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary-strong);
  border: 1px solid rgba(var(--accent-rgb), 0.24);
}

.tag-success {
  background: var(--color-success-soft);
  color: var(--color-success);
  border: 1px solid rgba(var(--color-success-rgb), 0.16);
}

.tag-warning {
  background: var(--color-warning-soft);
  color: var(--color-warning);
  border: 1px solid rgba(var(--color-warning-rgb), 0.16);
}

.tag-danger {
  background: var(--color-danger-soft);
  color: var(--color-danger);
  border: 1px solid rgba(var(--color-danger-rgb), 0.16);
}

.tag-info {
  background: var(--accent-softer);
  color: var(--color-info-strong);
  border: 1px solid rgba(var(--accent-rgb), 0.12);
}

.tag.clickable {
  cursor: pointer;
}

.tag.clickable:hover {
  transform: translateY(-1px);
}

.tag.clickable.active {
  border-color: var(--accent-border-strong);
  background: rgba(var(--accent-rgb), 0.14);
  color: var(--accent-primary-strong);
  box-shadow: 0 0 0 1px rgba(var(--accent-rgb), 0.08);
}
</style>
