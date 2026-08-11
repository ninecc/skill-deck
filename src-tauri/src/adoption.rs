use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    configuration,
    install::{
        copy_atomically, create_preferred_link, preferred_mode, remove_created_entry,
        RESTART_MESSAGE,
    },
    inventory::{self, Agent, AgentRoots, ExternalInstallation, InstallationKind},
    library::{
        copy_directory, ensure_name_available, ensure_unique_state_name, next_id, remove_directory,
    },
    skill::{self, SkillError, SkillErrorCode},
    state::{
        ConfigurationProvenance, DeploymentMode, Installation, InstalledRevision,
        ManagedSkillPackage, SkillSource, StateMode, StateStore,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInstallationIdentity {
    pub agent: Agent,
    pub logical_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionInstallationPlan {
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub resolved_target: PathBuf,
    pub kind: InstallationKind,
    pub preferred_mode: DeploymentMode,
    pub enabled: bool,
    pub configuration_provenance: ConfigurationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionPlan {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
    pub library_path: PathBuf,
    pub installations: Vec<AdoptionInstallationPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMigrationPlan {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
    pub library_path: PathBuf,
    pub legacy_path: PathBuf,
    pub resolved_target: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoptionResult {
    pub package: ManagedSkillPackage,
    pub restart_message: &'static str,
}

#[derive(Debug, Clone)]
struct PendingAdoption {
    public: AdoptionPlan,
    staged_path: PathBuf,
    roots: AgentRoots,
}

#[derive(Debug, Clone)]
struct PendingLegacyMigration {
    public: LegacyMigrationPlan,
    staged_path: PathBuf,
    roots: AgentRoots,
    kind: InstallationKind,
}

#[derive(Debug, Clone)]
enum PendingPlan {
    Adoption(PendingAdoption),
    Legacy(PendingLegacyMigration),
}

#[derive(Default)]
pub struct AdoptionManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl AdoptionManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_adoption(
        &self,
        app_data: &Path,
        identities: Vec<ExternalInstallationIdentity>,
    ) -> Result<AdoptionPlan, SkillError> {
        self.plan_adoption_for_roots(app_data, identities, inventory::agent_roots()?)
    }

    fn plan_adoption_for_roots(
        &self,
        app_data: &Path,
        identities: Vec<ExternalInstallationIdentity>,
        roots: AgentRoots,
    ) -> Result<AdoptionPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        if identities.is_empty() {
            return Err(SkillError::new(
                SkillErrorCode::InvalidPlan,
                "Adoption requires at least one explicitly selected external Installation",
                None,
            ));
        }

        let mut selected = Vec::with_capacity(identities.len());
        for identity in identities {
            if selected.iter().any(|entry: &SelectedExternal| {
                entry.identity.agent == identity.agent
                    || entry.identity.logical_path == identity.logical_path
            }) {
                return Err(SkillError::new(
                    SkillErrorCode::InvalidPlan,
                    "Each Agent Installation may be selected only once",
                    Some(identity.logical_path),
                ));
            }
            selected.push(select_external(identity, &roots, false)?);
        }
        let name = selected.first().expect("non-empty selection").name.clone();
        let fingerprint = selected
            .first()
            .expect("non-empty selection")
            .fingerprint
            .clone();
        if selected.iter().any(|entry| entry.name != name) {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "Grouped Adoption requires the same normalized Skill name",
                None,
            ));
        }
        if selected
            .iter()
            .any(|entry| entry.fingerprint != fingerprint)
        {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "Same-name external Installations have different content fingerprints",
                None,
            ));
        }
        ensure_name_available(app_data, &name)?;

        let id = next_id();
        let staged_path = stage_external(
            app_data,
            &id,
            &name,
            &fingerprint,
            &selected
                .first()
                .expect("non-empty selection")
                .resolved_target,
        )?;
        let result = (|| {
            for entry in &selected {
                ensure_selected_fresh(entry, &roots, false)?;
            }
            let installations = selected
                .into_iter()
                .map(|entry| {
                    let (enabled, configuration_provenance) = configuration::initial_configuration(
                        entry.identity.agent,
                        &entry.identity.logical_path,
                        &entry.name,
                        &roots,
                    )?;
                    Ok(AdoptionInstallationPlan {
                        agent: entry.identity.agent,
                        logical_path: entry.identity.logical_path,
                        resolved_target: entry.resolved_target,
                        kind: entry.kind,
                        preferred_mode: preferred_mode(),
                        enabled,
                        configuration_provenance,
                    })
                })
                .collect::<Result<Vec<_>, SkillError>>()?;
            let public = AdoptionPlan {
                id: id.clone(),
                name: name.clone(),
                fingerprint: fingerprint.clone(),
                library_path: library_path(app_data, &name),
                installations,
            };
            self.plans.lock().map_err(|_| lock_error())?.insert(
                id,
                PendingPlan::Adoption(PendingAdoption {
                    public: public.clone(),
                    staged_path: staged_path.clone(),
                    roots,
                }),
            );
            Ok(public)
        })();
        if result.is_err() {
            remove_plan_staging(&staged_path)?;
        }
        result
    }

    pub fn commit_adoption(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_copy_fallback: bool,
    ) -> Result<AdoptionResult, SkillError> {
        self.commit_adoption_with_linker(
            app_data,
            plan_id,
            confirm_copy_fallback,
            create_preferred_link,
        )
    }

    fn commit_adoption_with_linker<F>(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_copy_fallback: bool,
        mut linker: F,
    ) -> Result<AdoptionResult, SkillError>
    where
        F: FnMut(&Path, &Path) -> io::Result<DeploymentMode>,
    {
        self.commit_adoption_with_saver(
            app_data,
            plan_id,
            confirm_copy_fallback,
            &mut linker,
            &mut |state| StateStore::new(app_data.to_path_buf()).save(state),
        )
    }

    fn commit_adoption_with_saver<F, S>(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_copy_fallback: bool,
        linker: &mut F,
        saver: &mut S,
    ) -> Result<AdoptionResult, SkillError>
    where
        F: FnMut(&Path, &Path) -> io::Result<DeploymentMode>,
        S: FnMut(&crate::state::AppState) -> Result<(), SkillError>,
    {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let plan = match self
            .plans
            .lock()
            .map_err(|_| lock_error())?
            .remove(plan_id)
            .ok_or_else(invalid_plan)?
        {
            PendingPlan::Adoption(plan) => plan,
            PendingPlan::Legacy(_) => return Err(invalid_plan()),
        };
        let result = commit_adoption_pending(
            app_data,
            plan_id,
            plan,
            confirm_copy_fallback,
            linker,
            saver,
        );
        if result.is_err() {
            let _ = remove_plan_staging_path(app_data, plan_id);
        }
        result
    }

    pub fn plan_legacy_migration(
        &self,
        app_data: &Path,
        logical_path: PathBuf,
    ) -> Result<LegacyMigrationPlan, SkillError> {
        self.plan_legacy_for_roots(app_data, logical_path, inventory::agent_roots()?)
    }

    fn plan_legacy_for_roots(
        &self,
        app_data: &Path,
        logical_path: PathBuf,
        roots: AgentRoots,
    ) -> Result<LegacyMigrationPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let selected = select_external(
            ExternalInstallationIdentity {
                agent: Agent::Codex,
                logical_path,
            },
            &roots,
            true,
        )?;
        ensure_name_available(app_data, &selected.name)?;
        let id = next_id();
        let staged_path = stage_external(
            app_data,
            &id,
            &selected.name,
            &selected.fingerprint,
            &selected.resolved_target,
        )?;
        let result = (|| {
            ensure_selected_fresh(&selected, &roots, true)?;
            let name = selected.name.clone();
            let public = LegacyMigrationPlan {
                id: id.clone(),
                name: selected.name,
                fingerprint: selected.fingerprint,
                library_path: library_path(app_data, &name),
                legacy_path: selected.identity.logical_path,
                resolved_target: selected.resolved_target,
            };
            self.plans.lock().map_err(|_| lock_error())?.insert(
                id,
                PendingPlan::Legacy(PendingLegacyMigration {
                    public: public.clone(),
                    staged_path: staged_path.clone(),
                    roots,
                    kind: selected.kind,
                }),
            );
            Ok(public)
        })();
        if result.is_err() {
            remove_plan_staging(&staged_path)?;
        }
        result
    }

    pub fn commit_legacy_migration(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<AdoptionResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let plan = match self
            .plans
            .lock()
            .map_err(|_| lock_error())?
            .remove(plan_id)
            .ok_or_else(invalid_plan)?
        {
            PendingPlan::Legacy(plan) => plan,
            PendingPlan::Adoption(_) => return Err(invalid_plan()),
        };
        let result = commit_legacy_pending(app_data, plan);
        if result.is_err() {
            let _ = remove_plan_staging_path(app_data, plan_id);
        }
        result
    }
}

