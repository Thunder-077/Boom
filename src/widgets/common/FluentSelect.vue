<template>
  <div
    class="fluent-combo"
    :class="{ open: isOpen, disabled }"
    :tabindex="searchable ? -1 : 0"
    @keydown.esc.prevent="closeCombo"
    ref="comboRef"
  >
    <template v-if="searchable">
      <div class="fluent-trigger fluent-searchable-trigger" ref="triggerRef" @mousedown.prevent="onSearchTriggerMouseDown">
        <input
          ref="searchInputRef"
          class="fluent-searchable-input"
          :value="searchKeyword"
          :placeholder="isPlaceholder ? placeholder : ''"
          @input="handleSearchInput"
          @keydown.esc.prevent="closeCombo"
          @keydown.down.prevent.stop="navigateOptions(1)"
          @keydown.up.prevent.stop="navigateOptions(-1)"
          @keydown.enter.prevent.stop="selectHighlighted"
        />
        <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
      </div>
    </template>
    <template v-else>
      <div class="fluent-trigger" ref="triggerRef" @mousedown.prevent="toggleCombo">
        <span class="fluent-value" :class="{ placeholder: isPlaceholder }">
          {{ displayLabel }}
        </span>
        <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
      </div>
    </template>

    <Teleport to="body">
      <div
        v-show="isOpen"
        class="teleported-fluent-menu"
        :style="menuStyle"
        ref="menuRef"
      >
        <button
          v-for="(opt, idx) in filteredOptions"
          :key="opt.value"
          type="button"
          class="fluent-option"
          :class="{ selected: opt.value === modelValue, highlighted: idx === highlightIndex }"
          @click="selectOption(opt.value)"
        >
          {{ opt.label }}
        </button>
        <div v-if="filteredOptions.length === 0" class="menu-empty">无匹配选项</div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts" generic="T extends string | number">
import { computed, ref, reactive, onUnmounted, watch, nextTick } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: T | "";
    options: { label: string; value: T | "" }[];
    placeholder?: string;
    disabled?: boolean;
    searchable?: boolean;
  }>(),
  {
    placeholder: "请选择",
    disabled: false,
    searchable: false,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: T | ""];
  "change": [value: T | ""];
}>();

const isOpen = ref(false);
const comboRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchKeyword = ref("");
const highlightIndex = ref(-1);

const menuStyle = reactive({
  top: "0px",
  left: "0px",
  width: "auto",
});

const filteredOptions = computed(() => {
  if (!props.searchable || !searchKeyword.value.trim()) {
    return props.options;
  }
  const kw = searchKeyword.value.trim().toLowerCase();
  return props.options.filter((opt) => opt.label.toLowerCase().includes(kw));
});

const displayLabel = computed(() => {
  if (props.modelValue === "" || props.modelValue === null || props.modelValue === undefined) {
    const defaultPlaceholderOption = props.options.find((opt) => opt.value === "");
    return defaultPlaceholderOption ? defaultPlaceholderOption.label : props.placeholder;
  }
  const found = props.options.find((opt) => opt.value === props.modelValue);
  return found ? found.label : props.placeholder;
});

const isPlaceholder = computed(() => {
  if (props.modelValue === "" || props.modelValue === null || props.modelValue === undefined) {
    const defaultPlaceholderOption = props.options.find((opt) => opt.value === "");
    return !defaultPlaceholderOption;
  }
  return false;
});

function updatePosition() {
  if (!triggerRef.value) return;
  const rect = triggerRef.value.getBoundingClientRect();

  const viewportHeight = window.innerHeight;
  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  const estimatedHeight = Math.min(240, Math.max(spaceBelow, spaceAbove) - 12);

  if (spaceBelow < 240 && spaceAbove > spaceBelow) {
    menuStyle.top = `${rect.top - estimatedHeight - 6}px`;
  } else {
    menuStyle.top = `${rect.bottom + 6}px`;
  }

  menuStyle.left = `${rect.left}px`;
  menuStyle.width = `${rect.width}px`;
}

function handleScrollOrResize(e: Event) {
  if (!isOpen.value) return;
  if (e.type === "scroll" && e.target === menuRef.value) {
    return;
  }
  closeCombo();
}

function handleClickOutside(e: MouseEvent) {
  if (!isOpen.value) return;
  const target = e.target as Node;
  if (comboRef.value?.contains(target) || menuRef.value?.contains(target)) {
    return;
  }
  closeCombo();
}

function openCombo() {
  if (props.disabled) return;
  if (isOpen.value) return;
  highlightIndex.value = -1;
  updatePosition();
  isOpen.value = true;
  window.addEventListener("scroll", handleScrollOrResize, true);
  window.addEventListener("resize", handleScrollOrResize);
  document.addEventListener("mousedown", handleClickOutside);
  if (props.searchable) {
    nextTick(() => {
      nextTick(() => {
        const input = searchInputRef.value;
        if (!input) return;
        input.focus();
        const len = input.value.length;
        input.setSelectionRange(len, len);
      });
    });
  } else {
    comboRef.value?.focus();
  }
}

function onSearchTriggerMouseDown() {
  if (props.disabled) return;
  if (!isOpen.value) {
    openCombo();
  } else {
    const input = searchInputRef.value;
    if (!input) return;
    input.focus();
    const len = input.value.length;
    input.setSelectionRange(len, len);
  }
}

