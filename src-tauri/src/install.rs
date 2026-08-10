use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{
    configuration,
    inventory::{self, Agent, AgentRoots},
    library::{copy_directory, remove_directory},
    skill::{self, SkillError, SkillErrorCode},
    state::{AppState, DeploymentMode, Installation, ManagedSkillPackage, StateMode, StateStore},
};

pub(crate) const RESTART_MESSAGE: &str = "变更已保存；若 Agent 未反映，请重启";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallTargetPlan {
    pub agent: Agent,
    pub logical_path: PathBuf,
    pub root_exists: bool,
    pub preferred_mode: DeploymentMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlan {
    pub id: String,
    pub package_id: String,
    pub targets: Vec<InstallTargetPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub package: ManagedSkillPackage,
    pub restart_message: &'static str,
}

#[derive(Debug, Clone)]
struct PendingInstallPlan {
    public: InstallPlan,
    roots: AgentRoots,
    create_missing_roots: bool,
}

#[derive(Default)]
pub struct InstallManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingInstallPlan>>,
}

impl InstallManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan(
        &self,
        app_data: &Path,
        package_id: &str,
        targets: Vec<Agent>,
        create_missing_roots: bool,
    ) -> Result<InstallPlan, SkillError> {
        self.plan_for_roots(
            app_data,
            package_id,
            targets,
            create_missing_roots,
            inventory::agent_roots()?,
        )
    }

    pub(crate) fn plan_for_roots(
        &self,
        app_data: &Path,
        package_id: &str,
        targets: Vec<Agent>,
        create_missing_roots: bool,
        roots: AgentRoots,
    ) -> Result<InstallPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = package(&state, package_id)?;
        validate_library(package)?;
        let targets = preflight(package, &targets, create_missing_roots, &roots)?;
        let id = crate::library::next_id();
        let public = InstallPlan {
            id: id.clone(),
            package_id: package_id.to_owned(),
            targets,
        };
        self.plans.lock().map_err(|_| lock_error())?.insert(
            id,
            PendingInstallPlan {
                public: public.clone(),
                roots,
                create_missing_roots,
            },
        );
        Ok(public)
    }

    pub fn commit(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_copy_fallback: bool,
    ) -> Result<InstallResult, SkillError> {
        self.commit_with_linker(
            app_data,
            plan_id,
            confirm_copy_fallback,
            create_preferred_link,
        )
    }

    fn commit_with_linker<F>(
        &self,
        app_data: &Path,
        plan_id: &str,
        confirm_copy_fallback: bool,
        mut linker: F,
    ) -> Result<InstallResult, SkillError>
    where
        F: FnMut(&Path, &Path) -> io::Result<DeploymentMode>,
    {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let plan = self
            .plans
            .lock()
            .map_err(|_| lock_error())?
            .remove(plan_id)
            .ok_or_else(invalid_plan)?;
        let mut state = load_writable_state(app_data)?;
        let package_index = state
            .packages
            .iter()
            .position(|candidate| candidate.id == plan.public.package_id)
            .ok_or_else(|| missing_package(&plan.public.package_id))?;
        let package = &state.packages[package_index];
        validate_library(package)?;
        let agents = plan
            .public
            .targets
            .iter()
            .map(|target| target.agent)
            .collect::<Vec<_>>();
        let targets = preflight(package, &agents, plan.create_missing_roots, &plan.roots)?;

        let mut created_roots = Vec::new();
        for target in &targets {
            let root = target
                .logical_path
                .parent()
                .expect("installation path has an Agent root");
            if !root.exists() {
                let missing = missing_directories(root);
                if let Err(error) = fs::create_dir_all(root) {
                    created_roots.extend(missing);
                    rollback(&[], &created_roots)?;
                    return Err(SkillError::io(root, error));
                }
                created_roots.extend(missing);
            }
        }

        let mut created = Vec::new();
        let mut installations = Vec::new();
        for target in targets {
            let link_result = linker(&package.library_path, &target.logical_path);
            let deployment_mode = match link_result {
                Ok(mode) => mode,
                Err(_) if confirm_copy_fallback => {
                    if let Err(error) = remove_created_entry(&target.logical_path) {
                        rollback(&created, &created_roots)?;
                        return Err(error);
                    }
                    if let Err(error) = copy_atomically(
                        &package.library_path,
                        &target.logical_path,
                        plan_id,
                        &package.name,
                        &package.installed_revision.fingerprint,
                    ) {
                        rollback(&created, &created_roots)?;
                        return Err(error);
                    }
                    DeploymentMode::CopyFallback
                }
                Err(link_error) => {
                    let cleanup_error = remove_created_entry(&target.logical_path).err();
                    rollback(&created, &created_roots)?;
                    if let Some(error) = cleanup_error {
                        return Err(error);
                    }
                    return Err(SkillError::new(
                        SkillErrorCode::CopyFallbackRequired,
                        format!("Could not create the preferred linked installation: {link_error}"),
                        Some(target.logical_path),
                    ));
                }
            };
            created.push((target.logical_path.clone(), deployment_mode));
            let (enabled, configuration_provenance) = match configuration::initial_configuration(
                target.agent,
                &target.logical_path,
                &package.name,
                &plan.roots,
            ) {
                Ok(configuration) => configuration,
                Err(error) => {
                    rollback(&created, &created_roots)?;
                    return Err(error);
                }
            };
            installations.push(Installation {
                agent: target.agent,
                logical_path: target.logical_path.clone(),
                resolved_target: if deployment_mode == DeploymentMode::CopyFallback {
                    target.logical_path.clone()
                } else {
                    package.library_path.clone()
                },
                deployment_mode,
                enabled,
                last_known_fingerprint: package.installed_revision.fingerprint.clone(),
                configuration_provenance,
            });
        }

        state.packages[package_index]
            .installations
            .extend(installations);
        let result_package = state.packages[package_index].clone();
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            rollback(&created, &created_roots)?;
            return Err(error);
        }
        Ok(InstallResult {
            package: result_package,
            restart_message: RESTART_MESSAGE,
        })
    }
}

