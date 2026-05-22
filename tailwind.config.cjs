/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  corePlugins: {
    // The project already owns its global reset in src/shared/styles/base.css.
    preflight: false,
  },
  theme: {
    extend: {
      colors: {
        surface: {
          page: "var(--surface-page)",
          panel: "var(--surface-panel)",
          elevated: "var(--surface-elevated)",
          input: "var(--surface-input)",
        },
        text: {
          primary: "var(--text-primary)",
          secondary: "var(--text-secondary)",
          tertiary: "var(--text-tertiary)",
        },
        accent: {
          DEFAULT: "var(--accent-primary)",
          strong: "var(--accent-primary-strong)",
          soft: "var(--accent-soft)",
          softer: "var(--accent-softer)",
        },
        border: {
          DEFAULT: "var(--border-default)",
          strong: "var(--border-strong)",
        },
        success: "var(--color-success)",
        warning: "var(--color-warning)",
        danger: "var(--color-danger)",
        info: "var(--color-info)",
      },
      borderRadius: {
        xs: "var(--radius-xs)",
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
        xl: "var(--radius-xl)",
        pill: "var(--radius-pill)",
      },
      boxShadow: {
        soft: "var(--shadow-soft)",
        medium: "var(--shadow-medium)",
        strong: "var(--shadow-strong)",
      },
      fontFamily: {
        ui: "var(--font-ui)",
        mono: "var(--font-mono)",
      },
      fontSize: {
        xs: "var(--font-size-xs)",
        sm: "var(--font-size-sm)",
        base: "var(--font-size-base)",
        lg: "var(--font-size-lg)",
        xl: "var(--font-size-xl)",
        "2xl": "var(--font-size-2xl)",
        "3xl": "var(--font-size-3xl)",
      },
      spacing: {
        xs: "var(--space-xs)",
        sm: "var(--space-sm)",
        md: "var(--space-md)",
        lg: "var(--space-lg)",
        xl: "var(--space-xl)",
        "2xl": "var(--space-2xl)",
      },
      transitionDuration: {
        fast: "var(--transition-fast)",
        base: "var(--transition-base)",
        slow: "var(--transition-slow)",
      },
      transitionTimingFunction: {
        app: "var(--transition-ease)",
      },
    },
  },
  plugins: [],
};
