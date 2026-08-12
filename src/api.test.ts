import { describe, expect, it, vi } from "vitest";
import {
  addSkill,
  commandErrorCode,
  commandErrorMessage,
  translatePreview,
} from "./api";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

describe("command boundary", () => {
  it("omits CLI overrides by sending empty/default settings", async () => {
    invokeMock.mockResolvedValue({
      inventory: [],
      changedSkills: [],
      diagnostics: "",
    });
    await addSkill("owner/repo", null, { agents: [], copy: false });
    expect(invokeMock).toHaveBeenCalledWith("add_skill", {
      source: "owner/repo",
      skill: null,
      settings: { agents: [], copy: false },
    });
  });

  it("normalizes structured failures", () => {
    expect(commandErrorCode({ code: "busy" })).toBe("busy");
    expect(
      commandErrorMessage({ message: "Failed", diagnostics: "detail" }),
    ).toBe("Failed\ndetail");
  });

  it("passes the translation proxy override through the command boundary", async () => {
    invokeMock.mockResolvedValue({ translatedText: "hello" });
    await translatePreview("demo", "SKILL.md", "en", "http://127.0.0.1:7890");
    expect(invokeMock).toHaveBeenCalledWith("translate_preview", {
      skill: "demo",
      path: "SKILL.md",
      targetLanguage: "en",
      translationProxy: "http://127.0.0.1:7890",
    });
  });
});
