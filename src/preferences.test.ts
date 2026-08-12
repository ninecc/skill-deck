import { describe, expect, it } from "vitest";
import { agentOptions, defaultLanguage, resolvedTheme } from "./preferences";

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
});
