import { useMemo, useState } from "react";
import type { ThemeOption, ThemeId } from "../../../shared/theme/theme";
import { getCurrentTheme, setTheme, THEME_OPTIONS } from "../../../shared/theme/theme";
import { Tag } from "../../../widgets/common/index.react";

function previewStyle(theme: ThemeOption) {
  return {
    "--preview-surface": theme.surface,
    "--preview-accent": theme.accent,
  } as React.CSSProperties;
}

export default function SettingsPanel() {
  const [currentTheme, setCurrentTheme] = useState<ThemeId>(getCurrentTheme());
  const currentThemeLabel = useMemo(
    () => THEME_OPTIONS.find((option) => option.id === currentTheme)?.label ?? "未设置",
    [currentTheme],
  );

  function selectTheme(themeId: ThemeId) {
    setTheme(themeId);
    setCurrentTheme(themeId);
  }

  return (
    <section className="settings-card card-shell">
      <header className="settings-head">
        <div>
          <span className="section-kicker">外观</span>
          <h3>配色主题</h3>
        </div>
        <Tag>{currentThemeLabel}</Tag>
      </header>

      <div className="theme-grid">
        {THEME_OPTIONS.map((theme) => (
          <button
            key={theme.id}
            type="button"
            className={`theme-card ${theme.id === currentTheme ? "active" : ""}`}
            style={previewStyle(theme)}
            onClick={() => selectTheme(theme.id)}
          >
            <span className="theme-preview">
              <span className="preview-sidebar" />
              <span className="preview-panel" />
              <span className="preview-accent" />
            </span>
            <span className="theme-text">
              <strong>{theme.label}</strong>
              <small>{theme.description}</small>
            </span>
          </button>
        ))}
      </div>
    </section>
  );
}
