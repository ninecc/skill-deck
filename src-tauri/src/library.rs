use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{
    skill::{self, SkillError, SkillErrorCode, ValidatedSkill},
    state::{InstalledRevision, ManagedSkillPackage, SkillSource, StateMode, StateStore},
};

const LIBRARY_DIR: &str = "library";
const STAGING_DIR: &str = "staging";
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddToLibraryPlan {
    pub id: String,
    pub skill: ValidatedSkill,
    pub library_path: PathBuf,
}

#[derive(Debug)]
struct PendingPlan {
    public: AddToLibraryPlan,
    source_path: PathBuf,
    staged_path: PathBuf,
}

#[derive(Default)]
pub struct LibraryManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl LibraryManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_local(
        &self,
        app_data: &Path,
        source: &Path,
    ) -> Result<AddToLibraryPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let mut validated = skill::validate_skill_dir(source)?;
        let source = source
            .canonicalize()
            .map_err(|error| SkillError::io(source, error))?;
        validated.root = source.clone();
        ensure_name_available(app_data, &validated.metadata.name)?;

        let id = next_id();
        let plan_root = app_data.join(STAGING_DIR).join(&id);
        if plan_root.starts_with(&source) {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Managed Library staging cannot be placed inside the Skill source",
                Some(source),
            ));
        }
        let staged_path = plan_root.join(&validated.metadata.name);
        let result = (|| {
            copy_directory(&source, &staged_path)?;
            let staged = skill::validate_skill_dir(&staged_path)?;
            let current_source = skill::validate_skill_dir(&source)?;
            if staged.fingerprint != validated.fingerprint
                || current_source.fingerprint != validated.fingerprint
            {
                return Err(source_changed(&source));
            }

            let public = AddToLibraryPlan {
                id: id.clone(),
                skill: validated,
                library_path: app_data
                    .join(LIBRARY_DIR)
                    .join(&staged.metadata.name)
                    .join("current"),
            };
            self.plans.lock().map_err(|_| lock_error())?.insert(
                id,
                PendingPlan {
                    public: public.clone(),
                    source_path: source,
                    staged_path,
                },
            );
            Ok(public)
        })();

        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
    }

    pub fn commit_local(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<ManagedSkillPackage, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let plan = self
            .plans
            .lock()
            .map_err(|_| lock_error())?
            .remove(plan_id)
            .ok_or_else(|| {
                SkillError::new(
                    SkillErrorCode::InvalidPlan,
                    "The Add to Library plan is missing or was already committed",
                    None,
                )
            })?;
        let plan_root = plan
            .staged_path
            .parent()
            .expect("staging plan has a parent")
            .to_path_buf();
        let result = commit_pending(app_data, plan);
        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
    }
}

fn commit_pending(app_data: &Path, plan: PendingPlan) -> Result<ManagedSkillPackage, SkillError> {
    let staged = skill::validate_skill_dir(&plan.staged_path)?;
    let source = skill::validate_skill_dir(&plan.source_path)
        .map_err(|_| source_changed(&plan.source_path))?;
    if staged.fingerprint != plan.public.skill.fingerprint
        || source.fingerprint != plan.public.skill.fingerprint
    {
        return Err(source_changed(&plan.source_path));
    }

    let store = StateStore::new(app_data.to_path_buf());
    let loaded = store.load()?;
    if loaded.mode == StateMode::ReadOnlyRecovery {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Application state is in read-only recovery mode",
            Some(app_data.join("state.json")),
        ));
    }
    let mut state = loaded.state.expect("writable state mode contains state");
    ensure_unique_state_name(&state.packages, &plan.public.skill.metadata.name)?;

    let package_root = plan
        .public
        .library_path
        .parent()
        .expect("library current path has a parent")
        .to_path_buf();
    if package_root.exists() {
        return Err(conflict_error(&package_root));
    }
    fs::create_dir_all(&package_root).map_err(|error| SkillError::io(&package_root, error))?;
    if let Err(error) = fs::rename(&plan.staged_path, &plan.public.library_path) {
        remove_directory(&package_root)?;
        return Err(SkillError::io(&plan.staged_path, error));
    }
    if let Err(error) = remove_directory(
        plan.staged_path
            .parent()
            .expect("staging plan has a parent"),
    ) {
        remove_directory(&package_root)?;
        return Err(error);
    }

    let package = ManagedSkillPackage {
        id: format!("local-{}", next_id()),
        name: plan.public.skill.metadata.name,
        library_path: plan.public.library_path,
        source: SkillSource::LocalSnapshot,
        installed_revision: InstalledRevision {
            fingerprint: plan.public.skill.fingerprint,
            commit_oid: None,
        },
        previous_revision: None,
        installations: Vec::new(),
    };
    state.packages.push(package.clone());
    if let Err(error) = store.save(&state) {
        if let Err(rollback_error) = remove_directory(&package_root) {
            return Err(SkillError::new(
                SkillErrorCode::Io,
                format!(
                    "State write failed ({error:?}) and Managed Library rollback failed ({rollback_error:?})"
                ),
                Some(package_root),
            ));
        }
        return Err(error);
    }
    Ok(package)
}

pub(crate) fn ensure_name_available(app_data: &Path, name: &str) -> Result<(), SkillError> {
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    if loaded.mode == StateMode::ReadOnlyRecovery {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Application state is in read-only recovery mode",
            Some(app_data.join("state.json")),
        ));
    }
    ensure_unique_state_name(
        &loaded
            .state
            .expect("writable state mode contains state")
            .packages,
        name,
    )?;
    let package_root = app_data.join(LIBRARY_DIR).join(name);
    if package_root.exists() {
        return Err(conflict_error(&package_root));
    }
    Ok(())
}

