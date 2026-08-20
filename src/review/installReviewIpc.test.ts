// @vitest-environment jsdom
import { invoke } from "@tauri-apps/api/core";
import { clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";
import { installReviewIpc } from "./installReviewIpc";
import { reviewScenarios } from "./scenarios";

afterEach(() => clearMocks());

describe("UI review IPC", () => {
  it("routes shell commands to the selected deterministic scenario", async () => {
    const scenario = reviewScenarios["shell-ready"];
    installReviewIpc(scenario);

    await expect(invoke("runtime_status")).resolves.toEqual(scenario.runtime);
    await expect(
      invoke("preview_tree", { skill: "ask-matt" }),
    ).resolves.toEqual(scenario.tree);
    await expect(
      invoke("read_preview", { skill: "ask-matt", path: "SKILL.md" }),
    ).resolves.toEqual(scenario.preview);
  });

  it("rejects product commands that the shell scenario did not declare", async () => {
    installReviewIpc(reviewScenarios["shell-empty"]);

    await expect(invoke("remove_skill", { name: "ask-matt" })).rejects.toThrow(
      "Unhandled review IPC command: remove_skill",
    );
  });

  it("provides deterministic translation success and failure states", async () => {
    const success = reviewScenarios["content-translation"];
    installReviewIpc(success);
    await expect(invoke("translate_preview")).resolves.toMatchObject({
      translatedText: success.translatedText,
    });
    clearMocks();

    installReviewIpc(reviewScenarios["content-translation-error"]);
    await expect(invoke("translate_preview")).rejects.toMatchObject({
      code: "translation_unavailable",
    });
  });
});