fn preflight(
    package: &ManagedSkillPackage,
    agents: &[Agent],
    create_missing_roots: bool,
    roots: &AgentRoots,
) -> Result<Vec<InstallTargetPlan>, SkillError> {
    if agents.is_empty() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidPlan,
            "Install requires at least one explicitly selected Agent target",
            None,
        ));
    }
    let mut targets = Vec::with_capacity(agents.len());
    for &agent in agents {
        if targets
            .iter()
            .any(|target: &InstallTargetPlan| target.agent == agent)
        {
            return Err(SkillError::new(
                SkillErrorCode::InvalidPlan,
                "Each Agent target may be selected only once",
                None,
            ));
        }
        if package
            .installations
            .iter()
            .any(|installation| installation.agent == agent)
        {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "The Managed Skill Package is already installed for this Agent",
                Some(package.library_path.clone()),
            ));
        }
        let root = match agent {
            Agent::Codex => &roots.codex,
            Agent::Claude => &roots.claude,
        };
        let root_exists = root.is_dir();
        if !root_exists && !create_missing_roots {
            return Err(SkillError::new(
                SkillErrorCode::AgentRootMissing,
                "The Agent Skill root is missing and creation was not confirmed",
                Some(root.clone()),
            ));
        }
        if root.exists() && !root_exists {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "The Agent Skill root exists but is not a directory",
                Some(root.clone()),
            ));
        }
        let logical_path = root.join(&package.name);
        ensure_absent(&logical_path)?;
        if agent == Agent::Codex {
            let legacy = roots.codex_legacy.join(&package.name);
            if path_exists(&legacy)? {
                return Err(SkillError::new(
                    SkillErrorCode::LegacyConflict,
                    "A Codex legacy entry with this name must be removed outside Skill Deck before installation",
                    Some(legacy),
                ));
            }
        }
        targets.push(InstallTargetPlan {
            agent,
            logical_path,
            root_exists,
            preferred_mode: preferred_mode(),
        });
    }
    Ok(targets)
}

