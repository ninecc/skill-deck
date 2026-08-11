use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{
    configuration,
    install::RESTART_MESSAGE,
    inventory::{self, Agent, ManagedInstallationStatus},
    library::{copy_directory, next_id, remove_directory},
    skill::{self, SkillError, SkillErrorCode},
    state::{
        AppState, ConfigurationProvenance, DeploymentMode, Installation, InstalledRevision,
        ManagedSkillPackage, SkillSource, StateMode, StateStore,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallPlan {
    pub id: String,
    pub package_id: String,
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub deployment_mode: DeploymentMode,
    pub cleans_owned_configuration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachPlan {
    pub id: String,
    pub package_id: String,
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub deployment_mode: DeploymentMode,
    pub keeps_configuration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgetInstallationPlan {
    pub id: String,
    pub package_id: String,
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub status: ManagedInstallationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveLibraryPlan {
    pub id: String,
    pub package_id: String,
    pub name: String,
    pub source: SkillSource,
    pub current_revision: InstalledRevision,
    pub previous_revision: Option<InstalledRevision>,
    pub library_path: PathBuf,
    pub bytes: u64,
    pub root_existed: bool,
    pub local_snapshot_last_copy_warning: bool,
    pub export_current_path: PathBuf,
    pub unrecoverable_content_warning: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleResult {
    pub package: Option<ManagedSkillPackage>,
    pub restart_message: &'static str,
}

#[derive(Debug, Clone)]
enum PendingPlan {
    Uninstall(UninstallPlan),
    Detach(DetachPlan),
    Forget(ForgetInstallationPlan),
    Remove(Box<RemoveLibraryPlan>),
}

#[derive(Default)]
pub struct LifecycleManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl LifecycleManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_uninstall(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
    ) -> Result<UninstallPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let (package, installation) = find_installation(&state, package_id, agent)?;
        validate_installation(package, installation)?;
        configuration::validate_owned_configuration(&package.name, installation)?;
        let plan = UninstallPlan {
            id: next_id(),
            package_id: package_id.to_owned(),
            agent,
            logical_path: installation.logical_path.clone(),
            deployment_mode: installation.deployment_mode,
            cleans_owned_configuration: matches!(
                installation.configuration_provenance,
                ConfigurationProvenance::SkillDeck { .. }
            ),
        };
        self.insert(plan.id.clone(), PendingPlan::Uninstall(plan.clone()))?;
        Ok(plan)
    }

    pub fn commit_uninstall(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<LifecycleResult, SkillError> {
        self.commit_uninstall_with_saver(app_data, plan_id, |state| {
            StateStore::new(app_data.to_path_buf()).save(state)
        })
    }

    fn commit_uninstall_with_saver<F>(
        &self,
        app_data: &Path,
        plan_id: &str,
        save: F,
    ) -> Result<LifecycleResult, SkillError>
    where
        F: FnOnce(&AppState) -> Result<(), SkillError>,
    {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Uninstall(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Uninstall"));
        };
        let mut state = load_writable_state(app_data)?;
        let (package_index, installation_index) = indexes(&state, &plan.package_id, plan.agent)?;
        let package = &state.packages[package_index];
        let installation = &package.installations[installation_index];
        ensure_plan_matches(
            &plan.logical_path,
            plan.deployment_mode,
            installation,
            "Uninstall",
        )?;
        validate_installation(package, installation)?;
        configuration::validate_owned_configuration(&package.name, installation)?;

        let original_state = state.clone();
        let backup = sibling_backup(&installation.logical_path, plan_id);
        ensure_absent(&backup)?;
        fs::rename(&installation.logical_path, &backup)
            .map_err(|error| SkillError::io(&installation.logical_path, error))?;
        let cleanup = match configuration::cleanup_owned_configuration(&package.name, installation)
        {
            Ok(cleanup) => cleanup,
            Err(error) => {
                restore_entry(&backup, &installation.logical_path)?;
                return Err(error);
            }
        };
        state.packages[package_index]
            .installations
            .remove(installation_index);
        if let Err(error) = save(&state) {
            configuration::restore_configuration_cleanup(cleanup.as_ref())?;
            restore_entry(&backup, &plan.logical_path)?;
            return Err(error);
        }
        if let Err(error) = remove_entry(&backup) {
            rollback_committed_state(app_data, &original_state, cleanup.as_ref())?;
            restore_entry(&backup, &plan.logical_path)?;
            return Err(error);
        }
        Ok(LifecycleResult {
            package: Some(state.packages[package_index].clone()),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn plan_forget_installation(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
    ) -> Result<ForgetInstallationPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let (package, installation) = find_installation(&state, package_id, agent)?;
        let reconciliation = inventory::reconcile_installation(package, installation);
        ensure_forgettable(reconciliation.status, &installation.logical_path)?;
        let plan = ForgetInstallationPlan {
            id: next_id(),
            package_id: package_id.to_owned(),
            agent,
            logical_path: installation.logical_path.clone(),
            status: reconciliation.status,
        };
        self.insert(plan.id.clone(), PendingPlan::Forget(plan.clone()))?;
        Ok(plan)
    }

    pub fn commit_forget_installation(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<LifecycleResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Forget(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Forget Installation"));
        };
        let mut state = load_writable_state(app_data)?;
        let (package_index, installation_index) = indexes(&state, &plan.package_id, plan.agent)?;
        let package = &state.packages[package_index];
        let installation = &package.installations[installation_index];
        if installation.logical_path != plan.logical_path {
            return Err(stale_plan("Forget Installation"));
        }
        let reconciliation = inventory::reconcile_installation(package, installation);
        if !matches!(
            reconciliation.status,
            ManagedInstallationStatus::Missing
                | ManagedInstallationStatus::Retargeted
                | ManagedInstallationStatus::Broken
        ) {
            return Err(stale_plan("Forget Installation"));
        }
        state.packages[package_index]
            .installations
            .remove(installation_index);
        StateStore::new(app_data.to_path_buf()).save(&state)?;
        Ok(LifecycleResult {
            package: Some(state.packages[package_index].clone()),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn plan_detach(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
    ) -> Result<DetachPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let (package, installation) = find_installation(&state, package_id, agent)?;
        validate_detachable(package, installation)?;
        let plan = DetachPlan {
            id: next_id(),
            package_id: package_id.to_owned(),
            agent,
            logical_path: installation.logical_path.clone(),
            deployment_mode: installation.deployment_mode,
            keeps_configuration: true,
        };
        self.insert(plan.id.clone(), PendingPlan::Detach(plan.clone()))?;
        Ok(plan)
    }

    pub fn commit_detach(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<LifecycleResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Detach(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Detach"));
        };
        let mut state = load_writable_state(app_data)?;
        let (package_index, installation_index) = indexes(&state, &plan.package_id, plan.agent)?;
        let package = &state.packages[package_index];
        let installation = &package.installations[installation_index];
        ensure_plan_matches(
            &plan.logical_path,
            plan.deployment_mode,
            installation,
            "Detach",
        )?;
        validate_detachable(package, installation)?;

        let original_state = state.clone();
        let linked = installation.deployment_mode != DeploymentMode::CopyFallback;
        let backup = sibling_backup(&installation.logical_path, plan_id);
        let staging = sibling_staging(&installation.logical_path, plan_id);
        if linked {
            ensure_absent(&backup)?;
            ensure_absent(&staging)?;
            if let Err(error) = prepare_standalone(package, &staging) {
                let _ = remove_entry(&staging);
                return Err(error);
            }
            if let Err(error) = fs::rename(&installation.logical_path, &backup) {
                remove_entry(&staging)?;
                return Err(SkillError::io(&installation.logical_path, error));
            }
            if let Err(error) = fs::rename(&staging, &installation.logical_path) {
                restore_entry(&backup, &installation.logical_path)?;
                remove_entry(&staging)?;
                return Err(SkillError::io(&installation.logical_path, error));
            }
        }
        state.packages[package_index]
            .installations
            .remove(installation_index);
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            if linked {
                remove_entry(&plan.logical_path)?;
                restore_entry(&backup, &plan.logical_path)?;
            }
            return Err(error);
        }
        if linked {
            if let Err(error) = remove_entry(&backup) {
                StateStore::new(app_data.to_path_buf()).save(&original_state)?;
                remove_entry(&plan.logical_path)?;
                restore_entry(&backup, &plan.logical_path)?;
                return Err(error);
            }
        }
        Ok(LifecycleResult {
            package: Some(state.packages[package_index].clone()),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn plan_remove_library(
        &self,
        app_data: &Path,
        package_id: &str,
    ) -> Result<RemoveLibraryPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?;
        ensure_removable(app_data, package)?;
        let package_root = package_root(app_data, package)?;
        ensure_safe_package_root(&package_root)?;
        let root_exists = package_root.exists();
        let plan = RemoveLibraryPlan {
            id: next_id(),
            package_id: package_id.to_owned(),
            name: package.name.clone(),
            source: package.source.clone(),
            current_revision: package.installed_revision.clone(),
            previous_revision: package.previous_revision.clone(),
            library_path: package_root.clone(),
            bytes: if root_exists {
                directory_bytes(&package_root)?
            } else {
                0
            },
            root_existed: root_exists,
            local_snapshot_last_copy_warning: package.source == SkillSource::LocalSnapshot,
            export_current_path: package.library_path.clone(),
            unrecoverable_content_warning: validate_library(package).is_err(),
        };
        self.insert(plan.id.clone(), PendingPlan::Remove(Box::new(plan.clone())))?;
        Ok(plan)
    }

    pub fn commit_remove_library(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirmation_name: &str,
    ) -> Result<LifecycleResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Remove(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Remove from Library"));
        };
        if confirmation_name != plan.name {
            return Err(SkillError::new(
                SkillErrorCode::InvalidPlan,
                "The confirmation name must exactly match the Managed Skill Package name",
                None,
            ));
        }
        let mut state = load_writable_state(app_data)?;
        let package_index = state
            .packages
            .iter()
            .position(|package| package.id == plan.package_id)
            .ok_or_else(|| invalid_plan("Remove from Library"))?;
        let package = &state.packages[package_index];
        ensure_removable(app_data, package)?;
        let root = package_root(app_data, package)?;
        ensure_safe_package_root(&root)?;
        let root_exists = root.exists();
        let bytes = if root_exists {
            directory_bytes(&root)?
        } else {
            0
        };
        let unrecoverable_content_warning = validate_library(package).is_err();
        if root != plan.library_path
            || package.name != plan.name
            || package.source != plan.source
            || package.installed_revision != plan.current_revision
            || package.previous_revision != plan.previous_revision
            || root_exists != plan.root_existed
            || bytes != plan.bytes
            || unrecoverable_content_warning != plan.unrecoverable_content_warning
        {
            return Err(stale_plan("Remove from Library"));
        }

        let original_state = state.clone();
        let backup = app_data.join("staging").join(format!("remove-{plan_id}"));
        ensure_absent(&backup)?;
        if root_exists {
            if let Some(parent) = backup.parent() {
                fs::create_dir_all(parent).map_err(|error| SkillError::io(parent, error))?;
            }
            fs::rename(&root, &backup).map_err(|error| SkillError::io(&root, error))?;
        }
        state.packages.remove(package_index);
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            if root_exists {
                restore_entry(&backup, &root)?;
            }
            return Err(error);
        }
        if root_exists {
            if let Err(error) = remove_entry(&backup) {
                StateStore::new(app_data.to_path_buf()).save(&original_state)?;
                restore_entry(&backup, &root)?;
                return Err(error);
            }
        }
        Ok(LifecycleResult {
            package: None,
            restart_message: RESTART_MESSAGE,
        })
    }

    fn insert(&self, id: String, plan: PendingPlan) -> Result<(), SkillError> {
        self.plans
            .lock()
            .map_err(|_| lock_error())?
            .insert(id, plan);
        Ok(())
    }

    fn take(&self, id: &str) -> Result<PendingPlan, SkillError> {
        self.plans
            .lock()
            .map_err(|_| lock_error())?
            .remove(id)
            .ok_or_else(|| invalid_plan("Lifecycle"))
    }
}

fn validate_detachable(
    package: &ManagedSkillPackage,
    installation: &Installation,
) -> Result<(), SkillError> {
    let reconciliation = inventory::reconcile_installation(package, installation);
    if matches!(
        reconciliation.status,
        ManagedInstallationStatus::Healthy | ManagedInstallationStatus::Drifted
    ) {
        Ok(())
    } else {
        Err(reconciliation
            .diagnostic
            .map(|diagnostic| SkillError::new(diagnostic.code, diagnostic.message, diagnostic.path))
            .unwrap_or_else(|| unknown_topology(&installation.logical_path)))
    }
}

fn ensure_forgettable(status: ManagedInstallationStatus, path: &Path) -> Result<(), SkillError> {
    if matches!(
        status,
        ManagedInstallationStatus::Missing
            | ManagedInstallationStatus::Retargeted
            | ManagedInstallationStatus::Broken
    ) {
        Ok(())
    } else {
        Err(SkillError::new(
            SkillErrorCode::Conflict,
            "Forget Installation is available only for missing, retargeted, or broken managed installations",
            Some(path.to_path_buf()),
        ))
    }
}

pub(crate) fn validate_installation(
    package: &ManagedSkillPackage,
    installation: &Installation,
) -> Result<(), SkillError> {
    validate_library(package)?;
    let metadata = fs::symlink_metadata(&installation.logical_path)
        .map_err(|error| topology_error(&installation.logical_path, error))?;
    match installation.deployment_mode {
        DeploymentMode::CopyFallback => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(unknown_topology(&installation.logical_path));
            }
            let installed =
                skill::validate_installed_revision(&installation.logical_path, &package.name)?;
            if installed.fingerprint != installation.last_known_fingerprint
                || installed.fingerprint != package.installed_revision.fingerprint
            {
                return Err(content_drift(&installation.logical_path));
            }
        }
        DeploymentMode::Symlink => {
            if !metadata.file_type().is_symlink() {
                return Err(unknown_topology(&installation.logical_path));
            }
            ensure_link_target(package, installation)?;
        }
        DeploymentMode::Junction => {
            #[cfg(not(windows))]
            return Err(unknown_topology(&installation.logical_path));
            #[cfg(windows)]
            {
                if !metadata.is_dir() {
                    return Err(unknown_topology(&installation.logical_path));
                }
                ensure_link_target(package, installation)?;
            }
        }
    }
    Ok(())
}

fn ensure_link_target(
    package: &ManagedSkillPackage,
    installation: &Installation,
) -> Result<(), SkillError> {
    let actual = installation
        .logical_path
        .canonicalize()
        .map_err(|error| topology_error(&installation.logical_path, error))?;
    let expected = package
        .library_path
        .canonicalize()
        .map_err(|error| SkillError::io(&package.library_path, error))?;
    let recorded = installation
        .resolved_target
        .canonicalize()
        .map_err(|error| topology_error(&installation.resolved_target, error))?;
    if actual != expected || recorded != expected {
        return Err(unknown_topology(&installation.logical_path));
    }
    Ok(())
}

fn validate_library(package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let validated = skill::validate_installed_revision(&package.library_path, &package.name)?;
    if validated.fingerprint != package.installed_revision.fingerprint {
        return Err(content_drift(&package.library_path));
    }
    Ok(())
}

fn prepare_standalone(package: &ManagedSkillPackage, staging: &Path) -> Result<(), SkillError> {
    copy_directory(&package.library_path, staging)?;
    let copied = skill::validate_installed_revision(staging, &package.name)?;
    if copied.fingerprint != package.installed_revision.fingerprint {
        remove_directory(staging)?;
        return Err(content_drift(&package.library_path));
    }
    Ok(())
}

fn load_writable_state(app_data: &Path) -> Result<AppState, SkillError> {
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

fn find_package<'a>(
    state: &'a AppState,
    package_id: &str,
) -> Result<&'a ManagedSkillPackage, SkillError> {
    state
        .packages
        .iter()
        .find(|package| package.id == package_id)
        .ok_or_else(|| invalid_plan("Lifecycle"))
}

fn find_installation<'a>(
    state: &'a AppState,
    package_id: &str,
    agent: Agent,
) -> Result<(&'a ManagedSkillPackage, &'a Installation), SkillError> {
    let package = find_package(state, package_id)?;
    let installation = package
        .installations
        .iter()
        .find(|installation| installation.agent == agent)
        .ok_or_else(|| invalid_plan("Lifecycle"))?;
    Ok((package, installation))
}

fn indexes(state: &AppState, package_id: &str, agent: Agent) -> Result<(usize, usize), SkillError> {
    let package = state
        .packages
        .iter()
        .position(|package| package.id == package_id)
        .ok_or_else(|| invalid_plan("Lifecycle"))?;
    let installation = state.packages[package]
        .installations
        .iter()
        .position(|installation| installation.agent == agent)
        .ok_or_else(|| invalid_plan("Lifecycle"))?;
    Ok((package, installation))
}

fn ensure_plan_matches(
    path: &Path,
    mode: DeploymentMode,
    installation: &Installation,
    operation: &str,
) -> Result<(), SkillError> {
    if path != installation.logical_path || mode != installation.deployment_mode {
        return Err(stale_plan(operation));
    }
    Ok(())
}

fn ensure_removable(app_data: &Path, package: &ManagedSkillPackage) -> Result<(), SkillError> {
    if !package.installations.is_empty() {
        return Err(SkillError::new(
            SkillErrorCode::Conflict,
            "Remove from Library requires a Managed Skill Package with zero installations",
            Some(package.library_path.clone()),
        ));
    }
    package_root(app_data, package).map(|_| ())
}

fn package_root(app_data: &Path, package: &ManagedSkillPackage) -> Result<PathBuf, SkillError> {
    if !skill::valid_name(&package.name) {
        return Err(SkillError::new(
            SkillErrorCode::InvalidMetadata,
            "Managed Skill Package name is invalid",
            Some(package.library_path.clone()),
        ));
    }
    let expected = app_data.join("library").join(&package.name);
    if package.library_path != expected.join("current") {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Managed Library path is outside the owned package boundary",
            Some(package.library_path.clone()),
        ));
    }
    Ok(expected)
}

fn ensure_safe_package_root(root: &Path) -> Result<(), SkillError> {
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SkillError::io(root, error)),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(unknown_topology(root)),
    }
}

