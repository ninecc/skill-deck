import { describe, expect, it } from "vitest";
import {
  canonicalReviewMarker,
  isReviewScenarioId,
  reviewScenarioIds,
  reviewScenarios,
} from "./scenarios";

describe("UI review scenarios", () => {
  it("keeps every declared id backed by a typed scenario", () => {
    expect(Object.keys(reviewScenarios).sort()).toEqual(
      [...reviewScenarioIds].sort(),
    );
  });

  it("separates canonical, empty, long and Chinese shell pressure states", () => {
    expect(reviewScenarios["shell-ready"].runtime).not.toBe("pending");
    expect(reviewScenarios["shell-empty"].runtime).not.toBe("pending");
    expect(reviewScenarios["shell-loading"].runtime).toBe("pending");
    expect(reviewScenarios["shell-long"].preview.text?.length).toBeGreaterThan(
      reviewScenarios["shell-ready"].preview.text?.length ?? 0,
    );
    expect(reviewScenarios["shell-zh"].locale).toBe("zh-CN");
    expect(reviewScenarios["content-tree"].reviewState).toBe("tree");
    expect(
      reviewScenarios["content-tree"].tree.filter((entry) => !entry.directory),
    ).toHaveLength(5);
    expect(reviewScenarios["content-translation"].translatedText).toContain(
      "请教 Matt",
    );
    expect(reviewScenarios["lifecycle-loading"].runtime).toBe("pending");
    expect(reviewScenarios["lifecycle-runtime-failure"].runtime).toMatchObject({
      ready: false,
      errorCode: "runtime_not_found",
    });
    expect(reviewScenarios["lifecycle-preview-failure"].previewFailure).toBe(
      "SKILL.md could not be rendered",
    );
    expect(
      reviewScenarios["lifecycle-discovery-search"].searchResults,
    ).toHaveLength(3);
    expect(
      reviewScenarioIds.filter((id) => id.startsWith("settings-proof-")),
    ).toHaveLength(10);
    expect(reviewScenarios["settings-proof-general"].settingsState).toBe(
      "general",
    );
    expect(
      reviewScenarios["settings-proof-translation-invalid"].settingsState,
    ).toBe("translation-invalid");
    expect(
      reviewScenarios["settings-proof-installation-explicit"].preferences
        ?.agents,
    ).toEqual(["codex"]);
    expect(reviewScenarios["settings-proof-appearance-light"].theme).toBe(
      "light",
    );
    expect(reviewScenarios["settings-proof-installation-zh"].locale).toBe(
      "zh-CN",
    );
    expect(canonicalReviewMarker).toContain("CANONICAL_REVIEW_FIXTURE");
  });

  it("rejects unknown URL scenario ids", () => {
    expect(isReviewScenarioId("shell-ready")).toBe(true);
    expect(isReviewScenarioId("production")).toBe(false);
  });
});
