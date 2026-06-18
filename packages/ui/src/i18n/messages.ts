import enMessages from "./locales/en.json";
import zhCnMessages from "./locales/zh-CN.json";

export const supportedLocales = ["en", "zh-CN"] as const;

export type Locale = (typeof supportedLocales)[number];
export type TranslationParams = Record<string, string | number>;
export type TFunction = (
  key: string,
  paramsOrFallback?: TranslationParams | string,
  fallback?: string,
) => string;

type TranslationTree = {
  [key: string]: string | TranslationTree;
};

export const defaultLocale: Locale = "en";

const localeMessages: Record<Locale, TranslationTree> = {
  en: enMessages,
  "zh-CN": zhCnMessages,
};

const localeLabels: Record<Locale, string> = {
  en: "English",
  "zh-CN": "简体中文",
};

export function isSupportedLocale(locale: string): locale is Locale {
  return supportedLocales.includes(locale as Locale);
}

export function normalizeLocale(locale: string | null | undefined): Locale {
  if (!locale) {
    return defaultLocale;
  }

  if (isSupportedLocale(locale)) {
    return locale;
  }

  const normalized = locale.replace("_", "-").toLowerCase();
  if (normalized.startsWith("zh")) {
    return "zh-CN";
  }
  if (normalized.startsWith("en")) {
    return "en";
  }

  return defaultLocale;
}

export function getLocaleLabel(locale: Locale) {
  return localeLabels[locale];
}

export function translate(
  locale: Locale,
  key: string,
  paramsOrFallback?: TranslationParams | string,
  fallback?: string,
) {
  const params = typeof paramsOrFallback === "string" ? undefined : paramsOrFallback;
  const explicitFallback = typeof paramsOrFallback === "string" ? paramsOrFallback : fallback;
  const value =
    findMessage(localeMessages[locale], key) ??
    (locale === defaultLocale ? undefined : findMessage(localeMessages[defaultLocale], key)) ??
    explicitFallback ??
    key;

  return interpolate(value, params);
}

export function createTranslator(locale: Locale): TFunction {
  return (key, paramsOrFallback, fallback) =>
    translate(locale, key, paramsOrFallback, fallback);
}

function findMessage(messages: TranslationTree, key: string) {
  let current: string | TranslationTree | undefined = messages;

  for (const segment of key.split(".")) {
    if (!current || typeof current === "string") {
      return undefined;
    }
    current = current[segment];
  }

  return typeof current === "string" ? current : undefined;
}

function interpolate(message: string, params: TranslationParams | undefined) {
  if (!params) {
    return message;
  }

  return message.replace(/\{(\w+)\}/g, (match, key) =>
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : match,
  );
}