fn directory_bytes(path: &Path) -> Result<u64, SkillError> {
    let mut total = 0u64;
    for entry in fs::read_dir(path).map_err(|error| SkillError::io(path, error))? {
        let entry = entry.map_err(|error| SkillError::io(path, error))?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)
            .map_err(|error| SkillError::io(&entry_path, error))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            total = total.saturating_add(directory_bytes(&entry_path)?);
        } else if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

fn sibling_backup(path: &Path, id: &str) -> PathBuf {
    path.parent()
        .expect("managed installation has an Agent root")
        .join(format!(".skill-deck-backup-{id}"))
}

fn sibling_staging(path: &Path, id: &str) -> PathBuf {
    path.parent()
        .expect("managed installation has an Agent root")
        .join(format!(".skill-deck-detach-{id}"))
}

fn ensure_absent(path: &Path) -> Result<(), SkillError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SkillError::io(path, error)),
        Ok(_) => Err(SkillError::new(
            SkillErrorCode::Conflict,
            "A lifecycle transaction path already exists",
            Some(path.to_path_buf()),
        )),
    }
}

fn restore_entry(backup: &Path, logical: &Path) -> Result<(), SkillError> {
    fs::rename(backup, logical).map_err(|error| SkillError::io(logical, error))
}

fn remove_entry(path: &Path) -> Result<(), SkillError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(SkillError::io(path, error)),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).map_err(|error| SkillError::io(path, error))
    } else if metadata.is_dir() {
        remove_directory(path)
    } else {
        Err(unknown_topology(path))
    }
}

