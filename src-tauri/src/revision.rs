use std::{
    collections::{BTreeSet, HashMap},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{
    install::RESTART_MESSAGE,
    inventory::Agent,
    library::{copy_directory, next_id, remove_directory},
    lifecycle,
    skill::{self, SkillError, SkillErrorCode, ValidatedSkill},
    state::{
        AppState, DeploymentMode, Installation, InstalledRevision, ManagedSkillPackage,
        SkillSource, StateMode, StateStore,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueChanges<T> {
    pub added: Vec<T>,
    pub removed: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDisclosure {
    pub scripts: ValueChanges<PathBuf>,
    pub references: ValueChanges<PathBuf>,
    pub unknown_fields: ValueChanges<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceLocalPlan {
    pub id: String,
    pub package_id: String,
    pub source_path: PathBuf,
    pub candidate: ValidatedSkill,
    pub changes: ChangeDisclosure,
    pub installation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRevisionPlan {
    pub id: String,
    pub package_id: String,
    pub from_revision: InstalledRevision,
    pub to_revision: InstalledRevision,
    pub changes: ChangeDisclosure,
    pub installation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRevisionPlan {
    pub id: String,
    pub package_id: String,
    pub destination: PathBuf,
    pub revision: InstalledRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreInstallationPlan {
    pub id: String,
    pub package_id: String,
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub expected_fingerprint: String,
    pub observed_fingerprint: String,
    pub will_overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionResult {
    pub package: ManagedSkillPackage,
    pub restart_message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRevisionResult {
    pub destination: PathBuf,
    pub fingerprint: String,
}

#[derive(Debug, Clone)]
struct PendingReplace {
    public: ReplaceLocalPlan,
    source_path: PathBuf,
    staged_path: PathBuf,
    package: ManagedSkillPackage,
}

#[derive(Debug, Clone)]
enum PendingPlan {
    Replace(Box<PendingReplace>),
    Rollback {
        public: RollbackRevisionPlan,
        package: ManagedSkillPackage,
    },
    Export {
        public: ExportRevisionPlan,
        package: ManagedSkillPackage,
    },
    Restore {
        public: RestoreInstallationPlan,
        package: ManagedSkillPackage,
    },
}

#[derive(Default)]
pub struct RevisionManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl RevisionManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_replace_local(
        &self,
        app_data: &Path,
        package_id: &str,
        source: &Path,
    ) -> Result<ReplaceLocalPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?.clone();
        if package.source != SkillSource::LocalSnapshot {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "Only a Local Snapshot source can be replaced with a local directory",
                Some(package.library_path),
            ));
        }
        validate_package(app_data, &package)?;
        validate_previous_slot(&package)?;

        let mut candidate = skill::validate_skill_dir(source)?;
        let source_path = source
            .canonicalize()
            .map_err(|error| SkillError::io(source, error))?;
        candidate.root = source_path.clone();
        if candidate.metadata.name != package.name {
            return Err(SkillError::new(
                SkillErrorCode::InvalidMetadata,
                "A replacement revision must keep the Managed Skill Package name",
                Some(source_path),
            ));
        }

        let id = next_id();
        let staged_path = app_data.join("staging").join(&id).join(&package.name);
        if staged_path.starts_with(&source_path) {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Revision staging cannot be placed inside the local Skill source",
                Some(source_path),
            ));
        }
        let result = (|| {
            copy_directory(&source_path, &staged_path)?;
            let staged = skill::validate_installed_revision(&staged_path, &package.name)?;
            let source_now = skill::validate_installed_revision(&source_path, &package.name)?;
            if staged.fingerprint != candidate.fingerprint
                || source_now.fingerprint != candidate.fingerprint
            {
                return Err(source_changed(&source_path));
            }
            let current = skill::validate_installed_revision(&package.library_path, &package.name)?;
            let public = ReplaceLocalPlan {
                id: id.clone(),
                package_id: package_id.to_owned(),
                source_path: source_path.clone(),
                changes: changes(&current, &staged),
                candidate,
                installation_count: package.installations.len(),
            };
            self.insert(
                id,
                PendingPlan::Replace(Box::new(PendingReplace {
                    public: public.clone(),
                    source_path,
                    staged_path: staged_path.clone(),
                    package,
                })),
            )?;
            Ok(public)
        })();
        if result.is_err() {
            remove_directory(staged_path.parent().expect("staged Skill has a plan root"))?;
        }
        result
    }

    pub fn commit_replace_local(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<RevisionResult, SkillError> {
        self.commit_replace_with_saver(app_data, plan_id, |state| {
            StateStore::new(app_data.to_path_buf()).save(state)
        })
    }

    fn commit_replace_with_saver<F>(
        &self,
        app_data: &Path,
        plan_id: &str,
        save: F,
    ) -> Result<RevisionResult, SkillError>
    where
        F: FnOnce(&AppState) -> Result<(), SkillError>,
    {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Replace(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Replace Local Revision"));
        };
        let plan_root = plan
            .staged_path
            .parent()
            .expect("staged Skill has a plan root")
            .to_path_buf();
        let result = self.commit_replace_inner(app_data, plan_id, &plan, save);
        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
    }

    fn commit_replace_inner<F>(
        &self,
        app_data: &Path,
        plan_id: &str,
        plan: &PendingReplace,
        save: F,
    ) -> Result<RevisionResult, SkillError>
    where
        F: FnOnce(&AppState) -> Result<(), SkillError>,
    {
        let mut state = load_writable_state(app_data)?;
        let package_index = package_index(&state, &plan.public.package_id)?;
        if state.packages[package_index] != plan.package {
            return Err(stale_plan("Replace Local Revision"));
        }
        validate_package(app_data, &state.packages[package_index])?;
        validate_previous_slot(&state.packages[package_index])?;
        let staged = skill::validate_installed_revision(&plan.staged_path, &plan.package.name)?;
        let source = skill::validate_installed_revision(&plan.source_path, &plan.package.name)
            .map_err(|_| source_changed(&plan.source_path))?;
        if staged.fingerprint != plan.public.candidate.fingerprint
            || source.fingerprint != plan.public.candidate.fingerprint
        {
            return Err(source_changed(&plan.source_path));
        }

        let prepared = prepare_copies(
            &plan.package,
            &plan.staged_path,
            plan_id,
            &staged.fingerprint,
        )?;
        let library = match replace_library(&plan.package, &plan.staged_path) {
            Ok(library) => library,
            Err(error) => {
                cleanup_prepared(&prepared);
                return Err(error);
            }
        };
        let swapped = match swap_copies(&prepared) {
            Ok(swapped) => swapped,
            Err(error) => {
                rollback_library(&library)?;
                cleanup_prepared(&prepared);
                return Err(error);
            }
        };

        let old_revision = state.packages[package_index].installed_revision.clone();
        let package = &mut state.packages[package_index];
        package.installed_revision = InstalledRevision {
            fingerprint: staged.fingerprint.clone(),
            commit_oid: None,
        };
        package.previous_revision = Some(old_revision);
        update_installation_fingerprints(package, &staged.fingerprint);
        if let Err(error) = save(&state) {
            rollback_copies(&swapped)?;
            rollback_library(&library)?;
            return Err(error);
        }
        cleanup_committed(&swapped, &library);
        Ok(RevisionResult {
            package: state.packages[package_index].clone(),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn plan_rollback(
        &self,
        app_data: &Path,
        package_id: &str,
    ) -> Result<RollbackRevisionPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?.clone();
        validate_package(app_data, &package)?;
        let previous_revision = package.previous_revision.clone().ok_or_else(|| {
            SkillError::new(
                SkillErrorCode::Conflict,
                "Roll Back Revision requires a Previous Revision",
                Some(package.library_path.clone()),
            )
        })?;
        let previous_path = previous_path(&package)?;
        let current = skill::validate_installed_revision(&package.library_path, &package.name)?;
        let previous = skill::validate_installed_revision(&previous_path, &package.name)?;
        if previous.fingerprint != previous_revision.fingerprint {
            return Err(content_drift(&previous_path));
        }
        let id = next_id();
        let public = RollbackRevisionPlan {
            id: id.clone(),
            package_id: package_id.to_owned(),
            from_revision: package.installed_revision.clone(),
            to_revision: previous_revision,
            changes: changes(&current, &previous),
            installation_count: package.installations.len(),
        };
        self.insert(
            id,
            PendingPlan::Rollback {
                public: public.clone(),
                package,
            },
        )?;
        Ok(public)
    }

    pub fn commit_rollback(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<RevisionResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Rollback { public, package } = self.take(plan_id)? else {
            return Err(invalid_plan("Roll Back Revision"));
        };
        let mut state = load_writable_state(app_data)?;
        let package_index = package_index(&state, &public.package_id)?;
        if state.packages[package_index] != package {
            return Err(stale_plan("Roll Back Revision"));
        }
        validate_package(app_data, &package)?;
        let previous = previous_path(&package)?;
        let validated_previous = skill::validate_installed_revision(&previous, &package.name)?;
        if validated_previous.fingerprint != public.to_revision.fingerprint {
            return Err(content_drift(&previous));
        }

        let prepared = prepare_copies(
            &package,
            &previous,
            plan_id,
            &public.to_revision.fingerprint,
        )?;
        swap_revision_dirs(&package.library_path, &previous, plan_id)?;
        let swapped = match swap_copies(&prepared) {
            Ok(swapped) => swapped,
            Err(error) => {
                swap_revision_dirs(&package.library_path, &previous, plan_id)?;
                cleanup_prepared(&prepared);
                return Err(error);
            }
        };
        let current_revision = state.packages[package_index].installed_revision.clone();
        state.packages[package_index].installed_revision = public.to_revision;
        state.packages[package_index].previous_revision = Some(current_revision);
        let fingerprint = state.packages[package_index]
            .installed_revision
            .fingerprint
            .clone();
        update_installation_fingerprints(&mut state.packages[package_index], &fingerprint);
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            rollback_copies(&swapped)?;
            swap_revision_dirs(&package.library_path, &previous, plan_id)?;
            return Err(error);
        }
        cleanup_swapped(&swapped);
        cleanup_prepared(&prepared);
        Ok(RevisionResult {
            package: state.packages[package_index].clone(),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn plan_export(
        &self,
        app_data: &Path,
        package_id: &str,
        destination: &Path,
    ) -> Result<ExportRevisionPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?.clone();
        validate_package(app_data, &package)?;
        if destination.starts_with(&package.library_path) {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Export destination cannot be inside the current revision",
                Some(destination.to_path_buf()),
            ));
        }
        ensure_absent(destination)?;
        let parent = destination.parent().ok_or_else(|| {
            SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Export destination must have a parent directory",
                Some(destination.to_path_buf()),
            )
        })?;
        if !parent.is_dir() {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Export destination parent must already exist",
                Some(parent.to_path_buf()),
            ));
        }
        let id = next_id();
        let public = ExportRevisionPlan {
            id: id.clone(),
            package_id: package_id.to_owned(),
            destination: destination.to_path_buf(),
            revision: package.installed_revision.clone(),
        };
        self.insert(
            id,
            PendingPlan::Export {
                public: public.clone(),
                package,
            },
        )?;
        Ok(public)
    }

    pub fn commit_export(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<ExportRevisionResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Export { public, package } = self.take(plan_id)? else {
            return Err(invalid_plan("Export Revision"));
        };
        let state = load_writable_state(app_data)?;
        if find_package(&state, &public.package_id)? != &package {
            return Err(stale_plan("Export Revision"));
        }
        validate_package(app_data, &package)?;
        ensure_absent(&public.destination)?;
        let staging = public
            .destination
            .parent()
            .expect("planned export has a parent")
            .join(format!(".skill-deck-export-{plan_id}"));
        ensure_absent(&staging)?;
        let result = (|| {
            copy_directory(&package.library_path, &staging)?;
            let copied = skill::validate_installed_revision(&staging, &package.name)?;
            if copied.fingerprint != public.revision.fingerprint {
                return Err(content_drift(&package.library_path));
            }
            ensure_absent(&public.destination)?;
            fs::rename(&staging, &public.destination)
                .map_err(|error| SkillError::io(&public.destination, error))?;
            Ok(ExportRevisionResult {
                destination: public.destination,
                fingerprint: copied.fingerprint,
            })
        })();
        if result.is_err() {
            remove_directory(&staging)?;
        }
        result
    }

    pub fn plan_restore(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
    ) -> Result<RestoreInstallationPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?.clone();
        ensure_owned_library(app_data, &package)?;
        validate_library(&package)?;
        let installation = package
            .installations
            .iter()
            .find(|installation| installation.agent == agent)
            .ok_or_else(|| invalid_plan("Restore Installation"))?;
        ensure_copy_directory(installation)?;
        let observed =
            skill::validate_installed_revision(&installation.logical_path, &package.name)?;
        if observed.fingerprint == package.installed_revision.fingerprint {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "The selected Installation has no Content Drift to restore",
                Some(installation.logical_path.clone()),
            ));
        }
        let id = next_id();
        let public = RestoreInstallationPlan {
            id: id.clone(),
            package_id: package_id.to_owned(),
            agent,
            logical_path: installation.logical_path.clone(),
            expected_fingerprint: package.installed_revision.fingerprint.clone(),
            observed_fingerprint: observed.fingerprint,
            will_overwrite: true,
        };
        self.insert(
            id,
            PendingPlan::Restore {
                public: public.clone(),
                package,
            },
        )?;
        Ok(public)
    }

    pub fn commit_restore(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_overwrite: bool,
    ) -> Result<RevisionResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Restore { public, package } = self.take(plan_id)? else {
            return Err(invalid_plan("Restore Installation"));
        };
        if !confirm_overwrite {
            return Err(SkillError::new(
                SkillErrorCode::InvalidPlan,
                "Restore Installation requires explicit overwrite confirmation",
                Some(public.logical_path),
            ));
        }
        let mut state = load_writable_state(app_data)?;
        let package_index = package_index(&state, &public.package_id)?;
        if state.packages[package_index] != package {
            return Err(stale_plan("Restore Installation"));
        }
        ensure_owned_library(app_data, &package)?;
        validate_library(&package)?;
        let installation_index = package
            .installations
            .iter()
            .position(|installation| {
                installation.agent == public.agent
                    && installation.logical_path == public.logical_path
            })
            .ok_or_else(|| stale_plan("Restore Installation"))?;
        let installation = &package.installations[installation_index];
        ensure_copy_directory(installation)?;
        let observed =
            skill::validate_installed_revision(&installation.logical_path, &package.name)?;
        if observed.fingerprint != public.observed_fingerprint {
            return Err(stale_plan("Restore Installation"));
        }
        let staging = sibling(&installation.logical_path, "restore", plan_id);
        let backup = sibling(&installation.logical_path, "backup", plan_id);
        ensure_absent(&staging)?;
        ensure_absent(&backup)?;
        if let Err(error) = copy_directory(&package.library_path, &staging) {
            let _ = remove_directory(&staging);
            return Err(error);
        }
        let copied = skill::validate_installed_revision(&staging, &package.name)?;
        if copied.fingerprint != public.expected_fingerprint {
            remove_directory(&staging)?;
            return Err(content_drift(&package.library_path));
        }
        fs::rename(&installation.logical_path, &backup)
            .map_err(|error| SkillError::io(&installation.logical_path, error))?;
        if let Err(error) = fs::rename(&staging, &installation.logical_path) {
            fs::rename(&backup, &installation.logical_path)
                .map_err(|rollback| SkillError::io(&installation.logical_path, rollback))?;
            return Err(SkillError::io(&installation.logical_path, error));
        }
        state.packages[package_index].installations[installation_index].last_known_fingerprint =
            public.expected_fingerprint;
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            remove_directory(&installation.logical_path)?;
            fs::rename(&backup, &installation.logical_path)
                .map_err(|rollback| SkillError::io(&installation.logical_path, rollback))?;
            return Err(error);
        }
        // ponytail: committed cleanup is recoverable staging debt; add startup cleanup if it occurs in practice.
        let _ = remove_directory(&backup);
        Ok(RevisionResult {
            package: state.packages[package_index].clone(),
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
            .ok_or_else(|| invalid_plan("Revision"))
    }
}

