import { mockIPC } from "@tauri-apps/api/mocks";
import type { InvokeArgs } from "@tauri-apps/api/core";
import type { ReviewScenario } from "./scenarios";

export function installReviewIpc(scenario: ReviewScenario) {
  mockIPC((command: string, payload?: InvokeArgs) => {
    switch (command) {
      case "runtime_status":
      case "retry_runtime":
        return scenario.runtime === "pending"
          ? new Promise(() => undefined)
          : scenario.runtime;
      case "preview_tree":
        return scenario.tree;
      case "read_preview":
        return scenario.preview;
      case "reveal_path":
        return undefined;
      case "search_skills":
        return [];
      case "translate_preview":
        return {
          translatedText: scenario.preview.text ?? "",
          detectedSourceLanguage: "en",
        };
      default:
        if (command.startsWith("plugin:")) return undefined;
        throw new Error(
          `Unhandled review IPC command: ${command} ${JSON.stringify(payload ?? {})}`,
        );
    }
  });
}
