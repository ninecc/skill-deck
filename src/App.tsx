import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
} from "react";
import ReactMarkdown from "react-markdown";
import {
  addSkill,
  commandErrorCode,
  commandErrorMessage,
  previewTree,
  readPreview,
  removeSkill,
  retryRuntime,
  revealPath,
  runtimeStatus,
  searchSkills,
  translatePreview,
  updateSkill,
  type FileContent,
  type FileEntry,
  type InstalledSkill,
  type RuntimeStatus,
  type SearchResult,
} from "./api";
import { Icon } from "./icons";
import { catalogs, preferredLocale, type Locale } from "./i18n";
import {
  loadPreferences,
  resolvedTheme,
  savePreferences,
  type Preferences,
} from "./preferences";
import SettingsDialog from "./SettingsDialog";

const markdownComponents: ComponentProps<typeof ReactMarkdown>["components"] = {
  a: ({ children }) => <span>{children}</span>,
  img: ({ alt }) => <span>{alt ?? ""}</span>,
};

export default function App() {
  const [locale, setLocale] = useState<Locale>(() =>
    preferredLocale(
      localStorage.getItem("skill-deck-locale"),
      navigator.language,
    ),
  );
  const copy = catalogs[locale];
  const [preferences, setPreferences] = useState<Preferences>(() =>
    loadPreferences(),
  );
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [inventory, setInventory] = useState<InstalledSkill[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [tree, setTree] = useState<FileEntry[]>([]);
  const [file, setFile] = useState<FileContent | null>(null);
  const [treeOpen, setTreeOpen] = useState(false);
  const [translationOn, setTranslationOn] = useState(false);
  const [translationState, setTranslationState] = useState<{
    key: string;
    text?: string;
    error?: string;
  } | null>(null);
  const [mobilePane, setMobilePane] = useState<"original" | "translation">(
    "original",
  );
  const [filter, setFilter] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [discoveryOpen, setDiscoveryOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [source, setSource] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const previewRequest = useRef(0);
  const translationRequest = useRef(0);
  const [translationRetry, setTranslationRetry] = useState(0);

  function invalidateTranslation() {
    translationRequest.current += 1;
    setTranslationState(null);
  }

  function changePreferences(next: Preferences) {
    if (
      next.targetLanguage !== preferences.targetLanguage ||
      next.translationProxy !== preferences.translationProxy
    )
      invalidateTranslation();
    setPreferences(next);
  }

  useEffect(() => {
    void runtimeStatus()
      .then((status) => {
        setInventory(status.inventory);
        setRuntime(status);
      })
      .catch((value: unknown) => setError(commandErrorMessage(value)));
  }, []);

  useEffect(() => {
    savePreferences(preferences);
    const media = matchMedia("(prefers-color-scheme: dark)");
    const apply = () =>
      (document.documentElement.dataset.theme = resolvedTheme(
        preferences.theme,
        media.matches,
      ));
    apply();
    if (preferences.theme === "system") media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [preferences]);

  useEffect(() => {
    if (!selected) return;
    const request = ++previewRequest.current;
    void previewTree(selected)
      .then((entries) => {
        if (previewRequest.current !== request) return;
        setTree(entries);
        const first =
          entries.find(
            (entry) =>
              !entry.directory && entry.path.toLowerCase() === "skill.md",
          ) ??
          entries.find((entry) => !entry.directory && !entry.unsupportedReason);
        if (first)
          return readPreview(selected, first.path).then((content) => {
            if (previewRequest.current === request) setFile(content);
          });
      })
      .catch((value: unknown) => {
        if (previewRequest.current === request)
          setError(commandErrorMessage(value));
      });
  }, [selected]);

  useEffect(() => {
    const request = ++translationRequest.current;
    if (!translationOn || !selected || !file?.translatable) return;
    const key = `${selected}\n${file.path}\n${preferences.targetLanguage}\n${preferences.translationProxy}`;
    void translatePreview(
      selected,
      file.path,
      preferences.targetLanguage,
      preferences.translationProxy,
    )
      .then((result) => {
        if (translationRequest.current === request)
          setTranslationState({ key, text: result.translatedText });
      })
      .catch((value: unknown) => {
        if (translationRequest.current !== request) return;
        const code = commandErrorCode(value);
        setTranslationState({
          key,
          error:
            code === "invalid_proxy"
              ? copy.invalidProxy
              : code === "translation_timeout"
                ? copy.translationTimedOut
                : code === "translation_unavailable" ||
                    code === "translation_response"
                  ? copy.translationUnavailable
                  : commandErrorMessage(value),
        });
      });
  }, [
    translationOn,
    selected,
    file,
    preferences.targetLanguage,
    preferences.translationProxy,
    translationRetry,
    copy.invalidProxy,
    copy.translationTimedOut,
    copy.translationUnavailable,
  ]);

  const translationKey =
    selected && file
      ? `${selected}\n${file.path}\n${preferences.targetLanguage}\n${preferences.translationProxy}`
      : null;
  const currentTranslation =
    translationState?.key === translationKey ? translationState : null;

  const runtimeFailure = (() => {
    if (!runtime || runtime.ready) return null;
    switch (runtime.errorCode) {
      case "runtime_not_found":
        return [copy.runtimeNotFound, copy.runtimeNotFoundHint];
      case "node_too_old":
        return [copy.runtimeTooOld, copy.runtimeTooOldHint];
      case "incompatible_cli":
        return [copy.runtimeIncompatible, copy.runtimeIncompatibleHint];
      default:
        return [copy.runtimeUnavailable, copy.runtimeUnavailableHint];
    }
  })();

  const visible = useMemo(() => {
    const query = filter.trim().toLowerCase();
    return inventory.filter(
      (skill) =>
        !query ||
        skill.name.toLowerCase().includes(query) ||
        skill.source?.toLowerCase().includes(query) ||
        skill.path.toLowerCase().includes(query),
    );
  }, [filter, inventory]);

  function perform(
    label: string,
    operation: () => Promise<{
      inventory: InstalledSkill[];
      targetObserved?: boolean | null;
      diagnostics: string;
    }>,
    success: string,
  ) {
    setBusy(label);
    setError(null);
    setNotice(null);
    void operation()
      .then((result) => {
        setInventory(result.inventory);
        setNotice(
          [
            result.targetObserved === false ? copy.targetNotObserved : success,
            result.diagnostics,
          ]
            .filter(Boolean)
            .join("\n"),
        );
        if (
          selected &&
          !result.inventory.some((skill) => skill.name === selected)
        )
          setSelected(null);
      })
      .catch((value: unknown) => setError(commandErrorMessage(value)))
      .finally(() => setBusy(null));
  }

  function chooseFile(entry: FileEntry) {
    if (!selected || entry.directory) return;
    const request = ++previewRequest.current;
    invalidateTranslation();
    setTreeOpen(false);
    if (entry.unsupportedReason) {
      setFile({
        path: entry.path,
        viewer: "unsupported",
        size: entry.size,
        text: null,
        dataUrl: null,
        translatable: false,
        unsupportedReason: entry.unsupportedReason,
      });
      return;
    }
    setFile(null);
    void readPreview(selected, entry.path)
      .then((content) => {
        if (previewRequest.current === request) setFile(content);
      })
      .catch((value: unknown) => {
        if (previewRequest.current === request)
          setError(commandErrorMessage(value));
      });
  }

  function chooseSkill(name: string) {
    previewRequest.current += 1;
    invalidateTranslation();
    setTree([]);
    setFile(null);
    setSelected(name);
  }

  function moveFocus(event: React.KeyboardEvent<HTMLButtonElement>) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const rows = Array.from(
      event.currentTarget
        .closest(".source-list")
        ?.querySelectorAll<HTMLButtonElement>(".source-row") ?? [],
    );
    const index = rows.indexOf(event.currentTarget);
    rows[index + (event.key === "ArrowDown" ? 1 : -1)]?.focus();
  }

  return (
    <main className="app-shell">
      <header className="app-bar">
        <div className="brand">
          <span className="brand-mark">S</span>
          <strong>Skill Deck</strong>
        </div>
        <div className="bar-actions">
          <button
            type="button"
            className="button"
            disabled={!runtime?.ready || busy !== null}
            onClick={() =>
              window.confirm(copy.confirmUpdateAll) &&
              perform("update-all", () => updateSkill(null), copy.refreshed)
            }
          >
            <Icon name="refresh" />
            {copy.updateAll}
          </button>
          <button
            type="button"
            className="icon-button"
            onClick={() => setSettingsOpen(true)}
            aria-label={copy.settings}
            title={copy.settings}
          >
            <Icon name="settings" />
          </button>
          <select
            aria-label={copy.language}
            value={locale}
            onChange={(event) => {
              const next = event.target.value as Locale;
              localStorage.setItem("skill-deck-locale", next);
              setLocale(next);
            }}
          >
            <option value="en">EN</option>
            <option value="zh-CN">中文</option>
          </select>
        </div>
      </header>
      {runtime?.ready && (notice || error) && (
        <div className={error ? "status error" : "status"}>
          {error ?? notice}
        </div>
      )}
      {!runtime?.ready && (
        <section className="runtime-screen" aria-live="polite">
          {runtimeFailure || error ? (
            <>
              <h1>{copy.runtimeErrorTitle}</h1>
              <p>{runtimeFailure?.[0] ?? copy.runtimeUnavailable}</p>
              <small>
                {runtimeFailure?.[1] ?? copy.runtimeUnavailableHint}
              </small>
            </>
          ) : (
            <p className="loading-copy">
              <span className="spinner" aria-hidden="true" />
              {copy.loadingSkills}
            </p>
          )}
          {(runtime || error) && (
            <button
              type="button"
              onClick={() => {
                setRuntime(null);
                setError(null);
                void retryRuntime()
                  .then((status) => {
                    setInventory(status.inventory);
                    setRuntime(status);
                  })
                  .catch((value: unknown) =>
                    setError(commandErrorMessage(value)),
                  );
              }}
            >
              {copy.retry}
            </button>
          )}
        </section>
      )}
      <div className="workspace" inert={!runtime?.ready ? true : undefined}>
        <aside className="inventory-pane" aria-labelledby="installed-heading">
          <div className="pane-heading">
            <div>
              <h1 id="installed-heading">{copy.installed}</h1>
              <span>{inventory.length}</span>
              <button
                type="button"
                className="find-install-button"
                onClick={() => setDiscoveryOpen(true)}
              >
                <Icon name="search" />
                {copy.findInstall}
              </button>
            </div>
            <label className="search-field">
              <Icon name="search" />
              <input
                value={filter}
                onChange={(event) => setFilter(event.target.value)}
                placeholder={copy.filter}
                aria-label={copy.filter}
              />
            </label>
          </div>
          <div
            className="source-list"
            role="listbox"
            aria-label={copy.installed}
          >
            {visible.map((skill) => (
              <button
                key={skill.name}
                type="button"
                className="source-row"
                role="option"
                aria-selected={selected === skill.name}
                onKeyDown={moveFocus}
                onClick={() => chooseSkill(skill.name)}
              >
                <strong>{skill.name}</strong>
                <span>
                  {skill.source
                    ? `${skill.source} · ${skill.path}`
                    : skill.path}
                </span>
              </button>
            ))}
          </div>
          {runtime?.ready && inventory.length === 0 && !filter.trim() && (
            <p className="empty-list">{copy.noSkills}</p>
          )}
          {runtime?.ready && inventory.length > 0 && visible.length === 0 && (
            <p className="empty-list">{copy.noMatchingSkills}</p>
          )}
        </aside>
        <section className="detail-pane" aria-label={copy.preview}>
          {!selected ? (
            <p className="choose-placeholder">{copy.chooseSkill}</p>
          ) : (
            <>
              <div className="detail-toolbar">
                <button
                  type="button"
                  className="mobile-back"
                  onClick={() => setSelected(null)}
                >
                  <Icon name="chevron" />
                  {copy.backToInventory}
                </button>
                <div className="path-control">
                  <button
                    type="button"
                    className="path-button"
                    aria-haspopup="tree"
                    aria-expanded={treeOpen}
                    onClick={() => setTreeOpen((open) => !open)}
                  >
                    <span>{file?.path ?? copy.path}</span>
                    <Icon name="chevron" />
                  </button>
                  {treeOpen && (
                    <div
                      className="file-tree"
                      role="tree"
                      aria-label={copy.skillFiles}
                      onKeyDown={(event) => {
                        if (
                          event.key !== "ArrowDown" &&
                          event.key !== "ArrowUp"
                        )
                          return;
                        event.preventDefault();
                        const items = Array.from(
                          event.currentTarget.querySelectorAll<HTMLButtonElement>(
                            'button[role="treeitem"]:not(:disabled)',
                          ),
                        );
                        const current = items.indexOf(
                          document.activeElement as HTMLButtonElement,
                        );
                        items[
                          Math.max(
                            0,
                            Math.min(
                              items.length - 1,
                              current + (event.key === "ArrowDown" ? 1 : -1),
                            ),
                          )
                        ]?.focus();
                      }}
                    >
                      {tree.map((entry) => (
                        <button
                          type="button"
                          role="treeitem"
                          aria-level={entry.level}
                          aria-label={entry.path}
                          data-path={entry.path}
                          key={entry.path}
                          disabled={entry.directory}
                          aria-current={
                            file?.path === entry.path ? "true" : undefined
                          }
                          style={{
                            paddingInlineStart: `${10 + (entry.level - 1) * 18}px`,
                          }}
                          onClick={() => chooseFile(entry)}
                        >
                          <Icon name={entry.directory ? "folder" : "file"} />
                          <span>{entry.name}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
                <div className="preview-actions">
                  {file?.translatable && (
                    <>
                      <span className="egress">{copy.egress}</span>
                      <button
                        type="button"
                        aria-pressed={translationOn}
                        onClick={() => {
                          invalidateTranslation();
                          setTranslationOn((on) => !on);
                        }}
                      >
                        <Icon name="translate" />
                        {translationOn ? copy.translationOn : copy.translate}
                      </button>
                    </>
                  )}
                  <button
                    type="button"
                    className="icon-button"
                    onClick={() =>
                      void revealPath(selected, file?.path ?? null).catch(
                        (value: unknown) =>
                          setError(commandErrorMessage(value)),
                      )
                    }
                    aria-label={file ? copy.revealFile : copy.revealRoot}
                    title={file ? copy.revealFile : copy.revealRoot}
                  >
                    <Icon name="folder" />
                  </button>
                  <button
                    type="button"
                    className="danger"
                    onClick={() =>
                      window.confirm(copy.confirmRemove) &&
                      perform(
                        `remove-${selected}`,
                        () => removeSkill(selected),
                        copy.inventoryRefreshed,
                      )
                    }
                  >
                    <Icon name="trash" />
                    {copy.remove}
                  </button>
                  <button
                    type="button"
                    disabled={busy !== null}
                    onClick={() =>
                      perform(
                        `update-${selected}`,
                        () => updateSkill(selected),
                        copy.refreshed,
                      )
                    }
                  >
                    <Icon name="refresh" />
                    {copy.update}
                  </button>
                </div>
              </div>
              {translationOn && file?.translatable && (
                <div className="mobile-tabs" role="tablist">
                  <button
                    role="tab"
                    aria-selected={mobilePane === "original"}
                    onClick={() => setMobilePane("original")}
                  >
                    {copy.original}
                  </button>
                  <button
                    role="tab"
                    aria-selected={mobilePane === "translation"}
                    onClick={() => setMobilePane("translation")}
                  >
                    {copy.translation}
                  </button>
                </div>
              )}
              <div
                className={`viewer-grid ${translationOn && file?.translatable ? "translated" : ""}`}
                data-pane={mobilePane}
              >
                <Viewer
                  file={file}
                  label={copy.original}
                  unsupported={copy.unsupported}
                />
                {translationOn && file?.translatable && (
                  <article className="viewer translation-view">
                    <h2>{copy.translation}</h2>
                    {!currentTranslation ||
                    (currentTranslation.text === undefined &&
                      !currentTranslation.error) ? (
                      <p>{copy.translating}</p>
                    ) : currentTranslation.error ? (
                      <div className="translation-error">
                        <p className="error">{currentTranslation.error}</p>
                        <button
                          type="button"
                          onClick={() => {
                            invalidateTranslation();
                            setTranslationRetry((value) => value + 1);
                          }}
                        >
                          {copy.retry}
                        </button>
                      </div>
                    ) : file.viewer === "markdown" ? (
                      <ReactMarkdown skipHtml components={markdownComponents}>
                        {currentTranslation.text ?? ""}
                      </ReactMarkdown>
                    ) : (
                      <pre>{currentTranslation.text ?? ""}</pre>
                    )}
                  </article>
                )}
              </div>
            </>
          )}
        </section>
      </div>
      {discoveryOpen && (
        <div
          className="sheet-backdrop"
          role="presentation"
          onMouseDown={(event) =>
            event.target === event.currentTarget && setDiscoveryOpen(false)
          }
        >
          <section
            className="settings-sheet discovery-sheet"
            role="dialog"
            aria-modal="true"
            aria-labelledby="find-install-title"
          >
            <header>
              <h2 id="find-install-title">{copy.findInstall}</h2>
              <button
                className="icon-button"
                type="button"
                onClick={() => setDiscoveryOpen(false)}
                aria-label={copy.close}
                title={copy.close}
              >
                <Icon name="close" />
              </button>
            </header>
            <section className="discovery" aria-labelledby="find-heading">
              <h3 id="find-heading">{copy.search}</h3>
              <form
                onSubmit={(event) => {
                  event.preventDefault();
                  setError(null);
                  void searchSkills(searchQuery)
                    .then(setSearchResults)
                    .catch((value: unknown) =>
                      setError(commandErrorMessage(value)),
                    );
                }}
              >
                <input
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  aria-label={copy.searchQuery}
                  required
                />
                <button
                  type="submit"
                  className="icon-button"
                  aria-label={copy.search}
                  title={copy.search}
                >
                  <Icon name="search" />
                </button>
              </form>
              {error && <p className="error">{error}</p>}
              {searchResults.map((result) => (
                <div className="search-result" key={result.slug}>
                  <span>
                    <strong>{result.name}</strong>
                    <small>
                      {result.source} · {result.installs.toLocaleString()}
                    </small>
                  </span>
                  <button
                    type="button"
                    disabled={busy !== null}
                    onClick={() =>
                      perform(
                        `add-${result.slug}`,
                        () =>
                          addSkill(result.source, result.name, {
                            agents: preferences.agents,
                            copy: preferences.copy,
                          }),
                        copy.inventoryRefreshed,
                      )
                    }
                  >
                    {copy.install}
                  </button>
                </div>
              ))}
              <form
                className="source-install"
                onSubmit={(event) => {
                  event.preventDefault();
                  perform(
                    "add-source",
                    () =>
                      addSkill(source, null, {
                        agents: preferences.agents,
                        copy: preferences.copy,
                      }),
                    copy.inventoryRefreshed,
                  );
                }}
              >
                <label>
                  {copy.installSource}
                  <input
                    value={source}
                    onChange={(event) => setSource(event.target.value)}
                    required
                  />
                </label>
                <small>{copy.sourceHint}</small>
                <button type="submit" disabled={busy !== null}>
                  <Icon name="download" />
                  {copy.install}
                </button>
              </form>
            </section>
          </section>
        </div>
      )}
      {settingsOpen && (
        <SettingsDialog
          copy={copy}
          version={runtime?.version ?? null}
          preferences={preferences}
          onChange={changePreferences}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </main>
  );
}

function Viewer({
  file,
  label,
  unsupported,
}: {
  file: FileContent | null;
  label: string;
  unsupported: string;
}) {
  if (!file)
    return (
      <article className="viewer">
        <p>{unsupported}</p>
      </article>
    );
  return (
    <article className="viewer original-view">
      <h2>{label}</h2>
      {file.viewer === "markdown" ? (
        <ReactMarkdown skipHtml components={markdownComponents}>
          {file.text ?? ""}
        </ReactMarkdown>
      ) : file.viewer === "image" ? (
        <img src={file.dataUrl ?? ""} alt={file.path} />
      ) : file.viewer === "unsupported" ? (
        <p>
          {file.unsupportedReason ?? unsupported}
          <br />
          {file.size.toLocaleString()} bytes
        </p>
      ) : (
        <pre>
          <code>{file.text}</code>
        </pre>
      )}
    </article>
  );
}