fn validate_library(package: &ManagedSkillPackage) -> Result<(), SkillError> {
    let validated = skill::validate_installed_revision(&package.library_path, &package.name)?;
    if validated.metadata.name != package.name
        || validated.fingerprint != package.installed_revision.fingerprint
    {
        return Err(SkillError::new(
            SkillErrorCode::SourceChanged,
            "Managed Library content no longer matches the Installed Revision",
            Some(package.library_path.clone()),
        ));
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

fn package<'a>(
    state: &'a AppState,
    package_id: &str,
) -> Result<&'a ManagedSkillPackage, SkillError> {
    state
        .packages
        .iter()
        .find(|package| package.id == package_id)
        .ok_or_else(|| missing_package(package_id))
}

fn missing_package(package_id: &str) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        format!("Managed Skill Package {package_id} does not exist"),
        None,
    )
}

fn ensure_absent(path: &Path) -> Result<(), SkillError> {
    if path_exists(path)? {
        return Err(SkillError::new(
            SkillErrorCode::Conflict,
            "The Agent target already contains an entry with this Skill name",
            Some(path.to_path_buf()),
        ));
    }
    Ok(())
}

fn path_exists(path: &Path) -> Result<bool, SkillError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(SkillError::io(path, error)),
    }
}

fn missing_directories(path: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    let mut candidate = path;
    while !candidate.exists() {
        missing.push(candidate.to_path_buf());
        let Some(parent) = candidate.parent() else {
            break;
        };
        candidate = parent;
    }
    missing.reverse();
    missing
}

pub(crate) fn copy_atomically(
    source: &Path,
    destination: &Path,
    plan_id: &str,
    expected_name: &str,
    expected_fingerprint: &str,
) -> Result<(), SkillError> {
    let parent = destination
        .parent()
        .expect("installation path has an Agent root");
    let staging = parent.join(format!(".skill-deck-{plan_id}"));
    remove_directory(&staging)?;
    if let Err(error) = copy_directory(source, &staging) {
        remove_directory(&staging)?;
        return Err(error);
    }
    let copied = match skill::validate_installed_revision(&staging, expected_name) {
        Ok(copied) => copied,
        Err(error) => {
            remove_directory(&staging)?;
            return Err(error);
        }
    };
    if copied.fingerprint != expected_fingerprint {
        remove_directory(&staging)?;
        return Err(SkillError::new(
            SkillErrorCode::SourceChanged,
            "Managed Library content changed while Copy Fallback was being prepared",
            Some(source.to_path_buf()),
        ));
    }
    if let Err(error) = fs::rename(&staging, destination) {
        remove_directory(&staging)?;
        return Err(SkillError::io(destination, error));
    }
    Ok(())
}

fn rollback(
    created: &[(PathBuf, DeploymentMode)],
    created_roots: &[PathBuf],
) -> Result<(), SkillError> {
    let mut first_error = None;
    for (path, mode) in created.iter().rev() {
        let result = match mode {
            DeploymentMode::CopyFallback => remove_directory(path),
            DeploymentMode::Symlink => {
                fs::remove_file(path).map_err(|error| SkillError::io(path, error))
            }
            DeploymentMode::Junction => {
                fs::remove_dir(path).map_err(|error| SkillError::io(path, error))
            }
        };
        if first_error.is_none() {
            first_error = result.err();
        }
    }
    for root in created_roots.iter().rev() {
        if let Err(error) = fs::remove_dir(root) {
            if error.kind() != io::ErrorKind::NotFound && first_error.is_none() {
                first_error = Some(SkillError::io(root, error));
            }
        }
    }
    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(())
    }
}

