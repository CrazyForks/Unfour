import * as React from "react";

export type Theme = "light" | "dark";

export type ThemeContextValue = {
  setTheme: (theme: Theme) => void;
  theme: Theme;
  toggleTheme: () => void;
};

const DEFAULT_THEME: Theme = "dark";
const DEFAULT_STORAGE_KEY = "unfour.theme";

const ThemeContext = React.createContext<ThemeContextValue>({
  setTheme: () => undefined,
  theme: DEFAULT_THEME,
  toggleTheme: () => undefined,
});

export function ThemeProvider({
  children,
  defaultTheme = DEFAULT_THEME,
  storageKey = DEFAULT_STORAGE_KEY,
}: {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
}) {
  const [theme, setThemeState] = React.useState<Theme>(
    () => readStoredTheme(storageKey) ?? defaultTheme,
  );

  React.useLayoutEffect(() => {
    applyTheme(theme);
  }, [theme]);

  const setTheme = React.useCallback(
    (nextTheme: Theme) => {
      setThemeState(nextTheme);
      writeStoredTheme(storageKey, nextTheme);
    },
    [storageKey],
  );

  const toggleTheme = React.useCallback(() => {
    setThemeState((current) => {
      const next: Theme = current === "dark" ? "light" : "dark";
      writeStoredTheme(storageKey, next);
      return next;
    });
  }, [storageKey]);

  const value = React.useMemo<ThemeContextValue>(
    () => ({ setTheme, theme, toggleTheme }),
    [setTheme, theme, toggleTheme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  return React.useContext(ThemeContext);
}

export function initializeTheme({
  defaultTheme = DEFAULT_THEME,
  storageKey = DEFAULT_STORAGE_KEY,
}: {
  defaultTheme?: Theme;
  storageKey?: string;
} = {}): Theme {
  const theme = readStoredTheme(storageKey) ?? defaultTheme;
  applyTheme(theme);
  return theme;
}

function applyTheme(theme: Theme) {
  globalThis.document?.documentElement.setAttribute("data-theme", theme);
}

function readStoredTheme(storageKey: string): Theme | null {
  try {
    const stored = globalThis.localStorage?.getItem(storageKey);
    return stored === "light" || stored === "dark" ? stored : null;
  } catch {
    return null;
  }
}

function writeStoredTheme(storageKey: string, theme: Theme) {
  try {
    globalThis.localStorage?.setItem(storageKey, theme);
  } catch {
    // Ignore storage failures; theme still applies for the current session.
  }
}