#[derive(Debug)]
struct SelectedExternal {
    identity: ExternalInstallationIdentity,
    resolved_target: PathBuf,
    kind: InstallationKind,
    name: String,
    fingerprint: String,
}

fn select_external(
    identity: ExternalInstallationIdentity,
    roots: &AgentRoots,
    legacy: bool,
) -> Result<SelectedExternal, SkillError> {
    let root = if legacy {
        if identity.agent != Agent::Codex {
            return Err(invalid_external(&identity.logical_path));
        }
        &roots.codex_legacy
    } else {
        match identity.agent {
            Agent::Codex => &roots.codex,
            Agent::Claude => &roots.claude,
        }
    };
    if identity.logical_path.parent() != Some(root.as_path())
        || identity.logical_path.file_name().is_none()
    {
        return Err(SkillError::new(
            SkillErrorCode::InvalidPlan,
            "External Installation identity must be a direct child of the selected official Agent root",
            Some(identity.logical_path),
        ));
    }
    let external = inventory::inspect_entry(identity.agent, identity.logical_path.clone(), legacy);
    selected_from_inventory(identity, external, legacy)
}

fn selected_from_inventory(
    identity: ExternalInstallationIdentity,
    external: ExternalInstallation,
    legacy: bool,
) -> Result<SelectedExternal, SkillError> {
    let healthy = if legacy {
        matches!(
            external.kind,
            InstallationKind::LegacyDirectory | InstallationKind::LegacyLink
        )
    } else {
        matches!(
            external.kind,
            InstallationKind::Directory | InstallationKind::Link
        )
    };
    let (Some(skill), Some(resolved_target)) = (external.skill, external.resolved_target) else {
        return Err(external
            .diagnostic
            .map(|diagnostic| SkillError::new(diagnostic.code, diagnostic.message, diagnostic.path))
            .unwrap_or_else(|| invalid_external(&identity.logical_path)));
    };
    if !healthy {
        return Err(invalid_external(&identity.logical_path));
    }
    Ok(SelectedExternal {
        identity,
        resolved_target,
        kind: external.kind,
        name: skill.metadata.name,
        fingerprint: skill.fingerprint,
    })
}

