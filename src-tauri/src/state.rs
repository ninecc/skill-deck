use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomicwrites::{AllowOverwrite, AtomicFile};
use serde::{Deserialize, Serialize};

use crate::{
    inventory::Agent,
    skill::{SkillError, SkillErrorCode},
};

const STATE_VERSION: u32 = 1;
const STATE_FILE: &str = "state.json";
const BACKUP_FILE: &str = "state.backup.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum SkillSource {
    LocalSnapshot,
    Git {
        repository_url: String,
        subpath: String,
        tracked_branch: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledRevision {
    pub fingerprint: String,
    pub commit_oid: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    Symlink,
    Junction,
    CopyFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "owner")]
pub enum ConfigurationProvenance {
    SkillDeck { path: PathBuf },
    External { path: PathBuf },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub resolved_target: PathBuf,
    pub deployment_mode: DeploymentMode,
    pub enabled: bool,
    pub last_known_fingerprint: String,
    pub configuration_provenance: ConfigurationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSkillPackage {
    pub id: String,
    pub name: String,
    pub library_path: PathBuf,
    pub source: SkillSource,
    pub installed_revision: InstalledRevision,
    pub previous_revision: Option<InstalledRevision>,
    pub installations: Vec<Installation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub state_version: u32,
    pub packages: Vec<ManagedSkillPackage>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            state_version: STATE_VERSION,
            packages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateMode {
    Active,
    RecoveredBackup,
    ReadOnlyRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateLoad {
    pub mode: StateMode,
    pub state: Option<AppState>,
    pub diagnostic: Option<String>,
}

pub struct StateStore {
    root: PathBuf,
}

impl StateStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load(&self) -> Result<StateLoad, SkillError> {
        let state_path = self.root.join(STATE_FILE);
        if !state_path.exists() {
            return Ok(StateLoad {
                mode: StateMode::Active,
                state: Some(AppState::default()),
                diagnostic: None,
            });
        }

        match read_state(&state_path) {
            Ok(state) => Ok(StateLoad {
                mode: StateMode::Active,
                state: Some(state),
                diagnostic: None,
            }),
            Err(StateReadError::UnsupportedVersion(primary_error)) => Ok(StateLoad {
                mode: StateMode::ReadOnlyRecovery,
                state: None,
                diagnostic: Some(primary_error),
            }),
            Err(StateReadError::Invalid(primary_error)) => {
                let backup_path = self.root.join(BACKUP_FILE);
                match read_state(&backup_path) {
                    Ok(state) => Ok(StateLoad {
                        mode: StateMode::RecoveredBackup,
                        state: Some(state),
                        diagnostic: Some(primary_error),
                    }),
                    Err(backup_error) => Ok(StateLoad {
                        mode: StateMode::ReadOnlyRecovery,
                        state: None,
                        diagnostic: Some(format!(
                            "Primary state is invalid ({primary_error}); backup is unavailable or invalid ({backup_error})"
                        )),
                    }),
                }
            }
        }
    }

    pub fn save(&self, state: &AppState) -> Result<(), SkillError> {
        if state.state_version != STATE_VERSION {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Refusing to write an unsupported state version",
                Some(self.root.join(STATE_FILE)),
            ));
        }
        fs::create_dir_all(&self.root).map_err(|error| SkillError::io(&self.root, error))?;

        let state_path = self.root.join(STATE_FILE);
        if read_state(&state_path).is_ok() {
            let bytes =
                fs::read(&state_path).map_err(|error| SkillError::io(&state_path, error))?;
            atomic_write(&self.root.join(BACKUP_FILE), &bytes)?;
        }
        let bytes = serde_json::to_vec_pretty(state).map_err(|error| {
            SkillError::new(
                SkillErrorCode::Io,
                format!("Could not serialize application state: {error}"),
                Some(state_path.clone()),
            )
        })?;
        atomic_write(&state_path, &bytes)
    }
}

#[derive(Debug)]
enum StateReadError {
    Invalid(String),
    UnsupportedVersion(String),
}

impl std::fmt::Display for StateReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::UnsupportedVersion(message) => {
                formatter.write_str(message)
            }
        }
    }
}

