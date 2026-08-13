import { useMemo, useState } from "react";
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

type Section =
  "general" | "appearance" | "translation" | "installation" | "about";

export default function SettingsDialog({
  copy,
  version,
  preferences,
  onChange,
  onClose,
  returnFocus,
}: Props) {
  const [section, setSection] = useState<Section>("general");
  const [proxyDraft, setProxyDraft] = useState(preferences.translationProxy);
  const [proxyError, setProxyError] = useState(false);
  const [agentFilter, setAgentFilter] = useState("");
  const patch = (next: Partial<Preferences>) =>
    onChange({ ...preferences, ...next });
  const filteredAgents = useMemo(() => {
    const query = agentFilter.trim().toLowerCase();
    return agentOptions.filter((agent) => !query || agent.includes(query));
  }, [agentFilter]);
  const sections: readonly [Section, string][] = [
    ["general", copy.general],
    ["appearance", copy.appearance],
    ["translation", copy.translationSettings],
    ["installation", copy.installation],
    ["about", copy.about],
  ];

  return (
    <ModalShell
      labelledBy="settings-title"
      onClose={onClose}
      returnFocus={returnFocus}
      initialFocus=".settings-nav button"
      className="settings-dialog"
    >
      <header className="dialog-header">
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

      <div className="settings-layout">
        <nav className="settings-nav" aria-label={copy.settings}>
          {sections.map(([id, label]) => (
            <button
              key={id}
              type="button"
              aria-current={section === id ? "page" : undefined}
              onClick={() => setSection(id)}
            >
              {label}
            </button>
          ))}
        </nav>

        <div className="settings-content">
          {section === "general" && (
            <section aria-labelledby="general-heading">
              <div className="section-heading">
                <h3 id="general-heading">{copy.general}</h3>
                <small>{copy.settingsSavedImmediately}</small>
              </div>
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
            </section>
          )}

          {section === "appearance" && (
            <section aria-labelledby="appearance-heading">
              <div className="section-heading">
                <h3 id="appearance-heading">{copy.appearance}</h3>
                <small>{copy.settingsSavedImmediately}</small>
              </div>
              <fieldset>
                <legend className="sr-only">{copy.appearance}</legend>
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
            </section>
          )}

          {section === "translation" && (
            <section aria-labelledby="translation-heading">
              <div className="section-heading">
                <h3 id="translation-heading">{copy.translationSettings}</h3>
                <small>{copy.proxyApplyNotice}</small>
              </div>
              <label className="field">
                {copy.targetLanguage}
                <select
                  value={preferences.targetLanguage}
                  onChange={(event) =>
                    patch({
                      targetLanguage: event.target.value as TargetLanguage,
                    })
                  }
                >
                  {languages.map(([code, label]) => (
                    <option key={code} value={code}>
                      {label}
                    </option>
                  ))}
                </select>
                <small>{copy.settingsSavedImmediately}</small>
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
                {proxyError && (
                  <span className="field-error">{copy.invalidProxy}</span>
                )}
              </label>
              <button
                type="button"
                className="proxy-apply"
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
            </section>
          )}

          {section === "installation" && (
            <section aria-labelledby="installation-heading">
              <div className="section-heading">
                <h3 id="installation-heading">{copy.installation}</h3>
                <small>{copy.settingsSavedImmediately}</small>
              </div>
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
              <fieldset className="agent-targets">
                <legend>{copy.agentTargets}</legend>
                <div className="agent-summary">
                  <label className="choice">
                    <input
                      type="radio"
                      name="agents-mode"
                      checked={preferences.agents.length === 0}
                      onChange={() => patch({ agents: [] })}
                    />
                    {copy.automaticAgents}
                  </label>
                  <button
                    type="button"
                    className="text-button"
                    onClick={() =>
                      patch({
                        agents: preferences.agents.length ? [] : ["codex"],
                      })
                    }
                  >
                    {preferences.agents.length
                      ? copy.automaticAgents
                      : copy.chooseExplicitTargets}
                  </button>
                </div>
                <div className="agent-toolbar">
                  <label className="search-field">
                    <Icon name="search" />
                    <input
                      value={agentFilter}
                      onChange={(event) => setAgentFilter(event.target.value)}
                      placeholder={copy.agentSearch}
                      aria-label={copy.agentSearch}
                    />
                  </label>
                  <small>
                    {preferences.agents.length || copy.allTargets}{" "}
                    {preferences.agents.length ? copy.selectedTargets : ""}
                  </small>
                </div>
                <div className="agent-options">
                  {filteredAgents.map((agent) => (
                    <label className="choice" key={agent}>
                      <input
                        type="checkbox"
                        disabled={preferences.agents.length === 0}
                        checked={preferences.agents.includes(agent)}
                        onChange={(event) =>
                          patch({
                            agents: event.target.checked
                              ? [...preferences.agents, agent]
                              : preferences.agents.filter(
                                  (value) => value !== agent,
                                ),
                          })
                        }
                      />
                      {agent}
                    </label>
                  ))}
                  {filteredAgents.length === 0 && (
                    <p className="empty-list">{copy.noMatchingTargets}</p>
                  )}
                </div>
              </fieldset>
            </section>
          )}

          {section === "about" && (
            <section aria-labelledby="about-heading">
              <div className="section-heading">
                <h3 id="about-heading">{copy.about}</h3>
              </div>
              <p className="version-row">
                <span>{copy.cliVersion}</span>
                <code>{version ?? "—"}</code>
              </p>
            </section>
          )}
        </div>
      </div>

      <footer className="dialog-footer">
        <small>
          {section === "translation"
            ? copy.proxyApplyNotice
            : copy.settingsSavedImmediately}
        </small>
        <button type="button" onClick={onClose}>
          {copy.close}
        </button>
      </footer>
    </ModalShell>
  );
}
