import { describe, expect, it, vi } from "vitest";
import { addSkill, commandErrorCode, commandErrorMessage } from "./api";

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
});
