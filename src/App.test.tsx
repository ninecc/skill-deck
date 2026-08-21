// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const demoSkill = {
  name: "demo",
  path: "/tmp/demo",
  scope: "global",
  agents: ["Future Agent"],
  source: "owner/repo",
  sourceUrl: null,
  sourceType: "github",
};

describe("CLI-backed workspace", () => {
  let container: HTMLDivElement;
  let root: ReturnType<typeof createRoot>;

  beforeEach(() => {
    Object.defineProperty(globalThis, "matchMedia", {
      configurable: true,
      value: vi.fn(() => ({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      })),
    });
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [demoSkill],
        });
      if (command === "preview_tree") return Promise.resolve([]);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    container = document.createElement("div");
    document.body.append(container);
    root = createRoot(container);
    (
      globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT: boolean }
    ).IS_REACT_ACT_ENVIRONMENT = true;
  });

  afterEach(async () => {
    await act(async () => root.unmount());
    container.remove();
    localStorage.clear();
    invokeMock.mockReset();
  });

  it("publishes startup Inventory without rendering hidden Agent targets", async () => {
    await act(async () => root.render(<App />));
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("runtime_status");
    expect(container.textContent).not.toContain("Future Agent");
    expect(container.textContent).toContain("owner/repo");
    expect(container.textContent).toContain("/tmp/demo");
    expect(container.textContent).toContain("Select an installed Skill");
    expect(container.querySelector(".choose-placeholder .icon")).not.toBeNull();
    expect(
      container.querySelector('[role="option"]')?.getAttribute("aria-selected"),
    ).toBe("false");
  });

  it("composes selected Skill identity into title and location rows", async () => {
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );

    const identity = container.querySelector(".skill-identity") as HTMLElement;
    const titleRow = identity.querySelector(".skill-title-row") as HTMLElement;
    const locationRow = identity.querySelector(
      ".skill-location-row",
    ) as HTMLElement;
    expect(titleRow.querySelector("h1")?.textContent).toBe("demo");
    expect(titleRow.querySelector(".skill-source")?.textContent).toContain(
      "owner/repo",
    );
    expect(locationRow.querySelector(".skill-path")?.textContent).toContain(
      "/tmp/demo",
    );
    expect(
      locationRow.querySelector('[aria-label="Open file tree"]'),
    ).not.toBeNull();
    expect(locationRow.querySelector(".compact-path-label")).toBeNull();
    expect(
      locationRow
        .querySelector('[aria-label="Open file tree"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("folder");
    expect(locationRow.children[0].classList.contains("path-control")).toBe(
      true,
    );
    expect(locationRow.children[1].classList.contains("skill-path")).toBe(true);
    expect(identity.children[0]).toBe(titleRow);
    expect(identity.children[1]).toBe(locationRow);
  });

  it("keeps the full install path as non-visual identity metadata", async () => {
    const longPath =
      "~/.agents/skills/a-deliberately-long-skill-name-for-layout-pressure";
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [{ ...demoSkill, path: longPath }],
        });
      if (command === "preview_tree") return Promise.resolve([]);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );

    const trigger = container.querySelector(
      '[aria-label="Open file tree"]',
    ) as HTMLButtonElement;
    const path = container.querySelector(".skill-path") as HTMLElement;
    expect(path.textContent).toContain(longPath);
    expect(path.title).toBe(longPath);
    expect(trigger.querySelector("span")).toBeNull();
    expect(trigger.title).toBe("Browse files");
  });

  it("renders startup chrome before the runtime probe completes", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("Skill Deck");
    expect(container.textContent).toContain("Loading Skills");
    expect(container.textContent).not.toContain("No global Skills");
    expect(container.querySelector(".spinner")).not.toBeNull();
    expect(
      (container.querySelector(".bar-actions .button") as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(container.querySelector('[aria-label="Settings"]')).not.toBeNull();
    expect(
      container
        .querySelector('[aria-label="Refresh Inventory"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("refresh");
    expect(
      container
        .querySelector('[aria-label="Update all"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("update-all");
  });

  it("distinguishes empty Inventory from filter misses on visible fields", async () => {
    await act(async () => root.render(<App />));
    const filter = container.querySelector(
      '[aria-label="Filter installed Skills"]',
    ) as HTMLInputElement;
    const setFilter = (value: string) => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(filter, value);
      filter.dispatchEvent(new Event("input", { bubbles: true }));
    };

    await act(async () => setFilter("Future Agent"));
    expect(container.textContent).toContain("No matching Skills");
    expect(container.textContent).not.toContain("No global Skills");
    await act(async () => setFilter("owner/repo"));
    expect(container.querySelector('[role="option"]')).not.toBeNull();
    await act(async () => setFilter("/tmp/demo"));
    expect(container.querySelector('[role="option"]')).not.toBeNull();

    await act(async () => root.unmount());
    root = createRoot(container);
    invokeMock.mockResolvedValue({
      ready: true,
      errorCode: null,
      version: "1.5.22",
      nodeVersion: "22.20.0",
      message: null,
      inventory: [],
    });
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("No global Skills are installed");
    expect(container.textContent).toContain(
      "Install a Skill to inspect its files and keep it updated.",
    );
    const emptyRecovery = container.querySelector(
      ".inventory-empty-state button",
    ) as HTMLButtonElement;
    expect(emptyRecovery.textContent).toContain("Find & install");
    await act(async () => emptyRecovery.click());
    expect(container.querySelector("#find-install-title")).not.toBeNull();
    await act(async () =>
      (
        container.querySelector(
          '[role="dialog"] [aria-label="Close"]',
        ) as HTMLButtonElement
      ).click(),
    );
    const emptyFilter = container.querySelector(
      '[aria-label="Filter installed Skills"]',
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(emptyFilter, "missing");
      emptyFilter.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(container.textContent).not.toContain("No matching Skills");
    expect(container.textContent).not.toContain("No global Skills");
  });

  it("publishes Retry Inventory without a separate list command", async () => {
    invokeMock
      .mockResolvedValueOnce({
        ready: false,
        errorCode: "runtime_not_found",
        version: null,
        nodeVersion: null,
        message: "unavailable",
        inventory: [],
      })
      .mockResolvedValueOnce({
        ready: true,
        errorCode: null,
        version: "1.5.22",
        nodeVersion: "22.20.0",
        message: null,
        inventory: [demoSkill],
      });
    await act(async () => root.render(<App />));
    const retry = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Retry",
    ) as HTMLButtonElement;
    await act(async () => retry.click());
    expect(container.querySelector('[role="option"]')?.textContent).toContain(
      "demo",
    );
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual([
      "runtime_status",
      "retry_runtime",
    ]);
  });

  it("reports when Refresh removes the selected Skill", async () => {
    invokeMock
      .mockResolvedValueOnce({
        ready: true,
        errorCode: null,
        version: "1.5.22",
        nodeVersion: "22.20.0",
        message: null,
        inventory: [demoSkill],
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        ready: true,
        errorCode: null,
        version: "1.5.22",
        nodeVersion: "22.20.0",
        message: null,
        inventory: [],
      });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    await act(async () =>
      (
        container.querySelector(
          '[aria-label="Refresh Inventory"]',
        ) as HTMLButtonElement
      ).click(),
    );
    expect(container.textContent).toContain(
      "The selected Skill is no longer installed.",
    );
    expect(container.textContent).not.toContain(
      "The command completed and Inventory was refreshed.",
    );
  });

  it("opens catalog search and source install in a separate sheet", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [demoSkill],
        });
      if (command === "search_skills")
        return Promise.resolve([
          {
            name: "found",
            slug: "owner/repo/found",
            source: "owner/repo",
            installs: 42,
          },
        ]);
      if (command === "add_skill")
        return Promise.resolve({
          inventory: [demoSkill],
          changedSkills: [],
          targetObserved: true,
          diagnostics: "",
        });
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    const inventoryPane = container.querySelector(".inventory-pane")!;
    expect(
      inventoryPane.querySelector('[aria-label="Search query"]'),
    ).toBeNull();
    expect(inventoryPane.textContent).not.toContain("Install from source");

    const open = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Find & install"),
    ) as HTMLButtonElement;
    await act(async () => open.click());
    const dialog = container.querySelector(
      '[role="dialog"][aria-labelledby="find-install-title"]',
    )!;
    expect(dialog.classList.contains("settings-sheet")).toBe(true);
    expect(dialog.querySelector(".discovery-footer button")?.textContent).toBe(
      "Install from source…",
    );

    const search = dialog.querySelector(
      '[aria-label="Search query"]',
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(search, "found");
      search.dispatchEvent(new Event("input", { bubbles: true }));
      search
        .closest("form")
        ?.dispatchEvent(
          new Event("submit", { bubbles: true, cancelable: true }),
        );
    });
    expect(dialog.textContent).toContain("found");
    const installResult = Array.from(dialog.querySelectorAll("button")).find(
      (button) => button.textContent === "Install",
    ) as HTMLButtonElement;
    await act(async () => installResult.click());

    const sourceTab = Array.from(dialog.querySelectorAll("button")).find(
      (button) => button.textContent === "From source",
    ) as HTMLButtonElement;
    await act(async () => sourceTab.click());
    expect(dialog.querySelector(".discovery-footer button")).toBeNull();
    const source = dialog.querySelector(
      ".source-install input",
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(source, "another/repo");
      source.dispatchEvent(new Event("input", { bubbles: true }));
      source
        .closest("form")
        ?.dispatchEvent(
          new Event("submit", { bubbles: true, cancelable: true }),
        );
    });
    expect(
      invokeMock.mock.calls.filter(([command]) => command === "add_skill"),
    ).toEqual([
      [
        "add_skill",
        {
          source: "owner/repo",
          skill: "found",
          settings: { agents: [], copy: false },
        },
      ],
      [
        "add_skill",
        {
          source: "another/repo",
          skill: null,
          settings: { agents: [], copy: false },
        },
      ],
    ]);

    await act(async () =>
      (
        dialog.querySelector('[aria-label="Close"]') as HTMLButtonElement
      ).click(),
    );
    expect(container.querySelector("#find-install-title")).toBeNull();

    await act(async () => open.click());
    const reopened = container.querySelector(
      '[role="dialog"][aria-labelledby="find-install-title"]',
    )!;
    expect(
      reopened.querySelector('.discovery-tabs [aria-current="page"]')
        ?.textContent,
    ).toBe("From source");
    expect(
      (reopened.querySelector(".source-install input") as HTMLInputElement)
        .value,
    ).toBe("another/repo");
  });

  it("retains discovery drafts across tabs but resets a normal reopening", async () => {
    await act(async () => root.render(<App />));
    const open = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Find & install"),
    ) as HTMLButtonElement;
    await act(async () => open.click());
    const dialog = container.querySelector(
      '[role="dialog"][aria-labelledby="find-install-title"]',
    )!;
    const search = dialog.querySelector(
      '[aria-label="Search query"]',
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(search, "typescript");
      search.dispatchEvent(new Event("input", { bubbles: true }));
      const sourceTab = Array.from(dialog.querySelectorAll("button")).find(
        (button) => button.textContent === "From source",
      ) as HTMLButtonElement;
      sourceTab.click();
    });
    const source = dialog.querySelector(
      ".source-install input",
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(source, "owner/repo");
      source.dispatchEvent(new Event("input", { bubbles: true }));
      const searchTab = Array.from(dialog.querySelectorAll("button")).find(
        (button) => button.textContent === "Search",
      ) as HTMLButtonElement;
      searchTab.click();
    });
    expect(
      (dialog.querySelector('[aria-label="Search query"]') as HTMLInputElement)
        .value,
    ).toBe("typescript");

    await act(async () =>
      (
        dialog.querySelector('[aria-label="Close"]') as HTMLButtonElement
      ).click(),
    );
    await act(async () => open.click());
    const reopened = container.querySelector(
      '[role="dialog"][aria-labelledby="find-install-title"]',
    )!;
    expect(
      reopened.querySelector('.discovery-tabs [aria-current="page"]')
        ?.textContent,
    ).toBe("Search");
    expect(
      (
        reopened.querySelector(
          '[aria-label="Search query"]',
        ) as HTMLInputElement
      ).value,
    ).toBe("");
    await act(async () =>
      (
        Array.from(reopened.querySelectorAll("button")).find(
          (button) => button.textContent === "From source",
        ) as HTMLButtonElement
      ).click(),
    );
    expect(
      (reopened.querySelector(".source-install input") as HTMLInputElement)
        .value,
    ).toBe("");
  });

  it("replaces the Search footer shortcut with install progress", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [demoSkill],
        });
      if (command === "search_skills")
        return Promise.resolve([
          {
            name: "found",
            slug: "owner/repo/found",
            source: "owner/repo",
            installs: 42,
          },
        ]);
      if (command === "add_skill") return new Promise(() => undefined);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    const open = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Find & install"),
    ) as HTMLButtonElement;
    await act(async () => open.click());
    const dialog = container.querySelector(
      '[role="dialog"][aria-labelledby="find-install-title"]',
    )!;
    const search = dialog.querySelector(
      '[aria-label="Search query"]',
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(search, "found");
      search.dispatchEvent(new Event("input", { bubbles: true }));
      search
        .closest("form")
        ?.dispatchEvent(
          new Event("submit", { bubbles: true, cancelable: true }),
        );
    });
    const install = Array.from(dialog.querySelectorAll("button")).find(
      (button) => button.textContent === "Install",
    ) as HTMLButtonElement;
    await act(async () => install.click());

    const footer = dialog.querySelector(".discovery-footer")!;
    expect(footer.textContent).toContain(
      "Closing this dialog does not cancel the command.",
    );
    expect(footer.querySelector("button")).toBeNull();
  });

  it("describes the destructive target and restores focus after removal", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [demoSkill],
        });
      if (command === "preview_tree") return Promise.resolve([]);
      if (command === "remove_skill")
        return Promise.resolve({
          inventory: [],
          changedSkills: ["demo"],
          targetObserved: true,
          diagnostics: "removed demo",
        });
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    const trigger = container.querySelector(
      '[aria-label="Remove"]',
    ) as HTMLButtonElement;
    await act(async () => trigger.click());
    const dialog = container.querySelector(
      '[role="dialog"][aria-labelledby="remove-title"]',
    )!;
    expect(dialog.textContent).toContain("Remove “demo”?");
    expect(dialog.textContent).toContain("/tmp/demo");
    expect(document.activeElement?.classList.contains("cancel-button")).toBe(
      true,
    );
    const confirm = Array.from(dialog.querySelectorAll("button")).find(
      (button) => button.textContent === "Remove Skill",
    ) as HTMLButtonElement;
    await act(async () => confirm.click());
    await act(async () => Promise.resolve());
    expect(invokeMock).toHaveBeenCalledWith("remove_skill", { name: "demo" });
    expect(container.querySelector("#remove-title")).toBeNull();
    expect(document.activeElement?.id).toBe("installed-heading");
  });

  it("retries and reveals the same path from Preview failure recovery", async () => {
    invokeMock.mockImplementation((command: string, args?: unknown) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [demoSkill],
        });
      if (command === "preview_tree")
        return Promise.resolve([
          {
            path: "SKILL.md",
            name: "SKILL.md",
            level: 1,
            directory: false,
            size: 12,
            viewer: "markdown",
            unsupportedReason: null,
          },
        ]);
      if (command === "read_preview")
        return Promise.reject(new Error("SKILL.md could not be rendered"));
      if (command === "reveal_path") return Promise.resolve(args);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    const recovery = container.querySelector(".preview-error")!;
    expect(recovery.textContent).toContain("Preview couldn’t be loaded");
    expect(recovery.textContent).toContain("Retry the same path or reveal it.");
    const reveal = Array.from(recovery.querySelectorAll("button")).find(
      (button) => button.textContent === "Reveal file",
    ) as HTMLButtonElement;
    await act(async () => reveal.click());
    expect(invokeMock).toHaveBeenCalledWith("reveal_path", {
      skill: "demo",
      path: "SKILL.md",
    });
    const retry = Array.from(recovery.querySelectorAll("button")).find(
      (button) => button.textContent === "Retry",
    ) as HTMLButtonElement;
    await act(async () => retry.click());
    expect(
      invokeMock.mock.calls.filter(([command]) => command === "read_preview"),
    ).toHaveLength(2);
    expect(invokeMock).toHaveBeenLastCalledWith("read_preview", {
      skill: "demo",
      path: "SKILL.md",
    });
  });

  it("persists a translation proxy only after valid Apply", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );
    const translationSection = Array.from(
      container.querySelectorAll(".settings-nav button"),
    ).find(
      (button) => button.textContent === "Translation",
    ) as HTMLButtonElement;
    await act(async () => translationSection.click());
    expect(container.querySelector(".dialog-footer")?.textContent).toContain(
      "apply only when you choose Apply proxy",
    );
    const proxyLabel = Array.from(container.querySelectorAll("label")).find(
      (label) => label.textContent?.includes("Translation proxy"),
    ) as HTMLLabelElement;
    const input = proxyLabel.querySelector("input") as HTMLInputElement;
    expect(input.value).toBe("");
    expect(input.hasAttribute("placeholder")).toBe(false);
    const setInput = (value: string) => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    };
    await act(async () => {
      setInput("http://user:secret@proxy.example");
    });
    await act(async () =>
      (
        Array.from(container.querySelectorAll("button")).find(
          (button) => button.textContent === "Apply proxy",
        ) as HTMLButtonElement
      ).click(),
    );
    expect(localStorage.getItem("skill-deck-preferences")).not.toContain(
      "secret",
    );
    await act(async () => {
      setInput("http://127.0.0.1:7890");
    });
    expect(localStorage.getItem("skill-deck-preferences")).not.toContain(
      "7890",
    );
    await act(async () =>
      (
        Array.from(container.querySelectorAll(".settings-nav button")).find(
          (button) => button.textContent === "General",
        ) as HTMLButtonElement
      ).click(),
    );
    await act(async () => translationSection.click());
    expect(
      (
        Array.from(container.querySelectorAll("label"))
          .find((label) => label.textContent?.includes("Translation proxy"))
          ?.querySelector("input") as HTMLInputElement
      ).value,
    ).toBe("http://127.0.0.1:7890");
    await act(async () =>
      (
        Array.from(container.querySelectorAll("button")).find(
          (button) => button.textContent === "Apply proxy",
        ) as HTMLButtonElement
      ).click(),
    );
    expect(localStorage.getItem("skill-deck-preferences")).toContain("7890");
  });

  it("keeps non-proxy Settings changes immediate across sections", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );

    const navButtons = () =>
      Array.from(container.querySelectorAll(".settings-nav button"));
    const localeSelect = container.querySelector(
      ".general-settings select",
    ) as HTMLSelectElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLSelectElement.prototype,
        "value",
      )?.set?.call(localeSelect, "en");
      localeSelect.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"uiLocale":"en"',
    );
    await act(async () =>
      (
        navButtons().find(
          (button) => button.textContent === "Appearance",
        ) as HTMLButtonElement
      ).click(),
    );
    const sand = Array.from(container.querySelectorAll(".theme-tile"))
      .find((tile) => tile.textContent === "Sand")
      ?.querySelector("input") as HTMLInputElement;
    await act(async () => sand.click());
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"theme":"sand"',
    );

    await act(async () =>
      (
        navButtons().find(
          (button) => button.textContent === "Translation",
        ) as HTMLButtonElement
      ).click(),
    );
    const target = container.querySelector(
      ".translation-settings select",
    ) as HTMLSelectElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLSelectElement.prototype,
        "value",
      )?.set?.call(target, "en");
      target.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"targetLanguage":"en"',
    );

    await act(async () =>
      (
        navButtons().find(
          (button) => button.textContent === "Installation",
        ) as HTMLButtonElement
      ).click(),
    );
    const copyMethod = Array.from(
      container.querySelectorAll(".installation-settings .choice"),
    )
      .find((label) => label.textContent?.includes("Always copy"))
      ?.querySelector("input") as HTMLInputElement;
    await act(async () => copyMethod.click());
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"copy":true',
    );
    const explicit = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Choose explicit targets",
    ) as HTMLButtonElement;
    await act(async () => explicit.click());
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"agents":["codex"]',
    );
    const cline = Array.from(
      container.querySelectorAll(".agent-options .choice"),
    )
      .find((label) => label.textContent?.trim() === "cline")
      ?.querySelector("input") as HTMLInputElement;
    await act(async () => cline.click());
    expect(localStorage.getItem("skill-deck-preferences")).toContain(
      '"agents":["codex","cline"]',
    );
  });

  it("navigates Settings by keyboard and restores focus for every dismissal", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    const trigger = container.querySelector(
      '[aria-label="Settings"]',
    ) as HTMLButtonElement;
    trigger.focus();
    await act(async () => trigger.click());
    const nav = () =>
      Array.from(
        container.querySelectorAll<HTMLButtonElement>(".settings-nav button"),
      );
    expect(document.activeElement).toBe(nav()[0]);
    await act(async () =>
      nav()[0].dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(nav()[1]);
    expect(container.querySelector("#settings-appearance")).not.toBeNull();
    nav()[2].focus();
    await act(async () =>
      nav()[2].dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(nav()[3]);
    expect(container.querySelector("#settings-installation")).not.toBeNull();
    await act(async () =>
      nav()[3].dispatchEvent(
        new KeyboardEvent("keydown", { key: "End", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(nav()[4]);
    expect(container.querySelector("#settings-about")).not.toBeNull();
    await act(async () =>
      nav()[4].dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(nav()[3]);
    expect(container.querySelector("#settings-installation")).not.toBeNull();
    await act(async () =>
      nav()[3].dispatchEvent(
        new KeyboardEvent("keydown", { key: "Home", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(nav()[0]);
    expect(container.querySelector("#settings-general")).not.toBeNull();
    await act(async () =>
      nav()[0].dispatchEvent(
        new KeyboardEvent("keydown", { key: "End", bubbles: true }),
      ),
    );

    await act(async () =>
      nav()[4].dispatchEvent(
        new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
      ),
    );
    expect(container.querySelector("#settings-title")).toBeNull();
    expect(document.activeElement).toBe(trigger);

    await act(async () => trigger.click());
    const footerClose = container.querySelector(
      ".settings-dialog .dialog-footer button",
    ) as HTMLButtonElement;
    await act(async () => footerClose.click());
    expect(document.activeElement).toBe(trigger);

    await act(async () => trigger.click());
    const headerClose = container.querySelector(
      '.settings-dialog .dialog-header [aria-label="Close"]',
    ) as HTMLButtonElement;
    await act(async () => headerClose.click());
    expect(document.activeElement).toBe(trigger);
  });

  it("preserves upstream Agent order and presents empty filters and CLI version", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );
    const navButtons = Array.from(
      container.querySelectorAll<HTMLButtonElement>(".settings-nav button"),
    );
    await act(async () => navButtons[3].click());
    const explicit = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Choose explicit targets",
    ) as HTMLButtonElement;
    await act(async () => explicit.click());
    expect(
      Array.from(container.querySelectorAll(".agent-options .choice"))
        .slice(0, 6)
        .map((label) => label.textContent?.trim()),
    ).toEqual([
      "aider-desk",
      "amp",
      "replit",
      "universal",
      "antigravity",
      "antigravity-cli",
    ]);
    const filter = container.querySelector(
      '[aria-label="Filter Agent targets"]',
    ) as HTMLInputElement;
    await act(async () => {
      Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set?.call(filter, "no-such-agent");
      filter.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(container.textContent).toContain("No matching Agent targets.");

    await act(async () => navButtons[4].click());
    const version = container.querySelector(".version-row");
    expect(version?.textContent).toContain("Skills CLI version");
    expect(version?.textContent).toContain("—");
    expect(version?.classList.contains("version-unavailable")).toBe(true);
  });

  it("blocks the workspace when the runtime probe fails", async () => {
    invokeMock.mockImplementation((command: string) =>
      command === "runtime_status"
        ? Promise.resolve({
            ready: false,
            errorCode: "runtime_not_found",
            version: null,
            nodeVersion: null,
            message: "spawn node failed: os error 2; PATH=/usr/bin",
            inventory: [],
          })
        : Promise.reject(new Error("blocked")),
    );
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain(
      "could not find a supported Node.js and npx installation",
    );
    expect(container.textContent).toContain("Install Node.js 22.20 or newer");
    expect(container.textContent).not.toContain("os error");
    expect(container.textContent).not.toContain("PATH=");
    expect(container.querySelector(".workspace")?.hasAttribute("inert")).toBe(
      true,
    );
  });

  it("localizes actionable runtime failures", async () => {
    localStorage.setItem("skill-deck-locale", "zh-CN");
    invokeMock.mockImplementation((command: string) =>
      command === "runtime_status"
        ? Promise.resolve({
            ready: false,
            errorCode: "node_too_old",
            version: null,
            nodeVersion: null,
            message: "raw backend detail",
            inventory: [],
          })
        : Promise.reject(new Error("blocked")),
    );
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("已安装的 Node.js 版本过低");
    expect(container.textContent).toContain("请升级到 Node.js 22.20");
    expect(container.textContent).not.toContain("raw backend detail");
    expect(document.documentElement.lang).toBe("zh-CN");
  });

  it("keeps Markdown resources inert and lets unsupported files be selected", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [{ ...demoSkill, agents: [], source: null }],
        });
      if (command === "preview_tree")
        return Promise.resolve([
          {
            path: "SKILL.md",
            name: "SKILL.md",
            level: 1,
            directory: false,
            size: 40,
            viewer: "markdown",
            unsupportedReason: null,
          },
          {
            path: "archive.zip",
            name: "archive.zip",
            level: 1,
            directory: false,
            size: 123,
            viewer: "unsupported",
            unsupportedReason: "No viewer",
          },
        ]);
      if (command === "read_preview")
        return Promise.resolve({
          path: "SKILL.md",
          viewer: "markdown",
          size: 40,
          text: "[link](https://example.com) ![remote](https://example.com/a.png)",
          dataUrl: null,
          translatable: true,
        });
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    expect(container.querySelector(".viewer a")).toBeNull();
    expect(container.querySelector(".viewer img")).toBeNull();
    expect(
      container
        .querySelector('[aria-label="Translate"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("translate");
    expect(
      container
        .querySelector('[aria-label="Reveal file"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("folder-open");
    expect(
      container
        .querySelector('[aria-label="Update"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("update-skill");
    expect(
      container
        .querySelector('[aria-label="Remove"] .icon')
        ?.getAttribute("data-icon"),
    ).toBe("trash");

    await act(async () =>
      (container.querySelector(".path-button") as HTMLButtonElement).click(),
    );
    const pathButton = container.querySelector(
      ".path-button",
    ) as HTMLButtonElement;
    expect(pathButton.getAttribute("aria-label")).toBe("Open file tree");
    expect(pathButton.title).toBe("Browse files");
    const tree = container.querySelector(".file-tree") as HTMLElement;
    const ariaTree = tree.querySelector('[role="tree"]') as HTMLElement;
    expect(ariaTree.previousElementSibling?.className).toBe("file-tree-header");
    expect(ariaTree.firstElementChild?.getAttribute("aria-level")).toBe("1");
    await act(async () =>
      tree.dispatchEvent(
        new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
      ),
    );
    expect(document.activeElement).toBe(pathButton);
    await act(async () => pathButton.click());
    const unsupported = container.querySelector(
      '[data-path="archive.zip"]',
    ) as HTMLButtonElement;
    expect(unsupported.disabled).toBe(false);
    await act(async () => unsupported.click());
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    expect(document.activeElement).toBe(pathButton);
    expect(container.textContent).toContain("123 bytes");
    expect(container.querySelector(".preview-loading")).toBeNull();
    expect(
      container.querySelector('[aria-label="Reveal file"]'),
    ).not.toBeNull();
  });

  it("supports folder disclosure, tree navigation, and selected ancestor reveal", async () => {
    const entries = [
      {
        path: "SKILL.md",
        name: "SKILL.md",
        level: 1,
        directory: false,
        size: 40,
        viewer: "markdown",
        unsupportedReason: null,
      },
      {
        path: "references",
        name: "references",
        level: 1,
        directory: true,
        size: 0,
        viewer: "unsupported",
        unsupportedReason: null,
      },
      {
        path: "references/nested",
        name: "nested",
        level: 2,
        directory: true,
        size: 0,
        viewer: "unsupported",
        unsupportedReason: null,
      },
      {
        path: "references/nested/notes.md",
        name: "notes.md",
        level: 3,
        directory: false,
        size: 12,
        viewer: "markdown",
        unsupportedReason: null,
      },
      {
        path: "assets",
        name: "assets",
        level: 1,
        directory: true,
        size: 0,
        viewer: "unsupported",
        unsupportedReason: null,
      },
      {
        path: "assets/cover.png",
        name: "cover.png",
        level: 2,
        directory: false,
        size: 24,
        viewer: "image",
        unsupportedReason: null,
      },
    ] as const;
    let treeLoads = 0;
    invokeMock.mockImplementation(
      (command: string, args?: { path?: string }) => {
        if (command === "runtime_status")
          return Promise.resolve({
            ready: true,
            errorCode: null,
            version: "1.5.22",
            nodeVersion: "22.20.0",
            message: null,
            inventory: [demoSkill],
          });
        if (command === "preview_tree") {
          treeLoads += 1;
          return Promise.resolve(
            treeLoads === 1
              ? entries
              : entries.filter(
                  (entry) =>
                    entry.path !== "references/nested/notes.md" &&
                    entry.path !== "references/nested",
                ),
          );
        }
        if (command === "read_preview")
          return Promise.resolve({
            path: args?.path,
            viewer: "markdown",
            size: 12,
            text: args?.path,
            dataUrl: null,
            translatable: true,
          });
        return Promise.reject(new Error(`Unexpected command: ${command}`));
      },
    );
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    const trigger = container.querySelector(
      ".path-button",
    ) as HTMLButtonElement;
    await act(async () => trigger.click());
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    const references = container.querySelector(
      '[data-path="references"]',
    ) as HTMLButtonElement;
    expect(references.tagName).toBe("BUTTON");
    expect(references.getAttribute("aria-expanded")).toBe("true");
    await act(async () => references.focus());

    await act(async () =>
      references.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }),
      ),
    );
    expect(references.getAttribute("aria-expanded")).toBe("false");
    expect(references.tabIndex).toBe(0);
    expect(
      container
        .querySelector('[data-path="SKILL.md"]')
        ?.getAttribute("tabindex"),
    ).toBe("-1");
    expect(
      container.querySelector('[data-path="references/nested"]'),
    ).toBeNull();

    await act(async () =>
      (
        container.querySelector('[data-path="references"]') as HTMLButtonElement
      ).dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }),
      ),
    );
    expect(
      container
        .querySelector('[data-path="references"]')
        ?.getAttribute("aria-expanded"),
    ).toBe("true");
    expect((document.activeElement as HTMLElement).dataset.path).toBe(
      "references",
    );
    await act(async () =>
      (
        container.querySelector('[data-path="references"]') as HTMLButtonElement
      ).dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }),
      ),
    );
    expect((document.activeElement as HTMLElement).dataset.path).toBe(
      "references/nested",
    );

    const assets = container.querySelector(
      '[data-path="assets"]',
    ) as HTMLButtonElement;
    await act(async () => assets.click());
    expect(assets.getAttribute("aria-expanded")).toBe("false");
    const notes = container.querySelector(
      '[data-path="references/nested/notes.md"]',
    ) as HTMLButtonElement;
    await act(async () => notes.click());
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    expect(document.activeElement).toBe(trigger);

    await act(async () => trigger.click());
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    expect(
      container
        .querySelector('[data-path="references"]')
        ?.getAttribute("aria-expanded"),
    ).toBe("true");
    expect(
      container
        .querySelector('[data-path="references/nested"]')
        ?.getAttribute("aria-expanded"),
    ).toBe("true");
    expect(
      container
        .querySelector('[data-path="assets"]')
        ?.getAttribute("aria-expanded"),
    ).toBe("false");
    expect((document.activeElement as HTMLElement).dataset.path).toBe(
      "references/nested/notes.md",
    );

    await act(async () =>
      document.body.dispatchEvent(
        new MouseEvent("mousedown", { bubbles: true }),
      ),
    );
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    expect(container.querySelector(".file-tree")).toBeNull();
    expect(document.activeElement).toBe(trigger);

    await act(async () =>
      (
        container.querySelector(
          '[aria-label="Refresh Inventory"]',
        ) as HTMLButtonElement
      ).click(),
    );
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    await act(async () => trigger.click());
    await act(async () => new Promise((resolve) => setTimeout(resolve, 20)));
    expect((document.activeElement as HTMLElement).dataset.path).toBe(
      "SKILL.md",
    );
    expect(
      container
        .querySelector('[data-path="assets"]')
        ?.getAttribute("aria-expanded"),
    ).toBe("true");
  });

  it("keeps original content and retries sanitized translation failures", async () => {
    let translations = 0;
    let resolveStale: ((value: unknown) => void) | undefined;
    let rejectTranslation: ((value: unknown) => void) | undefined;
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          errorCode: null,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
          inventory: [{ ...demoSkill, agents: [], source: null }],
        });
      if (command === "preview_tree")
        return Promise.resolve([
          {
            path: "SKILL.md",
            name: "SKILL.md",
            level: 1,
            directory: false,
            size: 12,
            viewer: "markdown",
            unsupportedReason: null,
          },
        ]);
      if (command === "read_preview")
        return Promise.resolve({
          path: "SKILL.md",
          viewer: "markdown",
          size: 12,
          text: "secret source",
          dataUrl: null,
          translatable: true,
        });
      if (command === "translate_preview") {
        translations += 1;
        if (translations === 1)
          return new Promise((_resolve, reject) => {
            rejectTranslation = reject;
          });
        if (translations === 2)
          return Promise.resolve({
            translatedText: "translated",
            detectedSourceLanguage: "en",
          });
        if (translations === 3)
          return new Promise((resolve) => {
            resolveStale = resolve;
          });
        return new Promise(() => undefined);
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    await act(async () => root.render(<App />));
    await act(async () =>
      (container.querySelector('[role="option"]') as HTMLButtonElement).click(),
    );
    await act(async () =>
      (
        Array.from(container.querySelectorAll("button")).find((button) =>
          button.textContent?.includes("Translate"),
        ) as HTMLButtonElement
      ).click(),
    );
    expect(container.textContent).toContain("secret source");
    expect(container.textContent).toContain("Translating…");
    await act(async () =>
      rejectTranslation?.({
        code: "translation_timeout",
        message: "https://translate.googleapis.com/?q=secret",
      }),
    );
    expect(container.textContent).toContain("Translation timed out");
    expect(container.textContent).not.toContain("translate.googleapis.com");
    const retry = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Retry",
    ) as HTMLButtonElement;
    await act(async () => retry.click());
    expect(container.textContent).toContain("translated");
    expect(translations).toBe(2);

    const translationTab = container.querySelector(
      "#translation-tab",
    ) as HTMLButtonElement;
    const originalTab = container.querySelector(
      "#original-tab",
    ) as HTMLButtonElement;
    await act(async () => translationTab.click());
    expect(translationTab.getAttribute("aria-selected")).toBe("true");
    expect(container.textContent).toContain("translated");
    await act(async () => originalTab.click());
    expect(originalTab.getAttribute("aria-selected")).toBe("true");
    expect(container.textContent).toContain("translated");
    expect(translations).toBe(2);

    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );
    const target = container.querySelector(
      ".field select",
    ) as HTMLSelectElement;
    const setTarget = (value: string) => {
      Object.getOwnPropertyDescriptor(
        HTMLSelectElement.prototype,
        "value",
      )?.set?.call(target, value);
      target.dispatchEvent(new Event("change", { bubbles: true }));
    };
    await act(async () => setTarget("ja"));
    await act(async () => setTarget("ko"));
    await act(async () =>
      resolveStale?.({
        translatedText: "stale translation",
        detectedSourceLanguage: "en",
      }),
    );
    await act(async () => setTarget("ja"));
    expect(container.textContent).not.toContain("stale translation");
  });

  it("ends translation when switching Skills and rejects stale completion", async () => {
    const translations: {
      resolve: (value: unknown) => void;
      reject: (reason?: unknown) => void;
    }[] = [];
    invokeMock.mockImplementation(
      (command: string, args?: { path?: string; skill?: string }) => {
        if (command === "runtime_status")
          return Promise.resolve({
            ready: true,
            errorCode: null,
            version: "1.5.22",
            nodeVersion: "22.20.0",
            message: null,
            inventory: [demoSkill, { ...demoSkill, name: "other" }],
          });
        if (command === "preview_tree")
          return Promise.resolve([
            {
              path: "SKILL.md",
              name: "SKILL.md",
              level: 1,
              directory: false,
              size: 12,
              viewer: "markdown",
              unsupportedReason: null,
            },
            {
              path: "NOTES.md",
              name: "NOTES.md",
              level: 1,
              directory: false,
              size: 12,
              viewer: "markdown",
              unsupportedReason: null,
            },
          ]);
        if (command === "read_preview")
          return Promise.resolve({
            path: args?.path,
            viewer: "markdown",
            size: 12,
            text: `${args?.skill} ${args?.path} source`,
            dataUrl: null,
            translatable: true,
          });
        if (command === "translate_preview")
          return new Promise((resolve, reject) => {
            translations.push({ resolve, reject });
          });
        return Promise.reject(new Error(`Unexpected command: ${command}`));
      },
    );

    await act(async () => root.render(<App />));
    const skills =
      container.querySelectorAll<HTMLButtonElement>('[role="option"]');
    await act(async () => skills[0].click());
    const translate = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Translate"),
    ) as HTMLButtonElement;
    await act(async () => translate.click());
    expect(container.textContent).toContain("Translating…");

    await act(async () =>
      (container.querySelector(".path-button") as HTMLButtonElement).click(),
    );
    await act(async () =>
      (
        container.querySelector('[data-path="NOTES.md"]') as HTMLButtonElement
      ).click(),
    );
    expect(container.textContent).toContain("demo NOTES.md source");
    expect(container.querySelector(".viewer-grid.translated")).toBeNull();
    expect(
      invokeMock.mock.calls.filter(
        ([command]) => command === "translate_preview",
      ),
    ).toHaveLength(1);
    await act(async () =>
      (
        Array.from(container.querySelectorAll("button")).find((button) =>
          button.textContent?.includes("Translate"),
        ) as HTMLButtonElement
      ).click(),
    );

    await act(async () => skills[1].click());
    expect(container.textContent).toContain("other SKILL.md source");
    expect(container.textContent).not.toContain("Translating…");
    expect(container.querySelector(".viewer-grid.translated")).toBeNull();
    expect(
      Array.from(container.querySelectorAll("button"))
        .find((button) => button.textContent?.includes("Translate"))
        ?.getAttribute("aria-pressed"),
    ).toBe("false");
    expect(
      invokeMock.mock.calls.filter(
        ([command]) => command === "translate_preview",
      ),
    ).toHaveLength(2);

    await act(async () => {
      translations[0].resolve({
        translatedText: "stale translation",
        detectedSourceLanguage: "en",
      });
      translations[1].reject({
        code: "translation_timeout",
        message: "stale timeout",
      });
    });
    expect(container.textContent).not.toContain("stale translation");
    expect(container.textContent).not.toContain("Translation timed out");

    await act(async () => skills[0].click());
    expect(container.textContent).toContain("demo SKILL.md source");
    expect(container.querySelector(".viewer-grid.translated")).toBeNull();
    expect(
      invokeMock.mock.calls.filter(
        ([command]) => command === "translate_preview",
      ),
    ).toHaveLength(2);
  });
});