pub(crate) fn remove_created_entry(path: &Path) -> Result<(), SkillError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(SkillError::io(path, error)),
    };
    if metadata.file_type().is_symlink() {
        remove_platform_link(path)
    } else if metadata.is_dir() {
        remove_directory(path)
    } else {
        fs::remove_file(path).map_err(|error| SkillError::io(path, error))
    }
}

#[cfg(unix)]
fn remove_platform_link(path: &Path) -> Result<(), SkillError> {
    fs::remove_file(path).map_err(|error| SkillError::io(path, error))
}

#[cfg(windows)]
fn remove_platform_link(path: &Path) -> Result<(), SkillError> {
    fs::remove_dir(path).map_err(|error| SkillError::io(path, error))
}

#[cfg(unix)]
pub(crate) fn create_preferred_link(
    source: &Path,
    destination: &Path,
) -> io::Result<DeploymentMode> {
    use std::os::unix::fs::symlink;

    let parent = destination
        .parent()
        .expect("installation path has a parent");
    symlink(relative_path(parent, source)?, destination)?;
    Ok(DeploymentMode::Symlink)
}

#[cfg(windows)]
pub(crate) fn create_preferred_link(
    source: &Path,
    destination: &Path,
) -> io::Result<DeploymentMode> {
    use std::{os::windows::process::CommandExt, process::Command};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(destination)
        .arg(source)
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!("mklink /J exited with {status}")));
    }
    Ok(DeploymentMode::Junction)
}

#[cfg(unix)]
fn relative_path(from: &Path, to: &Path) -> io::Result<PathBuf> {
    use std::path::Component;

    let from = from.canonicalize()?;
    let to = to.canonicalize()?;
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut result = PathBuf::new();
    for component in &from[common..] {
        if matches!(component, Component::Normal(_)) {
            result.push("..");
        }
    }
    for component in &to[common..] {
        result.push(component.as_os_str());
    }
    Ok(result)
}

pub(crate) const fn preferred_mode() -> DeploymentMode {
    #[cfg(windows)]
    {
        DeploymentMode::Junction
    }
    #[cfg(not(windows))]
    {
        DeploymentMode::Symlink
    }
}

fn busy_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Busy,
        "Another installation mutation is already running",
        None,
    )
}

fn lock_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Io,
        "The in-memory installation plan store is unavailable",
        None,
    )
}

