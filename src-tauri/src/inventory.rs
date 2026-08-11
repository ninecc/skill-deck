use std::{env, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    configuration,
    skill::{self, SkillError, SkillErrorCode, ValidatedSkill},
    state::{
        ConfigurationProvenance, DeploymentMode, Installation, ManagedSkillPackage, SkillSource,
        StateMode, StateStore,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct AgentRoots {
    pub codex: PathBuf,
    pub claude: PathBuf,
    pub codex_legacy: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Agent {
    Codex,
    Claude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationKind {
    Directory,
    Link,
    LegacyDirectory,
    LegacyLink,
    BrokenLink,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionKind {
    BrokenExternalInstallation,
    InvalidInstallationCandidate,
    UnexpectedAgentRootEntry,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTarget {
    pub agent: Agent,
    pub root: PathBuf,
    pub exists: bool,
    pub legacy: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInstallation {
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub resolved_target: Option<PathBuf>,
    pub kind: InstallationKind,
    pub skill: Option<ValidatedSkill>,
    pub diagnostic: Option<InventoryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionEntry {
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub resolved_target: Option<PathBuf>,
    pub kind: AttentionKind,
    pub diagnostic: InventoryDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDiagnostic {
    pub code: SkillErrorCode,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedInstallationStatus {
    Healthy,
    Missing,
    Retargeted,
    Drifted,
    ConfigurationDrift,
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedInstallationAction {
    EnableConfiguration,
    DisableConfiguration,
    ReapplyConfiguration,
    ForgetConfiguration,
    Restore,
    Detach,
    Uninstall,
    ForgetInstallation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedPackageAction {
    Install,
    Replace,
    CheckUpdate,
    Rollback,
    Export,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationEvidence {
    pub logical_path: PathBuf,
    pub deployment_mode: DeploymentMode,
    pub expected_target: Option<PathBuf>,
    pub observed_target: Option<PathBuf>,
    pub recorded_fingerprint: String,
    pub library_fingerprint: String,
    pub observed_fingerprint: Option<String>,
    pub configuration_provenance: ConfigurationProvenance,
    pub deferred_checks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstallationReconciliation {
    pub package_id: String,
    pub agent: Agent,
    pub status: ManagedInstallationStatus,
    pub diagnostic: Option<InventoryDiagnostic>,
    pub evidence: ReconciliationEvidence,
    pub available_actions: Vec<ManagedInstallationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedPackageReconciliation {
    pub package_id: String,
    pub library_diagnostic: Option<InventoryDiagnostic>,
    pub available_actions: Vec<ManagedPackageAction>,
}

impl From<SkillError> for InventoryDiagnostic {
    fn from(error: SkillError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            path: error.path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub targets: Vec<AgentTarget>,
    pub external_installations: Vec<ExternalInstallation>,
    pub attention_entries: Vec<AttentionEntry>,
    pub managed_packages: Vec<ManagedSkillPackage>,
    pub managed_installation_statuses: Vec<ManagedInstallationReconciliation>,
    pub managed_package_reconciliations: Vec<ManagedPackageReconciliation>,
}

pub fn inventory(app_data: &std::path::Path) -> Result<Inventory, SkillError> {
    let roots = agent_roots()?;
    inventory_for_agent_roots(app_data, &roots)
}

pub(crate) fn inventory_for_agent_roots(
    app_data: &std::path::Path,
    roots: &AgentRoots,
) -> Result<Inventory, SkillError> {
    let mut inventory = inventory_for_roots([
        (Agent::Codex, roots.codex.clone(), false),
        (Agent::Claude, roots.claude.clone(), false),
        (Agent::Codex, roots.codex_legacy.clone(), true),
    ])?;
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    if loaded.mode != StateMode::ReadOnlyRecovery {
        inventory.managed_packages = loaded
            .state
            .expect("writable state mode contains state")
            .packages;
        for package in &inventory.managed_packages {
            let (package_reconciliation, installation_reconciliations) =
                reconcile_package(app_data, package);
            inventory
                .managed_package_reconciliations
                .push(package_reconciliation);
            inventory
                .managed_installation_statuses
                .extend(installation_reconciliations);
        }
        inventory.external_installations.retain(|external| {
            !inventory.managed_packages.iter().any(|package| {
                package.installations.iter().any(|installation| {
                    installation.logical_path == external.logical_path
                        && installation.agent == external.agent
                })
            })
        });
        inventory.attention_entries.retain(|entry| {
            !inventory.managed_packages.iter().any(|package| {
                package.installations.iter().any(|installation| {
                    installation.logical_path == entry.logical_path
                        && installation.agent == entry.agent
                })
            })
        });
    }
    Ok(inventory)
}

pub(crate) fn agent_roots() -> Result<AgentRoots, SkillError> {
    let home = home::home_dir().ok_or_else(|| {
        SkillError::new(
            SkillErrorCode::Io,
            "Could not resolve the operating-system home directory",
            None,
        )
    })?;
    let codex_state_root = env_root("CODEX_HOME")?.unwrap_or_else(|| home.join(".codex"));
    let claude_root = env_root("CLAUDE_CONFIG_DIR")?.unwrap_or_else(|| home.join(".claude"));
    Ok(AgentRoots {
        codex: home.join(".agents/skills"),
        claude: claude_root.join("skills"),
        codex_legacy: codex_state_root.join("skills"),
    })
}

fn env_root(name: &str) -> Result<Option<PathBuf>, SkillError> {
    let Some(value) = env::var_os(name) else {
        return Ok(None);
    };
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            format!("{name} must resolve to an absolute native path"),
            Some(path),
        ));
    }
    Ok(Some(path))
}

fn inventory_for_roots<const N: usize>(
    roots: [(Agent, PathBuf, bool); N],
) -> Result<Inventory, SkillError> {
    let mut targets = Vec::with_capacity(N);
    let mut external_installations = Vec::new();
    let mut attention_entries = Vec::new();

    for (agent, root, legacy) in roots {
        let exists = root.is_dir();
        targets.push(AgentTarget {
            agent,
            root: root.clone(),
            exists,
            legacy,
        });
        if !exists {
            continue;
        }

        let entries = fs::read_dir(&root).map_err(|error| SkillError::io(&root, error))?;
        for entry in entries {
            let entry = entry.map_err(|error| SkillError::io(&root, error))?;
            let logical_path = entry.path();
            if is_root_artifact(agent, legacy, &logical_path) {
                continue;
            }
            let metadata = fs::symlink_metadata(&logical_path)
                .map_err(|error| SkillError::io(&logical_path, error))?;
            if !metadata.is_dir() && !metadata.file_type().is_symlink() {
                attention_entries.push(AttentionEntry {
                    agent,
                    logical_path: logical_path.clone(),
                    resolved_target: None,
                    kind: AttentionKind::UnexpectedAgentRootEntry,
                    diagnostic: InventoryDiagnostic::from(SkillError::new(
                        SkillErrorCode::UnsupportedFileType,
                        "Agent root entry must be a directory or link",
                        Some(logical_path),
                    )),
                });
                continue;
            }
            let installation = inspect_entry(agent, logical_path, legacy);
            if installation.skill.is_some() {
                external_installations.push(installation);
            } else {
                attention_entries.push(attention_from_installation(installation));
            }
        }
    }

    external_installations.sort_by(|left, right| {
        left.logical_path
            .cmp(&right.logical_path)
            .then_with(|| agent_order(left.agent).cmp(&agent_order(right.agent)))
    });
    attention_entries.sort_by(|left, right| {
        left.logical_path
            .cmp(&right.logical_path)
            .then_with(|| agent_order(left.agent).cmp(&agent_order(right.agent)))
    });
    Ok(Inventory {
        targets,
        external_installations,
        attention_entries,
        managed_packages: Vec::new(),
        managed_installation_statuses: Vec::new(),
        managed_package_reconciliations: Vec::new(),
    })
}

pub(crate) fn reconcile_installation(
    package: &ManagedSkillPackage,
    installation: &Installation,
) -> ManagedInstallationReconciliation {
    let library_error = validate_library(package).err();
    reconcile_installation_with_library(package, installation, library_error.as_ref())
}

fn reconcile_package(
    app_data: &std::path::Path,
    package: &ManagedSkillPackage,
) -> (
    ManagedPackageReconciliation,
    Vec<ManagedInstallationReconciliation>,
) {
    let library_error = validate_library(package).err();
    let installations = package
        .installations
        .iter()
        .map(|installation| {
            reconcile_installation_with_library(package, installation, library_error.as_ref())
        })
        .collect::<Vec<_>>();
    let all_healthy = installations
        .iter()
        .all(|entry| entry.status == ManagedInstallationStatus::Healthy);
    let mut actions = Vec::new();
    if library_error.is_none() {
        actions.push(ManagedPackageAction::Export);
        if all_healthy {
            if package.installations.len() < 2 {
                actions.push(ManagedPackageAction::Install);
            }
            match package.source {
                SkillSource::LocalSnapshot => actions.push(ManagedPackageAction::Replace),
                SkillSource::Git { .. } => actions.push(ManagedPackageAction::CheckUpdate),
            }
            if package.previous_revision.is_some() {
                actions.push(ManagedPackageAction::Rollback);
            }
        }
    }
    if package.installations.is_empty() && safe_package_root(app_data, package) {
        actions.push(ManagedPackageAction::Remove);
    }
    (
        ManagedPackageReconciliation {
            package_id: package.id.clone(),
            library_diagnostic: library_error.map(Into::into),
            available_actions: actions,
        },
        installations,
    )
}

fn reconcile_installation_with_library(
    package: &ManagedSkillPackage,
    installation: &Installation,
    library_error: Option<&SkillError>,
) -> ManagedInstallationReconciliation {
    let mut evidence = ReconciliationEvidence {
        logical_path: installation.logical_path.clone(),
        deployment_mode: installation.deployment_mode,
        expected_target: (installation.deployment_mode != DeploymentMode::CopyFallback)
            .then(|| package.library_path.clone()),
        observed_target: None,
        recorded_fingerprint: installation.last_known_fingerprint.clone(),
        library_fingerprint: package.installed_revision.fingerprint.clone(),
        observed_fingerprint: None,
        configuration_provenance: installation.configuration_provenance.clone(),
        deferred_checks: false,
    };
    let (status, diagnostic) = if let Some(error) = library_error {
        evidence.deferred_checks = true;
        (
            ManagedInstallationStatus::Broken,
            Some(error.clone().into()),
        )
    } else {
        classify_projection(package, installation, &mut evidence)
    };
    let available_actions = installation_actions(status, installation);
    ManagedInstallationReconciliation {
        package_id: package.id.clone(),
        agent: installation.agent,
        status,
        diagnostic,
        evidence,
        available_actions,
    }
}

fn classify_projection(
    package: &ManagedSkillPackage,
    installation: &Installation,
    evidence: &mut ReconciliationEvidence,
) -> (ManagedInstallationStatus, Option<InventoryDiagnostic>) {
    if installation.deployment_mode == DeploymentMode::CopyFallback
        && installation.resolved_target != installation.logical_path
    {
        evidence.deferred_checks = true;
        return broken_topology(installation);
    }
    if installation.deployment_mode != DeploymentMode::CopyFallback
        && installation.last_known_fingerprint != package.installed_revision.fingerprint
    {
        evidence.deferred_checks = true;
        return (
            ManagedInstallationStatus::Broken,
            Some(diagnostic(
                SkillErrorCode::InvalidStructure,
                "Recorded Installation fingerprint does not match the Installed Revision",
                installation.logical_path.clone(),
            )),
        );
    }
    let metadata = match fs::symlink_metadata(&installation.logical_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            evidence.deferred_checks = true;
            return (
                ManagedInstallationStatus::Missing,
                Some(diagnostic(
                    SkillErrorCode::InstallationMissing,
                    "Managed Installation path is missing; content and configuration checks are deferred until refresh",
                    installation.logical_path.clone(),
                )),
            );
        }
        Err(error) => {
            evidence.deferred_checks = true;
            return (
                ManagedInstallationStatus::Broken,
                Some(SkillError::io(&installation.logical_path, error).into()),
            );
        }
    };
    match installation.deployment_mode {
        DeploymentMode::CopyFallback => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                evidence.deferred_checks = true;
                return broken_topology(installation);
            }
            match skill::validate_installed_revision(&installation.logical_path, &package.name) {
                Ok(observed) => {
                    evidence.observed_fingerprint = Some(observed.fingerprint.clone());
                    if observed.fingerprint != installation.last_known_fingerprint
                        || observed.fingerprint != package.installed_revision.fingerprint
                    {
                        evidence.deferred_checks = true;
                        return (
                            ManagedInstallationStatus::Drifted,
                            Some(diagnostic(
                                SkillErrorCode::ContentDrift,
                                "Copy Fallback content differs from its recorded or current Managed Library fingerprint; configuration check is deferred until refresh",
                                installation.logical_path.clone(),
                            )),
                        );
                    }
                }
                Err(error) => {
                    evidence.deferred_checks = true;
                    return (ManagedInstallationStatus::Broken, Some(error.into()));
                }
            }
        }
        DeploymentMode::Symlink => {
            if !metadata.file_type().is_symlink() {
                evidence.deferred_checks = true;
                return broken_topology(installation);
            }
            if let Some(result) = classify_link(package, installation, evidence) {
                return result;
            }
        }
        DeploymentMode::Junction => {
            #[cfg(not(windows))]
            {
                evidence.deferred_checks = true;
                return broken_topology(installation);
            }
            #[cfg(windows)]
            {
                if !crate::install::is_windows_directory_link(&metadata) {
                    evidence.deferred_checks = true;
                    return broken_topology(installation);
                }
                if let Some(result) = classify_link(package, installation, evidence) {
                    return result;
                }
            }
        }
    }
    if matches!(
        installation.configuration_provenance,
        ConfigurationProvenance::SkillDeck { .. }
    ) {
        if let Err(error) = configuration::validate_owned_configuration(&package.name, installation)
        {
            let status = if error.code == SkillErrorCode::ConfigurationDrift {
                ManagedInstallationStatus::ConfigurationDrift
            } else {
                ManagedInstallationStatus::Broken
            };
            return (status, Some(error.into()));
        }
    }
    (ManagedInstallationStatus::Healthy, None)
}

fn classify_link(
    package: &ManagedSkillPackage,
    installation: &Installation,
    evidence: &mut ReconciliationEvidence,
) -> Option<(ManagedInstallationStatus, Option<InventoryDiagnostic>)> {
    let actual = match installation.logical_path.canonicalize() {
        Ok(actual) => actual,
        Err(error) => {
            evidence.deferred_checks = true;
            return Some((
                ManagedInstallationStatus::Broken,
                Some(diagnostic(
                    SkillErrorCode::TopologyChanged,
                    format!("Managed link cannot be resolved safely: {error}"),
                    installation.logical_path.clone(),
                )),
            ));
        }
    };
    evidence.observed_target = Some(actual.clone());
    let expected = package
        .library_path
        .canonicalize()
        .expect("validated Managed Library has a canonical path");
    let recorded = installation.resolved_target.canonicalize().ok();
    if actual != expected || recorded.as_ref() != Some(&expected) {
        evidence.deferred_checks = true;
        return Some((
            ManagedInstallationStatus::Retargeted,
            Some(diagnostic(
                SkillErrorCode::TopologyChanged,
                "Managed link resolves somewhere other than the recorded current Managed Library; configuration check is deferred until refresh",
                installation.logical_path.clone(),
            )),
        ));
    }
    None
}

fn validate_library(package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let validated = skill::validate_installed_revision(&package.library_path, &package.name)?;
    if validated.fingerprint != package.installed_revision.fingerprint {
        return Err(SkillError::new(
            SkillErrorCode::ContentDrift,
            "Managed Library content no longer matches the Installed Revision",
            Some(package.library_path.clone()),
        ));
    }
    Ok(())
}

fn installation_actions(
    status: ManagedInstallationStatus,
    installation: &Installation,
) -> Vec<ManagedInstallationAction> {
    match status {
        ManagedInstallationStatus::Healthy => {
            let mut actions = Vec::new();
            if !matches!(
                installation.configuration_provenance,
                ConfigurationProvenance::External { .. }
            ) {
                actions.push(if installation.enabled {
                    ManagedInstallationAction::DisableConfiguration
                } else {
                    ManagedInstallationAction::EnableConfiguration
                });
            }
            actions.extend([
                ManagedInstallationAction::Detach,
                ManagedInstallationAction::Uninstall,
            ]);
            actions
        }
        ManagedInstallationStatus::Missing => vec![
            ManagedInstallationAction::Restore,
            ManagedInstallationAction::ForgetInstallation,
        ],
        ManagedInstallationStatus::Drifted => vec![
            ManagedInstallationAction::Restore,
            ManagedInstallationAction::Detach,
        ],
        ManagedInstallationStatus::ConfigurationDrift => vec![
            ManagedInstallationAction::ReapplyConfiguration,
            ManagedInstallationAction::ForgetConfiguration,
        ],
        ManagedInstallationStatus::Retargeted | ManagedInstallationStatus::Broken => {
            vec![ManagedInstallationAction::ForgetInstallation]
        }
    }
}

fn broken_topology(
    installation: &Installation,
) -> (ManagedInstallationStatus, Option<InventoryDiagnostic>) {
    (
        ManagedInstallationStatus::Broken,
        Some(diagnostic(
            SkillErrorCode::TopologyChanged,
            "Managed Installation topology no longer matches its recorded deployment mode",
            installation.logical_path.clone(),
        )),
    )
}

fn diagnostic(
    code: SkillErrorCode,
    message: impl Into<String>,
    path: PathBuf,
) -> InventoryDiagnostic {
    InventoryDiagnostic {
        code,
        message: message.into(),
        path: Some(path),
    }
}

fn safe_package_root(app_data: &std::path::Path, package: &ManagedSkillPackage) -> bool {
    if !skill::valid_name(&package.name) {
        return false;
    }
    let root = app_data.join("library").join(&package.name);
    if package.library_path != root.join("current") {
        return false;
    }
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => true,
        Ok(metadata) => metadata.is_dir() && !metadata.file_type().is_symlink(),
        _ => false,
    }
}

fn is_root_artifact(agent: Agent, legacy: bool, path: &std::path::Path) -> bool {
    path.file_name().is_some_and(|name| {
        name == ".DS_Store" || (agent == Agent::Codex && legacy && name == ".system")
    })
}

fn attention_from_installation(installation: ExternalInstallation) -> AttentionEntry {
    AttentionEntry {
        agent: installation.agent,
        logical_path: installation.logical_path,
        resolved_target: installation.resolved_target,
        kind: if installation.kind == InstallationKind::BrokenLink {
            AttentionKind::BrokenExternalInstallation
        } else {
            AttentionKind::InvalidInstallationCandidate
        },
        diagnostic: installation
            .diagnostic
            .expect("an installation without a validated Skill has a diagnostic"),
    }
}

pub(crate) fn inspect_entry(
    agent: Agent,
    logical_path: PathBuf,
    legacy: bool,
) -> ExternalInstallation {
    let metadata = match fs::symlink_metadata(&logical_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return invalid_installation(
                agent,
                logical_path.clone(),
                None,
                InstallationKind::Invalid,
                SkillError::io(&logical_path, error),
            );
        }
    };

    if metadata.file_type().is_symlink() {
        let target = match logical_path.canonicalize() {
            Ok(target) => target,
            Err(error) => {
                let unresolved_target = fs::read_link(&logical_path).ok().map(|target| {
                    if target.is_absolute() {
                        target
                    } else {
                        logical_path
                            .parent()
                            .unwrap_or_else(|| std::path::Path::new("."))
                            .join(target)
                    }
                });
                return invalid_installation(
                    agent,
                    logical_path.clone(),
                    unresolved_target,
                    InstallationKind::BrokenLink,
                    SkillError::new(
                        SkillErrorCode::UnsupportedFileType,
                        format!("Could not resolve external link: {error}"),
                        Some(logical_path),
                    ),
                );
            }
        };
        return validated_installation(
            agent,
            logical_path,
            Some(target.clone()),
            if legacy {
                InstallationKind::LegacyLink
            } else {
                InstallationKind::Link
            },
            skill::validate_skill_dir(&target),
        );
    }

    validated_installation(
        agent,
        logical_path.clone(),
        Some(logical_path.clone()),
        if legacy {
            InstallationKind::LegacyDirectory
        } else {
            InstallationKind::Directory
        },
        skill::validate_skill_dir(&logical_path),
    )
}

fn validated_installation(
    agent: Agent,
    logical_path: PathBuf,
    resolved_target: Option<PathBuf>,
    kind: InstallationKind,
    result: Result<ValidatedSkill, SkillError>,
) -> ExternalInstallation {
    match result {
        Ok(skill) if logical_path.file_name() == Some(skill.metadata.name.as_ref()) => {
            ExternalInstallation {
                agent,
                logical_path,
                resolved_target,
                kind,
                skill: Some(skill),
                diagnostic: None,
            }
        }
        Ok(_) => invalid_installation(
            agent,
            logical_path.clone(),
            resolved_target,
            InstallationKind::Invalid,
            SkillError::new(
                SkillErrorCode::InvalidMetadata,
                "Installation entry name must match Skill metadata name",
                Some(logical_path),
            ),
        ),
        Err(error) => invalid_installation(
            agent,
            logical_path,
            resolved_target,
            InstallationKind::Invalid,
            error,
        ),
    }
}

fn invalid_installation(
    agent: Agent,
    logical_path: PathBuf,
    resolved_target: Option<PathBuf>,
    kind: InstallationKind,
    error: SkillError,
) -> ExternalInstallation {
    ExternalInstallation {
        agent,
        logical_path,
        resolved_target,
        kind,
        skill: None,
        diagnostic: Some(error.into()),
    }
}

const fn agent_order(agent: Agent) -> u8 {
    match agent {
        Agent::Codex => 0,
        Agent::Claude => 1,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn write_skill(root: &std::path::Path, name: &str) {
        let skill = root.join(name);
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Inventory fixture\n---\n"),
        )
        .unwrap();
    }

    fn managed_copy(temp: &TempDir) -> ManagedSkillPackage {
        let library = temp.path().join("library/alpha-skill/current");
        write_skill(library.parent().unwrap(), "current");
        fs::write(
            library.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Inventory fixture\n---\n",
        )
        .unwrap();
        let fingerprint = skill::validate_installed_revision(&library, "alpha-skill")
            .unwrap()
            .fingerprint;
        let logical = temp.path().join("agent/alpha-skill");
        crate::library::copy_directory(&library, &logical).unwrap();
        ManagedSkillPackage {
            id: "package-1".to_owned(),
            name: "alpha-skill".to_owned(),
            library_path: library,
            source: SkillSource::LocalSnapshot,
            installed_revision: crate::state::InstalledRevision {
                fingerprint: fingerprint.clone(),
                commit_oid: None,
            },
            previous_revision: None,
            installations: vec![Installation {
                agent: Agent::Claude,
                logical_path: logical.clone(),
                resolved_target: logical,
                deployment_mode: DeploymentMode::CopyFallback,
                enabled: true,
                last_known_fingerprint: fingerprint,
                configuration_provenance: ConfigurationProvenance::None,
            }],
        }
    }

    #[test]
    fn managed_copy_reconciliation_distinguishes_healthy_missing_drift_and_broken() {
        let temp = TempDir::new().unwrap();
        let package = managed_copy(&temp);
        let installation = &package.installations[0];
        assert_eq!(
            reconcile_installation(&package, installation).status,
            ManagedInstallationStatus::Healthy
        );

        fs::remove_dir_all(&installation.logical_path).unwrap();
        assert_eq!(
            reconcile_installation(&package, installation).status,
            ManagedInstallationStatus::Missing
        );
        crate::library::copy_directory(&package.library_path, &installation.logical_path).unwrap();
        fs::write(installation.logical_path.join("changed.txt"), "changed").unwrap();
        assert_eq!(
            reconcile_installation(&package, installation).status,
            ManagedInstallationStatus::Drifted
        );
        fs::remove_file(installation.logical_path.join("SKILL.md")).unwrap();
        assert_eq!(
            reconcile_installation(&package, installation).status,
            ManagedInstallationStatus::Broken
        );
    }

    #[test]
    fn managed_copy_reconciliation_rejects_a_mismatched_recorded_target() {
        let temp = TempDir::new().unwrap();
        let mut package = managed_copy(&temp);
        package.installations[0].resolved_target = temp.path().join("outside");

        assert_eq!(
            reconcile_installation(&package, &package.installations[0]).status,
            ManagedInstallationStatus::Broken
        );
    }

    #[cfg(unix)]
    #[test]
    fn managed_link_reconciliation_reports_retargeted() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let mut package = managed_copy(&temp);
        let logical_path = package.installations[0].logical_path.clone();
        fs::remove_dir_all(&logical_path).unwrap();
        let external = temp.path().join("external");
        fs::create_dir_all(&external).unwrap();
        symlink(&external, &logical_path).unwrap();
        package.installations[0].deployment_mode = DeploymentMode::Symlink;
        package.installations[0].resolved_target = package.library_path.clone();

        assert_eq!(
            reconcile_installation(&package, &package.installations[0]).status,
            ManagedInstallationStatus::Retargeted
        );
    }

    #[test]
    fn managed_configuration_drift_is_visible_during_inventory() {
        let temp = TempDir::new().unwrap();
        let mut package = managed_copy(&temp);
        let config = temp.path().join("settings.json");
        fs::write(&config, r#"{"skillOverrides":{"alpha-skill":"off"}}"#).unwrap();
        package.installations[0].configuration_provenance =
            ConfigurationProvenance::SkillDeck { path: config };

        assert_eq!(
            reconcile_installation(&package, &package.installations[0]).status,
            ManagedInstallationStatus::ConfigurationDrift
        );
    }

    #[test]
    fn managed_reconciliation_serializes_closed_status_actions_and_evidence() {
        let temp = TempDir::new().unwrap();
        let package = managed_copy(&temp);
        let value =
            serde_json::to_value(reconcile_installation(&package, &package.installations[0]))
                .unwrap();

        assert_eq!(value["status"], "healthy");
        assert_eq!(value["availableActions"][0], "disable_configuration");
        assert_eq!(value["evidence"]["deploymentMode"], "copy_fallback");
        assert!(value["evidence"].get("libraryFingerprint").is_some());
    }

    #[test]
    fn missing_roots_are_advisory() {
        let temp = TempDir::new().unwrap();
        let inventory =
            inventory_for_roots([(Agent::Codex, temp.path().join("missing"), false)]).unwrap();

        assert!(!inventory.targets[0].exists);
        assert!(inventory.external_installations.is_empty());
        assert!(inventory.attention_entries.is_empty());
    }

    #[test]
    fn valid_entries_remain_external() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("skills");
        write_skill(&root, "alpha-skill");

        let inventory = inventory_for_roots([(Agent::Claude, root, false)]).unwrap();

        assert_eq!(inventory.external_installations.len(), 1);
        let installation = &inventory.external_installations[0];
        assert_eq!(installation.kind, InstallationKind::Directory);
        assert_eq!(
            installation.skill.as_ref().unwrap().metadata.name,
            "alpha-skill"
        );
        assert!(installation.diagnostic.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn healthy_and_broken_links_are_distinguished() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("skills");
        let canonical = temp.path().join("canonical");
        fs::create_dir_all(&root).unwrap();
        write_skill(&canonical, "linked-skill");
        symlink(canonical.join("linked-skill"), root.join("linked-skill")).unwrap();
        symlink(temp.path().join("absent"), root.join("broken-skill")).unwrap();

        let inventory = inventory_for_roots([(Agent::Codex, root, false)]).unwrap();

        assert_eq!(inventory.external_installations.len(), 1);
        assert!(inventory
            .external_installations
            .iter()
            .any(|installation| installation.kind == InstallationKind::Link));
        let broken = inventory
            .attention_entries
            .iter()
            .find(|entry| entry.kind == AttentionKind::BrokenExternalInstallation)
            .unwrap();
        assert_eq!(broken.resolved_target, Some(temp.path().join("absent")));
    }

    #[test]
    fn root_artifacts_are_ignored_without_hiding_other_entries() {
        let temp = TempDir::new().unwrap();
        let current = temp.path().join("current");
        let legacy = temp.path().join("legacy");
        fs::create_dir_all(current.join(".system")).unwrap();
        fs::create_dir_all(legacy.join(".system")).unwrap();
        fs::write(current.join(".DS_Store"), "noise").unwrap();
        fs::write(legacy.join(".DS_Store"), "noise").unwrap();

        let inventory = inventory_for_roots([
            (Agent::Codex, current.clone(), false),
            (Agent::Codex, legacy, true),
        ])
        .unwrap();

        assert!(inventory.external_installations.is_empty());
        assert_eq!(inventory.attention_entries.len(), 1);
        assert_eq!(
            inventory.attention_entries[0].logical_path,
            current.join(".system")
        );
        assert_eq!(
            inventory.attention_entries[0].kind,
            AttentionKind::InvalidInstallationCandidate
        );
    }

    #[test]
    fn ordinary_files_are_unexpected_root_entries() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("skills");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("notes.txt"), "not a skill").unwrap();

        let inventory = inventory_for_roots([(Agent::Claude, root.clone(), false)]).unwrap();

        assert!(inventory.external_installations.is_empty());
        assert_eq!(inventory.attention_entries.len(), 1);
        assert_eq!(
            inventory.attention_entries[0].kind,
            AttentionKind::UnexpectedAgentRootEntry
        );
        assert_eq!(
            inventory.attention_entries[0].diagnostic.path,
            Some(root.join("notes.txt"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn invalid_content_behind_a_healthy_link_is_not_broken_topology() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("skills");
        let target = temp.path().join("invalid-skill");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&target).unwrap();
        symlink(&target, root.join("invalid-skill")).unwrap();

        let inventory = inventory_for_roots([(Agent::Codex, root, false)]).unwrap();

        assert_eq!(inventory.attention_entries.len(), 1);
        assert_eq!(
            inventory.attention_entries[0].kind,
            AttentionKind::InvalidInstallationCandidate
        );
    }
}
