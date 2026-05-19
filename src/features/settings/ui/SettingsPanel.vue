<template>
  <section class="settings-card card-shell">
    <header class="settings-head">
      <div>
        <span class="section-kicker">外观</span>
        <h3>配色主题</h3>
      </div>
      <Tag>{{ currentThemeLabel }}</Tag>
    </header>

    <div class="theme-grid">
      <button
        v-for="theme in options"
        :key="theme.id"
        type="button"
        class="theme-card"
        :class="{ active: theme.id === currentTheme }"
        @click="setTheme(theme.id)"
      >
        <span class="theme-preview" :style="previewStyle(theme)">
          <span class="preview-sidebar" />
          <span class="preview-panel" />
          <span class="preview-accent" />
        </span>
        <span class="theme-text">
          <strong>{{ theme.label }}</strong>
          <small>{{ theme.description }}</small>
        </span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { ThemeOption } from "../../../shared/theme/theme";
import { useThemeState } from "../../../shared/theme/theme";
import { Tag } from "../../../widgets/common";

const { currentTheme, options, setTheme } = useThemeState();

const currentThemeLabel = computed(() => {
  const matched = options.find((option) => option.id === currentTheme.value);
  return matched?.label ?? "未设置";
});

function previewStyle(theme: ThemeOption) {
  return {
    "--preview-surface": theme.surface,
    "--preview-accent": theme.accent,
  };
}
</script>

<style scoped>
.settings-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-xl);
  padding: var(--space-4);
  border-radius: var(--radius-card-large);
  background: var(--surface-nav-panel);
}

.settings-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-head h3 {
  margin: var(--space-sm) 0 var(--space-2);
  font-size: var(--font-size-3xl);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.settings-head p {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  line-height: 1.55;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: var(--space-md);
}

.theme-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  padding: var(--space-md);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--surface-nav-item);
  cursor: pointer;
  text-align: left;
  transition:
    border-color var(--transition-base) var(--transition-ease),
    box-shadow var(--transition-base) var(--transition-ease),
    transform var(--transition-base) var(--transition-ease);
}

.theme-card:hover {
  transform: translateY(-1px);
  border-color: var(--accent-border-soft);
  box-shadow: var(--shadow-soft);
}

.theme-card.active {
  border-color: rgba(var(--accent-rgb), 0.24);
  box-shadow: 0 14px 26px rgba(var(--accent-rgb), 0.08);
}

.theme-preview {
  position: relative;
  display: block;
  height: 116px;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--preview-accent) 22%, rgba(15, 23, 42, 0.16));
  background:
    radial-gradient(circle at 82% 18%, color-mix(in srgb, var(--preview-accent) 16%, transparent), transparent 45%),
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--preview-surface) 88%, white),
      color-mix(in srgb, var(--preview-surface) 90%, rgba(15, 23, 42, 0.06))
    );
  overflow: hidden;
}

.preview-sidebar,
.preview-panel,
.preview-accent {
  position: absolute;
  border-radius: var(--radius-sm);
}

.preview-sidebar {
  top: var(--space-md);
  left: var(--space-md);
  width: 32px;
  bottom: var(--space-md);
  background: color-mix(in srgb, var(--preview-surface) 78%, var(--preview-accent) 22%);
  border: 1px solid color-mix(in srgb, var(--preview-accent) 24%, white);
}

.preview-panel {
  top: var(--space-md);
  left: 56px;
  right: var(--space-md);
  bottom: var(--space-md);
  background: color-mix(in srgb, var(--preview-surface) 86%, white);
  border: 1px solid color-mix(in srgb, var(--preview-accent) 20%, white);
}

.preview-accent {
  right: 22px;
  top: var(--space-5);
  width: 70px;
  height: 30px;
  background: var(--preview-accent);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.32);
}

.theme-text {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.theme-text strong {
  font-size: var(--font-size-lg);
  color: var(--text-primary);
}

.theme-text small {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}
</style>