struct LibrarySwap {
    current: PathBuf,
    previous: PathBuf,
    staged: PathBuf,
    older_previous: Option<PathBuf>,
}

struct PreparedCopy {
    logical: PathBuf,
    staged: PathBuf,
    backup: PathBuf,
}

fn replace_library(
    package: &ManagedSkillPackage,
    staged: &Path,
) -> Result<LibrarySwap, SkillError> {
    let previous = previous_path(package)?;
    let older_previous = if previous.exists() {
        let path = staged
            .parent()
            .expect("staged Skill has a plan root")
            .join("older-previous");
        ensure_absent(&path)?;
        fs::rename(&previous, &path).map_err(|error| SkillError::io(&previous, error))?;
        Some(path)
    } else {
        None
    };
    if let Err(error) = fs::rename(&package.library_path, &previous) {
        if let Some(older) = &older_previous {
            fs::rename(older, &previous).map_err(|rollback| SkillError::io(&previous, rollback))?;
        }
        return Err(SkillError::io(&package.library_path, error));
    }
    if let Err(error) = fs::rename(staged, &package.library_path) {
        fs::rename(&previous, &package.library_path)
            .map_err(|rollback| SkillError::io(&package.library_path, rollback))?;
        if let Some(older) = &older_previous {
            fs::rename(older, &previous).map_err(|rollback| SkillError::io(&previous, rollback))?;
        }
        return Err(SkillError::io(staged, error));
    }
    Ok(LibrarySwap {
        current: package.library_path.clone(),
        previous,
        staged: staged.to_path_buf(),
        older_previous,
    })
}