fn invalid_plan() -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        "The Install plan is missing or was already committed",
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibraryManager;
    use crate::state::{ConfigurationProvenance, InstalledRevision, SkillSource};
    use tempfile::TempDir;

    fn fixture() -> (TempDir, PathBuf, AgentRoots, ManagedSkillPackage) {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        let library_path = app_data.join("library/alpha-skill/current");
        fs::create_dir_all(&library_path).unwrap();
        fs::write(
            library_path.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Install fixture\n---\n",
        )
        .unwrap();
        let fingerprint = skill::validate_installed_revision(&library_path, "alpha-skill")
            .unwrap()
            .fingerprint;
        let package = ManagedSkillPackage {
            id: "package-1".to_owned(),
            name: "alpha-skill".to_owned(),
            library_path,
            source: SkillSource::LocalSnapshot,
            installed_revision: InstalledRevision {
                fingerprint,
                commit_oid: None,
            },
            previous_revision: None,
            installations: Vec::new(),
        };
        StateStore::new(app_data.clone())
            .save(&AppState {
                state_version: 1,
                packages: vec![package.clone()],
            })
            .unwrap();
        let roots = AgentRoots {
            codex: temp.path().join("codex/skills"),
            claude: temp.path().join("claude/skills"),
            codex_legacy: temp.path().join("legacy/skills"),
        };
        fs::create_dir_all(&roots.codex).unwrap();
        fs::create_dir_all(&roots.claude).unwrap();
        (temp, app_data, roots, package)
    }

    #[cfg(unix)]
    #[test]
    fn installs_two_relative_links_and_records_both_atomically() {
        let (_temp, app_data, roots, package) = fixture();
        let manager = InstallManager::default();
        let plan = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Codex, Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap();

        let result = manager
            .commit_with_linker(&app_data, &plan.id, false, create_preferred_link)
            .unwrap();

        assert_eq!(result.package.installations.len(), 2);
        assert_eq!(result.restart_message, RESTART_MESSAGE);
        for root in [&roots.codex, &roots.claude] {
            let logical = root.join("alpha-skill");
            assert!(!fs::read_link(&logical).unwrap().is_absolute());
            assert_eq!(
                logical.canonicalize().unwrap(),
                package.library_path.canonicalize().unwrap()
            );
        }
    }

    #[test]
    fn conflict_is_preflighted_without_overwrite() {
        let (_temp, app_data, roots, package) = fixture();
        let conflict = roots.claude.join("alpha-skill");
        fs::create_dir(&conflict).unwrap();
        fs::write(conflict.join("keep.txt"), "user data").unwrap();

        let error = InstallManager::default()
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Codex, Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::Conflict);
        assert_eq!(
            fs::read_to_string(conflict.join("keep.txt")).unwrap(),
            "user data"
        );
        assert!(!roots.codex.join("alpha-skill").exists());
    }

    #[test]
    fn install_plan_uses_the_frontend_camel_case_contract() {
        let (_temp, app_data, roots, package) = fixture();
        let plan = InstallManager::default()
            .plan_for_roots(&app_data, &package.id, vec![Agent::Claude], false, roots)
            .unwrap();
        let json = serde_json::to_value(plan).unwrap();

        assert_eq!(json["packageId"], "package-1");
        assert_eq!(json["targets"][0]["agent"], "claude");
        assert_eq!(json["targets"][0]["rootExists"], true);
        assert_eq!(json["targets"][0]["preferredMode"], preferred_mode_json());
        assert!(json.get("package_id").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn second_target_link_failure_rolls_back_first_target() {
        let (_temp, app_data, roots, package) = fixture();
        let manager = InstallManager::default();
        let plan = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Codex, Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap();
        let mut calls = 0;

        let error = manager
            .commit_with_linker(&app_data, &plan.id, false, |source, destination| {
                calls += 1;
                if calls == 2 {
                    Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
                } else {
                    create_preferred_link(source, destination)
                }
            })
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::CopyFallbackRequired);
        assert!(!path_exists(&roots.codex.join("alpha-skill")).unwrap());
        assert!(!path_exists(&roots.claude.join("alpha-skill")).unwrap());
        let state = StateStore::new(app_data).load().unwrap().state.unwrap();
        assert!(state.packages[0].installations.is_empty());
    }

    #[test]
    fn fallback_requires_a_fresh_explicit_confirmation() {
        let (_temp, app_data, roots, package) = fixture();
        let manager = InstallManager::default();
        let first = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap();
        let error = manager
            .commit_with_linker(&app_data, &first.id, false, |_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
            })
            .unwrap_err();
        assert_eq!(error.code, SkillErrorCode::CopyFallbackRequired);

        let confirmed = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap();
        let result = manager
            .commit_with_linker(&app_data, &confirmed.id, true, |_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
            })
            .unwrap();

        assert_eq!(
            result.package.installations[0].deployment_mode,
            DeploymentMode::CopyFallback
        );
        assert!(roots.claude.join("alpha-skill/SKILL.md").is_file());
    }

    #[test]
    fn codex_legacy_duplicate_blocks_current_root_install() {
        let (_temp, app_data, roots, package) = fixture();
        fs::create_dir_all(roots.codex_legacy.join("alpha-skill")).unwrap();

        let error = InstallManager::default()
            .plan_for_roots(&app_data, &package.id, vec![Agent::Codex], false, roots)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::LegacyConflict);
    }

    #[cfg(unix)]
    #[test]
    fn missing_root_is_created_only_after_explicit_confirmation() {
        let (_temp, app_data, mut roots, package) = fixture();
        roots.claude = roots.claude.join("missing/nested");
        let manager = InstallManager::default();
        let error = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Claude],
                false,
                roots.clone(),
            )
            .unwrap_err();
        assert_eq!(error.code, SkillErrorCode::AgentRootMissing);
        assert!(!roots.claude.exists());

        let plan = manager
            .plan_for_roots(
                &app_data,
                &package.id,
                vec![Agent::Claude],
                true,
                roots.clone(),
            )
            .unwrap();
        manager
            .commit_with_linker(&app_data, &plan.id, false, create_preferred_link)
            .unwrap();
        assert!(roots.claude.join("alpha-skill").exists());
    }

    #[test]
    fn fallback_decline_removes_every_created_root_directory() {
        let (temp, app_data, mut roots, package) = fixture();
        roots.claude = temp.path().join("new-agent/nested/skills");
        let manager = InstallManager::default();
        let plan = manager
            .plan_for_roots(&app_data, &package.id, vec![Agent::Claude], true, roots)
            .unwrap();

        let error = manager
            .commit_with_linker(&app_data, &plan.id, false, |_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
            })
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::CopyFallbackRequired);
        assert!(!temp.path().join("new-agent").exists());
    }

    #[test]
    fn install_preserves_external_claude_configuration() {
        let (_temp, app_data, roots, package) = fixture();
        let settings = roots.claude.parent().unwrap().join("settings.json");
        fs::write(
            &settings,
            r#"{"skillOverrides":{"alpha-skill":"off"},"theme":"dark"}"#,
        )
        .unwrap();
        let manager = InstallManager::default();
        let plan = manager
            .plan_for_roots(&app_data, &package.id, vec![Agent::Claude], false, roots)
            .unwrap();

        let result = manager
            .commit_with_linker(&app_data, &plan.id, true, |_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
            })
            .unwrap();

        let installation = &result.package.installations[0];
        assert!(!installation.enabled);
        assert_eq!(
            installation.configuration_provenance,
            ConfigurationProvenance::External { path: settings }
        );
    }

    #[test]
    fn shared_gate_rejects_cross_manager_mutations_without_losing_state() {
        let (temp, app_data, roots, package) = fixture();
        let source = temp.path().join("sources/beta-skill");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("SKILL.md"),
            "---\nname: beta-skill\ndescription: Shared gate fixture\n---\n",
        )
        .unwrap();
        let gate = Arc::new(Mutex::new(()));
        let library = LibraryManager::new(gate.clone());
        let installer = InstallManager::new(gate.clone());

        let held = gate.lock().unwrap();
        assert_eq!(
            library.plan_local(&app_data, &source).unwrap_err().code,
            SkillErrorCode::Busy
        );
        assert_eq!(
            installer
                .plan_for_roots(
                    &app_data,
                    &package.id,
                    vec![Agent::Claude],
                    false,
                    roots.clone(),
                )
                .unwrap_err()
                .code,
            SkillErrorCode::Busy
        );
        drop(held);

        let add = library.plan_local(&app_data, &source).unwrap();
        library.commit_local(&app_data, &add.id).unwrap();
        let install = installer
            .plan_for_roots(&app_data, &package.id, vec![Agent::Claude], false, roots)
            .unwrap();
        installer
            .commit_with_linker(&app_data, &install.id, true, |_, _| {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"))
            })
            .unwrap();

        let state = StateStore::new(app_data).load().unwrap().state.unwrap();
        assert_eq!(state.packages.len(), 2);
        assert_eq!(state.packages[0].installations.len(), 1);
        assert!(state
            .packages
            .iter()
            .any(|package| package.name == "beta-skill"));
    }

    const fn preferred_mode_json() -> &'static str {
        if cfg!(windows) {
            "junction"
        } else {
            "symlink"
        }
    }
}
