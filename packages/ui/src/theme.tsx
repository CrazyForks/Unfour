import * as React from "react";
import { applyTheme, readStoredTheme, writeStoredTheme } from "./theme-internal";

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

// eslint-disable-next-line react-refresh/only-export-components -- co-located hook with its context provider; splitting would create circular imports
export function useTheme(): ThemeContextValue {
  return React.useContext(ThemeContext);
}