fn rollback_library(swap: &LibrarySwap) -> Result<(), SkillError> {
    fs::rename(&swap.current, &swap.staged)
        .map_err(|error| SkillError::io(&swap.current, error))?;
    fs::rename(&swap.previous, &swap.current)
        .map_err(|error| SkillError::io(&swap.current, error))?;
    if let Some(older) = &swap.older_previous {
        fs::rename(older, &swap.previous).map_err(|error| SkillError::io(&swap.previous, error))?;
    }
    Ok(())
}

fn prepare_copies(
    package: &ManagedSkillPackage,
    source: &Path,
    id: &str,
    expected_fingerprint: &str,
) -> Result<Vec<PreparedCopy>, SkillError> {
    let mut prepared = Vec::new();
    for installation in &package.installations {
        if installation.deployment_mode != DeploymentMode::CopyFallback {
            continue;
        }
        let staged = sibling(&installation.logical_path, "revision", id);
        let backup = sibling(&installation.logical_path, "backup", id);
        if let Err(error) = ensure_absent(&staged).and_then(|_| ensure_absent(&backup)) {
            cleanup_prepared(&prepared);
            return Err(error);
        }
        if let Err(error) = copy_directory(source, &staged) {
            cleanup_prepared(&prepared);
            let _ = remove_directory(&staged);
            return Err(error);
        }
        let copied = match skill::validate_installed_revision(&staged, &package.name) {
            Ok(copied) => copied,
            Err(error) => {
                cleanup_prepared(&prepared);
                let _ = remove_directory(&staged);
                return Err(error);
            }
        };
        if copied.fingerprint != expected_fingerprint {
            cleanup_prepared(&prepared);
            let _ = remove_directory(&staged);
            return Err(content_drift(source));
        }
        prepared.push(PreparedCopy {
            logical: installation.logical_path.clone(),
            staged,
            backup,
        });
    }
    Ok(prepared)
}

