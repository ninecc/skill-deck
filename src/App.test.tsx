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

  it("blocks the workspace when the runtime probe fails", async () => {
    invokeMock.mockImplementation((command: string) =>
      command === "runtime_status"
        ? Promise.resolve({
            ready: false,
            version: null,
            nodeVersion: null,
            message: "Install Node.js 22.20.0 or newer",
          })
        : Promise.reject(new Error("blocked")),
    );
    await act(async () => root.render(<App />));
    expect(container.textContent).toContain("Install Node.js 22.20.0 or newer");
    expect(container.textContent).not.toContain("Installed Skills");
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
});
