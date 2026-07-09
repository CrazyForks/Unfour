import * as React from "react";
import {
  applyTheme,
  readStoredThemeMode,
  resolveTheme,
  writeStoredThemeMode,
} from "./theme-internal";

export type Theme = "light" | "dark";
export type ThemeMode = "light" | "dark" | "system";

export type ThemeContextValue = {
  setThemeMode: (mode: ThemeMode) => void;
  theme: Theme;
  themeMode: ThemeMode;
};

const DEFAULT_THEME_MODE: ThemeMode = "dark";
const DEFAULT_STORAGE_KEY = "unfour.theme";
const SYSTEM_MEDIA_QUERY = "(prefers-color-scheme: dark)";

const ThemeContext = React.createContext<ThemeContextValue>({
  setThemeMode: () => undefined,
  theme: DEFAULT_THEME_MODE,
  themeMode: DEFAULT_THEME_MODE,
});

export function ThemeProvider({
  children,
  defaultThemeMode = DEFAULT_THEME_MODE,
  storageKey = DEFAULT_STORAGE_KEY,
}: {
  children: React.ReactNode;
  defaultThemeMode?: ThemeMode;
  storageKey?: string;
}) {
  const [themeMode, setThemeModeState] = React.useState<ThemeMode>(
    () => readStoredThemeMode(storageKey) ?? defaultThemeMode,
  );
  const [theme, setTheme] = React.useState<Theme>(
    () => resolveTheme(themeMode),
  );

  // Resolve and apply the theme whenever the user preference changes.
  React.useLayoutEffect(() => {
    const resolved = resolveTheme(themeMode);
    setTheme(resolved);
    applyTheme(resolved);
  }, [themeMode]);

  // While in "system" mode, follow the webview's color scheme live. Uses the
  // webview `matchMedia` API (fast) instead of any native OS theme query.
  React.useEffect(() => {
    if (themeMode !== "system") {
      return;
    }
    const mql = globalThis.matchMedia?.(SYSTEM_MEDIA_QUERY);
    if (!mql) {
      return;
    }
    const handleChange = () => {
      const resolved = resolveTheme("system");
      setTheme(resolved);
      applyTheme(resolved);
    };
    mql.addEventListener("change", handleChange);
    return () => mql.removeEventListener("change", handleChange);
  }, [themeMode]);

  const setThemeMode = React.useCallback(
    (nextMode: ThemeMode) => {
      setThemeModeState(nextMode);
      writeStoredThemeMode(storageKey, nextMode);
    },
    [storageKey],
  );

  const value = React.useMemo<ThemeContextValue>(
    () => ({ setThemeMode, theme, themeMode }),
    [setThemeMode, theme, themeMode],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components -- co-located hook with its context provider; splitting would create circular imports
export function useTheme(): ThemeContextValue {
  return React.useContext(ThemeContext);
}