fn ensure_selected_fresh(
    expected: &SelectedExternal,
    roots: &AgentRoots,
    legacy: bool,
) -> Result<(), SkillError> {
    let observed = select_external(expected.identity.clone(), roots, legacy)
        .map_err(|_| source_changed(&expected.identity.logical_path))?;
    if observed.resolved_target != expected.resolved_target
        || observed.kind != expected.kind
        || observed.name != expected.name
        || observed.fingerprint != expected.fingerprint
    {
        return Err(source_changed(&expected.identity.logical_path));
    }
    Ok(())
}

fn commit_adoption_pending<F, S>(
    app_data: &Path,
    plan_id: &str,
    plan: PendingAdoption,
    confirm_copy_fallback: bool,
    linker: &mut F,
    saver: &mut S,
) -> Result<AdoptionResult, SkillError>
where
    F: FnMut(&Path, &Path) -> io::Result<DeploymentMode>,
    S: FnMut(&crate::state::AppState) -> Result<(), SkillError>,
{
    let staged = skill::validate_installed_revision(&plan.staged_path, &plan.public.name)?;
    if staged.fingerprint != plan.public.fingerprint {
        return Err(source_changed(&plan.staged_path));
    }
    let mut state = load_writable_state(app_data)?;
    ensure_unique_state_name(&state.packages, &plan.public.name)?;
    ensure_library_absent(&plan.public.library_path)?;

    for expected in &plan.public.installations {
        let selected = SelectedExternal {
            identity: ExternalInstallationIdentity {
                agent: expected.agent,
                logical_path: expected.logical_path.clone(),
            },
            resolved_target: expected.resolved_target.clone(),
            kind: expected.kind,
            name: plan.public.name.clone(),
            fingerprint: plan.public.fingerprint.clone(),
        };
        ensure_selected_fresh(&selected, &plan.roots, false)?;
        let configuration = configuration::initial_configuration(
            expected.agent,
            &expected.logical_path,
            &plan.public.name,
            &plan.roots,
        )?;
        if configuration != (expected.enabled, expected.configuration_provenance.clone()) {
            return Err(source_changed(&expected.logical_path));
        }
    }

    move_staging_to_library(&plan.staged_path, &plan.public.library_path)?;
    let package_root = plan
        .public
        .library_path
        .parent()
        .expect("library path has a package root")
        .to_path_buf();
    if let Err(error) = remove_plan_staging_path(app_data, plan_id) {
        remove_directory(&package_root)?;
        return Err(error);
    }
    let mut replacements = Vec::new();
    let mut installations = Vec::new();
    for (index, expected) in plan.public.installations.iter().enumerate() {
        let backup = expected
            .logical_path
            .parent()
            .expect("external Installation has an Agent root")
            .join(format!(".skill-deck-adopt-{plan_id}-{index}"));
        if let Err(error) = ensure_path_absent(&backup) {
            rollback_adoption(&replacements, &package_root)?;
            return Err(error);
        }
        if let Err(error) = fs::rename(&expected.logical_path, &backup) {
            rollback_adoption(&replacements, &package_root)?;
            return Err(SkillError::io(&expected.logical_path, error));
        }

        let mode = match linker(&plan.public.library_path, &expected.logical_path) {
            Ok(mode) => mode,
            Err(_) if confirm_copy_fallback => {
                if let Err(error) = remove_created_entry(&expected.logical_path).and_then(|_| {
                    copy_atomically(
                        &plan.public.library_path,
                        &expected.logical_path,
                        plan_id,
                        &plan.public.name,
                        &plan.public.fingerprint,
                    )
                }) {
                    rollback_current_and_previous(
                        &expected.logical_path,
                        &backup,
                        &replacements,
                        &package_root,
                    )?;
                    return Err(error);
                }
                DeploymentMode::CopyFallback
            }
            Err(error) => {
                rollback_current_and_previous(
                    &expected.logical_path,
                    &backup,
                    &replacements,
                    &package_root,
                )?;
                return Err(SkillError::new(
                    SkillErrorCode::CopyFallbackRequired,
                    format!("Could not create the preferred linked Installation: {error}"),
                    Some(expected.logical_path.clone()),
                ));
            }
        };
        replacements.push(Replacement {
            logical_path: expected.logical_path.clone(),
            backup,
        });
        installations.push(Installation {
            agent: expected.agent,
            logical_path: expected.logical_path.clone(),
            resolved_target: if mode == DeploymentMode::CopyFallback {
                expected.logical_path.clone()
            } else {
                plan.public.library_path.clone()
            },
            deployment_mode: mode,
            enabled: expected.enabled,
            last_known_fingerprint: plan.public.fingerprint.clone(),
            configuration_provenance: expected.configuration_provenance.clone(),
        });
    }

    let package = package_from_snapshot(
        format!("adopt-{}", next_id()),
        plan.public.name,
        plan.public.library_path,
        plan.public.fingerprint,
        installations,
    );
    state.packages.push(package.clone());
    if let Err(error) = saver(&state) {
        rollback_adoption(&replacements, &package_root)?;
        return Err(error);
    }
    // ponytail: committed backups are best-effort cleanup; startup recovery can own stale debris if observed.
    for replacement in replacements {
        let _ = remove_created_entry(&replacement.backup);
    }
    let _ = remove_plan_staging_path(app_data, plan_id);
    Ok(AdoptionResult {
        package,
        restart_message: RESTART_MESSAGE,
    })
}

