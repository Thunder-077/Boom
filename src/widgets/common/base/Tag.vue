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
  @apply inline-flex items-center justify-center whitespace-nowrap rounded-pill font-semibold;
  transition:
    background-color var(--transition-base) var(--transition-ease),
    color var(--transition-base) var(--transition-ease);
}

.tag-sm {
  @apply min-h-[22px] px-2 py-0.5 text-xs;
}

.tag-md {
  @apply min-h-[26px] px-2.5 py-1 text-xs;
}

.tag-lg {
  @apply min-h-[30px] px-3 py-1 text-sm;
}

.tag-default {
  @apply bg-[var(--accent-fill-soft)] text-text-secondary;
}

.tag-primary {
  @apply bg-accent-soft text-accent;
}

.tag-success {
  @apply bg-[var(--color-success-soft)] text-success;
}

.tag-warning {
  @apply bg-[var(--color-warning-soft)] text-warning;
}

.tag-danger {
  @apply bg-[var(--color-danger-soft)] text-danger;
}

.tag-info {
  @apply bg-accent-softer text-accent;
}

.tag.clickable {
  @apply cursor-pointer;
}

.tag.clickable:hover {
  filter: brightness(0.96);
}

.tag.clickable.active {
  @apply bg-accent-soft text-accent-strong;
}
</style>
