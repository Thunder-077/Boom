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
    transform var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease),
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
  outline: none;
}

.btn:disabled,
.btn.disabled {
  opacity: 0.56;
  cursor: not-allowed;
  transform: none !important;
  box-shadow: none !important;
}

.btn-sm {
  min-height: 32px;
  padding: 0 var(--space-md);
  font-size: var(--font-size-sm);
  border-radius: var(--radius-xs);
}

.btn-md {
  min-height: 42px;
  padding: 0 var(--space-4);
  font-size: var(--font-size-base);
  border-radius: var(--radius-sm);
}

.btn-lg {
  min-height: 48px;
  padding: 0 var(--space-5);
  font-size: var(--font-size-lg);
  border-radius: var(--radius-md);
}

.btn-primary {
  border-color: rgba(255, 255, 255, 0.12);
  color: var(--text-on-dark);
  background: linear-gradient(135deg, var(--accent-primary-strong), var(--accent-primary));
  box-shadow: 0 16px 30px rgba(var(--accent-rgb), 0.24);
}

.btn-primary:hover:not(:disabled):not(.loading) {
  transform: translateY(-1px);
  box-shadow: 0 18px 34px rgba(var(--accent-rgb), 0.3);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: 0 10px 20px rgba(var(--accent-rgb), 0.2);
}

.btn-secondary {
  border-color: var(--border-default);
  color: var(--accent-primary);
  background: var(--surface-panel);
}

.btn-secondary:hover:not(:disabled):not(.loading) {
  transform: translateY(-1px);
  border-color: var(--accent-border-strong);
  background: var(--surface-panel-strong);
  box-shadow: 0 10px 22px rgba(31, 60, 103, 0.1);
}

.btn-danger {
  border-color: rgba(209, 52, 56, 0.24);
  color: var(--color-danger);
  background: var(--surface-panel);
}

.btn-danger:hover:not(:disabled):not(.loading) {
  transform: translateY(-1px);
  border-color: var(--color-danger-action-strong);
  background: var(--color-danger-soft);
  box-shadow: 0 10px 22px rgba(var(--color-danger-rgb), 0.12);
}

.btn-ghost {
  border-color: transparent;
  color: var(--text-secondary);
  background: transparent;
}

.btn-ghost:hover:not(:disabled):not(.loading) {
  background: var(--accent-fill-soft);
  color: var(--accent-primary);
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
  width: 16px;
  height: 16px;
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
