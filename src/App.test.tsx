// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const inventory = {
  targets: [],
  managedPackages: [],
  managedInstallationStatuses: [],
  managedPackageReconciliations: [],
  externalInstallations: [
    {
      agent: "codex",
      logicalPath: "/tmp/skills/valid-skill",
      resolvedTarget: "/tmp/skills/valid-skill",
      kind: "directory",
      skill: {
        root: "/tmp/skills/valid-skill",
        fingerprint: "valid",
        metadata: {
          name: "valid-skill",
          description: "A valid external skill",
          unknownFields: {},
        },
        resources: {
          packageBytes: 1,
          fileCount: 1,
          largestFileBytes: 1,
          skillMarkdownBytes: 1,
        },
        scripts: [],
        references: [],
      },
      diagnostic: null,
    },
  ],
  attentionEntries: [
    {
      agent: "codex",
      logicalPath: "/tmp/skills/invalid-skill",
      resolvedTarget: "/tmp/skills/invalid-skill",
      kind: "invalid_installation_candidate",
      diagnostic: {
        code: "unsupported_file_type",
        message: "Links are unsupported",
        path: "/tmp/skills/invalid-skill/references/link.md",
      },
    },
    {
      agent: "claude",
      logicalPath: "/tmp/skills/notes.txt",
      resolvedTarget: null,
      kind: "unexpected_agent_root_entry",
      diagnostic: {
        code: "unsupported_file_type",
        message: "Not a skill",
        path: "/tmp/skills/notes.txt",
      },
    },
  ],
};

