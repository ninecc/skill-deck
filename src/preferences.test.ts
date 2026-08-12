// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import {
  agentOptions,
  defaultLanguage,
  loadPreferences,
  resolvedTheme,
  validateTranslationProxy,
} from "./preferences";

describe("preferences", () => {
  it("resolves system theme without changing explicit themes", () => {
    expect(resolvedTheme("system", true)).toBe("dark");
    expect(resolvedTheme("system", false)).toBe("light");
    expect(resolvedTheme("plum", false)).toBe("plum");
  });

  it("uses the fixed target list and falls back to English", () => {
    expect(defaultLanguage("zh-TW")).toBe("zh-Hant");
    expect(defaultLanguage("ja-JP")).toBe("ja");
    expect(defaultLanguage("nl-NL")).toBe("en");
  });

  it("ships the pinned CLI's global Agent override list", () => {
    expect(agentOptions).toContain("aider-desk");
    expect(agentOptions).not.toContain("future" as never);
    expect(agentOptions).not.toContain("eve" as never);
    expect(agentOptions).not.toContain("promptscript" as never);
  });

  it("accepts only credential-free root HTTP(S) proxy URLs", () => {
    expect(validateTranslationProxy("")).toBeNull();
    expect(validateTranslationProxy("http://127.0.0.1:7890")).toBeNull();
    expect(validateTranslationProxy("https://proxy.example")).toBeNull();
    expect(validateTranslationProxy("socks5://127.0.0.1:1080")).toBe("invalid");
    expect(validateTranslationProxy("http://user:secret@proxy.example")).toBe(
      "invalid",
    );
    expect(validateTranslationProxy("http://proxy.example/path")).toBe(
      "invalid",
    );
    expect(validateTranslationProxy("http://proxy.example/")).toBe("invalid");
  });

  it("drops an invalid persisted proxy", () => {
    localStorage.setItem(
      "skill-deck-preferences",
      JSON.stringify({ translationProxy: "http://user:secret@proxy.example" }),
    );
    expect(loadPreferences("en-US").translationProxy).toBe("");
  });

  it("migrates the legacy explicit UI locale once", () => {
    localStorage.clear();
    localStorage.setItem("skill-deck-locale", "zh-CN");
    expect(loadPreferences("en-US").uiLocale).toBe("zh-CN");
  });
});
