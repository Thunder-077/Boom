<template>
  <section class="settings-layout">
    <aside class="settings-nav card-shell">
      <p class="nav-kicker">设置分区</p>
      <button
        v-for="section in sections"
        :key="section.key"
        type="button"
        class="section-link"
        :class="{ active: section.key === activeSection }"
        @click="activeSection = section.key"
      >
        <span class="material-symbols-rounded" aria-hidden="true">{{ section.icon }}</span>
        <span class="section-copy">
          <strong>{{ section.label }}</strong>
          <small>{{ section.description }}</small>
        </span>
      </button>
    </aside>

    <div class="settings-content">
      <section v-if="activeSection === 'appearance'" class="settings-card card-shell">
        <header class="settings-head">
          <div>
            <span class="section-kicker">外观</span>
            <h3>主题配色</h3>
            <p>主题会立即应用并保存到本地，后续打开应用时自动恢复。</p>
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

      <section v-if="activeSection === 'workspace'" class="settings-card card-shell">
        <header class="settings-head">
          <div>
            <span class="section-kicker">工作区</span>
            <h3>界面偏好</h3>
            <p>这里预留给后续的布局密度、默认页签、导航行为等工作区偏好设置。</p>
          </div>
          <span class="coming-pill">规划中</span>
        </header>

        <div class="placeholder-grid">
          <article class="placeholder-item">
            <strong>布局密度</strong>
            <p>后续可配置舒适、标准、紧凑三种工作密度。</p>
          </article>
          <article class="placeholder-item">
            <strong>默认首页</strong>
            <p>后续可设置启动后默认进入的业务模块。</p>
          </article>
          <article class="placeholder-item">
            <strong>导航偏好</strong>
            <p>后续可选择是否默认展开二级菜单与记忆上次状态。</p>
          </article>
        </div>
      </section>

      <section v-if="activeSection === 'about'" class="settings-card card-shell">
        <header class="settings-head">
          <div>
            <span class="section-kicker">系统</span>
            <h3>说明与后续扩展</h3>
            <p>设置页采用分区结构，后续新增新能力时可以按模块直接扩展。</p>
          </div>
        </header>

        <ul class="about-list">
          <li>主题系统已支持本地持久化与即时切换。</li>
          <li>设置导航采用配置分区结构，便于后续继续添加。</li>
          <li>界面样式基于 CSS 变量实现，新增主题时只需要补 token。</li>
        </ul>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { ThemeOption } from "../../../shared/theme/theme";
import { useThemeState } from "../../../shared/theme/theme";

type SettingsSectionKey = "appearance" | "workspace" | "about";
type SettingsSection = {
  key: SettingsSectionKey;
  label: string;
  description: string;
  icon: string;
};

const { currentTheme, options, setTheme } = useThemeState();
const activeSection = ref<SettingsSectionKey>("appearance");

// Keep settings sections in a single registry so later additions only need one entry point.
const sections: SettingsSection[] = [
  { key: "appearance", label: "配色主题", description: "切换系统的整体视觉风格", icon: "palette" },
  { key: "workspace", label: "工作区偏好", description: "预留给布局与行为偏好", icon: "tune" },
  { key: "about", label: "说明", description: "查看设置结构与扩展说明", icon: "info" },
];

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
.settings-layout {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  gap: 20px;
  min-height: 0;
}

.settings-nav,
.settings-card {
  padding: 20px;
  border-radius: var(--radius-card-large);
}

.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: var(--surface-nav-panel);
}

.nav-kicker,
.section-kicker {
  margin: 0;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.section-link {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  width: 100%;
  padding: 12px 14px;
  border: 1px solid var(--border-default);
  border-radius: 18px;
  background: var(--surface-nav-item);
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.18s ease,
    background-color 0.18s ease,
    box-shadow 0.18s ease;
}

.section-link:hover {
  border-color: var(--accent-border-soft);
  background: var(--surface-nav-item-hover);
}

.section-link.active {
  border-color: rgba(var(--accent-rgb), 0.2);
  background: var(--surface-nav-item-active);
  box-shadow: 0 10px 22px rgba(var(--accent-rgb), 0.06);
}

.section-link .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 20px;
  line-height: 1;
  color: var(--accent-primary);
}

.section-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.section-copy strong {
  font-size: 14px;
}

.section-copy small {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.settings-content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-card {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
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

.theme-pill,
.coming-pill {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.theme-pill {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.coming-pill {
  background: rgba(255, 244, 223, 0.82);
  color: var(--color-warning);
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
  background: linear-gradient(180deg, color-mix(in srgb, var(--preview-surface) 86%, white), var(--preview-surface));
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
  background: color-mix(in srgb, var(--preview-surface) 74%, white);
  border: 1px solid rgba(255, 255, 255, 0.6);
}

.preview-panel {
  top: 12px;
  left: 56px;
  right: 12px;
  bottom: 12px;
  background: var(--surface-nav-panel);
  border: 1px solid rgba(255, 255, 255, 0.66);
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

.placeholder-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.placeholder-item {
  padding: 14px;
  border: 1px dashed var(--border-strong);
  border-radius: 18px;
  background: var(--surface-panel);
}

.placeholder-item strong {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
}

.placeholder-item p,
.about-list {
  margin: 0;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

.about-list {
  padding-left: 18px;
}

@media (max-width: 1080px) {
  .settings-layout {
    grid-template-columns: 1fr;
  }
}
</style>
