import { describe, expect, it } from "vitest";
import {
  catalogs,
  effectiveLocale,
  preferredLocale,
  systemLocale,
} from "./i18n";

describe("i18n catalog", () => {
  it("keeps both locales structurally aligned", () => {
    expect(Object.keys(catalogs.en).sort()).toEqual(
      Object.keys(catalogs["zh-CN"]).sort(),
    );
  });

  it("maps Chinese system locales and falls back to English", () => {
    expect(systemLocale("zh-Hans-CN")).toBe("zh-CN");
    expect(systemLocale("fr-FR")).toBe("en");
  });

  it("uses a valid persisted override before the system locale", () => {
    expect(preferredLocale("en", "zh-CN")).toBe("en");
    expect(preferredLocale("invalid", "zh-CN")).toBe("zh-CN");
  });

  it("resolves system language preference without rewriting intent", () => {
    expect(effectiveLocale("system", ["zh-Hant-TW", "en-US"])).toBe("zh-CN");
    expect(effectiveLocale("en", ["zh-CN"])).toBe("en");
  });
});
