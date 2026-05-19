<template>
  <div class="search-input" :class="{ focused, disabled }">
    <span class="search-icon material-symbols-rounded" aria-hidden="true">search</span>
    <input
      class="search-field"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      @input="handleInput"
      @focus="handleFocus"
      @blur="handleBlur"
      @keydown="handleKeydown"
    />
    <button
      v-if="modelValue && !disabled"
      type="button"
      class="search-clear"
      @click="clear"
    >
      <span class="material-symbols-rounded">close</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue?: string
    placeholder?: string
    disabled?: boolean
    debounceMs?: number
  }>(),
  {
    modelValue: "",
    placeholder: "搜索...",
    disabled: false,
    debounceMs: 300,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: string]
  "search": [value: string]
  "clear": []
}>();

const focused = ref(false);
let debounceTimer: number | null = null;

function handleInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  emit("update:modelValue", value);

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = window.setTimeout(() => {
    emit("search", value);
  }, props.debounceMs);
}

function handleFocus() {
  focused.value = true;
}

function handleBlur() {
  focused.value = false;
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    emit("search", props.modelValue);
  }
}

function clear() {
  emit("update:modelValue", "");
  emit("clear");
  emit("search", "");
}
</script>

<style scoped>
.search-input {
  position: relative;
  display: flex;
  align-items: center;
  min-height: 44px;
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-input);
  transition:
    border-color var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease),
    background-color var(--transition-base) var(--transition-ease);
}

.search-icon {
  color: var(--text-tertiary);
  font-size: 20px;
  margin-right: var(--space-sm);
  flex-shrink: 0;
}

.search-field {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  font-size: var(--font-size-base);
  color: var(--text-primary);
  font-family: var(--font-ui);
  min-height: 44px;
}

.search-field::placeholder {
  color: var(--text-tertiary);
}

.search-field:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.search-clear {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  color: var(--text-tertiary);
  transition:
    background-color var(--transition-fast) var(--transition-ease),
    color var(--transition-fast) var(--transition-ease);
  flex-shrink: 0;
}

.search-clear:hover {
  background: var(--accent-fill-soft);
  color: var(--accent-primary);
}

.search-clear .material-symbols-rounded {
  font-size: 18px;
}

.search-input.focused {
  border-color: rgba(var(--accent-rgb), 0.42);
  background: var(--surface-input-strong);
  box-shadow: 0 0 0 4px var(--accent-focus-ring);
}

.search-input.disabled {
  background: var(--surface-elevated);
  cursor: not-allowed;
}

.search-input:hover:not(.focused):not(.disabled) {
  border-color: var(--border-strong);
  background: var(--surface-panel-strong);
}
</style>
