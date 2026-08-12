import { describe, expect, it } from "vitest";
import { catalogs } from "./i18n";
import { commandLabels } from "./nativeMenu";

describe("native menu projection", () => {
  it("localizes labels and projects translation toggle state", () => {
    expect(commandLabels(catalogs.en, false)["translate-skill"]).toBe(
      "Show Translation",
    );
    expect(commandLabels(catalogs.en, true)["translate-skill"]).toBe(
      "Hide Translation",
    );
    expect(commandLabels(catalogs["zh-CN"], false).settings).toBe("设置");
  });
});