describe("App inventory", () => {
  let container: HTMLDivElement;
  let root: ReturnType<typeof createRoot>;

  beforeEach(() => {
    localStorage.setItem("skill-deck-locale", "en");
    invokeMock.mockImplementation((command: string) => {
      if (command === "inventory") return Promise.resolve(inventory);
      if (command === "state_status") {
        return Promise.resolve({
          mode: "active",
          state: { packages: [] },
          diagnostic: null,
        });
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    container = document.createElement("div");
    document.body.append(container);
    root = createRoot(container);
    HTMLDialogElement.prototype.showModal = vi.fn();
    HTMLDialogElement.prototype.close = vi.fn();
    (
      globalThis as typeof globalThis & {
        IS_REACT_ACT_ENVIRONMENT: boolean;
      }
    ).IS_REACT_ACT_ENVIRONMENT = true;
  });

  afterEach(async () => {
    await act(async () => root.unmount());
    container.remove();
    localStorage.clear();
    invokeMock.mockReset();
  });

  it("shows actionable diagnostics and keeps locale changes scan-free", async () => {
    await act(async () => {
      root.render(<App />);
    });

    expect(container.textContent).toContain(
      "1 Discovered external installations",
    );
    expect(container.textContent).toContain("2 Needs attention");
    expect(container.textContent).toContain(
      "/tmp/skills/invalid-skill/references/link.md",
    );
    expect(container.textContent).not.toContain("/tmp/skills/notes.txt");

    const settingsButton = Array.from(
      container.querySelectorAll("button"),
    ).find((button) => button.textContent?.includes("Paths & diagnostics"));
    await act(async () => settingsButton!.click());
    expect(container.textContent).toContain("/tmp/skills/notes.txt");
    await act(async () =>
      (
        container.querySelector(
          ".settings-dialog .icon-button",
        ) as HTMLButtonElement
      ).click(),
    );

    const enabledSelect = Array.from(container.querySelectorAll("label"))
      .find((label) => label.textContent?.includes("Enabled state"))
      ?.querySelector("select");
    expect(enabledSelect).toBeTruthy();
    await act(async () => {
      enabledSelect!.value = "enabled";
      enabledSelect!.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(container.textContent).toContain(
      "Enabled state applies only to Skill Deck-managed skills.",
    );
    expect(container.textContent).not.toContain("invalid-skill");

    const callsBeforeLocaleChange = invokeMock.mock.calls.length;
    const localeSelect = container.querySelector(".locale-control select");
    await act(async () => {
      (localeSelect as HTMLSelectElement).value = "zh-CN";
      localeSelect!.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(invokeMock).toHaveBeenCalledTimes(callsBeforeLocaleChange);
  });

  it("renders backend-owned managed status, evidence, and actions", async () => {
    const managedInventory = {
      ...inventory,
      externalInstallations: [],
      attentionEntries: [],
      managedPackages: [
        {
          id: "package-1",
          name: "managed-skill",
          libraryPath: "/library/managed-skill/current",
          source: { type: "local_snapshot" },
          installedRevision: {
            fingerprint: "aaaaaaaaaaaaaaaaaaaaaaaa",
            commitOid: null,
          },
          previousRevision: null,
          installations: [
            {
              agent: "claude",
              logicalPath: "/agents/skills/managed-skill",
              resolvedTarget: "/library/managed-skill/current",
              deploymentMode: "symlink",
              enabled: true,
              lastKnownFingerprint: "aaaaaaaaaaaaaaaaaaaaaaaa",
              configurationProvenance: { owner: "none" },
            },
          ],
        },
      ],
      managedInstallationStatuses: [
        {
          packageId: "package-1",
          agent: "claude",
          status: "retargeted",
          diagnostic: {
            code: "topology_changed",
            message: "Retargeted",
            path: "/agents/skills/managed-skill",
          },
          evidence: {
            logicalPath: "/agents/skills/managed-skill",
            deploymentMode: "symlink",
            expectedTarget: "/library/managed-skill/current",
            observedTarget: "/outside/managed-skill",
            recordedFingerprint: "aaaaaaaaaaaaaaaaaaaaaaaa",
            libraryFingerprint: "aaaaaaaaaaaaaaaaaaaaaaaa",
            observedFingerprint: null,
            configurationProvenance: { owner: "none" },
            deferredChecks: true,
          },
          availableActions: ["forget_installation"],
        },
      ],
      managedPackageReconciliations: [
        {
          packageId: "package-1",
          libraryDiagnostic: null,
          availableActions: ["export"],
        },
      ],
    };
    invokeMock.mockImplementation((command: string) => {
      if (command === "inventory") return Promise.resolve(managedInventory);
      if (command === "state_status") {
        return Promise.resolve({
          mode: "active",
          state: { packages: managedInventory.managedPackages },
          diagnostic: null,
        });
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });

    await act(async () => root.render(<App />));

    expect(container.textContent).toContain("managed-skill");
    expect(container.textContent).toContain("Retargeted");
    expect(container.textContent).toContain("/library/managed-skill/current");
    expect(container.textContent).toContain("/outside/managed-skill");
    expect(container.textContent).toContain("Forget installation");
    expect(container.textContent).not.toContain("Uninstall");
    expect(container.textContent).not.toContain("Install to agents");
  });

  it("displays a broken Managed Library diagnostic only once per package", async () => {
    const libraryDiagnostic = {
      code: "content_drift",
      message: "Managed Library changed",
      path: "/library/managed-skill/current",
    };
    const managedInventory = {
      ...inventory,
      managedPackages: [
        {
          id: "package-1",
          name: "managed-skill",
          libraryPath: "/library/managed-skill/current",
          source: { type: "local_snapshot" },
          installedRevision: { fingerprint: "aaaa", commitOid: null },
          previousRevision: null,
          installations: [
            {
              agent: "claude",
              logicalPath: "/agents/skills/managed-skill",
              resolvedTarget: "/library/managed-skill/current",
              deploymentMode: "symlink",
              enabled: true,
              lastKnownFingerprint: "aaaa",
              configurationProvenance: { owner: "none" },
            },
          ],
        },
      ],
      managedInstallationStatuses: [
        {
          packageId: "package-1",
          agent: "claude",
          status: "broken",
          diagnostic: libraryDiagnostic,
          evidence: {
            logicalPath: "/agents/skills/managed-skill",
            deploymentMode: "symlink",
            expectedTarget: "/library/managed-skill/current",
            observedTarget: null,
            recordedFingerprint: "aaaa",
            libraryFingerprint: "aaaa",
            observedFingerprint: null,
            configurationProvenance: { owner: "none" },
            deferredChecks: true,
          },
          availableActions: ["forget_installation"],
        },
      ],
      managedPackageReconciliations: [
        {
          packageId: "package-1",
          libraryDiagnostic,
          availableActions: [],
        },
      ],
    };
    invokeMock.mockImplementation((command: string) =>
      command === "inventory"
        ? Promise.resolve(managedInventory)
        : command === "state_status"
          ? Promise.resolve({
              mode: "active",
              state: { packages: managedInventory.managedPackages },
              diagnostic: null,
            })
          : Promise.reject(new Error(`Unexpected command: ${command}`)),
    );

    await act(async () => root.render(<App />));

    expect(
      container.textContent?.match(
        /Managed Skill content changed outside Skill Deck\./g,
      ),
    ).toHaveLength(1);
  });
});
