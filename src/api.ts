import { invoke } from "@tauri-apps/api/core";

export type Agent = "codex" | "claude";
export type InstallationKind =
  | "directory"
  | "link"
  | "legacy_directory"
  | "legacy_link"
  | "broken_link"
  | "invalid";

export interface ValidatedSkill {
  root: string;
  fingerprint: string;
  metadata: {
    name: string;
    description: string;
    unknownFields: Record<string, unknown>;
  };
  resources: {
    packageBytes: number;
    fileCount: number;
    largestFileBytes: number;
    skillMarkdownBytes: number;
  };
  scripts: string[];
  references: string[];
}

export interface ExternalInstallation {
  agent: Agent;
  logicalPath: string;
  resolvedTarget: string | null;
  kind: InstallationKind;
  skill: ValidatedSkill | null;
  diagnostic: {
    code: string;
    message: string;
    path: string | null;
  } | null;
}

export interface Inventory {
  targets: Array<{
    agent: Agent;
    root: string;
    exists: boolean;
    legacy: boolean;
  }>;
  externalInstallations: ExternalInstallation[];
  managedPackages: ManagedSkillPackage[];
}

export interface ManagedSkillPackage {
  id: string;
  name: string;
  libraryPath: string;
  source:
    | { type: "local_snapshot" }
    | {
        type: "git";
        repositoryUrl: string;
        subpath: string;
        trackedBranch: string;
      };
  installedRevision: { fingerprint: string; commitOid: string | null };
  previousRevision: { fingerprint: string; commitOid: string | null } | null;
  installations: Array<{
    agent: Agent;
    logicalPath: string;
    resolvedTarget: string;
    deploymentMode: "symlink" | "junction" | "copy_fallback";
    enabled: boolean;
    lastKnownFingerprint: string;
    configurationProvenance:
      | { owner: "skill_deck"; path: string }
      | { owner: "external"; path: string }
      | { owner: "none" };
  }>;
}

export interface StateStatus {
  mode: "active" | "recovered_backup" | "read_only_recovery";
  state: { packages: ManagedSkillPackage[] } | null;
  diagnostic: string | null;
}

export interface AddToLibraryPlan {
  id: string;
  skill: ValidatedSkill;
  libraryPath: string;
}

export interface InstallPlan {
  id: string;
  packageId: string;
  targets: Array<{
    agent: Agent;
    logicalPath: string;
    rootExists: boolean;
    preferredMode: "symlink" | "junction";
  }>;
}

export interface InstallResult {
  package: ManagedSkillPackage;
  restartMessage: string;
}

export interface ConfigurationPlan {
  id: string;
  packageId: string;
  agent: Agent;
  enabled: boolean;
  currentEnabled: boolean;
  configPath: string;
}

export interface UninstallPlan {
  id: string;
  packageId: string;
  agent: Agent;
  logicalPath: string;
  deploymentMode: "symlink" | "junction" | "copy_fallback";
  cleansOwnedConfiguration: boolean;
}

export interface DetachPlan {
  id: string;
  packageId: string;
  agent: Agent;
  logicalPath: string;
  deploymentMode: "symlink" | "junction" | "copy_fallback";
  keepsConfiguration: boolean;
}

export interface RemoveLibraryPlan {
  id: string;
  packageId: string;
  name: string;
  source: ManagedSkillPackage["source"];
  currentRevision: ManagedSkillPackage["installedRevision"];
  previousRevision: ManagedSkillPackage["previousRevision"];
  libraryPath: string;
  bytes: number;
  localSnapshotLastCopyWarning: boolean;
  exportCurrentPath: string;
}

export interface LifecycleResult {
  package: ManagedSkillPackage | null;
  restartMessage: string;
}

export interface ChangeDisclosure {
  scripts: { added: string[]; removed: string[] };
  references: { added: string[]; removed: string[] };
  unknownFields: { added: string[]; removed: string[] };
}

export interface ReplaceRevisionPlan {
  id: string;
  packageId: string;
  sourcePath: string;
  candidate: ValidatedSkill;
  changes: ChangeDisclosure;
  installationCount: number;
}

export interface RollbackRevisionPlan {
  id: string;
  packageId: string;
  fromRevision: ManagedSkillPackage["installedRevision"];
  toRevision: ManagedSkillPackage["installedRevision"];
  changes: ChangeDisclosure;
  installationCount: number;
}

export interface ExportRevisionPlan {
  id: string;
  packageId: string;
  destination: string;
  revision: ManagedSkillPackage["installedRevision"];
}

export interface ExportRevisionResult {
  destination: string;
  fingerprint: string;
}