pub(crate) fn ensure_unique_state_name(
    packages: &[ManagedSkillPackage],
    name: &str,
) -> Result<(), SkillError> {
    if let Some(existing) = packages
        .iter()
        .find(|package| package.name.eq_ignore_ascii_case(name))
    {
        return Err(SkillError::new(
            SkillErrorCode::Conflict,
            format!("A Managed Skill Package named {name} already exists"),
            Some(existing.library_path.clone()),
        ));
    }
    Ok(())
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) -> Result<(), SkillError> {
    fs::create_dir_all(destination).map_err(|error| SkillError::io(destination, error))?;
    for entry in fs::read_dir(source).map_err(|error| SkillError::io(source, error))? {
        let entry = entry.map_err(|error| SkillError::io(source, error))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| SkillError::io(&source_path, error))?;
        if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
            return Err(SkillError::new(
                SkillErrorCode::UnsupportedFileType,
                "Skill packages can contain only real files and directories",
                Some(source_path),
            ));
        }
        if metadata.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| SkillError::io(&source_path, error))?;
        }
    }
    Ok(())
}

pub(crate) fn next_id() -> String {
    format!(
        "{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn busy_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Busy,
        "Another Skill Deck mutation is already running",
        None,
    )
}

fn conflict_error(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::Conflict,
        "The normalized Skill name is already present in Managed Library",
        Some(path.to_path_buf()),
    )
}

fn source_changed(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        "The local Skill source changed after the plan was created",
        Some(path.to_path_buf()),
    )
}

fn lock_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Io,
        "The in-memory operation plan store is unavailable",
        None,
    )
}

pub(crate) fn remove_directory(path: &Path) -> Result<(), SkillError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| SkillError::io(path, error))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_skill(parent: &Path, name: &str, body: &str) -> PathBuf {
        let root = parent.join(name);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Fixture\n---\n{body}"),
        )
        .unwrap();
        root
    }

    #[test]
    fn adds_zero_target_local_snapshot_to_private_library() {
        let temp = TempDir::new().unwrap();
        let source = write_skill(&temp.path().join("sources"), "alpha-skill", "v1");
        let app_data = temp.path().join("app-data");
        let manager = LibraryManager::default();

        let plan = manager.plan_local(&app_data, &source).unwrap();
        assert!(!plan.library_path.exists());
        let package = manager.commit_local(&app_data, &plan.id).unwrap();

        assert!(package.library_path.join("SKILL.md").is_file());
        assert!(!app_data.join(STAGING_DIR).join(&plan.id).exists());
        assert!(package.installations.is_empty());
        assert_eq!(package.source, SkillSource::LocalSnapshot);
        let state = StateStore::new(app_data).load().unwrap().state.unwrap();
        assert_eq!(state.packages, [package]);
    }

    #[test]
    fn rejects_normalized_name_collision_without_touching_source() {
        let temp = TempDir::new().unwrap();
        let source = write_skill(&temp.path().join("sources"), "alpha-skill", "v1");
        let app_data = temp.path().join("app-data");
        let manager = LibraryManager::default();
        let first = manager.plan_local(&app_data, &source).unwrap();
        manager.commit_local(&app_data, &first.id).unwrap();

        let error = manager.plan_local(&app_data, &source).unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Conflict);
        assert!(source.join("SKILL.md").is_file());
    }

    #[test]
    fn changed_source_invalidates_plan_and_cleans_staging() {
        let temp = TempDir::new().unwrap();
        let source = write_skill(&temp.path().join("sources"), "alpha-skill", "v1");
        let app_data = temp.path().join("app-data");
        let manager = LibraryManager::default();
        let plan = manager.plan_local(&app_data, &source).unwrap();
        fs::write(
            source.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Fixture\n---\nv2",
        )
        .unwrap();

        let error = manager.commit_local(&app_data, &plan.id).unwrap_err();

        assert_eq!(error.code, SkillErrorCode::SourceChanged);
        assert!(!app_data.join(STAGING_DIR).join(&plan.id).exists());
        assert!(!app_data.join(LIBRARY_DIR).join("alpha-skill").exists());
    }

    #[test]
    fn state_write_failure_rolls_back_library_move() {
        let temp = TempDir::new().unwrap();
        let source = write_skill(&temp.path().join("sources"), "alpha-skill", "v1");
        let app_data = temp.path().join("app-data");
        let manager = LibraryManager::default();
        let plan = manager.plan_local(&app_data, &source).unwrap();
        fs::create_dir_all(app_data.join("state.json")).unwrap();
        fs::write(
            app_data.join("state.backup.json"),
            r#"{"stateVersion":1,"packages":[]}"#,
        )
        .unwrap();

        let error = manager.commit_local(&app_data, &plan.id).unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Io);
        assert!(!app_data.join(LIBRARY_DIR).join("alpha-skill").exists());
        assert!(source.join("SKILL.md").is_file());
    }

    #[test]
    fn concurrent_mutation_is_rejected_as_busy() {
        let temp = TempDir::new().unwrap();
        let source = write_skill(&temp.path().join("sources"), "alpha-skill", "v1");
        let app_data = temp.path().join("app-data");
        let manager = LibraryManager::default();
        let _held = manager.mutation.lock().unwrap();

        let error = manager.plan_local(&app_data, &source).unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Busy);
        assert!(!app_data.join(STAGING_DIR).exists());
    }
}
