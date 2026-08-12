import { Icon } from "./icons";
import type { Messages } from "./i18n";
import {
  agentOptions,
  languages,
  themes,
  type Preferences,
  type TargetLanguage,
  type Theme,
} from "./preferences";

interface Props {
  copy: Messages;
  version: string | null;
  preferences: Preferences;
  onChange: (preferences: Preferences) => void;
  onClose: () => void;
}

export default function SettingsDialog({
  copy,
  version,
  preferences,
  onChange,
  onClose,
}: Props) {
  const patch = (next: Partial<Preferences>) =>
    onChange({ ...preferences, ...next });
  return (
    <div
      className="sheet-backdrop"
      role="presentation"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section
        className="settings-sheet"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
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
              <label className={`theme-tile preview-${theme}`} key={theme}>
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
      </section>
    </div>
  );
}
