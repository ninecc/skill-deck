import {
  useEffect,
  useCallback,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
  type KeyboardEvent as ReactKeyboardEvent,
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
  type CommandResult,
  type FileContent,
  type FileEntry,
  type InstalledSkill,
  type RuntimeStatus,
  type SearchResult,
} from "./api";
import {
  commandAvailability,
  commandIds,
  createDispatcher,
  isNarrowBackShortcut,
  shortcutCommand,
  type CommandContext,
  type CommandId,
  type UnavailableReason,
} from "./commands";
import { effectiveLocale, catalogs, type Messages } from "./i18n";
import { Icon, type IconName } from "./icons";
import ModalShell from "./ModalShell";
import {
  installNativeMenu,
  popupSkillMenu,
  type CommandStates,
} from "./nativeMenu";
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

type OperationKind = "refresh" | "install" | "update" | "remove";
type DiscoveryTab = "search" | "source";
type Modal =
  | { kind: "settings"; trigger: HTMLElement | null }
  | { kind: "discovery"; trigger: HTMLElement | null }
  | {
      kind: "remove";
      name: string;
      trigger: HTMLElement | null;
      confirmed?: boolean;
    }
  | null;
type Feedback = {
  severity: "neutral" | "success" | "partial" | "error";
  summary: string;
  diagnostics?: string;
  review?: boolean;
};

