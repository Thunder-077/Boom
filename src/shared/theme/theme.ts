import { ref } from "vue";

export type ThemeId = "mist-blue" | "apricot-cream" | "mint-frost" | "sky-bloom" | "rose-dawn" | "lavender-mist";

export interface ThemeOption {
  id: ThemeId;
  label: string;
  description: string;
  accent: string;
  surface: string;
}

export const THEME_STORAGE_KEY = "academic-admin-theme";

export const THEME_OPTIONS: ThemeOption[] = [
  {
    id: "mist-blue",
    label: "雾蓝",
    description: "轻玻璃、通透、桌面感最强，适合作为默认控制台风格。",
    accent: "#1768ac",
    surface: "#eff4fb",
  },
  {
    id: "apricot-cream",
    label: "晴杏奶油",
    description: "更明亮温暖，像清晨阳光落在纸面上的办公桌，轻快但不甜腻。",
    accent: "#d7864d",
    surface: "#f8efe3",
  },
  {
    id: "mint-frost",
    label: "薄荷云白",
    description: "更清新透亮，像带一口凉意的早春空气，适合长时间盯着看。",
    accent: "#47a08b",
    surface: "#eaf8f3",
  },
  {
    id: "sky-bloom",
    label: "晴空柠蓝",
    description: "更明快清爽，像高亮度的天空和浅阳光，整体更有轻盈感。",
    accent: "#4f8fe8",
    surface: "#edf5ff",
  },
  {
    id: "rose-dawn",
    label: "玫瑰晨曦",
    description: "偏粉金的柔亮路线，像晨光落在浅色纸张上，明快又有一点精致感。",
    accent: "#db7b95",
    surface: "#fdf0f4",
  },
  {
    id: "lavender-mist",
    label: "雾光薰衣草",
    description: "偏冷白的淡紫路线，更轻盈梦幻，但整体仍保持办公界面的克制。",
    accent: "#8a86dc",
    surface: "#f4f2ff",
  },
];

const FALLBACK_THEME: ThemeId = "mist-blue";
const currentTheme = ref<ThemeId>(FALLBACK_THEME);

function isThemeId(value: string | null | undefined): value is ThemeId {
  return THEME_OPTIONS.some((option) => option.id === value);
}

function readStoredTheme(): ThemeId {
  if (typeof window === "undefined") {
    return FALLBACK_THEME;
  }
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  return isThemeId(stored) ? stored : FALLBACK_THEME;
}

function writeThemeToDom(themeId: ThemeId) {
  if (typeof document === "undefined") {
    return;
  }
  document.documentElement.dataset.theme = themeId;
}

export function initializeTheme() {
  const resolvedTheme = readStoredTheme();
  currentTheme.value = resolvedTheme;
  writeThemeToDom(resolvedTheme);
}

export function setTheme(themeId: ThemeId) {
  currentTheme.value = themeId;
  writeThemeToDom(themeId);
  if (typeof window !== "undefined") {
    window.localStorage.setItem(THEME_STORAGE_KEY, themeId);
  }
}

export function useThemeState() {
  return {
    currentTheme,
    setTheme,
    options: THEME_OPTIONS,
  };
}