fn commit_legacy_pending(
    app_data: &Path,
    plan: PendingLegacyMigration,
) -> Result<AdoptionResult, SkillError> {
    let staged = skill::validate_installed_revision(&plan.staged_path, &plan.public.name)?;
    if staged.fingerprint != plan.public.fingerprint {
        return Err(source_changed(&plan.staged_path));
    }
    let expected = SelectedExternal {
        identity: ExternalInstallationIdentity {
            agent: Agent::Codex,
            logical_path: plan.public.legacy_path.clone(),
        },
        resolved_target: plan.public.resolved_target,
        kind: plan.kind,
        name: plan.public.name.clone(),
        fingerprint: plan.public.fingerprint.clone(),
    };
    ensure_selected_fresh(&expected, &plan.roots, true)?;
    let mut state = load_writable_state(app_data)?;
    ensure_unique_state_name(&state.packages, &plan.public.name)?;
    ensure_library_absent(&plan.public.library_path)?;
    move_staging_to_library(&plan.staged_path, &plan.public.library_path)?;
    let package_root = plan
        .public
        .library_path
        .parent()
        .expect("library path has a package root")
        .to_path_buf();
    if let Err(error) = remove_plan_staging_path(app_data, &plan.public.id) {
        remove_directory(&package_root)?;
        return Err(error);
    }
    let package = package_from_snapshot(
        format!("legacy-{}", next_id()),
        plan.public.name,
        plan.public.library_path,
        plan.public.fingerprint,
        Vec::new(),
    );
    state.packages.push(package.clone());
    if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
        remove_directory(&package_root)?;
        return Err(error);
    }
    Ok(AdoptionResult {
        package,
        restart_message: RESTART_MESSAGE,
    })
}

