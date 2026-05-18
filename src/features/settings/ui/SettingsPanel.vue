<template>
  <section class="settings-card card-shell">
    <header class="settings-head">
      <div>
        <span class="section-kicker">外观</span>
        <h3>配色主题</h3>
      </div>
      <span class="theme-pill">{{ currentThemeLabel }}</span>
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
  gap: 18px;
  padding: 20px;
  border-radius: var(--radius-card-large);
  background: var(--surface-nav-panel);
}

.settings-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-head h3 {
  margin: 6px 0 8px;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.settings-head p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.55;
}

.theme-pill {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 14px;
}

.theme-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--border-default);
  border-radius: 20px;
  background: var(--surface-nav-item);
  cursor: pointer;
  text-align: left;
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    transform 0.18s ease;
}

.theme-card:hover {
  transform: translateY(-1px);
  border-color: var(--accent-border-soft);
  box-shadow: 0 12px 24px rgba(31, 60, 103, 0.06);
}

.theme-card.active {
  border-color: rgba(var(--accent-rgb), 0.24);
  box-shadow: 0 14px 26px rgba(var(--accent-rgb), 0.08);
}

.theme-preview {
  position: relative;
  display: block;
  height: 116px;
  border-radius: 16px;
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
  border-radius: 14px;
}

.preview-sidebar {
  top: 12px;
  left: 12px;
  width: 32px;
  bottom: 12px;
  background: color-mix(in srgb, var(--preview-surface) 78%, var(--preview-accent) 22%);
  border: 1px solid color-mix(in srgb, var(--preview-accent) 24%, white);
}

.preview-panel {
  top: 12px;
  left: 56px;
  right: 12px;
  bottom: 12px;
  background: color-mix(in srgb, var(--preview-surface) 86%, white);
  border: 1px solid color-mix(in srgb, var(--preview-accent) 20%, white);
}

.preview-accent {
  right: 22px;
  top: 24px;
  width: 70px;
  height: 30px;
  background: var(--preview-accent);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.32);
}

.theme-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.theme-text strong {
  font-size: 15px;
  color: var(--text-primary);
}

.theme-text small {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.5;
}
</style>
