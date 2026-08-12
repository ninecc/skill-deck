import { useState } from "react";
import { Icon } from "./icons";
import ModalShell from "./ModalShell";
import type { Messages } from "./i18n";
import {
  agentOptions,
  languages,
  themes,
  type Preferences,
  type TargetLanguage,
  type Theme,
  type UiLocale,
  validateTranslationProxy,
} from "./preferences";

interface Props {
  copy: Messages;
  version: string | null;
  preferences: Preferences;
  onChange: (preferences: Preferences) => void;
  onClose: () => void;
  returnFocus?: HTMLElement | null;
}

export default function SettingsDialog({
  copy,
  version,
  preferences,
  onChange,
  onClose,
  returnFocus,
}: Props) {
  const [proxyDraft, setProxyDraft] = useState(preferences.translationProxy);
  const [proxyError, setProxyError] = useState(false);
  const patch = (next: Partial<Preferences>) =>
    onChange({ ...preferences, ...next });
  return (
    <ModalShell
      labelledBy="settings-title"
      onClose={onClose}
      returnFocus={returnFocus}
      initialFocus="button"
    >
      <header>
        <h2 id="settings-title">{copy.settings}</h2>
        <button
          className="icon-button"
          type="button"
          onClick={onClose}
          aria-label={copy.close}
          title={copy.close}
        >
          <Icon name="close" />
        </button>
      </header>
      <fieldset>
        <legend>{copy.appearance}</legend>
        <div className="theme-grid">
          {themes.map((theme) => (
            <label
              className={`theme-tile preview-${theme} ${preferences.theme === theme ? "selected" : ""}`}
              key={theme}
            >
              <input
                type="radio"
                name="theme"
                value={theme}
                checked={preferences.theme === theme}
                onChange={() => patch({ theme: theme as Theme })}
              />
              <span className="theme-swatches">
                <i />
                <i />
                <i />
              </span>
              <span>{copy[theme]}</span>
            </label>
          ))}
        </div>
      </fieldset>
      <label className="field">
        {copy.targetLanguage}
        <select
          value={preferences.targetLanguage}
          onChange={(event) =>
            patch({ targetLanguage: event.target.value as TargetLanguage })
          }
        >
          {languages.map(([code, label]) => (
            <option key={code} value={code}>
              {label}
            </option>
          ))}
        </select>
      </label>
      <label className="field">
        {copy.uiLanguage}
        <select
          value={preferences.uiLocale}
          onChange={(event) =>
            patch({ uiLocale: event.target.value as UiLocale })
          }
        >
          <option value="system">{copy.systemLanguage}</option>
          <option value="en">English</option>
          <option value="zh-CN">简体中文</option>
        </select>
      </label>
      <label className="field">
        {copy.translationProxy}
        <input
          value={proxyDraft}
          aria-invalid={proxyError}
          onChange={(event) => {
            setProxyDraft(event.target.value);
            setProxyError(false);
          }}
        />
        <small>{copy.translationProxyHint}</small>
        {proxyError && <span className="field-error">{copy.invalidProxy}</span>}
      </label>
      <button
        type="button"
        onClick={() => {
          const value = proxyDraft.trim();
          if (validateTranslationProxy(value)) {
            setProxyError(true);
            return;
          }
          patch({ translationProxy: value });
        }}
      >
        {copy.applyProxy}
      </button>
      <fieldset>
        <legend>{copy.agentTargets}</legend>
        <label className="choice">
          <input
            type="radio"
            name="agents-mode"
            checked={preferences.agents.length === 0}
            onChange={() => patch({ agents: [] })}
          />
          {copy.automaticAgents}
        </label>
        <div className="agent-options">
          {agentOptions.map((agent) => (
            <label className="choice" key={agent}>
              <input
                type="checkbox"
                disabled={preferences.agents.length === 0}
                checked={preferences.agents.includes(agent)}
                onChange={(event) =>
                  patch({
                    agents: event.target.checked
                      ? [...preferences.agents, agent]
                      : preferences.agents.filter((value) => value !== agent),
                  })
                }
              />
              {agent}
            </label>
          ))}
        </div>
        <button
          type="button"
          className="text-button"
          onClick={() =>
            patch({ agents: preferences.agents.length ? [] : ["codex"] })
          }
        >
          {preferences.agents.length
            ? copy.automaticAgents
            : copy.chooseExplicitTargets}
        </button>
      </fieldset>
      <fieldset>
        <legend>{copy.installMethod}</legend>
        <label className="choice">
          <input
            type="radio"
            name="method"
            checked={!preferences.copy}
            onChange={() => patch({ copy: false })}
          />
          {copy.automaticMethod}
        </label>
        <label className="choice">
          <input
            type="radio"
            name="method"
            checked={preferences.copy}
            onChange={() => patch({ copy: true })}
          />
          {copy.copyMethod}
        </label>
      </fieldset>
      <p className="version-row">
        <span>{copy.cliVersion}</span>
        <code>{version ?? "—"}</code>
      </p>
      <button type="button" onClick={onClose}>
        {copy.close}
      </button>
    </ModalShell>
  );
}
