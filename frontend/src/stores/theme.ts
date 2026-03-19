import { create } from "zustand";

export type Theme = "light" | "dark" | "system";

const STORAGE_KEY = "lockso-theme";

function getStoredTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function applyTheme(theme: Theme) {
  const isDark =
    theme === "dark" ||
    (theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  document.documentElement.classList.toggle("dark", isDark);
}

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: getStoredTheme(),

  setTheme: (theme: Theme) => {
    localStorage.setItem(STORAGE_KEY, theme);
    applyTheme(theme);
    set({ theme });
  },
}));

// Listen for system preference changes when in "system" mode
if (typeof window !== "undefined") {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  mq.addEventListener("change", () => {
    const current = useThemeStore.getState().theme;
    if (current === "system") {
      applyTheme("system");
    }
  });

  // Apply on initial load (matches the inline script in index.html)
  applyTheme(getStoredTheme());
}
