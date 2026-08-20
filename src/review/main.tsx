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

const preferences: Preferences = {
  theme:
    requestedTheme && themes.includes(requestedTheme)
      ? requestedTheme
      : scenario.theme,
  uiLocale:
    requestedLocale && locales.includes(requestedLocale)
      ? requestedLocale
      : scenario.locale,
  targetLanguage: "zh-Hans",
  translationProxy: "",
  agents: [],
  copy: false,
};

localStorage.setItem("skill-deck-preferences", JSON.stringify(preferences));
installReviewIpc(scenario);
document.documentElement.dataset.reviewScenario = scenario.id;

function ReviewMount({ value }: { value: ReviewScenario }) {
  useEffect(() => {
    if (!value.autoSelect || value.runtime === "pending") return;
    const targetName = value.runtime.inventory[0]?.name;
    if (!targetName) return;
    let selected = false;
    let timer = 0;
    const activate = () => {
      const row = document.querySelector<HTMLButtonElement>(
        `[data-skill="${CSS.escape(targetName)}"]`,
      );
      if (!selected && row) {
        selected = true;
        row.click();
      }
      if (!selected || value.reviewState === "none") {
        if (selected) clearInterval(timer);
        return;
      }
      const selector =
        value.reviewState === "tree" ? ".path-button" : ".translation-toggle";
      const control = document.querySelector<HTMLButtonElement>(selector);
      if (!control) return;
      control.click();
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
