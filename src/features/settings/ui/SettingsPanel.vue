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
  padding: var(--space-xl);
  border-radius: var(--radius-card-large);
  background: var(--surface-nav-panel);
}

.settings-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
}

.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-head h3 {
  margin: var(--space-sm) 0 var(--space-xs);
  font-size: var(--font-size-title-md);
  font-weight: 600;
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
    box-shadow var(--transition-base) var(--transition-ease);
}

.theme-card:hover {
  border-color: var(--accent-border-soft);
  box-shadow: var(--shadow-soft);
}

.theme-card.active {
  border-color: var(--accent-primary);
  box-shadow: var(--shadow-soft);
}

.theme-preview {
  position: relative;
  display: block;
  height: 116px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--preview-surface);
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
  border: 1px solid var(--border-default);
}

.preview-panel {
  top: var(--space-md);
  left: 56px;
  right: var(--space-md);
  bottom: var(--space-md);
  background: var(--surface-panel);
  border: 1px solid var(--border-default);
}

.preview-accent {
  right: 22px;
  top: 24px;
  width: 70px;
  height: 30px;
  background: var(--preview-accent);
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
