// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import { I18nProvider } from "./provider";
import { useI18n } from "./context";

afterEach(cleanup);
beforeEach(() => globalThis.localStorage?.clear());

function LocaleProbe() {
  const { locale, setLocale, t } = useI18n();
  return (
    <div>
      <span data-testid="locale">{locale}</span>
      <span data-testid="nav">{t("app.nav.database")}</span>
      <button onClick={() => setLocale("zh-CN")} type="button">
        zh
      </button>
    </div>
  );
}

describe("I18nProvider", () => {
  it("uses the initialLocale and translates with it", () => {
    render(
      <I18nProvider initialLocale="en">
        <LocaleProbe />
      </I18nProvider>,
    );

    expect(screen.getByTestId("locale").textContent).toBe("en");
    expect(screen.getByTestId("nav").textContent).toBe("Database");
  });

  it("updates locale and persists it to storage when setLocale runs", () => {
    render(
      <I18nProvider initialLocale="en" storageKey="test.locale">
        <LocaleProbe />
      </I18nProvider>,
    );

    act(() => {
      screen.getByRole("button", { name: "zh" }).click();
    });

    expect(screen.getByTestId("locale").textContent).toBe("zh-CN");
    expect(screen.getByTestId("nav").textContent).toBe("数据库");
    expect(globalThis.localStorage.getItem("test.locale")).toBe("zh-CN");
  });

  it("restores a previously stored locale on mount", () => {
    globalThis.localStorage.setItem("test.locale", "zh-CN");

    render(
      <I18nProvider storageKey="test.locale">
        <LocaleProbe />
      </I18nProvider>,
    );

    expect(screen.getByTestId("locale").textContent).toBe("zh-CN");
  });
});