export default function App() {
  const [preferences, setPreferences] = useState<Preferences>(() =>
    loadPreferences(),
  );
  const [systemLanguages, setSystemLanguages] = useState<readonly string[]>(
    () =>
      navigator.languages.length ? navigator.languages : [navigator.language],
  );
  const locale = effectiveLocale(preferences.uiLocale, systemLanguages);
  const copy = catalogs[locale] ?? catalogs.en;
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [inventory, setInventory] = useState<InstalledSkill[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [tree, setTree] = useState<FileEntry[]>([]);
  const [expandedDirectories, setExpandedDirectories] = useState<Set<string>>(
    () => new Set(),
  );
  const [treeFocusPath, setTreeFocusPath] = useState<string | null>(null);
  const [file, setFile] = useState<FileContent | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [translationOn, setTranslationOn] = useState(false);
  const [translationState, setTranslationState] = useState<{
    key: string;
    text?: string;
    error?: string;
  } | null>(null);
  const [translationRetry, setTranslationRetry] = useState(0);
  const [mobilePane, setMobilePane] = useState<"original" | "translation">(
    "original",
  );
  const [filter, setFilter] = useState("");
  const [modal, setModal] = useState<Modal>(null);
  const [transient, setTransient] = useState<"tree" | "diagnostics" | null>(
    null,
  );
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [source, setSource] = useState("");
  const [discoveryTab, setDiscoveryTab] = useState<DiscoveryTab>("search");
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);
  const [unresolvedDiscovery, setUnresolvedDiscovery] = useState(false);
  const [lastDiscoveryTarget, setLastDiscoveryTarget] = useState<{
    source: string;
    name: string | null;
  } | null>(null);
  const [operation, setOperation] = useState<OperationKind | null>(null);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const [runtimeError, setRuntimeError] = useState<string | null>(null);
  const previewRequest = useRef(0);
  const previewPath = useRef<string | null>(null);
  const translationRequest = useRef(0);
  const filterRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const pathRef = useRef<HTMLButtonElement>(null);
  const dispatchRef = useRef<(id: CommandId) => void>(() => undefined);
  const contextRef = useRef<CommandContext>({
    runtimeReady: false,
    inventoryCount: 0,
    selected: false,
    mutationActive: false,
    modal: null,
    document: "none",
  });

  function invalidateTranslation() {
    translationRequest.current += 1;
    setTranslationState(null);
  }

  function hideTranslation() {
    invalidateTranslation();
    setTranslationOn(false);
    setMobilePane("original");
  }

  function changePreferences(next: Preferences) {
    if (
      next.targetLanguage !== preferences.targetLanguage ||
      next.translationProxy !== preferences.translationProxy
    )
      invalidateTranslation();
    setPreferences(next);
  }

  function closeModal() {
    setModal(null);
  }

  function openModal(next: Exclude<Modal, null>) {
    setTransient(null);
    setModal(next);
  }

  async function loadPreview(name: string, preferredPath?: string | null) {
    const request = ++previewRequest.current;
    setTree([]);
    setExpandedDirectories(new Set());
    setTreeFocusPath(null);
    setFile(null);
    setPreviewError(null);
    setPreviewLoading(true);
    previewPath.current = preferredPath ?? null;
    hideTranslation();
    try {
      const entries = await previewTree(name);
      if (previewRequest.current !== request) return;
      const candidate =
        entries.find(
          (entry) => !entry.directory && entry.path === preferredPath,
        ) ??
        entries.find(
          (entry) =>
            !entry.directory && entry.path.toLowerCase() === "skill.md",
        ) ??
        entries.find((entry) => !entry.directory && !entry.unsupportedReason) ??
        entries.find((entry) => !entry.directory);
      previewPath.current = candidate?.path ?? null;
      let content: FileContent | null = null;
      if (candidate?.unsupportedReason) {
        content = {
          path: candidate.path,
          viewer: "unsupported",
          size: candidate.size,
          text: null,
          dataUrl: null,
          translatable: false,
          unsupportedReason: candidate.unsupportedReason,
        };
      } else if (candidate) content = await readPreview(name, candidate.path);
      if (previewRequest.current === request) {
        setTree(entries);
        setExpandedDirectories(
          new Set(
            entries
              .filter((entry) => entry.directory)
              .map((entry) => normalizeDirectoryPath(entry.path)),
          ),
        );
        setFile(content);
      }
    } catch (value: unknown) {
      if (previewRequest.current === request) {
        const message = commandErrorMessage(value);
        setPreviewError(message);
        setFeedback({
          severity: "error",
          summary: copy.previewError,
          diagnostics: message,
        });
      }
    } finally {
      if (previewRequest.current === request) setPreviewLoading(false);
    }
  }

  function chooseSkill(name: string) {
    setSelected(name);
    void loadPreview(name);
  }

  const returnToInventory = useCallback(() => {
    const name = selected;
    const scroll = listRef.current?.scrollTop ?? 0;
    setSelected(null);
    requestAnimationFrame(() => {
      if (listRef.current) listRef.current.scrollTop = scroll;
      const row = Array.from(
        listRef.current?.querySelectorAll<HTMLButtonElement>(".source-row") ??
          [],
      ).find((item) => item.dataset.skill === name);
      (
        row ??
        filterRef.current ??
        document.getElementById("installed-heading")
      )?.focus();
    });
  }, [selected]);

  async function refreshInventory(): Promise<Feedback | void> {
    const previous = selected;
    const previousPath = file?.path;
    const status = await runtimeStatus();
    setRuntime(status);
    setInventory(status.inventory);
    if (!previous) return;
    if (status.inventory.some((skill) => skill.name === previous))
      await loadPreview(previous, previousPath);
    else {
      setSelected(null);
      setTree([]);
      setFile(null);
      return { severity: "neutral", summary: copy.selectionRemoved };
    }
  }

  function startOperation(
    kind: OperationKind,
    run: () => Promise<CommandResult | Feedback | void>,
    target?: { type: "search"; name: string } | { type: "source" },
  ) {
    setOperation(kind);
    setFeedback(null);
    setTransient(null);
    void run()
      .then((value) => {
        if (!value) {
          setFeedback({ severity: "neutral", summary: copy.neutralComplete });
          return;
        }
        if ("severity" in value) {
          setFeedback(value);
          return;
        }
        setInventory(value.inventory);
        const diagnostics = value.diagnostics || undefined;
        if (kind === "install" && target) {
          const matches =
            target.type === "search"
              ? value.inventory.filter((skill) => skill.name === target.name)
              : value.changedSkills.length === 1
                ? value.inventory.filter(
                    (skill) => skill.name === value.changedSkills[0],
                  )
                : [];
          if (matches.length === 1) {
            setUnresolvedDiscovery(false);
            setDiscoveryError(null);
            setModal(null);
            chooseSkill(matches[0].name);
            setFeedback({
              severity: "success",
              summary: copy.inventoryRefreshed,
              diagnostics,
            });
          } else {
            setUnresolvedDiscovery(true);
            setDiscoveryError(copy.targetNotObserved);
            setFeedback({
              severity:
                target.type === "source" && !value.changedSkills.length
                  ? "neutral"
                  : "partial",
              summary:
                target.type === "source" && !value.changedSkills.length
                  ? copy.neutralComplete
                  : copy.partialComplete,
              diagnostics,
              review: true,
            });
          }
        } else {
          const partial = value.targetObserved === false;
          setFeedback({
            severity:
              kind === "update" ? "neutral" : partial ? "partial" : "success",
            summary: partial ? copy.partialComplete : copy.neutralComplete,
            diagnostics,
          });
          if (
            selected &&
            !value.inventory.some((skill) => skill.name === selected)
          ) {
            setSelected(null);
            setTree([]);
            setFile(null);
          }
          if (kind === "remove") setModal(null);
        }
      })
      .catch((value: unknown) => {
        const message = commandErrorMessage(value);
        if (kind === "install") {
          setDiscoveryError(message);
          setUnresolvedDiscovery(true);
        }
        setFeedback({
          severity: "error",
          summary: message,
          review: kind === "install",
        });
      })
      .finally(() => setOperation(null));
  }

  function executeCommand(id: CommandId) {
    switch (id) {
      case "find-installed":
        setModal(null);
        filterRef.current?.focus();
        return;
      case "find-install":
        if (!unresolvedDiscovery) {
          setDiscoveryTab("search");
          setSearchQuery("");
          setSearchResults([]);
          setSource("");
          setDiscoveryError(null);
          setLastDiscoveryTarget(null);
        }
        openModal({
          kind: "discovery",
          trigger: document.activeElement as HTMLElement | null,
        });
        return;
      case "settings":
        openModal({
          kind: "settings",
          trigger: document.activeElement as HTMLElement | null,
        });
        return;
      case "refresh-inventory":
        startOperation("refresh", refreshInventory);
        return;
      case "update-all":
        startOperation("update", () => updateSkill(null));
        return;
      case "translate-skill":
        if (translationOn) hideTranslation();
        else setTranslationOn(true);
        return;
      case "reveal-skill":
        if (selected)
          void revealPath(selected, file?.path ?? previewPath.current).catch(
            (value: unknown) =>
              setFeedback({
                severity: "error",
                summary: commandErrorMessage(value),
              }),
          );
        return;
      case "update-skill":
        if (selected) startOperation("update", () => updateSkill(selected));
        return;
      case "remove-skill":
        if (selected)
          openModal({
            kind: "remove",
            name: selected,
            trigger: document.activeElement as HTMLElement | null,
          });
    }
  }

  const documentState: CommandContext["document"] = previewLoading
    ? "loading"
    : file?.translatable
      ? "supported"
      : file
        ? "unsupported"
        : "none";
  const commandContext = useMemo<CommandContext>(
    () => ({
      runtimeReady: runtime?.ready === true,
      inventoryCount: inventory.length,
      selected: selected !== null,
      mutationActive: operation !== null,
      modal: modal?.kind ?? null,
      document: documentState,
    }),
    [
      documentState,
      inventory.length,
      modal?.kind,
      operation,
      runtime?.ready,
      selected,
    ],
  );
  useEffect(() => {
    contextRef.current = commandContext;
    dispatchRef.current = executeCommand;
  });
  const dispatch = useCallback(
    (id: CommandId) =>
      createDispatcher(
        () => contextRef.current,
        (next) => dispatchRef.current(next),
      )(id),
    [],
  );
  const commandStates = useMemo(
    () =>
      Object.fromEntries(
        commandIds.map((id) => [id, commandAvailability(id, commandContext)]),
      ) as CommandStates,
    [commandContext],
  );

  useEffect(() => {
    void runtimeStatus()
      .then((status) => {
        setInventory(status.inventory);
        setRuntime(status);
      })
      .catch((value: unknown) => setRuntimeError(commandErrorMessage(value)));
  }, []);

  useEffect(() => {
    savePreferences(preferences);
    const media = matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      document.documentElement.dataset.theme = resolvedTheme(
        preferences.theme,
        media.matches,
      );
      document.documentElement.style.colorScheme =
        resolvedTheme(preferences.theme, media.matches) === "dark" ||
        resolvedTheme(preferences.theme, media.matches) === "plum"
          ? "dark"
          : "light";
    };
    apply();
    if (preferences.theme === "system") media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [preferences]);

  useEffect(() => {
    if (preferences.uiLocale !== "system") return;
    const update = () => setSystemLanguages([...navigator.languages]);
    window.addEventListener("languagechange", update);
    return () => window.removeEventListener("languagechange", update);
  }, [preferences.uiLocale]);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

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

  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (transient) {
          event.preventDefault();
          setTransient(null);
          if (transient === "tree") pathRef.current?.focus();
        } else if (!modal && selected) {
          event.preventDefault();
          returnToInventory();
        }
        return;
      }
      if (
        selected &&
        !modal &&
        isNarrowBackShortcut(event, matchMedia("(max-width: 820px)").matches)
      ) {
        event.preventDefault();
        returnToInventory();
        return;
      }
      const id = shortcutCommand(event);
      if (id) {
        event.preventDefault();
        dispatch(id);
      }
    };
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, [dispatch, modal, returnToInventory, selected, transient]);

  useEffect(() => {
    let cleanup: () => void = () => undefined;
    let disposed = false;
    void installNativeMenu(copy, commandStates, translationOn, dispatch)
      .then((next) => {
        if (disposed) void next();
        else cleanup = () => void next();
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      void cleanup();
    };
  }, [copy, dispatch, translationOn, commandStates]);

  useEffect(() => {
    if (!transient) return;
    const closeOutside = (event: MouseEvent) => {
      const target = event.target;
      if (
        target instanceof Element &&
        target.closest(
          ".file-tree, .path-button, .diagnostics-popover, .status-summary button",
        )
      )
        return;
      setTransient(null);
      if (
        !(target instanceof Element) ||
        !target.closest("button, input, select, a, [tabindex]")
      )
        requestAnimationFrame(() => pathRef.current?.focus());
    };
    document.addEventListener("mousedown", closeOutside);
    if (transient === "tree")
      requestAnimationFrame(() => {
        const treeElement = document.querySelector<HTMLElement>(".file-tree");
        const selectedPath = file?.path;
        const selectedItem = selectedPath
          ? Array.from(
              treeElement?.querySelectorAll<HTMLElement>(
                'button[role="treeitem"]',
              ) ?? [],
            ).find((item) => item.dataset.path === selectedPath)
          : null;
        (
          selectedItem ??
          treeElement?.querySelector<HTMLElement>('button[role="treeitem"]')
        )?.focus();
      });
    return () => document.removeEventListener("mousedown", closeOutside);
  }, [file?.path, transient]);

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
  const selectedSkill = selected
    ? (inventory.find((skill) => skill.name === selected) ?? null)
    : null;
  const translationKey =
    selected && file
      ? `${selected}\n${file.path}\n${preferences.targetLanguage}\n${preferences.translationProxy}`
      : null;
  const currentTranslation =
    translationState?.key === translationKey ? translationState : null;
  const visibleTree = useMemo(
    () => visibleTreeEntries(tree, expandedDirectories),
    [expandedDirectories, tree],
  );
  const treeTabPath =
    treeFocusPath && visibleTree.some((entry) => entry.path === treeFocusPath)
      ? treeFocusPath
      : visibleTree.some((entry) => entry.path === file?.path)
        ? file?.path
        : visibleTree[0]?.path;

  function openFileTree() {
    if (transient === "tree") {
      setTransient(null);
      return;
    }
    const selectedPath = file?.path;
    if (selectedPath) {
      const ancestors = directoryAncestors(tree, selectedPath);
      if (ancestors.length)
        setExpandedDirectories(
          (current) => new Set([...current, ...ancestors]),
        );
    }
    setTransient("tree");
  }

  function setDirectoryExpanded(path: string, expanded: boolean) {
    const normalized = normalizeDirectoryPath(path);
    setExpandedDirectories((current) => {
      const next = new Set(current);
      if (expanded) next.add(normalized);
      else next.delete(normalized);
      return next;
    });
  }
  const runtimeFailure = (() => {
    if (runtimeError) return [runtimeError, null] as const;
    if (!runtime || runtime.ready) return null;
    switch (runtime.errorCode) {
      case "runtime_not_found":
        return [copy.runtimeNotFound, copy.runtimeNotFoundHint] as const;
      case "node_too_old":
        return [copy.runtimeTooOld, copy.runtimeTooOldHint] as const;
      case "incompatible_cli":
        return [
          copy.runtimeIncompatible,
          copy.runtimeIncompatibleHint,
        ] as const;
      default:
        return [copy.runtimeUnavailable, copy.runtimeUnavailableHint] as const;
    }
  })();

  const reasonCopy = (reason: UnavailableReason | null) => {
    switch (reason) {
      case "runtime-unavailable":
        return copy.unavailableRuntime;
      case "inventory-empty":
        return copy.unavailableEmpty;
      case "no-skill-selected":
        return copy.unavailableSelection;
      case "mutation-active":
        return copy.unavailableBusy;
      case "modal-active":
        return copy.unavailableModal;
      case "document-loading":
        return copy.unavailableLoading;
      case "unsupported-document":
        return copy.unavailableDocument;
      default:
        return undefined;
    }
  };

  const commandButton = (
    id: CommandId,
    icon: IconName,
    label: string,
    className = "",
    iconOnly = id === "settings",
  ) => {
    const state = commandStates[id];
    const reason = reasonCopy(state.reason);
    return (
      <span
        className="command-wrapper"
        tabIndex={state.enabled ? undefined : 0}
        title={reason}
        aria-label={
          state.enabled || !reason ? undefined : `${label}: ${reason}`
        }
      >
        <button
          type="button"
          className={className}
          disabled={!state.enabled}
          aria-label={label}
          title={label}
          onClick={() => dispatch(id)}
        >
          <Icon name={icon} />
          {!iconOnly && <span className="command-label">{label}</span>}
        </button>
      </span>
    );
  };

  return (
    <main
      className={`app-shell ${selected ? "view-detail" : "view-inventory"}`}
    >
      <header className="app-bar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <strong>Skill Deck</strong>
          <span className="brand-count">{inventory.length}</span>
        </div>
        <div className="bar-actions">
          {commandButton(
            "find-install",
            "install",
            copy.findInstall,
            "primary",
          )}
          {commandButton("refresh-inventory", "refresh", copy.refreshInventory)}
          <span className="toolbar-separator" aria-hidden="true" />
          {commandButton("update-all", "update-all", copy.updateAll, "button")}
          {commandButton("settings", "settings", copy.settings, "icon-button")}
        </div>
      </header>

      {!runtime?.ready && (
        <section className="runtime-screen" aria-live="polite">
          {runtimeFailure ? (
            <div className="operational-state runtime-failure-state">
              <span className="state-symbol" aria-hidden="true">
                <Icon name="runtime-warning" />
              </span>
              <h1>{copy.runtimeErrorTitle}</h1>
              <p role="alert">{runtimeFailure[0]}</p>
              {runtimeFailure[1] && <small>{runtimeFailure[1]}</small>}
              <button
                type="button"
                onClick={() => {
                  setRuntime(null);
                  setRuntimeError(null);
                  void retryRuntime()
                    .then((status) => {
                      setRuntime(status);
                      setInventory(status.inventory);
                    })
                    .catch((value: unknown) =>
                      setRuntimeError(commandErrorMessage(value)),
                    );
                }}
              >
                <Icon name="refresh" />
                {copy.retry}
              </button>
            </div>
          ) : (
            <div className="operational-state loading-state">
              <span className="spinner" aria-hidden="true" />
              <h1>{copy.loadingSkills}</h1>
              <p>{copy.loadingDetail}</p>
            </div>
          )}
        </section>
      )}

      <div className="workspace" inert={!runtime?.ready ? true : undefined}>
        <aside className="inventory-pane" aria-labelledby="installed-heading">
          <div className="pane-heading">
            <div>
              <h1 id="installed-heading" tabIndex={-1}>
                {copy.installed}
              </h1>
              <span>{inventory.length}</span>
            </div>
            <label className="search-field">
              <Icon name="search" />
              <input
                ref={filterRef}
                value={filter}
                onChange={(event) => setFilter(event.target.value)}
                placeholder={copy.filter}
                aria-label={copy.filter}
              />
            </label>
          </div>
          <div
            ref={listRef}
            className={`source-list ${visible.length ? "has-items" : ""}`}
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
                data-skill={skill.name}
                onKeyDown={moveListFocus}
                onClick={() => chooseSkill(skill.name)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  setTransient(null);
                  const changingSelection = selected !== skill.name;
                  if (changingSelection) chooseSkill(skill.name);
                  const context = {
                    ...contextRef.current,
                    selected: true,
                    document: changingSelection
                      ? ("loading" as const)
                      : contextRef.current.document,
                  };
                  const states = Object.fromEntries(
                    commandIds.map((id) => [
                      id,
                      commandAvailability(id, context),
                    ]),
                  ) as CommandStates;
                  void popupSkillMenu(
                    copy,
                    states,
                    changingSelection ? false : translationOn,
                    dispatch,
                  );
                }}
              >
                <strong>{skill.name}</strong>
                {skill.source && (
                  <span title={skill.source}>{skill.source}</span>
                )}
                <code title={skill.path}>{skill.path}</code>
              </button>
            ))}
          </div>
          {runtime?.ready && inventory.length === 0 && !filter.trim() && (
            <div className="inventory-empty-state operational-state">
              <span className="state-symbol" aria-hidden="true">
                <Icon name="empty-inventory" />
              </span>
              <strong>{copy.noSkills}</strong>
              <p>{copy.emptyInventoryMessage}</p>
              <button type="button" onClick={() => dispatch("find-install")}>
                <Icon name="install" />
                {copy.findInstall}
              </button>
            </div>
          )}
          {runtime?.ready && inventory.length > 0 && visible.length === 0 && (
            <p className="empty-list">{copy.noMatchingSkills}</p>
          )}
        </aside>

        <section className="detail-pane" aria-label={copy.preview}>
          {!selected ? (
            <div className="choose-placeholder operational-state">
              <span className="state-symbol" aria-hidden="true">
                <Icon name="preview-placeholder" />
              </span>
              <strong>{copy.emptyPreviewTitle}</strong>
              <p>{copy.emptyPreviewMessage}</p>
            </div>
          ) : (
            <>
              <div className="detail-toolbar">
                <button
                  type="button"
                  className="mobile-back"
                  onClick={returnToInventory}
                >
                  <Icon name="chevron" />
                  {copy.backToInventory}
                </button>
                <div className="skill-header">
                  <div className="skill-identity">
                    <div className="skill-title-row">
                      <h1 title={selectedSkill?.name}>{selectedSkill?.name}</h1>
                      {selectedSkill?.source && (
                        <span
                          className="skill-source"
                          title={selectedSkill.source}
                        >
                          <span className="sr-only">{copy.skillSource}: </span>
                          {selectedSkill.source}
                        </span>
                      )}
                    </div>
                    <div className="skill-location-row">
                      <div className="path-control">
                        <button
                          ref={pathRef}
                          type="button"
                          className="path-button"
                          aria-haspopup="tree"
                          aria-expanded={transient === "tree"}
                          aria-controls="skill-file-tree"
                          aria-label={copy.openFileTree}
                          title={copy.browseFiles}
                          onClick={openFileTree}
                        >
                          <Icon name="folder" />
                        </button>
                        {transient === "tree" && (
                          <div
                            id="skill-file-tree"
                            className="file-tree"
                            onKeyDown={(event) =>
                              moveTreeFocus(
                                event,
                                () => {
                                  setTransient(null);
                                  pathRef.current?.focus();
                                },
                                setDirectoryExpanded,
                              )
                            }
                          >
                            <div className="file-tree-header">
                              <div>
                                <strong>{copy.skillFiles}</strong>
                                <span>
                                  {
                                    tree.filter((entry) => !entry.directory)
                                      .length
                                  }{" "}
                                  {copy.fileCount}
                                </span>
                              </div>
                              <code title={selectedSkill?.name}>
                                {selectedSkill?.name}
                              </code>
                            </div>
                            <div role="tree" aria-label={copy.skillFiles}>
                              {visibleTree.map((entry) =>
                                entry.directory ? (
                                  <button
                                    type="button"
                                    role="treeitem"
                                    aria-level={entry.level}
                                    aria-label={entry.path}
                                    aria-expanded={expandedDirectories.has(
                                      normalizeDirectoryPath(entry.path),
                                    )}
                                    data-directory="true"
                                    data-path={entry.path}
                                    tabIndex={
                                      treeTabPath === entry.path ? 0 : -1
                                    }
                                    className="tree-directory"
                                    style={{
                                      paddingInlineStart: `${10 + (entry.level - 1) * 18}px`,
                                    }}
                                    key={entry.path}
                                    onClick={() =>
                                      setDirectoryExpanded(
                                        entry.path,
                                        !expandedDirectories.has(
                                          normalizeDirectoryPath(entry.path),
                                        ),
                                      )
                                    }
                                    onFocus={(event) =>
                                      setTreeFocusPath(
                                        event.currentTarget.dataset.path ??
                                          null,
                                      )
                                    }
                                  >
                                    <Icon name="chevron" />
                                    <Icon name="folder" />
                                    <span>{entry.name}</span>
                                  </button>
                                ) : (
                                  <button
                                    type="button"
                                    role="treeitem"
                                    aria-level={entry.level}
                                    aria-label={entry.path}
                                    data-path={entry.path}
                                    tabIndex={
                                      treeTabPath === entry.path ? 0 : -1
                                    }
                                    key={entry.path}
                                    aria-current={
                                      file?.path === entry.path
                                        ? "true"
                                        : undefined
                                    }
                                    style={{
                                      paddingInlineStart: `${10 + (entry.level - 1) * 18}px`,
                                    }}
                                    onClick={() => chooseFile(entry)}
                                    onFocus={(event) =>
                                      setTreeFocusPath(
                                        event.currentTarget.dataset.path ??
                                          null,
                                      )
                                    }
                                  >
                                    <Icon
                                      name={
                                        entry.viewer === "image"
                                          ? "image"
                                          : "file"
                                      }
                                    />
                                    <span>{entry.name}</span>
                                  </button>
                                ),
                              )}
                            </div>
                          </div>
                        )}
                      </div>
                      <span className="skill-path" title={selectedSkill?.path}>
                        <span className="sr-only">{copy.installPath}: </span>
                        {selectedSkill?.path}
                      </span>
                    </div>
                  </div>
                  <div className="skill-command-groups">
                    <div className="preview-actions content-actions">
                      {file?.translatable && (
                        <>
                          <span className="egress">{copy.egress}</span>
                          <button
                            type="button"
                            className="translation-toggle"
                            aria-pressed={translationOn}
                            aria-label={
                              translationOn
                                ? copy.hideTranslation
                                : copy.translate
                            }
                            title={
                              translationOn
                                ? copy.hideTranslation
                                : copy.translate
                            }
                            onClick={() => dispatch("translate-skill")}
                          >
                            <Icon name="translate" />
                            <span className="command-label">
                              {translationOn
                                ? copy.hideTranslation
                                : copy.translate}
                            </span>
                          </button>
                        </>
                      )}
                      {commandButton(
                        "reveal-skill",
                        "folder-open",
                        file ? copy.revealFile : copy.revealRoot,
                        "icon-button",
                        true,
                      )}
                    </div>
                    <div className="preview-actions lifecycle-actions">
                      {commandButton(
                        "update-skill",
                        "update-skill",
                        copy.update,
                        "update-emphasis",
                      )}
                      {commandButton(
                        "remove-skill",
                        "trash",
                        copy.remove,
                        "danger quiet-danger icon-button",
                        true,
                      )}
                    </div>
                  </div>
                </div>
              </div>
              {translationOn && file?.translatable && (
                <TranslationTabs
                  copy={copy}
                  value={mobilePane}
                  onChange={setMobilePane}
                />
              )}
              {previewError ? (
                <div className="preview-error operational-state" role="alert">
                  <span className="state-symbol" aria-hidden="true">
                    <Icon name="preview-warning" />
                  </span>
                  <strong>{copy.previewError}</strong>
                  <p>{copy.previewErrorMessage}</p>
                  <div className="state-actions">
                    <button
                      type="button"
                      onClick={() =>
                        void loadPreview(selected, previewPath.current)
                      }
                    >
                      <Icon name="refresh" />
                      {copy.retry}
                    </button>
                    <button
                      type="button"
                      className="secondary"
                      onClick={() => dispatch("reveal-skill")}
                    >
                      <Icon name="folder-open" />
                      {copy.revealFile}
                    </button>
                  </div>
                  <small className="state-diagnostic">{previewError}</small>
                </div>
              ) : previewLoading ? (
                <p className="loading-copy preview-loading">
                  <span className="spinner" aria-hidden="true" />
                  {copy.loadingSkills}
                </p>
              ) : (
                <div
                  className={`viewer-grid ${translationOn && file?.translatable ? "translated" : ""}`}
                  data-pane={mobilePane}
                >
                  <Viewer
                    file={file}
                    label={copy.original}
                    unsupported={copy.unsupported}
                    id="original-panel"
                    labelledBy="original-tab"
                  />
                  {translationOn && file?.translatable && (
                    <article
                      className="viewer translation-view"
                      id="translation-panel"
                      role="tabpanel"
                      aria-labelledby="translation-tab"
                    >
                      <h2>{copy.translation}</h2>
                      {!currentTranslation ||
                      (currentTranslation.text === undefined &&
                        !currentTranslation.error) ? (
                        <p>{copy.translating}</p>
                      ) : currentTranslation.error ? (
                        <div className="translation-error">
                          <p className="error" role="alert">
                            {currentTranslation.error}
                          </p>
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
              )}
            </>
          )}
        </section>
      </div>

      <footer
        className={`status-bar status-${runtimeFailure ? "error" : (feedback?.severity ?? "ready")}`}
      >
        <div className="status-summary">
          {!runtime && !runtimeError ? (
            <span className="status-announcement" aria-live="polite">
              {copy.starting}
            </span>
          ) : runtimeFailure ? (
            <span className="status-announcement" aria-live="polite">
              {copy.runtimeUnavailableStatus}
            </span>
          ) : operation ? (
            <span className="status-announcement" aria-live="polite">
              <span className="spinner" aria-hidden="true" />
              {copy.busy}
            </span>
          ) : feedback ? (
            <>
              <span className="status-announcement" aria-live="polite">
                {feedback.summary}
              </span>
              {feedback.review && (
                <button
                  type="button"
                  className="text-button"
                  onClick={() =>
                    openModal({
                      kind: "discovery",
                      trigger: document.activeElement as HTMLElement | null,
                    })
                  }
                >
                  {copy.review}
                </button>
              )}
              {feedback.diagnostics && (
                <button
                  type="button"
                  className="text-button"
                  aria-expanded={transient === "diagnostics"}
                  onClick={() =>
                    setTransient((value) =>
                      value === "diagnostics" ? null : "diagnostics",
                    )
                  }
                >
                  {copy.details}
                </button>
              )}
            </>
          ) : (
            <span>{copy.ready}</span>
          )}
          {transient === "diagnostics" && feedback?.diagnostics && (
            <div className="diagnostics-popover">
              <pre>{feedback.diagnostics}</pre>
            </div>
          )}
        </div>
        <span className="status-facts">
          {runtimeFailure
            ? copy.actionRequired
            : `${copy.installed}: ${inventory.length} · CLI ${runtime?.version ?? "—"}`}
        </span>
      </footer>

      {modal?.kind === "settings" && (
        <SettingsDialog
          copy={copy}
          version={runtime?.version ?? null}
          preferences={preferences}
          onChange={changePreferences}
          onClose={closeModal}
          returnFocus={modal.trigger}
        />
      )}
      {modal?.kind === "discovery" && (
        <ModalShell
          labelledBy="find-install-title"
          onClose={closeModal}
          returnFocus={modal.trigger}
          initialFocus="input"
          className="discovery-modal"
        >
          <header className="discovery-header">
            <div>
              <h2 id="find-install-title">{copy.findInstall}</h2>
              <p>{copy.findInstallSubtitle}</p>
            </div>
            <button
              className="icon-button"
              type="button"
              onClick={closeModal}
              aria-label={copy.close}
            >
              <Icon name="close" />
            </button>
          </header>
          <nav className="discovery-tabs" aria-label={copy.installMethod}>
            <button
              type="button"
              aria-current={discoveryTab === "search" ? "page" : undefined}
              onClick={() => setDiscoveryTab("search")}
            >
              {copy.searchTab}
            </button>
            <button
              type="button"
              aria-current={discoveryTab === "source" ? "page" : undefined}
              onClick={() => setDiscoveryTab("source")}
            >
              {copy.fromSource}
            </button>
          </nav>
          <section className="discovery">
            {discoveryError && (
              <p className="error" role="alert">
                {discoveryError}
              </p>
            )}
            {discoveryTab === "search" ? (
              <div className="discovery-panel" id="discovery-search-panel">
                <form
                  className="catalog-search"
                  onSubmit={(event) => {
                    event.preventDefault();
                    setDiscoveryError(null);
                    setUnresolvedDiscovery(false);
                    setLastDiscoveryTarget(null);
                    setFeedback(null);
                    void searchSkills(searchQuery)
                      .then(setSearchResults)
                      .catch((value: unknown) =>
                        setDiscoveryError(commandErrorMessage(value)),
                      );
                  }}
                >
                  <Icon name="search" />
                  <input
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                    aria-label={copy.searchQuery}
                    required
                  />
                  <button type="submit">{copy.searchAction}</button>
                </form>
                <div className="search-results" aria-live="polite">
                  {searchResults.map((result) => (
                    <div className="search-result" key={result.slug}>
                      <span>
                        <strong>{result.name}</strong>
                        <small>
                          {result.source} ·{" "}
                          {result.installs.toLocaleString(locale)}{" "}
                          {copy.installs}
                        </small>
                      </span>
                      <button
                        type="button"
                        disabled={operation !== null}
                        onClick={() => {
                          setLastDiscoveryTarget({
                            source: result.source,
                            name: result.name,
                          });
                          startOperation(
                            "install",
                            () =>
                              addSkill(result.source, result.name, {
                                agents: preferences.agents,
                                copy: preferences.copy,
                              }),
                            { type: "search", name: result.name },
                          );
                        }}
                      >
                        <Icon name="download" />
                        {copy.install}
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            ) : (
              <form
                className="source-install discovery-panel"
                onSubmit={(event) => {
                  event.preventDefault();
                  setLastDiscoveryTarget({ source, name: null });
                  startOperation(
                    "install",
                    () =>
                      addSkill(source, null, {
                        agents: preferences.agents,
                        copy: preferences.copy,
                      }),
                    { type: "source" },
                  );
                }}
              >
                <label htmlFor="install-source-input">
                  {copy.installSource}
                </label>
                <input
                  id="install-source-input"
                  value={source}
                  onChange={(event) => setSource(event.target.value)}
                  required
                />
                <small>{copy.sourceHint}</small>
                <button type="submit" disabled={operation !== null}>
                  <Icon name="download" />
                  {copy.install}
                </button>
              </form>
            )}
            {unresolvedDiscovery && (
              <button
                className="discovery-retry"
                type="button"
                disabled={operation !== null || !lastDiscoveryTarget}
                onClick={() => {
                  if (!lastDiscoveryTarget) return;
                  startOperation(
                    "install",
                    () =>
                      addSkill(
                        lastDiscoveryTarget.source,
                        lastDiscoveryTarget.name,
                        {
                          agents: preferences.agents,
                          copy: preferences.copy,
                        },
                      ),
                    lastDiscoveryTarget.name
                      ? { type: "search", name: lastDiscoveryTarget.name }
                      : { type: "source" },
                  );
                }}
              >
                {copy.retry}
              </button>
            )}
          </section>
          <footer className="discovery-footer">
            <small>{copy.pinnedCliTrust}</small>
            {operation === "install" ? (
              <small>{copy.commandContinues}</small>
            ) : discoveryTab === "search" ? (
              <button type="button" onClick={() => setDiscoveryTab("source")}>
                {copy.installFromSourceAction}
              </button>
            ) : null}
          </footer>
        </ModalShell>
      )}
      {modal?.kind === "remove" && (
        <ModalShell
          labelledBy="remove-title"
          onClose={closeModal}
          returnFocus={modal.confirmed ? null : modal.trigger}
          fallbackFocus="#installed-heading"
          initialFocus=".cancel-button"
          className="confirmation-modal"
        >
          <header className="remove-dialog-header">
            <h2 id="remove-title">
              {copy.removeTitle.replace("{name}", modal.name)}
            </h2>
          </header>
          <div className="remove-dialog-content">
            <p>{copy.removeExplanation}</p>
            <code
              title={inventory.find((skill) => skill.name === modal.name)?.path}
            >
              {inventory.find((skill) => skill.name === modal.name)?.path}
            </code>
            {operation === "remove" && <small>{copy.commandContinues}</small>}
          </div>
          <footer className="modal-actions">
            <button
              type="button"
              className="cancel-button"
              onClick={closeModal}
            >
              {copy.cancel}
            </button>
            <button
              type="button"
              className="danger"
              disabled={operation !== null}
              onClick={() => {
                setModal((current) =>
                  current?.kind === "remove"
                    ? { ...current, confirmed: true }
                    : current,
                );
                startOperation("remove", () => removeSkill(modal.name));
              }}
            >
              {copy.confirm}
            </button>
          </footer>
        </ModalShell>
      )}
    </main>
  );

  function chooseFile(entry: FileEntry) {
    if (!selected || entry.directory) return;
    const request = ++previewRequest.current;
    previewPath.current = entry.path;
    hideTranslation();
    setTransient(null);
    requestAnimationFrame(() => pathRef.current?.focus());
    setPreviewError(null);
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
      setPreviewLoading(false);
      return;
    }
    setFile(null);
    setPreviewLoading(true);
    void readPreview(selected, entry.path)
      .then((content) => {
        if (previewRequest.current === request) setFile(content);
      })
      .catch((value: unknown) => {
        if (previewRequest.current === request)
          setPreviewError(commandErrorMessage(value));
      })
      .finally(() => {
        if (previewRequest.current === request) setPreviewLoading(false);
      });
  }
}

function moveListFocus(event: ReactKeyboardEvent<HTMLButtonElement>) {
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

function moveTreeFocus(
  event: ReactKeyboardEvent<HTMLDivElement>,
  close: () => void,
  setDirectoryExpanded: (path: string, expanded: boolean) => void,
) {
  if (event.key === "Escape") {
    event.preventDefault();
    close();
    return;
  }
  const items = Array.from(
    event.currentTarget.querySelectorAll<HTMLButtonElement>(
      'button[role="treeitem"]',
    ),
  );
  const current = items.indexOf(document.activeElement as HTMLButtonElement);
  if (current < 0) return;
  let next = current;
  if (event.key === "ArrowDown") next = Math.min(items.length - 1, current + 1);
  else if (event.key === "ArrowUp") next = Math.max(0, current - 1);
  else if (event.key === "Home") next = 0;
  else if (event.key === "End") next = items.length - 1;
  else if (event.key === "ArrowRight") {
    const item = items[current];
    if (item.dataset.directory !== "true") return;
    if (item.getAttribute("aria-expanded") === "false") {
      event.preventDefault();
      setDirectoryExpanded(item.dataset.path ?? "", true);
      return;
    }
    const level = Number(item.getAttribute("aria-level"));
    const child = items[current + 1];
    if (child && Number(child.getAttribute("aria-level")) > level) {
      event.preventDefault();
      child.focus();
    }
    return;
  } else if (event.key === "ArrowLeft") {
    const item = items[current];
    if (
      item.dataset.directory === "true" &&
      item.getAttribute("aria-expanded") === "true"
    ) {
      event.preventDefault();
      setDirectoryExpanded(item.dataset.path ?? "", false);
      return;
    }
    const level = Number(item.getAttribute("aria-level"));
    for (let index = current - 1; index >= 0; index -= 1) {
      if (Number(items[index].getAttribute("aria-level")) < level) {
        event.preventDefault();
        items[index].focus();
        return;
      }
    }
    return;
  } else return;
  event.preventDefault();
  items[next]?.focus();
}

function normalizeDirectoryPath(path: string) {
  return path.replace(/\/+$/, "");
}

function directoryAncestors(tree: FileEntry[], path: string) {
  return tree
    .filter(
      (entry) =>
        entry.directory &&
        path.startsWith(`${normalizeDirectoryPath(entry.path)}/`),
    )
    .map((entry) => normalizeDirectoryPath(entry.path));
}

function visibleTreeEntries(
  tree: FileEntry[],
  expandedDirectories: ReadonlySet<string>,
) {
  return tree.filter((entry) =>
    directoryAncestors(tree, entry.path).every((path) =>
      expandedDirectories.has(path),
    ),
  );
}

function TranslationTabs({
  copy,
  value,
  onChange,
}: {
  copy: Messages;
  value: "original" | "translation";
  onChange: (value: "original" | "translation") => void;
}) {
  const keydown = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const next = value === "original" ? "translation" : "original";
    onChange(next);
    document.getElementById(`${next}-tab`)?.focus();
  };
  return (
    <div className="mobile-tabs" role="tablist">
      <button
        id="original-tab"
        role="tab"
        aria-selected={value === "original"}
        aria-controls="original-panel"
        tabIndex={value === "original" ? 0 : -1}
        onKeyDown={keydown}
        onClick={() => onChange("original")}
      >
        {copy.original}
      </button>
      <button
        id="translation-tab"
        role="tab"
        aria-selected={value === "translation"}
        aria-controls="translation-panel"
        tabIndex={value === "translation" ? 0 : -1}
        onKeyDown={keydown}
        onClick={() => onChange("translation")}
      >
        {copy.translation}
      </button>
    </div>
  );
}

function Viewer({
  file,
  label,
  unsupported,
  id,
  labelledBy,
}: {
  file: FileContent | null;
  label: string;
  unsupported: string;
  id: string;
  labelledBy: string;
}) {
  if (!file)
    return (
      <article className="viewer">
        <p>{unsupported}</p>
      </article>
    );
  return (
    <article
      className="viewer original-view"
      id={id}
      role="tabpanel"
      aria-labelledby={labelledBy}
    >
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
