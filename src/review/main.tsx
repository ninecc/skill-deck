import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import App from "../App";
import type { Preferences, Theme, UiLocale } from "../preferences";
import "../styles.css";
import { installReviewIpc } from "./installReviewIpc";
import {
  isReviewScenarioId,
  reviewScenarios,
  type ReviewScenario,
} from "./scenarios";

const params = new URLSearchParams(location.search);
const requestedScenario = params.get("scenario");
const scenarioId = isReviewScenarioId(requestedScenario)
  ? requestedScenario
  : "shell-ready";
const scenario = reviewScenarios[scenarioId];
const themes: Theme[] = ["system", "light", "dark", "sand", "plum"];
const locales: UiLocale[] = ["system", "en", "zh-CN"];
const requestedTheme = params.get("theme") as Theme | null;
const requestedLocale = params.get("locale") as UiLocale | null;

const deterministicLanguages = scenario.locale === "zh-CN" ? ["zh-CN"] : ["en"];
Object.defineProperty(navigator, "languages", {
  configurable: true,
  value: deterministicLanguages,
});
Object.defineProperty(navigator, "language", {
  configurable: true,
  value: deterministicLanguages[0],
});

const preferences: Preferences = {
  theme:
    requestedTheme && themes.includes(requestedTheme)
      ? requestedTheme
      : (scenario.preferences?.theme ?? scenario.theme),
  uiLocale:
    requestedLocale && locales.includes(requestedLocale)
      ? requestedLocale
      : (scenario.preferences?.uiLocale ?? scenario.locale),
  targetLanguage: scenario.preferences?.targetLanguage ?? "zh-Hans",
  translationProxy: scenario.preferences?.translationProxy ?? "",
  agents: scenario.preferences?.agents ?? [],
  copy: scenario.preferences?.copy ?? false,
};

localStorage.setItem("skill-deck-preferences", JSON.stringify(preferences));
installReviewIpc(scenario);
document.documentElement.dataset.reviewScenario = scenario.id;

function ReviewMount({ value }: { value: ReviewScenario }) {
  useEffect(() => {
    if (value.runtime === "pending") return;
    const settingsState = value.settingsState;
    if (settingsState) {
      let opened = false;
      let timer = 0;
      const activateSettings = () => {
        const dialog = document.querySelector<HTMLElement>(
          '[role="dialog"][aria-labelledby="settings-title"]',
        );
        if (!dialog) {
          if (opened) return;
          const trigger = document
            .querySelector('[data-icon="settings"]')
            ?.closest<HTMLButtonElement>("button");
          if (!trigger) return;
          opened = true;
          trigger.click();
          return;
        }
        const section = settingsState.startsWith("translation")
          ? "translation"
          : settingsState.startsWith("installation")
            ? "installation"
            : settingsState;
        const sectionButton = dialog.querySelector<HTMLButtonElement>(
          `[aria-controls="settings-${section}"]`,
        );
        sectionButton?.focus();
        sectionButton?.click();
        if (settingsState === "translation-invalid") {
          const input = dialog.querySelector<HTMLInputElement>(
            ".translation-settings input",
          );
          if (!input) return;
          const setter = Object.getOwnPropertyDescriptor(
            HTMLInputElement.prototype,
            "value",
          )?.set;
          setter?.call(input, "http://user:••••@proxy.example");
          input.dispatchEvent(new Event("input", { bubbles: true }));
          dialog.querySelector<HTMLButtonElement>(".proxy-apply")?.click();
        }
        if (
          settingsState === "installation-explicit" ||
          settingsState === "installation-no-match"
        ) {
          const input = dialog.querySelector<HTMLInputElement>(
            '[aria-label="Filter Agent targets"], [aria-label="筛选 Agent target"]',
          );
          if (!input) return;
          const setter = Object.getOwnPropertyDescriptor(
            HTMLInputElement.prototype,
            "value",
          )?.set;
          setter?.call(
            input,
            settingsState === "installation-explicit" ? "cod" : "no-such-agent",
          );
          input.dispatchEvent(new Event("input", { bubbles: true }));
        }
        clearInterval(timer);
      };
      timer = window.setInterval(activateSettings, 25);
      return () => clearInterval(timer);
    }
    if (!value.autoSelect) return;
    const targetName = value.runtime.inventory[0]?.name;
    if (!targetName) return;
    let selected = false;
    let modalOpened = false;
    let finished = false;
    let timer = 0;
    const activate = () => {
      const row = document.querySelector<HTMLButtonElement>(
        `[data-skill="${CSS.escape(targetName)}"]`,
      );
      if (!selected && row) {
        selected = true;
        row.click();
      }
      if (!selected || finished) return;
      if (
        value.reviewState === "none" ||
        value.reviewState === "preview-error"
      ) {
        clearInterval(timer);
        return;
      }
      if (
        value.reviewState === "discovery-search" ||
        value.reviewState === "discovery-source"
      ) {
        const dialog = document.querySelector<HTMLElement>(
          '[role="dialog"][aria-labelledby="find-install-title"]',
        );
        if (!dialog) {
          if (modalOpened) return;
          const open = document.querySelector<HTMLButtonElement>(
            ".bar-actions .primary",
          );
          if (!open) return;
          modalOpened = true;
          open.click();
          return;
        }
        if (value.reviewState === "discovery-source") {
          const tab = dialog.querySelector<HTMLButtonElement>(
            ".discovery-tabs button:nth-child(2)",
          );
          tab?.click();
          finished = true;
          clearInterval(timer);
          return;
        }
        const input = dialog.querySelector<HTMLInputElement>(
          ".catalog-search input",
        );
        if (!input) return;
        const setter = Object.getOwnPropertyDescriptor(
          HTMLInputElement.prototype,
          "value",
        )?.set;
        setter?.call(input, "typescript");
        input.dispatchEvent(new Event("input", { bubbles: true }));
        input
          .closest("form")
          ?.dispatchEvent(
            new Event("submit", { bubbles: true, cancelable: true }),
          );
        finished = true;
        clearInterval(timer);
        return;
      }
      if (value.reviewState === "remove") {
        const remove =
          document.querySelector<HTMLButtonElement>(".quiet-danger");
        if (!remove) return;
        remove.click();
        finished = true;
        clearInterval(timer);
        return;
      }
      const selector =
        value.reviewState === "tree" ? ".path-button" : ".translation-toggle";
      const control = document.querySelector<HTMLButtonElement>(selector);
      if (!control) return;
      control.click();
      finished = true;
      clearInterval(timer);
    };
    timer = window.setInterval(activate, 25);
    return () => clearInterval(timer);
  }, [value]);

  return <App />;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ReviewMount value={scenario} />
  </StrictMode>,
);
