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
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.input-label {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.input-field-wrap {
  position: relative;
  display: flex;
  align-items: center;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-input);
  transition:
    border-color var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease);
}

.input-field {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  color: var(--text-primary);
  font-family: var(--font-ui);
}

.input-field::placeholder {
  color: var(--text-tertiary);
}

.input-field:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.input-sm .input-field-wrap {
  min-height: 32px;
}

.input-sm .input-field {
  padding: 0 var(--space-md);
  font-size: var(--font-size-sm);
}

.input-md .input-field-wrap {
  min-height: 38px;
}

.input-md .input-field {
  padding: 0 var(--space-lg);
  font-size: var(--font-size-base);
}

.input-lg .input-field-wrap {
  min-height: 44px;
}

.input-lg .input-field {
  padding: 0 var(--space-xl);
  font-size: var(--font-size-base);
}

.input-prefix,
.input-suffix {
  display: inline-flex;
  align-items: center;
  color: var(--text-tertiary);
  font-size: var(--font-size-base);
  flex-shrink: 0;
}

.input-prefix {
  padding-left: var(--space-md);
}

.input-suffix {
  padding-right: var(--space-md);
}

.input-wrapper.focused .input-field-wrap {
  border-color: rgba(var(--accent-rgb), 0.5);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
}

.input-wrapper.disabled .input-field-wrap {
  background: var(--surface-elevated);
  cursor: not-allowed;
}

.input-wrapper .input-field-wrap:hover:not(.focused):not(.disabled) {
  border-color: var(--border-strong);
}

.input-error {
  margin: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

.input-help {
  margin: 0;
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
}
</style>
