import { ref } from "vue";

export type ThemeId = "mist-blue" | "sand-dune-mist" | "glacier-dew" | "sky-bloom" | "peach-blush" | "lavender-mist";

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
    id: "sand-dune-mist",
    label: "简白清蓝",
    description: "纯白基底配合清远蓝色点缀，整体更安静克制，边界清晰利落。",
    accent: "#0078d4",
    surface: "#fbfcfe",
  },
  {
    id: "glacier-dew",
    label: "冰川晨露",
    description: "冷白与浅青蓝的通透路线，像清晨冰川空气，视觉更干净克制。",
    accent: "#4a9ab3",
    surface: "#f3fbfe",
  },
  {
    id: "sky-bloom",
    label: "晴空柠蓝",
    description: "更明快清爽，像高亮度的天空和浅阳光，整体更有轻盈感。",
    accent: "#4f8fe8",
    surface: "#edf5ff",
  },
  {
    id: "peach-blush",
    label: "白桃雾粉",
    description: "更浅更淡的白桃粉雾感，减少甜度，保持柔和精致。",
    accent: "#d68ea2",
    surface: "#fff8fb",
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
