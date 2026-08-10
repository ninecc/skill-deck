use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use atomicwrites::{AtomicFile, DisallowOverwrite};
use serde::Serialize;

use crate::{
    inventory::{self, Agent, Inventory},
    library::next_id,
    skill::{SkillError, SkillErrorCode},
    state::{StateMode, StateStore},
};

const STAGING_DIR: &str = "staging";
const LIBRARY_DIR: &str = "library";
const RECOVERY_SCOPE: &str = "Startup recovery removes only stale Skill Deck staging. Recovery of interrupted changes in Agent roots is deferred.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticTargetSummary {
    pub agent: Agent,
    pub root: PathBuf,
    pub exists: bool,
    pub legacy: bool,
    pub external_installation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsReport {
    pub state_mode: StateMode,
    pub targets: Vec<DiagnosticTargetSummary>,
    pub managed_package_count: usize,
    pub external_installation_count: usize,
    pub orphaned_package_paths: Vec<PathBuf>,
    pub destination: PathBuf,
    pub omitted: Vec<String>,
    pub recovery_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsExportPlan {
    pub id: String,
    pub report: DiagnosticsReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsExportResult {
    pub destination: PathBuf,
}

#[derive(Default)]
pub struct DiagnosticsManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, DiagnosticsReport>>,
}

impl DiagnosticsManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_export(
        &self,
        app_data: &Path,
        destination: &Path,
    ) -> Result<DiagnosticsExportPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        validate_destination(destination)?;
        let report = build_report(app_data, destination, inventory::inventory(app_data)?)?;
        let id = next_id();
        self.plans
            .lock()
            .map_err(|_| plan_store_error())?
            .insert(id.clone(), report.clone());
        Ok(DiagnosticsExportPlan { id, report })
    }

    pub fn commit_export(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<DiagnosticsExportResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let planned = self
            .plans
            .lock()
            .map_err(|_| plan_store_error())?
            .remove(plan_id)
            .ok_or_else(invalid_plan)?;
        validate_destination(&planned.destination)?;
        let current = build_report(
            app_data,
            &planned.destination,
            inventory::inventory(app_data)?,
        )?;
        if current != planned {
            return Err(SkillError::new(
                SkillErrorCode::SourceChanged,
                "Diagnostics changed after the export preview was created",
                Some(planned.destination),
            ));
        }
        let bytes = serde_json::to_vec_pretty(&planned).map_err(|error| {
            SkillError::new(
                SkillErrorCode::Io,
                format!("Could not serialize diagnostics: {error}"),
                Some(planned.destination.clone()),
            )
        })?;
        write_new_atomic(&planned.destination, &bytes)?;
        Ok(DiagnosticsExportResult {
            destination: planned.destination,
        })
    }

    pub fn cancel_staging(&self, app_data: &Path) -> Result<(), SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        cleanup_stale_staging(app_data)
    }
}

pub(crate) fn cleanup_stale_staging(app_data: &Path) -> Result<(), SkillError> {
    let staging = app_data.join(STAGING_DIR);
    let metadata = match fs::symlink_metadata(&staging) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(SkillError::io(&staging, error)),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(&staging).map_err(|error| SkillError::io(&staging, error))
    } else {
        fs::remove_file(&staging).map_err(|error| SkillError::io(&staging, error))
    }
}

fn build_report(
    app_data: &Path,
    destination: &Path,
    inventory: Inventory,
) -> Result<DiagnosticsReport, SkillError> {
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    let targets = inventory
        .targets
        .iter()
        .map(|target| DiagnosticTargetSummary {
            agent: target.agent,
            root: target.root.clone(),
            exists: target.exists,
            legacy: target.legacy,
            external_installation_count: inventory
                .external_installations
                .iter()
                .filter(|installation| installation.agent == target.agent)
                .filter(|installation| {
                    target.legacy
                        == matches!(
                            installation.kind,
                            inventory::InstallationKind::LegacyDirectory
                                | inventory::InstallationKind::LegacyLink
                        )
                })
                .count(),
        })
        .collect();
    Ok(DiagnosticsReport {
        state_mode: loaded.mode,
        targets,
        managed_package_count: inventory.managed_packages.len(),
        external_installation_count: inventory.external_installations.len(),
        orphaned_package_paths: orphaned_package_paths(app_data, loaded.state.as_ref())?,
        destination: destination.to_path_buf(),
        omitted: vec![
            "skill_content".to_owned(),
            "environment_variables".to_owned(),
            "credentials".to_owned(),
        ],
        recovery_scope: RECOVERY_SCOPE.to_owned(),
    })
}

fn orphaned_package_paths(
    app_data: &Path,
    state: Option<&crate::state::AppState>,
) -> Result<Vec<PathBuf>, SkillError> {
    let library = app_data.join(LIBRARY_DIR);
    let metadata = match fs::symlink_metadata(&library) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(SkillError::io(&library, error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Managed Library must be a real directory",
            Some(library),
        ));
    }
    let referenced: HashSet<&Path> = state
        .into_iter()
        .flat_map(|state| state.packages.iter())
        .filter_map(|package| package.library_path.parent())
        .collect();
    let mut orphaned = Vec::new();
    for entry in fs::read_dir(&library).map_err(|error| SkillError::io(&library, error))? {
        let entry = entry.map_err(|error| SkillError::io(&library, error))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| SkillError::io(&path, error))?;
        if metadata.is_dir()
            && !metadata.file_type().is_symlink()
            && !referenced.contains(path.as_path())
        {
            orphaned.push(path);
        }
    }
    orphaned.sort();
    Ok(orphaned)
}