export interface RestoreInstallationPlan {
  id: string;
  packageId: string;
  agent: Agent;
  logicalPath: string;
  expectedFingerprint: string;
  observedFingerprint: string;
  willOverwrite: true;
}

export interface ExternalInstallationIdentity {
  agent: Agent;
  logicalPath: string;
}

export interface AdoptionPlan {
  id: string;
  name: string;
  fingerprint: string;
  libraryPath: string;
  installations: Array<{
    agent: Agent;
    logicalPath: string;
    resolvedTarget: string;
    kind: "directory" | "link";
    preferredMode: "symlink" | "junction";
    enabled: boolean;
    configurationProvenance: ManagedSkillPackage["installations"][number]["configurationProvenance"];
  }>;
}

export interface LegacyMigrationPlan {
  id: string;
  name: string;
  fingerprint: string;
  libraryPath: string;
  legacyPath: string;
  resolvedTarget: string;
}

export interface GitImportPlan {
  id: string;
  repositoryUrl: string;
  subpath: string;
  trackedBranch: string;
  commitOid: string;
  skill: ValidatedSkill;
  libraryPath: string;
}

export type GitUpdateStatus =
  | "equal"
  | "fast_forward"
  | "diverged"
  | "source_unreachable"
  | "source_missing";

export interface GitUpdatePlan {
  id: string;
  packageId: string;
  fromCommitOid: string;
  toCommitOid: string;
  candidate: ValidatedSkill;
  changes: ChangeDisclosure;
  installationCount: number;
}

export interface GitUpdateCheck {
  status: GitUpdateStatus;
  packageId: string;
  installedCommitOid: string;
  remoteCommitOid?: string;
  plan?: GitUpdatePlan;
}

export interface DiagnosticsReport {
  stateMode: StateStatus["mode"];
  targets: Array<{
    agent: Agent;
    root: string;
    exists: boolean;
    legacy: boolean;
    externalInstallationCount: number;
  }>;
  managedPackageCount: number;
  externalInstallationCount: number;
  orphanedPackagePaths: string[];
  destination: string;
  omitted: string[];
  recoveryScope: string;
}

export interface DiagnosticsExportPlan {
  id: string;
  report: DiagnosticsReport;
}

export function loadInventory(): Promise<Inventory> {
  return invoke<Inventory>("inventory");
}

export function loadStateStatus(): Promise<StateStatus> {
  return invoke<StateStatus>("state_status");
}

export function cancelStaging(): Promise<void> {
  return invoke<void>("cancel_staging");
}

export function planAddLocalSkill(path: string): Promise<AddToLibraryPlan> {
  return invoke<AddToLibraryPlan>("plan_add_local_skill", { path });
}

export function commitAddLocalSkill(
  planId: string,
): Promise<ManagedSkillPackage> {
  return invoke<ManagedSkillPackage>("commit_add_local_skill", { planId });
}

export function planInstall(
  packageId: string,
  targets: Agent[],
  createMissingRoots: boolean,
): Promise<InstallPlan> {
  return invoke<InstallPlan>("plan_install", {
    packageId,
    targets,
    createMissingRoots,
  });
}

export function commitInstall(
  planId: string,
  confirmCopyFallback: boolean,
): Promise<InstallResult> {
  return invoke<InstallResult>("commit_install", {
    planId,
    confirmCopyFallback,
  });
}

export function planConfiguration(
  packageId: string,
  agent: Agent,
  enabled: boolean,
): Promise<ConfigurationPlan> {
  return invoke<ConfigurationPlan>("plan_configuration", {
    packageId,
    agent,
    enabled,
  });
}

export function commitConfiguration(planId: string): Promise<InstallResult> {
  return invoke<InstallResult>("commit_configuration", { planId });
}

export function resolveConfiguration(
  packageId: string,
  agent: Agent,
  resolution: "reapply" | "forget",
): Promise<InstallResult> {
  return invoke<InstallResult>("resolve_configuration", {
    packageId,
    agent,
    resolution,
  });
}

export function planUninstall(
  packageId: string,
  agent: Agent,
): Promise<UninstallPlan> {
  return invoke<UninstallPlan>("plan_uninstall", { packageId, agent });
}

export function commitUninstall(planId: string): Promise<LifecycleResult> {
  return invoke<LifecycleResult>("commit_uninstall", { planId });
}

export function planDetach(
  packageId: string,
  agent: Agent,
): Promise<DetachPlan> {
  return invoke<DetachPlan>("plan_detach", { packageId, agent });
}

export function commitDetach(planId: string): Promise<LifecycleResult> {
  return invoke<LifecycleResult>("commit_detach", { planId });
}

export function planRemoveLibrary(
  packageId: string,
): Promise<RemoveLibraryPlan> {
  return invoke<RemoveLibraryPlan>("plan_remove_library", { packageId });
}