fn swap_copies(prepared: &[PreparedCopy]) -> Result<Vec<&PreparedCopy>, SkillError> {
    let mut swapped = Vec::new();
    for copy in prepared {
        if let Err(error) = fs::rename(&copy.logical, &copy.backup) {
            rollback_copies(&swapped)?;
            return Err(SkillError::io(&copy.logical, error));
        }
        if let Err(error) = fs::rename(&copy.staged, &copy.logical) {
            fs::rename(&copy.backup, &copy.logical)
                .map_err(|rollback| SkillError::io(&copy.logical, rollback))?;
            rollback_copies(&swapped)?;
            return Err(SkillError::io(&copy.logical, error));
        }
        swapped.push(copy);
    }
    Ok(swapped)
}

fn rollback_copies(swapped: &[&PreparedCopy]) -> Result<(), SkillError> {
    for copy in swapped.iter().rev() {
        remove_directory(&copy.logical)?;
        fs::rename(&copy.backup, &copy.logical)
            .map_err(|error| SkillError::io(&copy.logical, error))?;
    }
    Ok(())
}

fn cleanup_prepared(prepared: &[PreparedCopy]) {
    for copy in prepared {
        let _ = remove_directory(&copy.staged);
    }
}

fn cleanup_swapped(swapped: &[&PreparedCopy]) {
    for copy in swapped {
        let _ = remove_directory(&copy.backup);
    }
}

