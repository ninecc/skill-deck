import { describe, expect, it } from "vitest";
import {
  commandErrorCode,
  commandErrorMessage,
  inventoryDiagnosticMessage,
} from "./api";

describe("commandErrorMessage", () => {
  it("normalizes Tauri payloads without unchecked casts", () => {
    expect(commandErrorMessage({ message: "Invalid skill" })).toBe(
      "Invalid skill",
    );
    expect(commandErrorMessage(new Error("Unavailable"))).toBe("Unavailable");
    expect(commandErrorMessage({ message: 42 })).toBeNull();
    expect(
      commandErrorMessage({
        message: "Resource limit",
        limit: 10,
        observed: 11,
      }),
    ).toBe("Resource limit (10 / 11)");
    expect(
      commandErrorMessage(
        {
          code: "io",
          message: "Could not inspect skill source",
          path: "/tmp/example-skill",
        },
        { io: "无法读取或写入本机文件。" },
      ),
    ).toBe("无法读取或写入本机文件。 (/tmp/example-skill)");
    expect(
      commandErrorMessage(
        {
          code: "unsupported_file_type",
          message: "Could not resolve external link",
          path: "/tmp/broken-skill",
        },
        { unsupported_file_type: "Skill 包含不支持的文件类型或链接。" },
      ),
    ).toBe("Skill 包含不支持的文件类型或链接。 (/tmp/broken-skill)");
  });

  it("preserves structured command error codes", () => {
    expect(commandErrorCode({ code: "conflict", message: "Exists" })).toBe(
      "conflict",
    );
    expect(commandErrorCode("conflict")).toBeNull();
  });

  it("does not repeat a diagnostic path that matches the logical path", () => {
    expect(
      inventoryDiagnosticMessage(
        {
          code: "invalid_structure",
          message: "Invalid entry",
          path: "/tmp/skills/invalid",
        },
        "/tmp/skills/invalid",
        { invalid_structure: "Invalid structure" },
      ),
    ).toBe("Invalid structure");
  });
});
