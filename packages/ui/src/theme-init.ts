import { applyTheme, readStoredTheme } from "./theme-internal";
import type { Theme } from "./theme";

const DEFAULT_THEME: Theme = "dark";
const DEFAULT_STORAGE_KEY = "unfour.theme";

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
