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
    expect(container.textContent).toContain("Choose an installed Skill");
    expect(container.querySelector(".choose-placeholder .icon")).toBeNull();
    expect(
      container.querySelector('[role="option"]')?.getAttribute("aria-selected"),
    ).toBe("false");
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
  });

  it("persists a translation proxy only after valid Apply", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );
    const proxyLabel = Array.from(container.querySelectorAll("label")).find(
      (label) => label.textContent?.includes("Translation proxy"),
    ) as HTMLLabelElement;
    const input = proxyLabel.querySelector("input") as HTMLInputElement;
    expect(input.value).toBe("");
    expect(input.hasAttribute("placeholder")).toBe(false);
    const apply = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent === "Apply proxy",
    ) as HTMLButtonElement;
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
    await act(async () => apply.click());
    expect(localStorage.getItem("skill-deck-preferences")).not.toContain(
      "secret",
    );
    await act(async () => {
      setInput("http://127.0.0.1:7890");
    });
    expect(localStorage.getItem("skill-deck-preferences")).not.toContain(
      "7890",
    );
    await act(async () => apply.click());
    expect(localStorage.getItem("skill-deck-preferences")).toContain("7890");
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

    await act(async () =>
      (container.querySelector(".path-button") as HTMLButtonElement).click(),
    );
    const unsupported = container.querySelector(
      '[data-path="archive.zip"]',
    ) as HTMLButtonElement;
    expect(unsupported.disabled).toBe(false);
    await act(async () => unsupported.click());
    expect(container.textContent).toContain("123 bytes");
    expect(container.querySelector(".preview-loading")).toBeNull();
    expect(
      container.querySelector('[aria-label="Reveal file"]'),
    ).not.toBeNull();
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