fn cleanup_committed(swapped: &[&PreparedCopy], library: &LibrarySwap) {
    cleanup_swapped(swapped);
    if let Some(older) = &library.older_previous {
        let _ = remove_directory(older);
    }
    if let Some(plan_root) = library.staged.parent() {
        let _ = remove_directory(plan_root);
    }
}

fn swap_revision_dirs(current: &Path, previous: &Path, id: &str) -> Result<(), SkillError> {
    let temporary = current
        .parent()
        .expect("current revision has a package root")
        .join(format!(".swap-{id}"));
    ensure_absent(&temporary)?;
    fs::rename(current, &temporary).map_err(|error| SkillError::io(current, error))?;
    if let Err(error) = fs::rename(previous, current) {
        fs::rename(&temporary, current).map_err(|rollback| SkillError::io(current, rollback))?;
        return Err(SkillError::io(previous, error));
    }
    if let Err(error) = fs::rename(&temporary, previous) {
        fs::rename(current, previous).map_err(|rollback| SkillError::io(previous, rollback))?;
        fs::rename(&temporary, current).map_err(|rollback| SkillError::io(current, rollback))?;
        return Err(SkillError::io(&temporary, error));
    }
    Ok(())
}

fn changes(from: &ValidatedSkill, to: &ValidatedSkill) -> ChangeDisclosure {
    ChangeDisclosure {
        scripts: set_changes(&from.scripts, &to.scripts),
        references: set_changes(&from.references, &to.references),
        unknown_fields: set_changes(
            &from
                .metadata
                .unknown_fields
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            &to.metadata
                .unknown_fields
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
        ),
    }
}

pub(crate) fn change_disclosure(from: &ValidatedSkill, to: &ValidatedSkill) -> ChangeDisclosure {
    changes(from, to)
}