fn package_from_snapshot(
    id: String,
    name: String,
    library_path: PathBuf,
    fingerprint: String,
    installations: Vec<Installation>,
) -> ManagedSkillPackage {
    ManagedSkillPackage {
        id,
        name,
        library_path,
        source: SkillSource::LocalSnapshot,
        installed_revision: InstalledRevision {
            fingerprint,
            commit_oid: None,
        },
        previous_revision: None,
        installations,
    }
}

fn stage_external(
    app_data: &Path,
    id: &str,
    name: &str,
    fingerprint: &str,
    source: &Path,
) -> Result<PathBuf, SkillError> {
    let plan_root = app_data.join("staging").join(id);
    if plan_root.starts_with(source) {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Adoption staging cannot be placed inside the external Skill source",
            Some(source.to_path_buf()),
        ));
    }
    ensure_path_absent(&plan_root)?;
    let staged = plan_root.join(name);
    let result = (|| {
        copy_directory(source, &staged)?;
        let validated = skill::validate_installed_revision(&staged, name)?;
        if validated.fingerprint != fingerprint {
            return Err(source_changed(source));
        }
        Ok(staged.clone())
    })();
    if result.is_err() {
        remove_plan_staging(&staged)?;
    }
    result
}

fn move_staging_to_library(staged: &Path, library_path: &Path) -> Result<(), SkillError> {
    let package_root = library_path
        .parent()
        .expect("library path has a package root");
    fs::create_dir_all(package_root).map_err(|error| SkillError::io(package_root, error))?;
    if let Err(error) = fs::rename(staged, library_path) {
        remove_directory(package_root)?;
        return Err(SkillError::io(staged, error));
    }
    Ok(())
}

#[derive(Debug)]
struct Replacement {
    logical_path: PathBuf,
    backup: PathBuf,
}

