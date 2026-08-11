import { useCallback, useEffect, useState } from "react";
import {
  cancelStaging,
  commandErrorCode,
  commandErrorMessage,
  commitConfiguration,
  inventoryDiagnosticMessage,
  loadInventory,
  loadStateStatus,
  planConfiguration,
  resolveConfiguration,
  type AttentionEntry,
  type ExternalInstallation,
  type Inventory,
  type ManagedInstallationAction,
  type ManagedInstallationReconciliation,
  type ManagedPackageAction,
  type ManagedSkillPackage,
  type StateStatus,
} from "./api";
import AdoptionDialog from "./AdoptionDialog";
import GitUpdateDialog from "./GitUpdateDialog";
import ImportDialog from "./ImportDialog";
import InstallDialog from "./InstallDialog";
import LifecycleDialog, { type LifecycleAction } from "./LifecycleDialog";
import RevisionDialog, { type RevisionAction } from "./RevisionDialog";
import { catalogs, preferredLocale, type Locale, type Messages } from "./i18n";
import SettingsDialog from "./SettingsDialog";

function inventoryEntryName(logicalPath: string, skillName?: string) {
  return (
    skillName ??
    logicalPath.split(/[\\/]/).filter(Boolean).at(-1) ??
    logicalPath
  );
}

function attentionLabel(entry: AttentionEntry, copy: Messages) {
  switch (entry.kind) {
    case "broken_external_installation":
      return copy.brokenExternalInstallation;
    case "invalid_installation_candidate":
      return copy.invalidInstallationCandidate;
    case "unexpected_agent_root_entry":
      return copy.unexpectedAgentRootEntry;
  }
}

function managedStatusLabel(
  status: ManagedInstallationReconciliation["status"],
  copy: Messages,
) {
  switch (status) {
    case "healthy":
    case "missing":
    case "retargeted":
    case "drifted":
    case "configuration_drift":
    case "broken":
      return copy[`status_${status}`];
    default:
      return unreachable(status);
  }
}

function shortFingerprint(value: string) {
  return value.slice(0, 12);
}

function unreachable(value: never): never {
  throw new Error(`Unexpected backend enum value: ${String(value)}`);
}

