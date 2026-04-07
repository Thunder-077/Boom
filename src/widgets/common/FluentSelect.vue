<template>
  <div
    class="fluent-combo"
    :class="{ open: isOpen, disabled }"
    @keydown.esc.prevent="closeCombo"
    tabindex="0"
    ref="comboRef"
  >
    <div class="fluent-trigger" ref="triggerRef" @mousedown.prevent="toggleCombo">
      <span class="fluent-value" :class="{ placeholder: isPlaceholder }">
        {{ displayLabel }}
      </span>
      <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
    </div>

    <Teleport to="body">
      <div
        v-show="isOpen"
        class="teleported-fluent-menu"
        :style="menuStyle"
        ref="menuRef"
      >
        <button
          v-for="opt in options"
          :key="opt.value"
          type="button"
          class="fluent-option"
          :class="{ selected: opt.value === modelValue }"
          @click="selectOption(opt.value)"
        >
          {{ opt.label }}
        </button>
        <div v-if="options.length === 0" class="menu-empty">无选项</div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts" generic="T extends string | number">
import { computed, ref, reactive, onUnmounted } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: T | "";
    options: { label: string; value: T | "" }[];
    placeholder?: string;
    disabled?: boolean;
  }>(),
  {
    placeholder: "请选择",
    disabled: false,
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

const menuStyle = reactive({
  top: "0px",
  left: "0px",
  width: "auto",
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
  
  // Calculate top and bottom space
  const viewportHeight = window.innerHeight;
  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  const estimatedHeight = Math.min(240, Math.max(spaceBelow, spaceAbove) - 12);
  
  // Choose direction (below by default, above if not enough space below)
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

function toggleCombo() {
  if (props.disabled) return;
  if (isOpen.value) {
    closeCombo();
  } else {
    updatePosition();
    isOpen.value = true;
    comboRef.value?.focus();
    window.addEventListener("scroll", handleScrollOrResize, true);
    window.addEventListener("resize", handleScrollOrResize);
    document.addEventListener("mousedown", handleClickOutside);
  }
}

function closeCombo() {
  isOpen.value = false;
  window.removeEventListener("scroll", handleScrollOrResize, true);
  window.removeEventListener("resize", handleScrollOrResize);
  document.removeEventListener("mousedown", handleClickOutside);
  comboRef.value?.blur();
}

function selectOption(value: T | "") {
  if (props.disabled) return;
  emit("update:modelValue", value);
  emit("change", value);
  closeCombo();
}

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
  min-height: 42px;
  padding: 0 34px 0 14px;
  border-radius: 14px;
  border: 1px solid var(--color-border-soft);
  background: var(--surface-panel);
  cursor: pointer;
  user-select: none;
  font-size: 14px;
  color: var(--color-text);
  transition: all 0.2s ease;
}

.fluent-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fluent-value.placeholder {
  color: var(--color-text-muted);
}

.combo-icon {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 18px;
  color: var(--text-secondary);
  pointer-events: none;
  transition: transform 0.2s ease;
}

.fluent-combo.open .combo-icon {
  transform: translateY(-50%) rotate(180deg);
}

.fluent-combo:focus-within .fluent-trigger,
.fluent-combo.open .fluent-trigger {
  border-color: var(--accent-border-strong);
  box-shadow: 0 0 0 3px var(--accent-focus-ring);
  background: var(--surface-input-strong);
}

.fluent-combo.disabled .fluent-trigger {
  opacity: 0.6;
  cursor: not-allowed;
  background: var(--surface-elevated);
}
</style>

<style>
/* Global styles for teleported menu */
.teleported-fluent-menu {
  position: fixed;
  max-height: 240px;
  padding: 6px;
  border: 1px solid var(--color-border-soft);
  border-radius: 14px;
  background: var(--surface-input-strong);
  box-shadow: var(--shadow-strong);
  backdrop-filter: blur(20px);
  overflow-y: auto;
  z-index: 99999;
  animation: slide-down 0.15s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
  transform-origin: top;
  box-sizing: border-box;
}

@keyframes slide-down {
  from {
    opacity: 0;
    transform: scaleY(0.95);
  }
  to {
    opacity: 1;
    transform: scaleY(1);
  }
}

.teleported-fluent-menu .fluent-option {
  width: 100%;
  min-height: 38px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  text-align: left;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  box-sizing: border-box;
}

.teleported-fluent-menu .fluent-option:hover {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.teleported-fluent-menu .fluent-option.selected {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  font-weight: 600;
}

.teleported-fluent-menu .menu-empty {
  padding: 10px 12px;
  color: var(--color-text-muted);
  font-size: 13px;
  text-align: center;
}
</style>
