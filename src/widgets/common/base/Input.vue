<template>
  <label class="input-wrapper" :class="[sizeClass, { focused, disabled }]">
    <span v-if="label" class="input-label">{{ label }}</span>
    <div class="input-field-wrap">
      <span v-if="prefix" class="input-prefix">
        <slot name="prefix" />
      </span>
      <input
        class="input-field"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :maxlength="maxlength"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
      />
      <span v-if="suffix" class="input-suffix">
        <slot name="suffix" />
      </span>
    </div>
    <p v-if="error" class="input-error">{{ error }}</p>
    <p v-else-if="helpText" class="input-help">{{ helpText }}</p>
  </label>
</template>

<script setup lang="ts">
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue?: string
    type?: string
    label?: string
    placeholder?: string
    prefix?: boolean
    suffix?: boolean
    disabled?: boolean
    readonly?: boolean
    maxlength?: number
    size?: "sm" | "md" | "lg"
    error?: string
    helpText?: string
  }>(),
  {
    modelValue: "",
    type: "text",
    placeholder: "",
    prefix: false,
    suffix: false,
    disabled: false,
    readonly: false,
    size: "md",
    error: "",
    helpText: "",
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: string]
  "focus": [event: FocusEvent]
  "blur": [event: FocusEvent]
}>();

const focused = ref(false);
const sizeClass = `input-${props.size}`;

function handleInput(event: Event) {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", target.value);
}

function handleFocus(event: FocusEvent) {
  focused.value = true;
  emit("focus", event);
}

function handleBlur(event: FocusEvent) {
  focused.value = false;
  emit("blur", event);
}
</script>

<style scoped>
.input-wrapper {
  @apply flex flex-col gap-sm;
}

.input-label {
  @apply text-sm font-semibold text-text-secondary;
}

.input-field-wrap {
  @apply relative flex items-center rounded-sm border border-border bg-surface-input;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease);
}

.input-field {
  @apply flex-1 border-0 bg-transparent font-ui text-text-primary outline-none;
}

.input-field::placeholder {
  @apply text-text-tertiary;
}

.input-field:disabled {
  @apply cursor-not-allowed opacity-50;
}

.input-sm .input-field-wrap {
  @apply min-h-8;
}

.input-sm .input-field {
  @apply px-md text-sm;
}

.input-md .input-field-wrap {
  @apply min-h-[38px];
}

.input-md .input-field {
  @apply px-lg text-base;
}

.input-lg .input-field-wrap {
  @apply min-h-11;
}

.input-lg .input-field {
  @apply px-xl text-base;
}

.input-prefix,
.input-suffix {
  @apply inline-flex shrink-0 items-center text-base text-text-tertiary;
}

.input-prefix {
  @apply pl-md;
}

.input-suffix {
  @apply pr-md;
}

.input-wrapper.focused .input-field-wrap {
  border-color: rgba(var(--accent-rgb), 0.5);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
}

.input-wrapper.disabled .input-field-wrap {
  @apply cursor-not-allowed bg-surface-elevated;
}

.input-wrapper .input-field-wrap:hover:not(.focused):not(.disabled) {
  @apply border-border-strong;
}

.input-error {
  @apply m-0 text-sm text-danger;
}

.input-help {
  @apply m-0 text-sm text-text-tertiary;
}
</style>