export default function App() {
  const [locale, setLocale] = useState<Locale>(() =>
    preferredLocale(
      localStorage.getItem("skill-deck-locale"),
      navigator.language,
    ),
  );
  const copy = catalogs[locale];
  const [inventory, setInventory] = useState<Inventory | null>(null);
  const [stateStatus, setStateStatus] = useState<StateStatus | null>(null);
  const [inventoryError, setInventoryError] = useState<{
    value: unknown;
  } | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [configurationBusy, setConfigurationBusy] = useState<string | null>(
    null,
  );
  const [configurationDrift, setConfigurationDrift] = useState<{
    packageId: string;
    agent: "codex" | "claude";
  } | null>(null);
  const [installSkill, setInstallSkill] = useState<ManagedSkillPackage | null>(
    null,
  );
  const [lifecycleAction, setLifecycleAction] =
    useState<LifecycleAction | null>(null);
  const [revisionAction, setRevisionAction] = useState<RevisionAction | null>(
    null,
  );
  const [adoptionEntry, setAdoptionEntry] =
    useState<ExternalInstallation | null>(null);
  const [gitUpdateSkill, setGitUpdateSkill] =
    useState<ManagedSkillPackage | null>(null);
  const [query, setQuery] = useState("");
  const [agentFilter, setAgentFilter] = useState<"all" | "codex" | "claude">(
    "all",
  );
  const [enabledFilter, setEnabledFilter] = useState<
    "all" | "enabled" | "disabled"
  >("all");
  const [ownershipFilter, setOwnershipFilter] = useState<
    "all" | "managed" | "external"
  >("all");

  const refresh = useCallback(() => {
    void Promise.all([loadInventory(), loadStateStatus()])
      .then(([nextInventory, nextStateStatus]) => {
        setInventory(nextInventory);
        setStateStatus(nextStateStatus);
        setInventoryError(null);
      })
      .catch((error: unknown) => {
        setInventoryError({ value: error });
      });
  }, []);

  useEffect(refresh, [refresh]);

  const installations = inventory?.externalInstallations ?? [];
  const attentionEntries = inventory?.attentionEntries ?? [];
  const listedAttentionEntries = attentionEntries.filter(
    (entry) => entry.kind !== "unexpected_agent_root_entry",
  );
  const managedPackages = inventory?.managedPackages ?? [];
  const managedStatuses = new Map(
    (inventory?.managedInstallationStatuses ?? []).map((entry) => [
      `${entry.packageId}:${entry.agent}`,
      entry,
    ]),
  );
  const packageReconciliations = new Map(
    (inventory?.managedPackageReconciliations ?? []).map((entry) => [
      entry.packageId,
      entry,
    ]),
  );
  const readOnly = stateStatus?.mode === "read_only_recovery";
  const normalizedQuery = query.trim().toLowerCase();
  const visibleManaged = managedPackages.filter(
    (skill) =>
      ownershipFilter !== "external" &&
      (agentFilter === "all" ||
        skill.installations.some(
          (installation) => installation.agent === agentFilter,
        )) &&
      (enabledFilter === "all" ||
        skill.installations.some(
          (installation) =>
            installation.enabled === (enabledFilter === "enabled"),
        )) &&
      (!normalizedQuery || skill.name.toLowerCase().includes(normalizedQuery)),
  );
  const visibleExternal = installations.filter(
    (installation) =>
      ownershipFilter !== "managed" &&
      enabledFilter === "all" &&
      (agentFilter === "all" || installation.agent === agentFilter) &&
      (!normalizedQuery ||
        installation.skill.metadata.name
          .toLowerCase()
          .includes(normalizedQuery) ||
        installation.skill.metadata.description
          .toLowerCase()
          .includes(normalizedQuery) ||
        installation.logicalPath.toLowerCase().includes(normalizedQuery)),
  );
  const visibleAttention = listedAttentionEntries.filter(
    (entry) =>
      ownershipFilter !== "managed" &&
      enabledFilter === "all" &&
      (agentFilter === "all" || entry.agent === agentFilter) &&
      (!normalizedQuery ||
        entry.logicalPath.toLowerCase().includes(normalizedQuery)),
  );

  function toggleConfiguration(
    packageId: string,
    agent: "codex" | "claude",
    enabled: boolean,
  ) {
    const key = `${packageId}:${agent}`;
    setConfigurationBusy(key);
    setActionError(null);
    setConfigurationDrift(null);
    void planConfiguration(packageId, agent, enabled)
      .then((plan) => commitConfiguration(plan.id))
      .then(() => {
        setNotice(copy.restartFallback);
        refresh();
      })
      .catch((error: unknown) => {
        if (commandErrorCode(error) === "configuration_drift") {
          setConfigurationDrift({ packageId, agent });
          setActionError(copy.configurationDrift);
        } else {
          setActionError(
            commandErrorMessage(error, copy.errors) ?? copy.unknownError,
          );
        }
      })
      .finally(() => setConfigurationBusy(null));
  }

  function resolveDrift(resolution: "reapply" | "forget") {
    if (!configurationDrift) return;
    setConfigurationBusy(
      `${configurationDrift.packageId}:${configurationDrift.agent}`,
    );
    void resolveConfiguration(
      configurationDrift.packageId,
      configurationDrift.agent,
      resolution,
    )
      .then(() => {
        setNotice(copy.restartFallback);
        setActionError(null);
        setConfigurationDrift(null);
        refresh();
      })
      .catch((error: unknown) => {
        setActionError(
          commandErrorMessage(error, copy.errors) ?? copy.unknownError,
        );
      })
      .finally(() => setConfigurationBusy(null));
  }

  function resolveInventoryDrift(
    packageId: string,
    agent: "codex" | "claude",
    resolution: "reapply" | "forget",
  ) {
    setConfigurationBusy(`${packageId}:${agent}`);
    setActionError(null);
    void resolveConfiguration(packageId, agent, resolution)
      .then(() => {
        setNotice(copy.restartFallback);
        refresh();
      })
      .catch((error: unknown) =>
        setActionError(
          commandErrorMessage(error, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setConfigurationBusy(null));
  }

  function installationActionButton(
    skill: ManagedSkillPackage,
    installation: ManagedSkillPackage["installations"][number],
    action: ManagedInstallationAction,
  ) {
    const busy = configurationBusy === `${skill.id}:${installation.agent}`;
    switch (action) {
      case "enable_configuration":
      case "disable_configuration":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly || busy}
            onClick={() =>
              toggleConfiguration(
                skill.id,
                installation.agent,
                action === "enable_configuration",
              )
            }
          >
            {busy
              ? copy.saving
              : action === "enable_configuration"
                ? copy.enable
                : copy.disable}
          </button>
        );
      case "reapply_configuration":
      case "forget_configuration":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly || busy}
            onClick={() =>
              resolveInventoryDrift(
                skill.id,
                installation.agent,
                action === "reapply_configuration" ? "reapply" : "forget",
              )
            }
          >
            {action === "reapply_configuration"
              ? copy.reapplyConfiguration
              : copy.forgetConfiguration}
          </button>
        );
      case "restore":
        return (
          <button
            className="text-button destructive-text"
            type="button"
            disabled={readOnly}
            onClick={() =>
              setRevisionAction({
                mode: "restore",
                skill,
                agent: installation.agent,
              })
            }
          >
            {copy.restoreAction}
          </button>
        );
      case "detach":
      case "uninstall":
      case "forget_installation": {
        const mode = action === "forget_installation" ? "forget" : action;
        return (
          <button
            className={`text-button ${action === "detach" ? "" : "destructive-text"}`}
            type="button"
            disabled={readOnly}
            onClick={() =>
              setLifecycleAction({ mode, skill, agent: installation.agent })
            }
          >
            {action === "detach"
              ? copy.detachAction
              : action === "uninstall"
                ? copy.uninstallAction
                : copy.forgetInstallationAction}
          </button>
        );
      }
      default:
        return unreachable(action);
    }
  }

  function packageActionButton(
    skill: ManagedSkillPackage,
    action: ManagedPackageAction,
  ) {
    switch (action) {
      case "install":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly}
            onClick={() => setInstallSkill(skill)}
          >
            {copy.installAction}
          </button>
        );
      case "replace":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly}
            onClick={() => setRevisionAction({ mode: "replace", skill })}
          >
            {copy.replaceAction}
          </button>
        );
      case "check_update":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly}
            onClick={() => setGitUpdateSkill(skill)}
          >
            {copy.checkUpdateAction}
          </button>
        );
      case "rollback":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly}
            onClick={() => setRevisionAction({ mode: "rollback", skill })}
          >
            {copy.rollbackAction}
          </button>
        );
      case "export":
        return (
          <button
            className="text-button"
            type="button"
            disabled={readOnly}
            onClick={() => setRevisionAction({ mode: "export", skill })}
          >
            {copy.exportAction}
          </button>
        );
      case "remove":
        return (
          <button
            className="text-button destructive-text"
            type="button"
            disabled={readOnly}
            onClick={() => setLifecycleAction({ mode: "remove", skill })}
          >
            {copy.removeLibraryAction}
          </button>
        );
      default:
        return unreachable(action);
    }
  }

  function closeStagedDialog(close: () => void) {
    close();
    void cancelStaging().catch((error: unknown) => {
      setActionError(
        commandErrorMessage(error, copy.errors) ?? copy.unknownError,
      );
    });
  }

  return (
    <main className="shell" id="top">
      <nav className="nav" aria-label={copy.primaryNavigation}>
        <a className="brand" href="#top" aria-label={copy.home}>
          <span className="brand-mark" aria-hidden="true">
            S
          </span>
          Skill Deck
        </a>
        <div className="nav-actions">
          <span className="offline-dot">{copy.offline}</span>
          <label className="locale-control">
            <span className="sr-only">{copy.language}</span>
            <select
              value={locale}
              onChange={(event) => {
                const next = event.target.value as Locale;
                localStorage.setItem("skill-deck-locale", next);
                setLocale(next);
              }}
            >
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </label>
        </div>
      </nav>

      <section className="library" aria-labelledby="library-title">
        <div className="library-heading">
          <div>
            <h2 id="library-title">{copy.skills}</h2>
            <p className="inventory-summary">
              {managedPackages.length} {copy.managedLibrary} ·{" "}
              {installations.length} {copy.discovered} ·{" "}
              {attentionEntries.length} {copy.needsAttention}
            </p>
          </div>
          <div className="library-actions">
            <button
              className="secondary"
              type="button"
              disabled={!inventory || !stateStatus}
              onClick={() => setSettingsOpen(true)}
            >
              {copy.settingsAction}
            </button>
            <button
              className="primary"
              type="button"
              disabled={readOnly}
              onClick={() => setImportOpen(true)}
            >
              {copy.importAction}
            </button>
          </div>
        </div>

        {readOnly && <p className="recovery-banner">{copy.readOnlyRecovery}</p>}
        {notice && <p className="success-banner">{notice}</p>}
        {actionError && (
          <div className="recovery-banner" role="alert">
            <p>{actionError}</p>
            {configurationDrift && (
              <div className="library-actions">
                <button
                  className="secondary"
                  type="button"
                  disabled={configurationBusy !== null}
                  onClick={() => resolveDrift("reapply")}
                >
                  {copy.reapplyConfiguration}
                </button>
                <button
                  className="secondary"
                  type="button"
                  disabled={configurationBusy !== null}
                  onClick={() => resolveDrift("forget")}
                >
                  {copy.forgetConfiguration}
                </button>
              </div>
            )}
          </div>
        )}

        {(managedPackages.length > 0 ||
          installations.length > 0 ||
          listedAttentionEntries.length > 0) && (
          <div className="inventory-filters" aria-label={copy.filters}>
            <label className="field search-field">
              <span>{copy.search}</span>
              <input
                type="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={copy.searchPlaceholder}
              />
            </label>
            <label className="filter-field">
              <span>{copy.agentFilter}</span>
              <select
                value={agentFilter}
                onChange={(event) =>
                  setAgentFilter(
                    event.target.value as "all" | "codex" | "claude",
                  )
                }
              >
                <option value="all">{copy.all}</option>
                <option value="codex">Codex</option>
                <option value="claude">Claude Code</option>
              </select>
            </label>
            <label className="filter-field">
              <span>{copy.enabledFilter}</span>
              <select
                value={enabledFilter}
                onChange={(event) =>
                  setEnabledFilter(
                    event.target.value as "all" | "enabled" | "disabled",
                  )
                }
              >
                <option value="all">{copy.all}</option>
                <option value="enabled">{copy.enabled}</option>
                <option value="disabled">{copy.disabled}</option>
              </select>
            </label>
            <label className="filter-field">
              <span>{copy.managementScope}</span>
              <select
                value={ownershipFilter}
                onChange={(event) =>
                  setOwnershipFilter(
                    event.target.value as "all" | "managed" | "external",
                  )
                }
              >
                <option value="all">{copy.all}</option>
                <option value="managed">{copy.managed}</option>
                <option value="external">{copy.outsideLibrary}</option>
              </select>
            </label>
            {enabledFilter !== "all" && (
              <p className="filter-scope-note">{copy.enabledManagedOnly}</p>
            )}
          </div>
        )}

        {(!inventory || !stateStatus) && !inventoryError ? (
          <div className="loading-state" aria-live="polite">
            {copy.loading}
          </div>
        ) : inventoryError ? (
          <div className="error-state" role="alert">
            <p>
              {commandErrorMessage(inventoryError.value, copy.errors) ??
                copy.unknownError}
            </p>
            <button className="secondary" type="button" onClick={refresh}>
              {copy.retry}
            </button>
          </div>
        ) : visibleManaged.length > 0 ||
          visibleExternal.length > 0 ||
          visibleAttention.length > 0 ? (
          <ul className="skill-list">
            {visibleManaged.map((skill) => {
              const packageReconciliation = packageReconciliations.get(
                skill.id,
              );
              const reconciliations = skill.installations
                .map((installation) =>
                  managedStatuses.get(`${skill.id}:${installation.agent}`),
                )
                .filter(
                  (entry): entry is ManagedInstallationReconciliation =>
                    entry !== undefined,
                );
              const hasAbnormal = reconciliations.some(
                (entry) => entry.status !== "healthy",
              );
              return (
                <li className="skill-row" key={skill.id}>
                  <div>
                    <h3>{skill.name}</h3>
                    <p>
                      {skill.installations.length > 0
                        ? `${copy.installedTo}: ${skill.installations
                            .map((installation) => installation.agent)
                            .join(", ")}`
                        : copy.libraryOnly}
                    </p>
                    {packageReconciliation?.libraryDiagnostic && (
                      <p className="skill-description">
                        {inventoryDiagnosticMessage(
                          packageReconciliation.libraryDiagnostic,
                          skill.libraryPath,
                          copy.errors,
                        )}
                      </p>
                    )}
                    {hasAbnormal && (
                      <p className="filter-scope-note">
                        {copy.packageActionsFrozen}
                      </p>
                    )}
                  </div>
                  <div className="skill-meta">
                    <span>{skill.source.type.replace("_", " ")}</span>
                    <span>{copy.managed}</span>
                    {skill.installations.map((installation) => {
                      const reconciliation = managedStatuses.get(
                        `${skill.id}:${installation.agent}`,
                      );
                      if (!reconciliation) return null;
                      return (
                        <div
                          className="installation-actions"
                          key={installation.agent}
                        >
                          <div className="skill-title-line">
                            <span>{installation.agent}</span>
                            <span className="status-badge">
                              {managedStatusLabel(reconciliation.status, copy)}
                            </span>
                          </div>
                          <p className="installation-path">
                            {reconciliation.evidence.logicalPath}
                          </p>
                          {reconciliation.diagnostic &&
                            !packageReconciliation?.libraryDiagnostic && (
                              <p className="skill-description">
                                {inventoryDiagnosticMessage(
                                  reconciliation.diagnostic,
                                  reconciliation.evidence.logicalPath,
                                  copy.errors,
                                )}
                              </p>
                            )}
                          {reconciliation.evidence.expectedTarget && (
                            <p className="installation-path">
                              {copy.expectedTarget}:{" "}
                              {reconciliation.evidence.expectedTarget}
                            </p>
                          )}
                          {reconciliation.evidence.observedTarget && (
                            <p className="installation-path">
                              {copy.observedTarget}:{" "}
                              {reconciliation.evidence.observedTarget}
                            </p>
                          )}
                          {reconciliation.evidence.observedFingerprint && (
                            <p className="installation-path">
                              {copy.expectedFingerprint}:{" "}
                              {shortFingerprint(
                                reconciliation.evidence.libraryFingerprint,
                              )}{" "}
                              · {copy.observedFingerprint}:{" "}
                              {shortFingerprint(
                                reconciliation.evidence.observedFingerprint,
                              )}
                            </p>
                          )}
                          {reconciliation.evidence.configurationProvenance
                            .owner === "external" && (
                            <p>{copy.externallyControlledConfiguration}</p>
                          )}
                          {reconciliation.evidence.deferredChecks && (
                            <p>{copy.deferredChecks}</p>
                          )}
                          <span>
                            {reconciliation.availableActions.map((action) => (
                              <span key={action}>
                                {installationActionButton(
                                  skill,
                                  installation,
                                  action,
                                )}
                              </span>
                            ))}
                          </span>
                        </div>
                      );
                    })}
                    {packageReconciliation?.availableActions.map((action) => (
                      <span key={action}>
                        {packageActionButton(skill, action)}
                      </span>
                    ))}
                  </div>
                </li>
              );
            })}
            {visibleExternal.map((installation) => (
              <li
                className="skill-row"
                key={`${installation.agent}:${installation.logicalPath}`}
              >
                <div>
                  <div className="skill-title-line">
                    <h3>
                      {inventoryEntryName(
                        installation.logicalPath,
                        installation.skill.metadata.name,
                      )}
                    </h3>
                  </div>
                  <p className="skill-description">
                    {installation.skill.metadata.description}
                  </p>
                  <p className="installation-path">
                    {installation.logicalPath}
                  </p>
                </div>
                <div className="skill-meta">
                  <span>{installation.agent}</span>
                  <span>{copy.external}</span>
                  <button
                    className="text-button"
                    type="button"
                    disabled={readOnly}
                    onClick={() => setAdoptionEntry(installation)}
                  >
                    {installation.kind.startsWith("legacy_")
                      ? copy.migrateAction
                      : copy.adoptAction}
                  </button>
                </div>
              </li>
            ))}
            {visibleAttention.map((entry) => (
              <li
                className="skill-row"
                key={`${entry.agent}:${entry.logicalPath}`}
              >
                <div>
                  <div className="skill-title-line">
                    <h3>{inventoryEntryName(entry.logicalPath)}</h3>
                    <span className="status-badge">{copy.invalid}</span>
                  </div>
                  <p className="skill-description">
                    {inventoryDiagnosticMessage(
                      entry.diagnostic,
                      entry.logicalPath,
                      copy.errors,
                    )}
                  </p>
                  <p className="installation-path">{entry.logicalPath}</p>
                </div>
                <div className="skill-meta">
                  <span>{entry.agent}</span>
                  <span>{attentionLabel(entry, copy)}</span>
                </div>
              </li>
            ))}
          </ul>
        ) : managedPackages.length > 0 ||
          installations.length > 0 ||
          listedAttentionEntries.length > 0 ? (
          <div className="empty-state compact-empty">
            <h3>{copy.noResults}</h3>
          </div>
        ) : (
          <div className="empty-state">
            <div className="empty-glyph" aria-hidden="true">
              <span />
              <span />
              <span />
            </div>
            <h3>{copy.emptyTitle}</h3>
            <p>{copy.emptyBody}</p>
            <button
              className="primary"
              type="button"
              onClick={() => setImportOpen(true)}
            >
              {copy.importAction}
            </button>
          </div>
        )}
      </section>

      {importOpen && (
        <ImportDialog
          copy={copy}
          onClose={() => closeStagedDialog(() => setImportOpen(false))}
          onCommitted={refresh}
        />
      )}
      {settingsOpen && inventory && stateStatus && (
        <SettingsDialog
          copy={copy}
          inventory={inventory}
          stateStatus={stateStatus}
          onClose={() => setSettingsOpen(false)}
        />
      )}
      {installSkill && inventory && (
        <InstallDialog
          copy={copy}
          inventory={inventory}
          skill={installSkill}
          onClose={() => setInstallSkill(null)}
          onCommitted={(message) => {
            setNotice(message);
            refresh();
          }}
        />
      )}
      {lifecycleAction && (
        <LifecycleDialog
          action={lifecycleAction}
          copy={copy}
          onClose={() => setLifecycleAction(null)}
          onCommitted={(message) => {
            setNotice(message);
            refresh();
          }}
        />
      )}
      {revisionAction && (
        <RevisionDialog
          action={revisionAction}
          copy={copy}
          onClose={() => closeStagedDialog(() => setRevisionAction(null))}
          onCommitted={(message) => {
            setNotice(message);
            refresh();
          }}
        />
      )}
      {adoptionEntry && (
        <AdoptionDialog
          copy={copy}
          entry={adoptionEntry}
          candidates={installations.filter(
            (candidate) =>
              candidate.skill.metadata.name ===
                adoptionEntry.skill.metadata.name &&
              candidate.skill.fingerprint === adoptionEntry.skill.fingerprint &&
              (candidate.kind === "directory" || candidate.kind === "link"),
          )}
          onClose={() => closeStagedDialog(() => setAdoptionEntry(null))}
          onCommitted={(message) => {
            setNotice(message);
            refresh();
          }}
        />
      )}
      {gitUpdateSkill && (
        <GitUpdateDialog
          copy={copy}
          skill={gitUpdateSkill}
          onClose={() => closeStagedDialog(() => setGitUpdateSkill(null))}
          onCommitted={(message) => {
            setNotice(message);
            refresh();
          }}
        />
      )}
    </main>
  );
}