fn rollback_adoption(replacements: &[Replacement], package_root: &Path) -> Result<(), SkillError> {
    let mut first_error = None;
    for replacement in replacements.iter().rev() {
        if let Err(error) = remove_created_entry(&replacement.logical_path) {
            first_error.get_or_insert(error);
            continue;
        }
        if let Err(error) = fs::rename(&replacement.backup, &replacement.logical_path) {
            first_error.get_or_insert_with(|| SkillError::io(&replacement.logical_path, error));
        }
    }
    if let Err(error) = remove_directory(package_root) {
        first_error.get_or_insert(error);
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn restore_current_backup(logical_path: &Path, backup: &Path) -> Result<(), SkillError> {
    remove_created_entry(logical_path)?;
    fs::rename(backup, logical_path).map_err(|error| SkillError::io(logical_path, error))
}

fn rollback_current_and_previous(
    logical_path: &Path,
    backup: &Path,
    replacements: &[Replacement],
    package_root: &Path,
) -> Result<(), SkillError> {
    let current = restore_current_backup(logical_path, backup);
    let previous = rollback_adoption(replacements, package_root);
    current.and(previous)
}

fn library_path(app_data: &Path, name: &str) -> PathBuf {
    app_data.join("library").join(name).join("current")
}

fn ensure_library_absent(library_path: &Path) -> Result<(), SkillError> {
    let package_root = library_path
        .parent()
        .expect("library path has a package root");
    ensure_path_absent(package_root)
}

fn ensure_path_absent(path: &Path) -> Result<(), SkillError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SkillError::io(path, error)),
        Ok(_) => Err(SkillError::new(
            SkillErrorCode::Conflict,
            "A filesystem entry already exists at the transaction path",
            Some(path.to_path_buf()),
        )),
    }
}

fn remove_plan_staging(staged_path: &Path) -> Result<(), SkillError> {
    let plan_root = staged_path.parent().expect("staged Skill has a plan root");
    remove_directory(plan_root)
}

fn remove_plan_staging_path(app_data: &Path, plan_id: &str) -> Result<(), SkillError> {
    remove_directory(&app_data.join("staging").join(plan_id))
}

fn load_writable_state(app_data: &Path) -> Result<crate::state::AppState, SkillError> {
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    if loaded.mode == StateMode::ReadOnlyRecovery {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Application state is in read-only recovery mode",
            Some(app_data.join("state.json")),
        ));
    }
    Ok(loaded.state.expect("writable state mode contains state"))
}

fn source_changed(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        "The selected external Installation changed after the plan was created",
        Some(path.to_path_buf()),
    )
}

fn invalid_external(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        "Only a healthy external directory or link can be adopted or migrated",
        Some(path.to_path_buf()),
    )
}

fn invalid_plan() -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        "The Adoption or Legacy Migration plan is missing, has the wrong kind, or was already committed",
        None,
    )
}

fn busy_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Busy,
        "Another Skill Deck mutation is already running",
        None,
    )
}