fn set_changes<T: Clone + Ord>(from: &[T], to: &[T]) -> ValueChanges<T> {
    let from = from.iter().cloned().collect::<BTreeSet<_>>();
    let to = to.iter().cloned().collect::<BTreeSet<_>>();
    ValueChanges {
        added: to.difference(&from).cloned().collect(),
        removed: from.difference(&to).cloned().collect(),
    }
}

fn validate_package(app_data: &Path, package: &ManagedSkillPackage) -> Result<(), SkillError> {
    ensure_owned_library(app_data, package)?;
    validate_library(package)?;
    for installation in &package.installations {
        lifecycle::validate_installation(package, installation)?;
    }
    Ok(())
}

pub(crate) fn validate_managed_package(
    app_data: &Path,
    package: &ManagedSkillPackage,
) -> Result<(), SkillError> {
    validate_package(app_data, package)?;
    validate_previous_slot(package)
}

pub(crate) fn apply_staged_revision(
    app_data: &Path,
    plan_id: &str,
    staged_path: &Path,
    expected_package: &ManagedSkillPackage,
    new_revision: InstalledRevision,
) -> Result<RevisionResult, SkillError> {
    let mut state = load_writable_state(app_data)?;
    let package_index = package_index(&state, &expected_package.id)?;
    if &state.packages[package_index] != expected_package {
        return Err(stale_plan("Git Update"));
    }
    validate_managed_package(app_data, expected_package)?;
    let staged = skill::validate_installed_revision(staged_path, &expected_package.name)?;
    if staged.fingerprint != new_revision.fingerprint {
        return Err(content_drift(staged_path));
    }

    let prepared = prepare_copies(
        expected_package,
        staged_path,
        plan_id,
        &new_revision.fingerprint,
    )?;
    let library = match replace_library(expected_package, staged_path) {
        Ok(library) => library,
        Err(error) => {
            cleanup_prepared(&prepared);
            return Err(error);
        }
    };
    let swapped = match swap_copies(&prepared) {
        Ok(swapped) => swapped,
        Err(error) => {
            rollback_library(&library)?;
            cleanup_prepared(&prepared);
            return Err(error);
        }
    };

    let package = &mut state.packages[package_index];
    package.previous_revision = Some(package.installed_revision.clone());
    package.installed_revision = new_revision;
    let fingerprint = package.installed_revision.fingerprint.clone();
    update_installation_fingerprints(package, &fingerprint);
    if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
        rollback_copies(&swapped)?;
        rollback_library(&library)?;
        return Err(error);
    }
    cleanup_committed(&swapped, &library);
    Ok(RevisionResult {
        package: state.packages[package_index].clone(),
        restart_message: RESTART_MESSAGE,
    })
}

fn ensure_owned_library(app_data: &Path, package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let expected = app_data.join("library").join(&package.name).join("current");
    if package.library_path != expected {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Managed Library path is outside the owned package boundary",
            Some(package.library_path.clone()),
        ));
    }
    Ok(())
}

fn validate_library(package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let current = skill::validate_installed_revision(&package.library_path, &package.name)?;
    if current.fingerprint != package.installed_revision.fingerprint {
        return Err(content_drift(&package.library_path));
    }
    Ok(())
}

fn validate_previous_slot(package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let path = previous_path(package)?;
    match &package.previous_revision {
        Some(revision) => {
            let previous = skill::validate_installed_revision(&path, &package.name)?;
            if previous.fingerprint != revision.fingerprint {
                return Err(content_drift(&path));
            }
        }
        None if path.exists() => {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "An untracked Previous Revision occupies the managed revision slot",
                Some(path),
            ));
        }
        None => {}
    }
    Ok(())
}

fn ensure_copy_directory(installation: &Installation) -> Result<(), SkillError> {
    if installation.deployment_mode != DeploymentMode::CopyFallback {
        return Err(SkillError::new(
            SkillErrorCode::Conflict,
            "Restore Installation is only needed for a copied Installation",
            Some(installation.logical_path.clone()),
        ));
    }
    let metadata = fs::symlink_metadata(&installation.logical_path)
        .map_err(|error| topology_error(&installation.logical_path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(topology_changed(&installation.logical_path));
    }
    Ok(())
}

fn update_installation_fingerprints(package: &mut ManagedSkillPackage, fingerprint: &str) {
    for installation in &mut package.installations {
        installation.last_known_fingerprint = fingerprint.to_owned();
        if installation.deployment_mode != DeploymentMode::CopyFallback {
            installation.resolved_target = package.library_path.clone();
        }
    }
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
        .ok_or_else(|| invalid_plan("Revision"))
}

fn package_index(state: &AppState, package_id: &str) -> Result<usize, SkillError> {
    state
        .packages
        .iter()
        .position(|package| package.id == package_id)
        .ok_or_else(|| invalid_plan("Revision"))
}

fn previous_path(package: &ManagedSkillPackage) -> Result<PathBuf, SkillError> {
    let root = package.library_path.parent().ok_or_else(|| {
        SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Managed Library current revision has no package root",
            Some(package.library_path.clone()),
        )
    })?;
    if package
        .library_path
        .file_name()
        .and_then(|name| name.to_str())
        != Some("current")
    {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Managed Library revision path must end in current",
            Some(package.library_path.clone()),
        ));
    }
    Ok(root.join("previous"))
}