function toggleCombo() {
  if (props.disabled) return;
  if (isOpen.value) {
    closeCombo();
  } else {
    openCombo();
  }
}

function closeCombo() {
  isOpen.value = false;
  if (!props.searchable) {
    searchKeyword.value = "";
  }
  highlightIndex.value = -1;
  window.removeEventListener("scroll", handleScrollOrResize, true);
  window.removeEventListener("resize", handleScrollOrResize);
  document.removeEventListener("mousedown", handleClickOutside);
  comboRef.value?.blur();
}

function selectOption(value: T | "") {
  if (props.disabled) return;
  if (props.searchable) {
    const selected = props.options.find((opt) => opt.value === value);
    searchKeyword.value = selected ? selected.label : "";
  }
  emit("update:modelValue", value);
  emit("change", value);
  closeCombo();
}

function handleSearchInput(e: Event) {
  searchKeyword.value = (e.target as HTMLInputElement).value;
  highlightIndex.value = -1;
  if (!isOpen.value) {
    openCombo();
  }
}

function navigateOptions(direction: number) {
  const len = filteredOptions.value.length;
  if (len === 0) return;
  if (highlightIndex.value === -1) {
    highlightIndex.value = direction > 0 ? 0 : len - 1;
  } else {
    highlightIndex.value = (highlightIndex.value + direction + len) % len;
  }
  scrollHighlightedIntoView();
}

function selectHighlighted() {
  if (highlightIndex.value >= 0 && highlightIndex.value < filteredOptions.value.length) {
    selectOption(filteredOptions.value[highlightIndex.value].value);
  }
}

function scrollHighlightedIntoView() {
  nextTick(() => {
    const menu = menuRef.value;
    if (!menu) return;
    const highlighted = menu.querySelector(".fluent-option.highlighted") as HTMLElement;
    if (highlighted) {
      highlighted.scrollIntoView({ block: "nearest" });
    }
  });
}

watch(() => isOpen.value, (val) => {
  if (!val) {
    if (!props.searchable) {
      searchKeyword.value = "";
    }
    highlightIndex.value = -1;
  }
});

watch(() => props.modelValue, (val) => {
  if (props.searchable && (val === "" || val === null || val === undefined)) {
    searchKeyword.value = "";
    return;
  }
  if (props.searchable && val !== "") {
    const selected = props.options.find((opt) => opt.value === val);
    if (selected) {
      searchKeyword.value = selected.label;
    }
  }
}, { immediate: true });

onUnmounted(() => {
  window.removeEventListener("scroll", handleScrollOrResize, true);
  window.removeEventListener("resize", handleScrollOrResize);
  document.removeEventListener("mousedown", handleClickOutside);
});
</script>

<style scoped>
.fluent-combo {
  position: relative;
  display: flex;
  outline: none;
  min-width: 120px;
}

.fluent-trigger {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  min-height: 38px;
  padding: 0 32px 0 var(--space-md);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-input);
  cursor: pointer;
  font-size: var(--font-size-base);
  color: var(--text-primary);
  transition:
    border-color var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease);
}

.fluent-searchable-trigger {
  cursor: text;
}

.fluent-searchable-input {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  font-size: var(--font-size-base);
  color: var(--text-primary);
  font-family: var(--font-ui);
  width: 100%;
  min-height: 38px;
}

.fluent-searchable-input::placeholder {
  color: var(--text-tertiary);
}

.fluent-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fluent-value.placeholder {
  color: var(--text-tertiary);
}

.combo-icon {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 18px;
  color: var(--text-tertiary);
  pointer-events: none;
  transition: transform var(--transition-base) var(--transition-ease);
}

.fluent-combo.open .combo-icon {
  transform: translateY(-50%) rotate(180deg);
}

.fluent-combo:focus-within .fluent-trigger,
.fluent-combo.open .fluent-trigger {
  border-color: rgba(var(--accent-rgb), 0.5);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
}

.fluent-combo.disabled .fluent-trigger {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--surface-elevated);
}
</style>

<style>
.teleported-fluent-menu {
  position: fixed;
  max-height: 240px;
  padding: 4px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-panel);
  box-shadow: var(--shadow-strong);
  overflow-y: auto;
  z-index: 99999;
  animation: menu-appear 0.12s var(--transition-ease) forwards;
  transform-origin: top;
  box-sizing: border-box;
}

@keyframes menu-appear {
  from {
    opacity: 0;
    transform: scaleY(0.96) translateY(-4px);
  }
  to {
    opacity: 1;
    transform: scaleY(1) translateY(0);
  }
}

.teleported-fluent-menu .fluent-option {
  width: 100%;
  min-height: 34px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  text-align: left;
  padding: 6px var(--space-md);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  transition:
    background-color var(--transition-fast) var(--transition-ease),
    color var(--transition-fast) var(--transition-ease);
  display: flex;
  align-items: center;
  box-sizing: border-box;
  font-family: var(--font-ui);
}

.teleported-fluent-menu .fluent-option:hover {
  background: var(--accent-fill-soft);
  color: var(--accent-primary);
}

.teleported-fluent-menu .fluent-option.selected {
  background: var(--accent-soft);
  color: var(--accent-primary);
  font-weight: 600;
}

.teleported-fluent-menu .fluent-option.highlighted {
  background: var(--accent-fill-soft);
}

.teleported-fluent-menu .menu-empty {
  padding: var(--space-sm) var(--space-md);
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
  text-align: center;
}
</style>