fn lock_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Io,
        "The in-memory Adoption plan store is unavailable",
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::install::InstallManager;
    #[cfg(unix)]
    use crate::state::AppState;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        app_data: PathBuf,
        roots: AgentRoots,
    }

    fn fixture() -> Fixture {
        let temp = TempDir::new().unwrap();
        let roots = AgentRoots {
            codex: temp.path().join("home/.agents/skills"),
            claude: temp.path().join("home/.claude/skills"),
            codex_legacy: temp.path().join("home/.codex/skills"),
        };
        fs::create_dir_all(&roots.codex).unwrap();
        fs::create_dir_all(&roots.claude).unwrap();
        fs::create_dir_all(&roots.codex_legacy).unwrap();
        Fixture {
            app_data: temp.path().join("app-data"),
            roots,
            _temp: temp,
        }
    }

    fn write_skill(path: &Path, body: &str) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("SKILL.md"),
            format!("---\nname: alpha-skill\ndescription: Adoption fixture\n---\n{body}"),
        )
        .unwrap();
    }

    fn identity(agent: Agent, logical_path: PathBuf) -> ExternalInstallationIdentity {
        ExternalInstallationIdentity {
            agent,
            logical_path,
        }
    }

    #[cfg(unix)]
    #[test]
    fn adopts_third_party_link_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let fixture = fixture();
        let external = fixture._temp.path().join("third-party/alpha-skill");
        write_skill(&external, "original");
        let logical = fixture.roots.claude.join("alpha-skill");
        symlink(&external, &logical).unwrap();
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![identity(Agent::Claude, logical.clone())],
                fixture.roots.clone(),
            )
            .unwrap();

        let result = manager
            .commit_adoption_with_linker(&fixture.app_data, &plan.id, false, create_preferred_link)
            .unwrap();

        assert!(external.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_to_string(external.join("SKILL.md")).unwrap(),
            "---\nname: alpha-skill\ndescription: Adoption fixture\n---\noriginal"
        );
        assert_eq!(
            logical.canonicalize().unwrap(),
            result.package.library_path.canonicalize().unwrap()
        );
        assert_eq!(result.package.installations.len(), 1);
        assert_eq!(result.package.source, SkillSource::LocalSnapshot);
    }

    #[cfg(unix)]
    #[test]
    fn grouped_same_fingerprint_adopts_both_agents() {
        let fixture = fixture();
        let codex = fixture.roots.codex.join("alpha-skill");
        let claude = fixture.roots.claude.join("alpha-skill");
        write_skill(&codex, "same");
        write_skill(&claude, "same");
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![
                    identity(Agent::Codex, codex.clone()),
                    identity(Agent::Claude, claude.clone()),
                ],
                fixture.roots,
            )
            .unwrap();

        let result = manager
            .commit_adoption_with_linker(&fixture.app_data, &plan.id, false, create_preferred_link)
            .unwrap();

        assert_eq!(result.package.installations.len(), 2);
        assert_eq!(
            codex.canonicalize().unwrap(),
            claude.canonicalize().unwrap()
        );
    }

    #[test]
    fn grouped_different_fingerprint_is_rejected_without_writes() {
        let fixture = fixture();
        let codex = fixture.roots.codex.join("alpha-skill");
        let claude = fixture.roots.claude.join("alpha-skill");
        write_skill(&codex, "one");
        write_skill(&claude, "two");

        let error = AdoptionManager::default()
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![
                    identity(Agent::Codex, codex.clone()),
                    identity(Agent::Claude, claude.clone()),
                ],
                fixture.roots,
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Conflict);
        assert!(codex.join("SKILL.md").is_file());
        assert!(claude.join("SKILL.md").is_file());
        assert!(!fixture.app_data.join("library/alpha-skill").exists());
    }

    #[test]
    fn changed_external_entry_invalidates_plan_before_replacement() {
        let fixture = fixture();
        let logical = fixture.roots.claude.join("alpha-skill");
        write_skill(&logical, "before");
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![identity(Agent::Claude, logical.clone())],
                fixture.roots,
            )
            .unwrap();
        fs::write(logical.join("extra.txt"), "changed").unwrap();

        let error = manager
            .commit_adoption_with_linker(&fixture.app_data, &plan.id, false, create_preferred_link)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::SourceChanged);
        assert!(logical.join("extra.txt").is_file());
        assert!(!fixture.app_data.join("library/alpha-skill").exists());
    }

    #[cfg(unix)]
    #[test]
    fn second_replacement_failure_restores_every_external_entry() {
        let fixture = fixture();
        let codex = fixture.roots.codex.join("alpha-skill");
        let claude = fixture.roots.claude.join("alpha-skill");
        write_skill(&codex, "same");
        write_skill(&claude, "same");
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![
                    identity(Agent::Codex, codex.clone()),
                    identity(Agent::Claude, claude.clone()),
                ],
                fixture.roots,
            )
            .unwrap();
        let mut calls = 0;

        let error = manager
            .commit_adoption_with_linker(
                &fixture.app_data,
                &plan.id,
                false,
                |source, destination| {
                    calls += 1;
                    if calls == 2 {
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
                    } else {
                        create_preferred_link(source, destination)
                    }
                },
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::CopyFallbackRequired);
        assert!(!fs::symlink_metadata(&codex)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(!fs::symlink_metadata(&claude)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(codex.join("SKILL.md").is_file());
        assert!(claude.join("SKILL.md").is_file());
        assert!(!fixture.app_data.join("library/alpha-skill").exists());
        assert_eq!(
            StateStore::new(fixture.app_data)
                .load()
                .unwrap()
                .state
                .unwrap(),
            AppState::default()
        );
    }

    #[cfg(unix)]
    #[test]
    fn state_save_failure_restores_original_entries_and_external_target() {
        use std::os::unix::fs::symlink;

        let fixture = fixture();
        let codex = fixture.roots.codex.join("alpha-skill");
        let external = fixture._temp.path().join("third-party/alpha-skill");
        let claude = fixture.roots.claude.join("alpha-skill");
        write_skill(&codex, "same");
        write_skill(&external, "same");
        symlink(&external, &claude).unwrap();
        let original_external = fs::read(external.join("SKILL.md")).unwrap();
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![
                    identity(Agent::Codex, codex.clone()),
                    identity(Agent::Claude, claude.clone()),
                ],
                fixture.roots,
            )
            .unwrap();

        let error = manager
            .commit_adoption_with_saver(
                &fixture.app_data,
                &plan.id,
                false,
                &mut create_preferred_link,
                &mut |_| {
                    Err(SkillError::new(
                        SkillErrorCode::Io,
                        "injected state failure",
                        None,
                    ))
                },
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Io);
        assert!(!fs::symlink_metadata(&codex)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(fs::symlink_metadata(&claude)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            claude.canonicalize().unwrap(),
            external.canonicalize().unwrap()
        );
        assert_eq!(
            fs::read(external.join("SKILL.md")).unwrap(),
            original_external
        );
        assert!(!fixture.app_data.join("library/alpha-skill").exists());
        assert_eq!(
            StateStore::new(fixture.app_data)
                .load()
                .unwrap()
                .state
                .unwrap(),
            AppState::default()
        );
    }

    #[test]
    fn legacy_migration_adds_only_to_library_and_keeps_install_blocked() {
        let fixture = fixture();
        let legacy = fixture.roots.codex_legacy.join("alpha-skill");
        write_skill(&legacy, "legacy");
        let manager = AdoptionManager::default();
        let plan = manager
            .plan_legacy_for_roots(&fixture.app_data, legacy.clone(), fixture.roots.clone())
            .unwrap();

        let result = manager
            .commit_legacy_migration(&fixture.app_data, &plan.id)
            .unwrap();

        assert!(legacy.join("SKILL.md").is_file());
        assert!(result.package.installations.is_empty());
        assert_eq!(result.package.source, SkillSource::LocalSnapshot);
        let error = InstallManager::default()
            .plan_for_roots(
                &fixture.app_data,
                &result.package.id,
                vec![Agent::Codex],
                false,
                fixture.roots,
            )
            .unwrap_err();
        assert_eq!(error.code, SkillErrorCode::LegacyConflict);
    }

    #[test]
    fn plan_contract_is_camel_case_and_preserves_external_configuration() {
        let fixture = fixture();
        let logical = fixture.roots.claude.join("alpha-skill");
        write_skill(&logical, "configured");
        let settings = fixture.roots.claude.parent().unwrap().join("settings.json");
        fs::write(&settings, r#"{"skillOverrides":{"alpha-skill":"off"}}"#).unwrap();

        let plan = AdoptionManager::default()
            .plan_adoption_for_roots(
                &fixture.app_data,
                vec![identity(Agent::Claude, logical)],
                fixture.roots,
            )
            .unwrap();
        let json = serde_json::to_value(plan).unwrap();

        assert_eq!(json["installations"][0]["agent"], "claude");
        assert_eq!(json["installations"][0]["enabled"], false);
        assert_eq!(
            json["installations"][0]["configurationProvenance"]["owner"],
            "external"
        );
        assert_eq!(
            json["installations"][0]["configurationProvenance"]["path"],
            settings.to_string_lossy().as_ref()
        );
        assert!(json["installations"][0]
            .get("configuration_provenance")
            .is_none());
    }
}
