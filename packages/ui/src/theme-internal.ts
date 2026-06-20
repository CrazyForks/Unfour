import type { Theme } from "./theme";

export function applyTheme(theme: Theme) {
  globalThis.document?.documentElement.setAttribute("data-theme", theme);
}

export function readStoredTheme(storageKey: string): Theme | null {
  try {
    const stored = globalThis.localStorage?.getItem(storageKey);
    return stored === "light" || stored === "dark" ? stored : null;
  } catch {
    return null;
  }
}

export function writeStoredTheme(storageKey: string, theme: Theme) {
  try {
    globalThis.localStorage?.setItem(storageKey, theme);
  } catch {
    // Ignore storage failures; theme still applies for the current session.
  }
}
