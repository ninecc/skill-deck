import { describe, expect, it, vi } from "vitest";
import {
  commandAvailability,
  createDispatcher,
  isNarrowBackShortcut,
  shortcutCommand,
  type CommandContext,
} from "./commands";

const ready: CommandContext = {
  runtimeReady: true,
  inventoryCount: 1,
  selected: true,
  mutationActive: false,
  modal: null,
  document: "supported",
};

describe("application commands", () => {
  it("revalidates availability when dispatching", () => {
    let context = ready;
    const execute = vi.fn();
    const dispatch = createDispatcher(() => context, execute);
    expect(dispatch("update-all").enabled).toBe(true);
    context = { ...ready, mutationActive: true };
    expect(dispatch("update-all")).toEqual({
      enabled: false,
      reason: "mutation-active",
    });
    expect(execute).toHaveBeenCalledTimes(1);
  });

  it("separates root and document command availability", () => {
    const loading = { ...ready, document: "loading" as const };
    expect(commandAvailability("reveal-skill", loading).enabled).toBe(true);
    expect(commandAvailability("translate-skill", loading).reason).toBe(
      "document-loading",
    );
  });

  it("keeps loaded Preview reading available during a mutation", () => {
    const mutating = { ...ready, mutationActive: true };
    expect(commandAvailability("translate-skill", mutating).enabled).toBe(true);
    expect(commandAvailability("reveal-skill", mutating).enabled).toBe(true);
    expect(commandAvailability("refresh-inventory", mutating).reason).toBe(
      "mutation-active",
    );
    expect(commandAvailability("remove-skill", mutating).reason).toBe(
      "mutation-active",
    );
  });

  it("does not replace an open top-level modal with Remove", () => {
    expect(
      commandAvailability("remove-skill", { ...ready, modal: "settings" }),
    ).toEqual({ enabled: false, reason: "modal-active" });
    expect(
      commandAvailability("find-installed", { ...ready, modal: "settings" }),
    ).toEqual({ enabled: false, reason: "modal-active" });
  });

  it("maps shared desktop shortcuts without claiming New", () => {
    expect(
      shortcutCommand({
        key: "f",
        metaKey: true,
        ctrlKey: false,
        shiftKey: false,
      }),
    ).toBe("find-installed");
    expect(
      shortcutCommand({
        key: "I",
        metaKey: false,
        ctrlKey: true,
        shiftKey: true,
      }),
    ).toBe("find-install");
    expect(
      shortcutCommand({
        key: "n",
        metaKey: true,
        ctrlKey: false,
        shiftKey: false,
      }),
    ).toBeNull();
    expect(
      isNarrowBackShortcut(
        { key: "ArrowLeft", metaKey: false, altKey: true },
        true,
      ),
    ).toBe(true);
    expect(
      isNarrowBackShortcut({ key: "[", metaKey: true, altKey: false }, false),
    ).toBe(false);
  });
});