fn ensure_absent(path: &Path) -> Result<(), SkillError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SkillError::io(path, error)),
        Ok(_) => Err(SkillError::new(
            SkillErrorCode::Conflict,
            "The transaction destination already exists and will not be overwritten",
            Some(path.to_path_buf()),
        )),
    }
}

fn sibling(path: &Path, kind: &str, id: &str) -> PathBuf {
    path.parent()
        .expect("managed Installation has an Agent root")
        .join(format!(".skill-deck-{kind}-{id}"))
}

fn source_changed(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        "The local Skill source changed after the plan was created",
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

fn topology_error(path: &Path, error: io::Error) -> SkillError {
    SkillError::new(
        SkillErrorCode::TopologyChanged,
        format!("Managed Installation topology is unavailable: {error}"),
        Some(path.to_path_buf()),
    )
}

fn topology_changed(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::TopologyChanged,
        "Managed Installation topology no longer matches its recorded deployment mode",
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

fn invalid_plan(operation: &str) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        format!("The {operation} plan is missing, stale, or already committed"),
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
        "The in-memory revision plan store is unavailable",
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ConfigurationProvenance;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        app_data: PathBuf,
        package: ManagedSkillPackage,
        sources: PathBuf,
    }

    fn write_skill(parent: &Path, name: &str, body: &str) -> PathBuf {
        let root = parent.join(name);
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(
            root.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Fixture\n---\n{body}"),
        )
        .unwrap();
        fs::write(root.join("scripts/run.sh"), body).unwrap();
        root
    }

    fn fixture(installation_count: usize) -> Fixture {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        let library = app_data.join("library/alpha-skill/current");
        fs::create_dir_all(library.join("scripts")).unwrap();
        fs::write(
            library.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Fixture\n---\nv1",
        )
        .unwrap();
        fs::write(library.join("scripts/run.sh"), "v1").unwrap();
        let fingerprint = skill::validate_installed_revision(&library, "alpha-skill")
            .unwrap()
            .fingerprint;
        let mut installations = Vec::new();
        for (index, agent) in [Agent::Codex, Agent::Claude]
            .into_iter()
            .take(installation_count)
            .enumerate()
        {
            let logical = temp
                .path()
                .join(format!("agent-{index}/skills/alpha-skill"));
            copy_directory(&library, &logical).unwrap();
            installations.push(Installation {
                agent,
                logical_path: logical.clone(),
                resolved_target: logical,
                deployment_mode: DeploymentMode::CopyFallback,
                enabled: true,
                last_known_fingerprint: fingerprint.clone(),
                configuration_provenance: ConfigurationProvenance::None,
            });
        }
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
            installations,
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
            sources: PathBuf::new(),
        }
    }

    fn replacement(fixture: &mut Fixture, name: &str, body: &str) -> PathBuf {
        fixture.sources = fixture._temp.path().join("sources");
        write_skill(&fixture.sources, name, body)
    }

    #[test]
    fn replacement_rejects_name_change() {
        let mut fixture = fixture(0);
        let source = replacement(&mut fixture, "renamed-skill", "v2");

        let error = RevisionManager::default()
            .plan_replace_local(&fixture.app_data, &fixture.package.id, &source)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::InvalidMetadata);
    }

    #[test]
    fn changed_source_invalidates_replacement_plan() {
        let mut fixture = fixture(0);
        let source = replacement(&mut fixture, "alpha-skill", "v2");
        let manager = RevisionManager::default();
        let plan = manager
            .plan_replace_local(&fixture.app_data, &fixture.package.id, &source)
            .unwrap();
        fs::write(source.join("scripts/run.sh"), "v3").unwrap();

        let error = manager
            .commit_replace_local(&fixture.app_data, &plan.id)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::SourceChanged);
        assert_eq!(
            skill::validate_installed_revision(&fixture.package.library_path, "alpha-skill")
                .unwrap()
                .fingerprint,
            fixture.package.installed_revision.fingerprint
        );
    }

    #[test]
    fn state_failure_rolls_back_library_and_two_copy_targets() {
        let mut fixture = fixture(2);
        let source = replacement(&mut fixture, "alpha-skill", "v2");
        let manager = RevisionManager::default();
        let plan = manager
            .plan_replace_local(&fixture.app_data, &fixture.package.id, &source)
            .unwrap();

        let error = manager
            .commit_replace_with_saver(&fixture.app_data, &plan.id, |_| {
                Err(SkillError::new(
                    SkillErrorCode::Io,
                    "injected state failure",
                    None,
                ))
            })
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Io);
        for path in std::iter::once(&fixture.package.library_path).chain(
            fixture
                .package
                .installations
                .iter()
                .map(|installation| &installation.logical_path),
        ) {
            assert_eq!(
                skill::validate_installed_revision(path, "alpha-skill")
                    .unwrap()
                    .fingerprint,
                fixture.package.installed_revision.fingerprint
            );
        }
        assert!(!fixture
            .package
            .library_path
            .parent()
            .unwrap()
            .join("previous")
            .exists());
    }

    #[test]
    fn rollback_swaps_current_and_previous_for_one_step_redo() {
        let mut fixture = fixture(0);
        let source = replacement(&mut fixture, "alpha-skill", "v2");
        let manager = RevisionManager::default();
        let replace = manager
            .plan_replace_local(&fixture.app_data, &fixture.package.id, &source)
            .unwrap();
        let updated = manager
            .commit_replace_local(&fixture.app_data, &replace.id)
            .unwrap()
            .package;

        let rollback = manager
            .plan_rollback(&fixture.app_data, &fixture.package.id)
            .unwrap();
        let old = manager
            .commit_rollback(&fixture.app_data, &rollback.id)
            .unwrap()
            .package;
        assert_eq!(old.installed_revision, fixture.package.installed_revision);

        let redo = manager
            .plan_rollback(&fixture.app_data, &fixture.package.id)
            .unwrap();
        let redone = manager
            .commit_rollback(&fixture.app_data, &redo.id)
            .unwrap()
            .package;
        assert_eq!(redone.installed_revision, updated.installed_revision);
    }

    #[test]
    fn restore_overwrites_only_the_selected_copy() {
        let fixture = fixture(2);
        let selected = &fixture.package.installations[0];
        fs::write(selected.logical_path.join("scripts/run.sh"), "drift").unwrap();
        let untouched = &fixture.package.installations[1];
        let manager = RevisionManager::default();

        let plan = manager
            .plan_restore(&fixture.app_data, &fixture.package.id, selected.agent)
            .unwrap();
        assert!(plan.will_overwrite);
        assert_ne!(plan.observed_fingerprint, plan.expected_fingerprint);
        manager
            .commit_restore(&fixture.app_data, &plan.id, true)
            .unwrap();

        assert_eq!(
            skill::validate_installed_revision(&selected.logical_path, "alpha-skill")
                .unwrap()
                .fingerprint,
            fixture.package.installed_revision.fingerprint
        );
        assert_eq!(
            skill::validate_installed_revision(&untouched.logical_path, "alpha-skill")
                .unwrap()
                .fingerprint,
            fixture.package.installed_revision.fingerprint
        );
    }

    #[test]
    fn export_never_overwrites_a_destination_created_after_plan() {
        let fixture = fixture(0);
        let destination = fixture._temp.path().join("exported-skill");
        let manager = RevisionManager::default();
        let plan = manager
            .plan_export(&fixture.app_data, &fixture.package.id, &destination)
            .unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("keep.txt"), "user data").unwrap();

        let error = manager
            .commit_export(&fixture.app_data, &plan.id)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Conflict);
        assert_eq!(
            fs::read_to_string(destination.join("keep.txt")).unwrap(),
            "user data"
        );
    }

    #[test]
    fn change_disclosure_and_plan_use_camel_case_json() {
        let mut fixture = fixture(0);
        let source = replacement(&mut fixture, "alpha-skill", "v2");
        fs::create_dir(source.join("references")).unwrap();
        fs::write(source.join("references/new.md"), "new").unwrap();
        let plan = RevisionManager::default()
            .plan_replace_local(&fixture.app_data, &fixture.package.id, &source)
            .unwrap();
        let json = serde_json::to_value(plan).unwrap();

        assert_eq!(json["packageId"], "package-1");
        assert_eq!(json["installationCount"], 0);
        assert_eq!(
            json["changes"]["references"]["added"][0],
            "references/new.md"
        );
        assert!(json["changes"].get("unknownFields").is_some());
        assert!(json.get("package_id").is_none());
    }
}
