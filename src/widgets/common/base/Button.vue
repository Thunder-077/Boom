<template>
  <button
    class="btn"
    :class="[variantClass, sizeClass, { disabled, loading }]"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <span v-if="loading" class="btn-icon btn-spinner" />
    <span class="btn-content">
      <slot />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "danger" | "ghost"
    size?: "sm" | "md" | "lg"
    disabled?: boolean
    loading?: boolean
  }>(),
  {
    variant: "primary",
    size: "md",
    disabled: false,
    loading: false,
  }
);

const emit = defineEmits<{
  click: [event: MouseEvent]
}>();

const variantClass = computed(() => `btn-${props.variant}`);
const sizeClass = computed(() => `btn-${props.size}`);

function handleClick(event: MouseEvent) {
  if (props.disabled || props.loading) return;
  emit("click", event);
}
</script>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  cursor: pointer;
  font-family: var(--font-ui);
  font-weight: 600;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
  outline: none;
}

.btn:disabled,
.btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-sm {
  min-height: 30px;
  padding: 0 var(--space-md);
  font-size: var(--font-size-sm);
}

.btn-md {
  min-height: 36px;
  padding: 0 var(--space-lg);
  font-size: var(--font-size-base);
}

.btn-lg {
  min-height: 42px;
  padding: 0 var(--space-xl);
  font-size: var(--font-size-base);
}

.btn-primary {
  border-color: transparent;
  color: var(--color-on-primary);
  background: var(--accent-primary);
}

.btn-primary:hover:not(:disabled):not(.loading) {
  background: var(--accent-primary-strong);
}

.btn-primary:active:not(:disabled) {
  filter: brightness(0.95);
}

.btn-secondary {
  border-color: var(--border-default);
  color: var(--text-primary);
  background: var(--surface-panel);
}

.btn-secondary:hover:not(:disabled):not(.loading) {
  border-color: var(--border-strong);
  background: var(--surface-panel);
}

.btn-danger {
  border-color: var(--border-default);
  color: var(--color-danger);
  background: var(--surface-panel);
}

.btn-danger:hover:not(:disabled):not(.loading) {
  border-color: var(--color-danger-action-strong);
  background: var(--color-danger-soft);
}

.btn-ghost {
  border-color: transparent;
  color: var(--text-secondary);
  background: transparent;
}

.btn-ghost:hover:not(:disabled):not(.loading) {
  background: var(--accent-fill-soft);
  color: var(--text-primary);
}

.btn-content {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
}

.btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
