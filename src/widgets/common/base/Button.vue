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
  @apply inline-flex cursor-pointer items-center justify-center gap-sm rounded-sm border border-transparent font-ui font-semibold outline-none;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.btn:disabled,
.btn.disabled {
  @apply cursor-not-allowed opacity-50;
}

.btn-sm {
  @apply min-h-[30px] px-md text-sm;
}

.btn-md {
  @apply min-h-9 px-lg text-base;
}

.btn-lg {
  @apply min-h-[42px] px-xl text-base;
}

.btn-primary {
  @apply border-transparent bg-accent text-[var(--color-on-primary)];
}

.btn-primary:hover:not(:disabled):not(.loading) {
  @apply bg-accent-strong;
}

.btn-primary:active:not(:disabled) {
  filter: brightness(0.95);
}

.btn-secondary {
  @apply border-border bg-surface-panel text-text-primary;
}

.btn-secondary:hover:not(:disabled):not(.loading) {
  @apply border-border-strong bg-surface-panel;
}

.btn-danger {
  @apply border-border bg-surface-panel text-danger;
}

.btn-danger:hover:not(:disabled):not(.loading) {
  @apply border-[var(--color-danger-action-strong)] bg-[var(--color-danger-soft)];
}

.btn-ghost {
  @apply border-transparent bg-transparent text-text-secondary;
}

.btn-ghost:hover:not(:disabled):not(.loading) {
  @apply bg-[var(--accent-fill-soft)] text-text-primary;
}

.btn-content {
  @apply inline-flex items-center gap-sm;
}

.btn-icon {
  @apply inline-flex items-center justify-center;
}

.btn-spinner {
  @apply h-[14px] w-[14px] rounded-full border-2;
  border-color: rgba(255, 255, 255, 0.3);
  border-top-color: currentColor;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
