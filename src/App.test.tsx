// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

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
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
        });
      if (command === "list_skills")
        return Promise.resolve([
          {
            name: "demo",
            path: "/tmp/demo",
            scope: "global",
            agents: ["Future Agent"],
            source: "owner/repo",
            sourceUrl: null,
            sourceType: "github",
          },
        ]);
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

  it("shows arbitrary agents and starts without a selected Skill", async () => {
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("Future Agent");
    expect(container.textContent).toContain("Choose an installed Skill");
    expect(
      container.querySelector('[role="option"]')?.getAttribute("aria-selected"),
    ).toBe("false");
  });

  it("renders startup chrome before the runtime probe completes", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("Skill Deck");
    expect(container.textContent).toContain("Connecting to the Skills CLI");
    expect(
      (container.querySelector(".bar-actions .button") as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(container.querySelector('[aria-label="Settings"]')).not.toBeNull();
    expect(container.querySelector('[aria-label="Language"]')).not.toBeNull();
  });

  it("persists a translation proxy only after valid Apply", async () => {
    invokeMock.mockImplementation(() => new Promise(() => undefined));
    await act(async () => root.render(<App />));
    await act(async () =>
      (
        container.querySelector('[aria-label="Settings"]') as HTMLButtonElement
      ).click(),
    );
    const input = container.querySelector(
      'input[placeholder="http://127.0.0.1:7890"]',
    ) as HTMLInputElement;
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
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
        });
      if (command === "list_skills")
        return Promise.resolve([
          {
            name: "demo",
            path: "/tmp/demo",
            scope: "global",
            agents: [],
            source: null,
            sourceUrl: null,
            sourceType: null,
          },
        ]);
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
    expect(
      container.querySelector('[aria-label="Reveal file"]'),
    ).not.toBeNull();
  });

  it("keeps original content and retries sanitized translation failures", async () => {
    let translations = 0;
    let resolveStale: ((value: unknown) => void) | undefined;
    invokeMock.mockImplementation((command: string) => {
      if (command === "runtime_status")
        return Promise.resolve({
          ready: true,
          version: "1.5.22",
          nodeVersion: "22.20.0",
          message: null,
        });
      if (command === "list_skills")
        return Promise.resolve([
          {
            name: "demo",
            path: "/tmp/demo",
            scope: "global",
            agents: [],
            source: null,
            sourceUrl: null,
            sourceType: null,
          },
        ]);
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
          return Promise.reject({
            code: "translation_timeout",
            message: "https://translate.googleapis.com/?q=secret",
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
    const translate = Array.from(container.querySelectorAll("button")).find(
      (button) => button.textContent?.includes("Translate"),
    ) as HTMLButtonElement;
    await act(async () => translate.click());
    expect(container.textContent).toContain("secret source");
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
});