fn validate_destination(destination: &Path) -> Result<(), SkillError> {
    if !destination.is_absolute() || destination.file_name().is_none() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Diagnostics destination must be an absolute file path",
            Some(destination.to_path_buf()),
        ));
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => return Err(conflict(destination)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SkillError::io(destination, error)),
    }
    let parent = destination
        .parent()
        .expect("absolute file path has a parent");
    if !parent.is_dir() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Diagnostics destination parent must be an existing directory",
            Some(parent.to_path_buf()),
        ));
    }
    Ok(())
}

fn write_new_atomic(path: &Path, bytes: &[u8]) -> Result<(), SkillError> {
    match AtomicFile::new(path, DisallowOverwrite).write(|file| -> std::io::Result<()> {
        file.write_all(bytes)?;
        file.sync_all()
    }) {
        Ok(()) => Ok(()),
        Err(error) => {
            let error: std::io::Error = error.into();
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Err(conflict(path))
            } else {
                Err(SkillError::io(path, error))
            }
        }
    }
}

fn conflict(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::Conflict,
        "Diagnostics export never overwrites an existing file or directory",
        Some(path.to_path_buf()),
    )
}

fn busy_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Busy,
        "Another Skill Deck mutation is already running",
        None,
    )
}

fn invalid_plan() -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        "The diagnostics export plan is missing or was already committed",
        None,
    )
}

fn plan_store_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Io,
        "The in-memory operation plan store is unavailable",
        None,
    )
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        inventory::{AgentTarget, ExternalInstallation, InstallationKind},
        state::AppState,
    };

    #[test]
    fn startup_cleanup_is_bounded_to_staging() {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        let outside = temp.path().join("outside");
        fs::create_dir_all(app_data.join("staging/old-plan")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(app_data.join("staging/old-plan/payload"), "stale").unwrap();
        fs::write(outside.join("keep"), "untouched").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, app_data.join("staging/old-plan/outside-link"))
            .unwrap();

        cleanup_stale_staging(&app_data).unwrap();

        assert!(!app_data.join("staging").exists());
        assert_eq!(
            fs::read_to_string(outside.join("keep")).unwrap(),
            "untouched"
        );
    }

    #[test]
    fn cancel_staging_cleans_preview_data_and_respects_the_mutation_gate() {
        let temp = TempDir::new().unwrap();
        let staging = temp.path().join("staging/plan");
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("payload"), "preview").unwrap();
        let manager = DiagnosticsManager::default();
        let held = manager.mutation.lock().unwrap();
        assert_eq!(
            manager.cancel_staging(temp.path()).unwrap_err().code,
            SkillErrorCode::Busy
        );
        drop(held);

        manager.cancel_staging(temp.path()).unwrap();
        assert!(!temp.path().join("staging").exists());
    }

    #[test]
    fn unreferenced_library_package_is_reported_without_deletion() {
        let temp = TempDir::new().unwrap();
        let orphan = temp.path().join("library/orphan");
        fs::create_dir_all(orphan.join("current")).unwrap();

        let paths = orphaned_package_paths(temp.path(), Some(&AppState::default())).unwrap();

        assert_eq!(paths, vec![orphan.clone()]);
        assert!(orphan.exists());
    }

    #[test]
    fn export_conflicts_and_payload_omits_skill_content() {
        let temp = TempDir::new().unwrap();
        let destination = temp.path().join("diagnostics.json");
        fs::write(&destination, "keep").unwrap();
        assert_eq!(
            validate_destination(&destination).unwrap_err().code,
            SkillErrorCode::Conflict
        );
        assert_eq!(fs::read_to_string(&destination).unwrap(), "keep");

        fs::remove_file(&destination).unwrap();
        let secret = "TOP_SECRET_SKILL_BODY";
        fs::create_dir_all(temp.path().join("library/orphan/current")).unwrap();
        fs::write(temp.path().join("library/orphan/current/SKILL.md"), secret).unwrap();
        let inventory = Inventory {
            targets: vec![AgentTarget {
                agent: Agent::Codex,
                root: temp.path().join("home/.agents/skills"),
                exists: false,
                legacy: false,
            }],
            external_installations: Vec::<ExternalInstallation>::new(),
            managed_packages: Vec::new(),
        };
        let report = build_report(temp.path(), &destination, inventory).unwrap();
        let bytes = serde_json::to_vec_pretty(&report).unwrap();
        write_new_atomic(&destination, &bytes).unwrap();
        let exported = fs::read_to_string(&destination).unwrap();
        assert!(!exported.contains(secret));
        assert!(exported.contains("skill_content"));
        assert!(exported.contains("environment_variables"));
        assert!(exported.contains("credentials"));
    }

    #[test]
    fn target_counts_keep_legacy_entries_separate() {
        let temp = TempDir::new().unwrap();
        let current_root = temp.path().join("current");
        let legacy_root = temp.path().join("legacy");
        let inventory = Inventory {
            targets: vec![
                AgentTarget {
                    agent: Agent::Codex,
                    root: current_root.clone(),
                    exists: true,
                    legacy: false,
                },
                AgentTarget {
                    agent: Agent::Codex,
                    root: legacy_root.clone(),
                    exists: true,
                    legacy: true,
                },
            ],
            external_installations: vec![ExternalInstallation {
                agent: Agent::Codex,
                logical_path: legacy_root.join("old"),
                resolved_target: None,
                kind: InstallationKind::LegacyDirectory,
                skill: None,
                diagnostic: None,
            }],
            managed_packages: Vec::new(),
        };

        let report = build_report(
            temp.path(),
            &temp.path().join("diagnostics.json"),
            inventory,
        )
        .unwrap();

        assert_eq!(report.targets[0].external_installation_count, 0);
        assert_eq!(report.targets[1].external_installation_count, 1);
    }
}