fn read_state(path: &Path) -> Result<AppState, StateReadError> {
    let bytes = fs::read(path).map_err(|error| StateReadError::Invalid(error.to_string()))?;
    let state: AppState = serde_json::from_slice(&bytes)
        .map_err(|error| StateReadError::Invalid(error.to_string()))?;
    if state.state_version != STATE_VERSION {
        return Err(StateReadError::UnsupportedVersion(format!(
            "unsupported state version {} (expected {STATE_VERSION})",
            state.state_version
        )));
    }
    Ok(state)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), SkillError> {
    AtomicFile::new(path, AllowOverwrite)
        .write(|file| {
            file.write_all(bytes)?;
            file.sync_all()
        })
        .map_err(|error| {
            SkillError::new(
                SkillErrorCode::Io,
                format!("Could not atomically write state: {error}"),
                Some(path.to_path_buf()),
            )
        })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn missing_state_starts_empty_without_writing() {
        let temp = TempDir::new().unwrap();
        let store = StateStore::new(temp.path().to_path_buf());

        let loaded = store.load().unwrap();

        assert_eq!(loaded.mode, StateMode::Active);
        assert_eq!(loaded.state, Some(AppState::default()));
        assert!(!temp.path().join(STATE_FILE).exists());
    }

    #[test]
    fn save_keeps_one_last_valid_backup() {
        let temp = TempDir::new().unwrap();
        let store = StateStore::new(temp.path().to_path_buf());
        let first = AppState::default();
        store.save(&first).unwrap();

        let mut second = first.clone();
        second.packages.push(package("example"));
        store.save(&second).unwrap();

        assert_eq!(store.load().unwrap().state, Some(second));
        assert_eq!(read_state(&temp.path().join(BACKUP_FILE)).unwrap(), first);
    }

    #[test]
    fn corrupt_primary_recovers_only_from_valid_backup() {
        let temp = TempDir::new().unwrap();
        let store = StateStore::new(temp.path().to_path_buf());
        let first = AppState::default();
        store.save(&first).unwrap();
        store.save(&first).unwrap();
        fs::write(temp.path().join(STATE_FILE), "not json").unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded.mode, StateMode::RecoveredBackup);
        assert_eq!(loaded.state, Some(first));
    }

    #[test]
    fn invalid_primary_and_backup_enter_read_only_recovery() {
        let temp = TempDir::new().unwrap();
        let store = StateStore::new(temp.path().to_path_buf());
        fs::write(
            temp.path().join(STATE_FILE),
            r#"{"stateVersion":99,"packages":[]}"#,
        )
        .unwrap();
        fs::write(temp.path().join(BACKUP_FILE), "not json").unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded.mode, StateMode::ReadOnlyRecovery);
        assert!(loaded.state.is_none());
    }

    #[test]
    fn future_primary_version_never_falls_back_to_an_older_backup() {
        let temp = TempDir::new().unwrap();
        let store = StateStore::new(temp.path().to_path_buf());
        fs::write(
            temp.path().join(STATE_FILE),
            r#"{"stateVersion":99,"packages":[]}"#,
        )
        .unwrap();
        fs::write(
            temp.path().join(BACKUP_FILE),
            r#"{"stateVersion":1,"packages":[]}"#,
        )
        .unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded.mode, StateMode::ReadOnlyRecovery);
        assert!(loaded.state.is_none());
        assert!(loaded
            .diagnostic
            .unwrap()
            .contains("unsupported state version"));
    }

    #[test]
    fn git_source_uses_snake_case_type_and_camel_case_fields() {
        let source = SkillSource::Git {
            repository_url: "https://example.com/repo.git".to_owned(),
            subpath: "skills/example".to_owned(),
            tracked_branch: "main".to_owned(),
        };

        let json = serde_json::to_value(source).unwrap();

        assert_eq!(json["type"], "git");
        assert_eq!(json["repositoryUrl"], "https://example.com/repo.git");
        assert_eq!(json["trackedBranch"], "main");
        assert!(json.get("repository_url").is_none());
    }

    fn package(name: &str) -> ManagedSkillPackage {
        ManagedSkillPackage {
            id: "stable-id".to_owned(),
            name: name.to_owned(),
            library_path: PathBuf::from("library/example/current"),
            source: SkillSource::LocalSnapshot,
            installed_revision: InstalledRevision {
                fingerprint: "fingerprint".to_owned(),
                commit_oid: None,
            },
            previous_revision: None,
            installations: Vec::new(),
        }
    }
}