fn rollback_committed_state(
    app_data: &Path,
    state: &AppState,
    cleanup: Option<&configuration::ConfigurationCleanup>,
) -> Result<(), SkillError> {
    StateStore::new(app_data.to_path_buf()).save(state)?;
    configuration::restore_configuration_cleanup(cleanup)
}

fn topology_error(path: &Path, error: io::Error) -> SkillError {
    SkillError::new(
        SkillErrorCode::TopologyChanged,
        format!("Managed Installation topology is broken or unavailable: {error}"),
        Some(path.to_path_buf()),
    )
}

fn unknown_topology(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::TopologyChanged,
        "Managed Installation topology no longer matches its recorded deployment mode",
        Some(path.to_path_buf()),
    )
}

fn content_drift(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::ContentDrift,
        "Managed Skill content changed outside Skill Deck",
        Some(path.to_path_buf()),
    )
}

fn stale_plan(operation: &str) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        format!("The {operation} plan is stale because managed state changed"),
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
        "The in-memory lifecycle plan store is unavailable",
        None,
    )
}

fn invalid_plan(operation: &str) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        format!("The {operation} plan is missing, stale, or already committed"),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        app_data: PathBuf,
        package: ManagedSkillPackage,
        logical: PathBuf,
    }

    fn fixture(mode: DeploymentMode, provenance: ConfigurationProvenance) -> Fixture {
        #[cfg(unix)]
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        let library = app_data.join("library/alpha-skill/current");
        fs::create_dir_all(&library).unwrap();
        fs::write(
            library.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Lifecycle fixture\n---\n",
        )
        .unwrap();
        let fingerprint = skill::validate_installed_revision(&library, "alpha-skill")
            .unwrap()
            .fingerprint;
        let logical = temp.path().join("agent/skills/alpha-skill");
        fs::create_dir_all(logical.parent().unwrap()).unwrap();
        match mode {
            DeploymentMode::Symlink => {
                #[cfg(unix)]
                symlink(&library, &logical).unwrap();
                #[cfg(not(unix))]
                unreachable!();
            }
            DeploymentMode::CopyFallback => copy_directory(&library, &logical).unwrap(),
            DeploymentMode::Junction => unreachable!(),
        }
        let installation = Installation {
            agent: Agent::Claude,
            logical_path: logical.clone(),
            resolved_target: if mode == DeploymentMode::CopyFallback {
                logical.clone()
            } else {
                library.clone()
            },
            deployment_mode: mode,
            enabled: true,
            last_known_fingerprint: fingerprint.clone(),
            configuration_provenance: provenance,
        };
        let package = ManagedSkillPackage {
            id: "package-1".to_owned(),
            name: "alpha-skill".to_owned(),
            library_path: library,
            source: SkillSource::LocalSnapshot,
            installed_revision: InstalledRevision {
                fingerprint,
                commit_oid: None,
            },
            previous_revision: None,
            installations: vec![installation],
        };
        StateStore::new(app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![package.clone()],
            })
            .unwrap();
        Fixture {
            _temp: temp,
            app_data,
            package,
            logical,
        }
    }

    #[cfg(unix)]
    #[test]
    fn uninstall_link_removes_only_entry_and_keeps_package() {
        let fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        let result = manager
            .commit_uninstall(&fixture.app_data, &plan.id)
            .unwrap();

        assert!(fs::symlink_metadata(&fixture.logical).is_err());
        assert!(fixture.package.library_path.exists());
        assert!(result.package.unwrap().installations.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn detach_link_becomes_standalone_and_preserves_configuration() {
        let config = PathBuf::from("/external/settings.json");
        let fixture = fixture(
            DeploymentMode::Symlink,
            ConfigurationProvenance::External {
                path: config.clone(),
            },
        );
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_detach(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        manager.commit_detach(&fixture.app_data, &plan.id).unwrap();

        assert!(!fs::symlink_metadata(&fixture.logical)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(fixture.logical.join("SKILL.md").is_file());
        assert_eq!(
            fixture.package.installations[0].configuration_provenance,
            ConfigurationProvenance::External { path: config }
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_drift_blocks_uninstall_but_can_detach_without_touching_content() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        fs::write(fixture.logical.join("changed.txt"), "external edit").unwrap();
        let manager = LifecycleManager::default();

        assert_eq!(
            manager
                .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
                .unwrap_err()
                .code,
            SkillErrorCode::ContentDrift
        );
        let plan = manager
            .plan_detach(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        let result = manager.commit_detach(&fixture.app_data, &plan.id).unwrap();
        assert!(fixture.logical.join("changed.txt").is_file());
        assert!(result.package.unwrap().installations.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn forget_missing_installation_changes_only_state() {
        let fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        fs::remove_file(&fixture.logical).unwrap();
        let library_bytes = fs::read(fixture.package.library_path.join("SKILL.md")).unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_forget_installation(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        let result = manager
            .commit_forget_installation(&fixture.app_data, &plan.id)
            .unwrap();

        assert!(result.package.unwrap().installations.is_empty());
        assert_eq!(
            fs::read(fixture.package.library_path.join("SKILL.md")).unwrap(),
            library_bytes
        );
        assert!(!fixture.logical.exists());
    }

    #[cfg(unix)]
    #[test]
    fn forget_plan_is_stale_after_missing_installation_is_restored() {
        use std::os::unix::fs::symlink;

        let fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        fs::remove_file(&fixture.logical).unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_forget_installation(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        symlink(&fixture.package.library_path, &fixture.logical).unwrap();

        assert_eq!(
            manager
                .commit_forget_installation(&fixture.app_data, &plan.id)
                .unwrap_err()
                .code,
            SkillErrorCode::InvalidPlan
        );
    }

    #[cfg(unix)]
    #[test]
    fn retargeted_link_is_never_removed() {
        use std::os::unix::fs::symlink;

        let fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        fs::remove_file(&fixture.logical).unwrap();
        let external = fixture._temp.path().join("external");
        fs::create_dir(&external).unwrap();
        symlink(&external, &fixture.logical).unwrap();

        let error = LifecycleManager::default()
            .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::TopologyChanged);
        assert_eq!(
            fixture.logical.canonicalize().unwrap(),
            external.canonicalize().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn forgetting_retargeted_installation_preserves_link_target_and_configuration() {
        use std::os::unix::fs::symlink;

        let mut fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let config = fixture._temp.path().join("settings.json");
        let config_bytes = br#"{"skillOverrides":{"alpha-skill":"off"}}"#;
        fs::write(&config, config_bytes).unwrap();
        fixture.package.installations[0].configuration_provenance =
            ConfigurationProvenance::SkillDeck {
                path: config.clone(),
            };
        StateStore::new(fixture.app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![fixture.package.clone()],
            })
            .unwrap();
        fs::remove_file(&fixture.logical).unwrap();
        let external = fixture._temp.path().join("external");
        fs::create_dir(&external).unwrap();
        fs::write(external.join("keep.txt"), "keep").unwrap();
        symlink(&external, &fixture.logical).unwrap();
        let link_target = fs::read_link(&fixture.logical).unwrap();

        let manager = LifecycleManager::default();
        let plan = manager
            .plan_forget_installation(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        let result = manager
            .commit_forget_installation(&fixture.app_data, &plan.id)
            .unwrap();

        assert!(result.package.unwrap().installations.is_empty());
        assert_eq!(fs::read_link(&fixture.logical).unwrap(), link_target);
        assert_eq!(
            fs::read_to_string(external.join("keep.txt")).unwrap(),
            "keep"
        );
        assert_eq!(fs::read(config).unwrap(), config_bytes);
    }

    #[cfg(unix)]
    #[test]
    fn remove_library_requires_zero_installations_and_exact_name() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        let manager = LifecycleManager::default();
        assert_eq!(
            manager
                .plan_remove_library(&fixture.app_data, &fixture.package.id)
                .unwrap_err()
                .code,
            SkillErrorCode::Conflict
        );

        let mut state = StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap();
        state.packages[0].installations.clear();
        StateStore::new(fixture.app_data.clone())
            .save(&state)
            .unwrap();
        let plan = manager
            .plan_remove_library(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert!(plan.local_snapshot_last_copy_warning);
        assert!(plan.bytes > 0);
        assert_eq!(
            manager
                .commit_remove_library(&fixture.app_data, &plan.id, "Alpha-Skill")
                .unwrap_err()
                .code,
            SkillErrorCode::InvalidPlan
        );
        assert!(fixture.package.library_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn broken_zero_installation_package_can_be_removed_from_safe_owned_root() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        let mut state = StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap();
        state.packages[0].installations.clear();
        StateStore::new(fixture.app_data.clone())
            .save(&state)
            .unwrap();
        fs::remove_file(fixture.package.library_path.join("SKILL.md")).unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_remove_library(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert!(plan.unrecoverable_content_warning);
        manager
            .commit_remove_library(&fixture.app_data, &plan.id, "alpha-skill")
            .unwrap();
        assert!(!fixture.package.library_path.parent().unwrap().exists());
    }

    #[test]
    fn removing_an_absent_package_root_changes_only_state() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        let mut state = StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap();
        state.packages[0].installations.clear();
        StateStore::new(fixture.app_data.clone())
            .save(&state)
            .unwrap();
        fs::remove_dir_all(fixture.package.library_path.parent().unwrap()).unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_remove_library(&fixture.app_data, &fixture.package.id)
            .unwrap();

        manager
            .commit_remove_library(&fixture.app_data, &plan.id, "alpha-skill")
            .unwrap();

        assert!(StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap()
            .packages
            .is_empty());
        assert!(!fixture.app_data.join("staging").exists());
    }

    #[test]
    fn broken_package_name_cannot_escape_the_owned_library_boundary() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        let mut state = StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap();
        state.packages[0].installations.clear();
        state.packages[0].name = "../outside".to_owned();
        state.packages[0].library_path = fixture.app_data.join("library/../outside/current");
        StateStore::new(fixture.app_data.clone())
            .save(&state)
            .unwrap();

        assert_eq!(
            LifecycleManager::default()
                .plan_remove_library(&fixture.app_data, &fixture.package.id)
                .unwrap_err()
                .code,
            SkillErrorCode::InvalidMetadata
        );
    }

    #[cfg(unix)]
    #[test]
    fn uninstall_cleans_only_owned_claude_entry() {
        let mut fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let config = fixture._temp.path().join("home/.claude/settings.json");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        fs::write(
            &config,
            r#"{"skillOverrides":{"alpha-skill":"on"},"theme":"dark"}"#,
        )
        .unwrap();
        fixture.package.installations[0].configuration_provenance =
            ConfigurationProvenance::SkillDeck {
                path: config.clone(),
            };
        StateStore::new(fixture.app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![fixture.package.clone()],
            })
            .unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();

        manager
            .commit_uninstall(&fixture.app_data, &plan.id)
            .unwrap();

        let json: serde_json::Value = serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
        assert_eq!(json["theme"], "dark");
        assert!(json.get("skillOverrides").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn uninstall_preserves_external_configuration_bytes() {
        let mut fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let config = fixture._temp.path().join("home/.claude/settings.json");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let original = br#"{"skillOverrides":{"alpha-skill":"name-only"},"theme":"dark"}"#;
        fs::write(&config, original).unwrap();
        fixture.package.installations[0].configuration_provenance =
            ConfigurationProvenance::External {
                path: config.clone(),
            };
        StateStore::new(fixture.app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![fixture.package.clone()],
            })
            .unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();

        manager
            .commit_uninstall(&fixture.app_data, &plan.id)
            .unwrap();

        assert_eq!(fs::read(config).unwrap(), original);
    }

    #[cfg(unix)]
    #[test]
    fn uninstall_state_failure_restores_entry_configuration_and_state() {
        let mut fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let config = fixture._temp.path().join("home/.claude/settings.json");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let original = br#"{"skillOverrides":{"alpha-skill":"on"},"theme":"dark"}"#;
        fs::write(&config, original).unwrap();
        fixture.package.installations[0].configuration_provenance =
            ConfigurationProvenance::SkillDeck {
                path: config.clone(),
            };
        StateStore::new(fixture.app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![fixture.package.clone()],
            })
            .unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_uninstall(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();

        let error = manager
            .commit_uninstall_with_saver(&fixture.app_data, &plan.id, |_| {
                Err(SkillError::new(
                    SkillErrorCode::Io,
                    "injected state failure",
                    None,
                ))
            })
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Io);
        assert!(fs::symlink_metadata(&fixture.logical)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read(config).unwrap(), original);
        assert_eq!(
            StateStore::new(fixture.app_data)
                .load()
                .unwrap()
                .state
                .unwrap()
                .packages[0]
                .installations
                .len(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn lifecycle_plan_uses_camel_case_json() {
        let fixture = fixture(DeploymentMode::Symlink, ConfigurationProvenance::None);
        let plan = LifecycleManager::default()
            .plan_detach(&fixture.app_data, &fixture.package.id, Agent::Claude)
            .unwrap();
        let json = serde_json::to_value(plan).unwrap();

        assert_eq!(json["packageId"], "package-1");
        assert_eq!(
            json["logicalPath"],
            fixture.logical.to_string_lossy().as_ref()
        );
        assert_eq!(json["keepsConfiguration"], true);
        assert!(json.get("package_id").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn remove_library_deletes_owned_package_after_confirmation() {
        let fixture = fixture(DeploymentMode::CopyFallback, ConfigurationProvenance::None);
        let mut state = StateStore::new(fixture.app_data.clone())
            .load()
            .unwrap()
            .state
            .unwrap();
        state.packages[0].installations.clear();
        StateStore::new(fixture.app_data.clone())
            .save(&state)
            .unwrap();
        let manager = LifecycleManager::default();
        let plan = manager
            .plan_remove_library(&fixture.app_data, &fixture.package.id)
            .unwrap();

        let result = manager
            .commit_remove_library(&fixture.app_data, &plan.id, "alpha-skill")
            .unwrap();

        assert!(result.package.is_none());
        assert!(!fixture.package.library_path.exists());
        assert!(StateStore::new(fixture.app_data)
            .load()
            .unwrap()
            .state
            .unwrap()
            .packages
            .is_empty());
    }
}