export function commitRemoveLibrary(
  planId: string,
  confirmationName: string,
): Promise<LifecycleResult> {
  return invoke<LifecycleResult>("commit_remove_library", {
    planId,
    confirmationName,
  });
}

export function planReplaceLocalRevision(
  packageId: string,
  path: string,
): Promise<ReplaceRevisionPlan> {
  return invoke<ReplaceRevisionPlan>("plan_replace_local_revision", {
    packageId,
    path,
  });
}

export function commitReplaceLocalRevision(
  planId: string,
): Promise<InstallResult> {
  return invoke<InstallResult>("commit_replace_local_revision", { planId });
}

export function planRollbackRevision(
  packageId: string,
): Promise<RollbackRevisionPlan> {
  return invoke<RollbackRevisionPlan>("plan_rollback_revision", { packageId });
}

export function commitRollbackRevision(planId: string): Promise<InstallResult> {
  return invoke<InstallResult>("commit_rollback_revision", { planId });
}

export function planExportRevision(
  packageId: string,
  destination: string,
): Promise<ExportRevisionPlan> {
  return invoke<ExportRevisionPlan>("plan_export_revision", {
    packageId,
    destination,
  });
}

export function commitExportRevision(
  planId: string,
): Promise<ExportRevisionResult> {
  return invoke<ExportRevisionResult>("commit_export_revision", { planId });
}

export function planRestoreInstallation(
  packageId: string,
  agent: Agent,
): Promise<RestoreInstallationPlan> {
  return invoke<RestoreInstallationPlan>("plan_restore_installation", {
    packageId,
    agent,
  });
}

export function commitRestoreInstallation(
  planId: string,
  confirmOverwrite: boolean,
): Promise<InstallResult> {
  return invoke<InstallResult>("commit_restore_installation", {
    planId,
    confirmOverwrite,
  });
}

export function planAdoption(
  installations: ExternalInstallationIdentity[],
): Promise<AdoptionPlan> {
  return invoke<AdoptionPlan>("plan_adoption", { installations });
}

export function commitAdoption(
  planId: string,
  confirmCopyFallback: boolean,
): Promise<InstallResult> {
  return invoke<InstallResult>("commit_adoption", {
    planId,
    confirmCopyFallback,
  });
}

export function planLegacyMigration(
  logicalPath: string,
): Promise<LegacyMigrationPlan> {
  return invoke<LegacyMigrationPlan>("plan_legacy_migration", { logicalPath });
}

export function commitLegacyMigration(planId: string): Promise<InstallResult> {
  return invoke<InstallResult>("commit_legacy_migration", { planId });
}

export function planGitImport(
  repositoryUrl: string,
  subpath: string,
  trackedBranch: string,
): Promise<GitImportPlan> {
  return invoke<GitImportPlan>("plan_git_import", {
    repositoryUrl,
    subpath,
    trackedBranch,
  });
}

export function commitGitImport(planId: string): Promise<ManagedSkillPackage> {
  return invoke<ManagedSkillPackage>("commit_git_import", { planId });
}

export function checkGitUpdate(packageId: string): Promise<GitUpdateCheck> {
  return invoke<GitUpdateCheck>("check_git_update", { packageId });
}

export function commitGitUpdate(planId: string): Promise<InstallResult> {
  return invoke<InstallResult>("commit_git_update", { planId });
}

export function planDiagnosticsExport(
  destination: string,
): Promise<DiagnosticsExportPlan> {
  return invoke<DiagnosticsExportPlan>("plan_diagnostics_export", {
    destination,
  });
}

export function commitDiagnosticsExport(
  planId: string,
): Promise<{ destination: string }> {
  return invoke<{ destination: string }>("commit_diagnostics_export", {
    planId,
  });
}

export function commandErrorCode(error: unknown): string | null {
  if (typeof error === "object" && error !== null && "code" in error) {
    return typeof error.code === "string" ? error.code : null;
  }
  return null;
}

export function commandErrorMessage(
  error: unknown,
  messages?: Record<string, string>,
): string | null {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = error.message;
    if (typeof message !== "string") return null;
    const code = commandErrorCode(error);
    const localized = (code && messages?.[code]) || message;
    const path =
      "path" in error && typeof error.path === "string" ? error.path : null;
    const limit = "limit" in error ? error.limit : null;
    const observed = "observed" in error ? error.observed : null;
    const details = [
      path,
      typeof limit === "number" && typeof observed === "number"
        ? `${limit} / ${observed}`
        : null,
    ].filter((detail): detail is string => detail !== null);
    return details.length > 0
      ? `${localized} (${details.join(" · ")})`
      : localized;
  }
  return null;
}
