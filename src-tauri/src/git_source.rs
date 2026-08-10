use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use git2::{build::RepoBuilder, FetchOptions, Oid, RemoteCallbacks, Repository};
use serde::Serialize;

use crate::{
    library::{copy_directory, ensure_name_available, next_id, remove_directory},
    revision::{self, ChangeDisclosure, RevisionResult},
    skill::{self, SkillError, SkillErrorCode, ValidatedSkill},
    state::{InstalledRevision, ManagedSkillPackage, SkillSource, StateMode, StateStore},
};

const MIB: u64 = 1024 * 1024;
const TRANSFER_LIMIT: u64 = 250 * MIB;
const CHECKOUT_LIMIT: u64 = 500 * MIB;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitImportPlan {
    pub id: String,
    pub repository_url: String,
    pub subpath: String,
    pub tracked_branch: String,
    pub commit_oid: String,
    pub skill: ValidatedSkill,
    pub library_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitUpdateStatus {
    Equal,
    FastForward,
    Diverged,
    SourceUnreachable,
    SourceMissing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitUpdatePlan {
    pub id: String,
    pub package_id: String,
    pub from_commit_oid: String,
    pub to_commit_oid: String,
    pub candidate: ValidatedSkill,
    pub changes: ChangeDisclosure,
    pub installation_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitUpdateCheck {
    pub status: GitUpdateStatus,
    pub package_id: String,
    pub installed_commit_oid: String,
    pub remote_commit_oid: Option<String>,
    pub plan: Option<GitUpdatePlan>,
}

#[derive(Debug, Clone)]
struct PendingImport {
    public: GitImportPlan,
    repository_path: PathBuf,
    staged_skill: PathBuf,
}

#[derive(Debug, Clone)]
struct PendingUpdate {
    public: GitUpdatePlan,
    repository_path: PathBuf,
    staged_skill: PathBuf,
    package: ManagedSkillPackage,
    source: GitCoordinates,
}

#[derive(Debug, Clone)]
struct GitCoordinates {
    repository_url: String,
    subpath: String,
    tracked_branch: String,
}

#[derive(Debug, Clone)]
enum PendingPlan {
    Import(Box<PendingImport>),
    Update(Box<PendingUpdate>),
}

#[derive(Default)]
pub struct GitSourceManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl GitSourceManager {
    pub fn new(mutation: Arc<Mutex<()>>) -> Self {
        Self {
            mutation,
            plans: Mutex::default(),
        }
    }

    pub fn plan_import(
        &self,
        app_data: &Path,
        repository_url: &str,
        subpath: &str,
        tracked_branch: &str,
    ) -> Result<GitImportPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let coordinates = validate_coordinates(repository_url, subpath, tracked_branch)?;
        self.plan_import_coordinates(app_data, coordinates)
    }

    fn plan_import_coordinates(
        &self,
        app_data: &Path,
        coordinates: GitCoordinates,
    ) -> Result<GitImportPlan, SkillError> {
        let id = next_id();
        let plan_root = app_data.join("staging").join(&id);
        let result = (|| {
            let repository_path = plan_root.join("repository");
            let (repository, commit_oid) = clone_branch(&coordinates, &repository_path)?;
            validate_repository(&repository, &repository_path)?;
            let staged_skill = plan_root.join(selected_directory_name(&coordinates)?);
            stage_selected_skill(&repository_path, &coordinates.subpath, &staged_skill)?;
            let skill = skill::validate_skill_dir(&staged_skill)?;
            ensure_name_available(app_data, &skill.metadata.name)?;
            let public = GitImportPlan {
                id: id.clone(),
                repository_url: coordinates.repository_url,
                subpath: coordinates.subpath,
                tracked_branch: coordinates.tracked_branch,
                commit_oid,
                library_path: app_data
                    .join("library")
                    .join(&skill.metadata.name)
                    .join("current"),
                skill,
            };
            self.insert(
                id,
                PendingPlan::Import(Box::new(PendingImport {
                    public: public.clone(),
                    repository_path,
                    staged_skill,
                })),
            )?;
            Ok(public)
        })();
        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
    }

    pub fn commit_import(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<ManagedSkillPackage, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Import(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Git Import"));
        };
        let plan_root = plan
            .repository_path
            .parent()
            .expect("repository has a plan root")
            .to_path_buf();
        let result = self.commit_import_inner(app_data, *plan);
        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
    }

    fn commit_import_inner(
        &self,
        app_data: &Path,
        plan: PendingImport,
    ) -> Result<ManagedSkillPackage, SkillError> {
        let coordinates = GitCoordinates {
            repository_url: plan.public.repository_url.clone(),
            subpath: plan.public.subpath.clone(),
            tracked_branch: plan.public.tracked_branch.clone(),
        };
        let repository = Repository::open(&plan.repository_path)
            .map_err(|error| source_changed(&plan.repository_path, error))?;
        let current_oid = fetch_branch(&repository, &coordinates)?;
        if current_oid != plan.public.commit_oid {
            return Err(stale_remote(&plan.repository_path));
        }
        drop(repository);
        let staged = skill::validate_installed_revision(
            &plan.staged_skill,
            &plan.public.skill.metadata.name,
        )?;
        if staged.fingerprint != plan.public.skill.fingerprint
            || staged.metadata.name != plan.public.skill.metadata.name
        {
            return Err(source_changed_message(&plan.staged_skill));
        }

        let store = StateStore::new(app_data.to_path_buf());
        let loaded = store.load()?;
        if loaded.mode == StateMode::ReadOnlyRecovery {
            return Err(read_only(app_data));
        }
        let mut state = loaded.state.expect("writable state mode contains state");
        crate::library::ensure_unique_state_name(&state.packages, &staged.metadata.name)?;
        if plan.public.library_path.exists() {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "The normalized Skill name is already present in Managed Library",
                Some(plan.public.library_path.clone()),
            ));
        }
        let package_root = plan
            .public
            .library_path
            .parent()
            .expect("library current has a package root")
            .to_path_buf();
        fs::create_dir_all(&package_root).map_err(|error| SkillError::io(&package_root, error))?;
        if let Err(error) = fs::rename(&plan.staged_skill, &plan.public.library_path) {
            remove_directory(&package_root)?;
            return Err(SkillError::io(&plan.staged_skill, error));
        }

        let package = ManagedSkillPackage {
            id: format!("git-{}", next_id()),
            name: staged.metadata.name,
            library_path: plan.public.library_path,
            source: SkillSource::Git {
                repository_url: coordinates.repository_url,
                subpath: coordinates.subpath,
                tracked_branch: coordinates.tracked_branch,
            },
            installed_revision: InstalledRevision {
                fingerprint: staged.fingerprint,
                commit_oid: Some(plan.public.commit_oid),
            },
            previous_revision: None,
            installations: Vec::new(),
        };
        state.packages.push(package.clone());
        if let Err(error) = store.save(&state) {
            remove_directory(&package_root)?;
            return Err(error);
        }
        remove_directory(
            plan.repository_path
                .parent()
                .expect("repository has a plan root"),
        )?;
        Ok(package)
    }

    pub fn check_update(
        &self,
        app_data: &Path,
        package_id: &str,
    ) -> Result<GitUpdateCheck, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = state
            .packages
            .iter()
            .find(|package| package.id == package_id)
            .ok_or_else(|| invalid_plan("Git Update"))?
            .clone();
        revision::validate_managed_package(app_data, &package)?;
        let source = coordinates_from_package(&package)?;
        let installed = package
            .installed_revision
            .commit_oid
            .clone()
            .ok_or_else(|| invalid_plan("Git Update"))?;
        let id = next_id();
        let plan_root = app_data.join("staging").join(&id);
        let repository_path = plan_root.join("repository");
        let cloned = clone_branch(&source, &repository_path);
        let (repository, remote) = match cloned {
            Ok(value) => value,
            Err(error) if error.code == SkillErrorCode::SourceMissing => {
                remove_directory(&plan_root)?;
                return Ok(check_without_plan(
                    GitUpdateStatus::SourceMissing,
                    package_id,
                    installed,
                    None,
                ));
            }
            Err(error) if error.code == SkillErrorCode::SourceUnreachable => {
                remove_directory(&plan_root)?;
                return Ok(check_without_plan(
                    GitUpdateStatus::SourceUnreachable,
                    package_id,
                    installed,
                    None,
                ));
            }
            Err(error) => return Err(error),
        };
        let checked_remote = remote.clone();
        let result = self.finish_update_check(
            id,
            package,
            source,
            installed.clone(),
            repository,
            repository_path,
            remote,
        );
        let result = match result {
            Err(error) if error.code == SkillErrorCode::SourceMissing => Ok(check_without_plan(
                GitUpdateStatus::SourceMissing,
                package_id,
                installed.clone(),
                Some(checked_remote),
            )),
            other => other,
        };
        if !matches!(
            &result,
            Ok(GitUpdateCheck {
                status: GitUpdateStatus::FastForward,
                ..
            })
        ) {
            remove_directory(&plan_root)?;
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_update_check(
        &self,
        id: String,
        package: ManagedSkillPackage,
        source: GitCoordinates,
        installed: String,
        repository: Repository,
        repository_path: PathBuf,
        remote: String,
    ) -> Result<GitUpdateCheck, SkillError> {
        if remote == installed {
            return Ok(check_without_plan(
                GitUpdateStatus::Equal,
                &package.id,
                installed,
                Some(remote),
            ));
        }
        let installed_oid = match Oid::from_str(&installed)
            .ok()
            .filter(|oid| repository.find_commit(*oid).is_ok())
        {
            Some(oid) => oid,
            None => {
                return Ok(check_without_plan(
                    GitUpdateStatus::Diverged,
                    &package.id,
                    installed,
                    Some(remote),
                ));
            }
        };
        let remote_oid = Oid::from_str(&remote).map_err(|error| {
            SkillError::new(
                SkillErrorCode::SourceMissing,
                format!("Remote branch HEAD is invalid: {error}"),
                Some(repository_path.clone()),
            )
        })?;
        if !repository
            .graph_descendant_of(remote_oid, installed_oid)
            .unwrap_or(false)
        {
            return Ok(check_without_plan(
                GitUpdateStatus::Diverged,
                &package.id,
                installed,
                Some(remote),
            ));
        }
        validate_repository(&repository, &repository_path)?;
        let staged_skill = repository_path
            .parent()
            .expect("repository has a plan root")
            .join(&package.name);
        stage_selected_skill(&repository_path, &source.subpath, &staged_skill).map_err(
            |error| {
                if error.code == SkillErrorCode::Io
                    || error.code == SkillErrorCode::InvalidStructure
                {
                    SkillError::new(
                        SkillErrorCode::SourceMissing,
                        "The configured Skill subpath is missing from the tracked branch",
                        Some(repository_path.join(&source.subpath)),
                    )
                } else {
                    error
                }
            },
        )?;
        let candidate = skill::validate_installed_revision(&staged_skill, &package.name)?;
        if candidate.metadata.name != package.name {
            return Err(SkillError::new(
                SkillErrorCode::InvalidMetadata,
                "A Git update cannot change the Managed Skill Package name",
                Some(staged_skill),
            ));
        }
        let current = skill::validate_installed_revision(&package.library_path, &package.name)?;
        let public = GitUpdatePlan {
            id: id.clone(),
            package_id: package.id.clone(),
            from_commit_oid: installed.clone(),
            to_commit_oid: remote.clone(),
            changes: revision::change_disclosure(&current, &candidate),
            candidate,
            installation_count: package.installations.len(),
        };
        self.insert(
            id,
            PendingPlan::Update(Box::new(PendingUpdate {
                public: public.clone(),
                repository_path,
                staged_skill,
                package: package.clone(),
                source,
            })),
        )?;
        Ok(GitUpdateCheck {
            status: GitUpdateStatus::FastForward,
            package_id: package.id,
            installed_commit_oid: installed,
            remote_commit_oid: Some(remote),
            plan: Some(public),
        })
    }

    pub fn commit_update(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<RevisionResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let PendingPlan::Update(plan) = self.take(plan_id)? else {
            return Err(invalid_plan("Git Update"));
        };
        let plan_root = plan
            .repository_path
            .parent()
            .expect("repository has a plan root")
            .to_path_buf();
        let result = (|| {
            let state = load_writable_state(app_data)?;
            let current_package = state
                .packages
                .iter()
                .find(|package| package.id == plan.package.id)
                .ok_or_else(|| invalid_plan("Git Update"))?;
            if current_package != &plan.package {
                return Err(stale_remote(&plan.repository_path));
            }
            let repository = Repository::open(&plan.repository_path)
                .map_err(|error| source_changed(&plan.repository_path, error))?;
            let remote = fetch_branch(&repository, &plan.source)?;
            if remote != plan.public.to_commit_oid {
                return Err(stale_remote(&plan.repository_path));
            }
            drop(repository);
            let candidate =
                skill::validate_installed_revision(&plan.staged_skill, &plan.package.name)?;
            if candidate.fingerprint != plan.public.candidate.fingerprint
                || candidate.metadata.name != plan.package.name
            {
                return Err(source_changed_message(&plan.staged_skill));
            }
            revision::apply_staged_revision(
                app_data,
                plan_id,
                &plan.staged_skill,
                &plan.package,
                InstalledRevision {
                    fingerprint: candidate.fingerprint,
                    commit_oid: Some(remote),
                },
            )
        })();
        if result.is_err() {
            remove_directory(&plan_root)?;
        }
        result
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
            .ok_or_else(|| invalid_plan("Git Source"))
    }
}

fn validate_coordinates(
    repository_url: &str,
    subpath: &str,
    tracked_branch: &str,
) -> Result<GitCoordinates, SkillError> {
    let repository_url = repository_url.trim();
    let authority = repository_url
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .filter(|authority| !authority.is_empty() && !authority.contains('@'));
    if authority.is_none() || repository_url.contains(['?', '#']) {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Git sources must use a public HTTPS repository URL without credentials or query data",
            None,
        ));
    }
    if !valid_branch(tracked_branch) {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Tracked branch must be an explicit branch name",
            None,
        ));
    }
    let subpath = normalize_subpath(subpath)?;
    Ok(GitCoordinates {
        repository_url: repository_url.to_owned(),
        subpath,
        tracked_branch: tracked_branch.to_owned(),
    })
}

fn valid_branch(branch: &str) -> bool {
    !branch.is_empty()
        && !branch.starts_with('-')
        && !branch.starts_with('/')
        && !branch.starts_with('.')
        && !branch.ends_with('/')
        && !branch.ends_with('.')
        && !branch.starts_with("refs/")
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.contains("@{")
        && branch
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.'))
}

fn normalize_subpath(subpath: &str) -> Result<String, SkillError> {
    if subpath == "." {
        return Ok(".".to_owned());
    }
    if subpath.is_empty() || subpath.starts_with('/') || subpath.contains('\\') {
        return Err(invalid_subpath());
    }
    let parts = subpath.split('/').collect::<Vec<_>>();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err(invalid_subpath());
    }
    Ok(parts.join("/"))
}

fn selected_directory_name(coordinates: &GitCoordinates) -> Result<String, SkillError> {
    let name = if coordinates.subpath == "." {
        coordinates
            .repository_url
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .strip_suffix(".git")
            .unwrap_or_else(|| {
                coordinates
                    .repository_url
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
            })
    } else {
        coordinates.subpath.rsplit('/').next().unwrap_or_default()
    };
    if name.is_empty() {
        return Err(invalid_subpath());
    }
    Ok(name.to_owned())
}

fn clone_branch(
    coordinates: &GitCoordinates,
    destination: &Path,
) -> Result<(Repository, String), SkillError> {
    let observed = Arc::new(AtomicU64::new(0));
    let callbacks = transfer_callbacks(observed.clone());
    let mut fetch = FetchOptions::new();
    fetch.remote_callbacks(callbacks);
    let mut builder = RepoBuilder::new();
    builder
        .branch(&coordinates.tracked_branch)
        .fetch_options(fetch);
    let repository = builder
        .clone(&coordinates.repository_url, destination)
        .map_err(|error| map_git_error(error, destination, observed.load(Ordering::Relaxed)))?;
    enforce_checkout_limit(destination)?;
    let commit_oid = remote_head(&repository, &coordinates.tracked_branch)?;
    Ok((repository, commit_oid))
}

fn fetch_branch(
    repository: &Repository,
    coordinates: &GitCoordinates,
) -> Result<String, SkillError> {
    let observed = Arc::new(AtomicU64::new(0));
    let callbacks = transfer_callbacks(observed.clone());
    let mut options = FetchOptions::new();
    options.remote_callbacks(callbacks);
    let mut remote = repository.find_remote("origin").map_err(|error| {
        SkillError::new(
            SkillErrorCode::SourceMissing,
            format!("Git origin is unavailable: {error}"),
            Some(repository.path().to_path_buf()),
        )
    })?;
    if remote.url().ok() != Some(coordinates.repository_url.as_str()) {
        return Err(SkillError::new(
            SkillErrorCode::SourceChanged,
            "The staged Git origin changed after planning",
            Some(repository.path().to_path_buf()),
        ));
    }
    remote
        .fetch(
            &[format!(
                "+refs/heads/{0}:refs/remotes/origin/{0}",
                coordinates.tracked_branch
            )],
            Some(&mut options),
            None,
        )
        .map_err(|error| {
            map_git_error(error, repository.path(), observed.load(Ordering::Relaxed))
        })?;
    enforce_checkout_limit(repository.workdir().unwrap_or_else(|| repository.path()))?;
    remote_head(repository, &coordinates.tracked_branch)
}

fn transfer_callbacks(observed: Arc<AtomicU64>) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| Err(git2::Error::from_str("authentication is disabled")));
    callbacks.transfer_progress(move |progress| {
        let bytes = progress.received_bytes() as u64;
        observed.store(bytes, Ordering::Relaxed);
        transfer_within_limit(bytes)
    });
    callbacks
}

fn transfer_within_limit(observed: u64) -> bool {
    observed <= TRANSFER_LIMIT
}

fn remote_head(repository: &Repository, branch: &str) -> Result<String, SkillError> {
    repository
        .find_reference(&format!("refs/remotes/origin/{branch}"))
        .and_then(|reference| {
            reference
                .target()
                .ok_or_else(|| git2::Error::from_str("remote branch has no direct target"))
        })
        .map(|oid| oid.to_string())
        .map_err(|error| {
            SkillError::new(
                SkillErrorCode::SourceMissing,
                format!("Tracked branch is missing: {error}"),
                Some(repository.path().to_path_buf()),
            )
        })
}

fn validate_repository(repository: &Repository, checkout: &Path) -> Result<(), SkillError> {
    enforce_checkout_limit(checkout)?;
    if !repository
        .submodules()
        .map_err(|error| {
            SkillError::new(
                SkillErrorCode::UnsupportedGitFeature,
                format!("Could not inspect Git submodules: {error}"),
                Some(checkout.to_path_buf()),
            )
        })?
        .is_empty()
    {
        return Err(SkillError::new(
            SkillErrorCode::UnsupportedGitFeature,
            "Git submodules are not supported in the MVP",
            Some(checkout.join(".gitmodules")),
        ));
    }
    reject_lfs(checkout)
}

fn reject_lfs(root: &Path) -> Result<(), SkillError> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| SkillError::io(&directory, error))? {
            let entry = entry.map_err(|error| SkillError::io(&directory, error))?;
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| SkillError::io(&path, error))?;
            if metadata.is_dir() && entry.file_name() != ".git" {
                pending.push(path);
            } else if metadata.is_file() && entry.file_name() == ".gitattributes" {
                let reader = BufReader::new(
                    fs::File::open(&path).map_err(|error| SkillError::io(&path, error))?,
                );
                for line in reader.lines() {
                    let line = line.map_err(|error| SkillError::io(&path, error))?;
                    if line.split_whitespace().any(|part| part == "filter=lfs") {
                        return Err(SkillError::new(
                            SkillErrorCode::UnsupportedGitFeature,
                            "Git LFS is not supported in the MVP",
                            Some(path),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn enforce_checkout_limit(root: &Path) -> Result<(), SkillError> {
    enforce_checkout_limit_with(root, CHECKOUT_LIMIT)
}

fn enforce_checkout_limit_with(root: &Path, limit: u64) -> Result<(), SkillError> {
    let mut observed = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| SkillError::io(&directory, error))? {
            let entry = entry.map_err(|error| SkillError::io(&directory, error))?;
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| SkillError::io(&path, error))?;
            if metadata.is_dir() {
                pending.push(path);
            } else {
                observed = observed.saturating_add(metadata.len());
                if observed > limit {
                    return Err(SkillError::limit(
                        "Git checkout repository exceeds its size limit",
                        Some(root.to_path_buf()),
                        limit,
                        observed,
                    ));
                }
            }
        }
    }
    Ok(())
}

fn stage_selected_skill(
    checkout: &Path,
    subpath: &str,
    destination: &Path,
) -> Result<(), SkillError> {
    let source = if subpath == "." {
        checkout
    } else {
        &checkout.join(subpath)
    };
    let metadata = fs::symlink_metadata(source).map_err(|error| SkillError::io(source, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(SkillError::new(
            SkillErrorCode::InvalidStructure,
            "The selected Git Skill subpath must be a real directory",
            Some(source.to_path_buf()),
        ));
    }
    if subpath != "." {
        return copy_directory(source, destination);
    }
    fs::create_dir_all(destination).map_err(|error| SkillError::io(destination, error))?;
    for entry in fs::read_dir(source).map_err(|error| SkillError::io(source, error))? {
        let entry = entry.map_err(|error| SkillError::io(source, error))?;
        if entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        let target = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&path).map_err(|error| SkillError::io(&path, error))?;
        if metadata.is_dir() {
            copy_directory(&path, &target)?;
        } else if metadata.is_file() {
            fs::copy(&path, &target).map_err(|error| SkillError::io(&path, error))?;
        } else {
            return Err(SkillError::new(
                SkillErrorCode::UnsupportedFileType,
                "Skill packages can contain only real files and directories",
                Some(path),
            ));
        }
    }
    Ok(())
}

fn coordinates_from_package(package: &ManagedSkillPackage) -> Result<GitCoordinates, SkillError> {
    let SkillSource::Git {
        repository_url,
        subpath,
        tracked_branch,
    } = &package.source
    else {
        return Err(SkillError::new(
            SkillErrorCode::Conflict,
            "Only a Git source can be checked for Git updates",
            Some(package.library_path.clone()),
        ));
    };
    Ok(GitCoordinates {
        repository_url: repository_url.clone(),
        subpath: subpath.clone(),
        tracked_branch: tracked_branch.clone(),
    })
}

fn load_writable_state(app_data: &Path) -> Result<crate::state::AppState, SkillError> {
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    if loaded.mode == StateMode::ReadOnlyRecovery {
        return Err(read_only(app_data));
    }
    Ok(loaded.state.expect("writable state mode contains state"))
}

fn check_without_plan(
    status: GitUpdateStatus,
    package_id: &str,
    installed: String,
    remote: Option<String>,
) -> GitUpdateCheck {
    GitUpdateCheck {
        status,
        package_id: package_id.to_owned(),
        installed_commit_oid: installed,
        remote_commit_oid: remote,
        plan: None,
    }
}

fn map_git_error(error: git2::Error, path: &Path, observed: u64) -> SkillError {
    if observed > TRANSFER_LIMIT {
        return SkillError::limit(
            "Git transfer exceeds its size limit",
            Some(path.to_path_buf()),
            TRANSFER_LIMIT,
            observed,
        );
    }
    let code = if matches!(
        error.code(),
        git2::ErrorCode::NotFound | git2::ErrorCode::InvalidSpec
    ) {
        SkillErrorCode::SourceMissing
    } else {
        SkillErrorCode::SourceUnreachable
    };
    SkillError::new(
        code,
        format!("Git source could not be read: {error}"),
        Some(path.to_path_buf()),
    )
}

fn source_changed(path: &Path, error: git2::Error) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        format!("The staged Git repository changed after planning: {error}"),
        Some(path.to_path_buf()),
    )
}

fn source_changed_message(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        "The staged Git Skill changed after planning",
        Some(path.to_path_buf()),
    )
}

fn stale_remote(path: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::SourceChanged,
        "The tracked branch changed after the plan was created; check again",
        Some(path.to_path_buf()),
    )
}

fn invalid_subpath() -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidStructure,
        "Git Skill subpath must be '.' for repository root or a normalized relative path",
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
        "The in-memory Git operation plan store is unavailable",
        None,
    )
}

fn read_only(app_data: &Path) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidStructure,
        "Application state is in read-only recovery mode",
        Some(app_data.join("state.json")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{IndexAddOption, Signature};
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        remote: PathBuf,
        app_data: PathBuf,
        package: ManagedSkillPackage,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            let remote = temp.path().join("remote");
            let repository = Repository::init(&remote).unwrap();
            repository.set_head("refs/heads/main").unwrap();
            write_skill(&remote, "alpha-skill", "v1", "");
            let oid = commit_all(&repository, "v1");
            let app_data = temp.path().join("app-data");
            let library_path = app_data.join("library/alpha-skill/current");
            copy_directory(&remote.join("skills/alpha-skill"), &library_path).unwrap();
            let fingerprint = skill::validate_installed_revision(&library_path, "alpha-skill")
                .unwrap()
                .fingerprint;
            let package = ManagedSkillPackage {
                id: "package-1".to_owned(),
                name: "alpha-skill".to_owned(),
                library_path,
                source: SkillSource::Git {
                    repository_url: remote.to_string_lossy().into_owned(),
                    subpath: "skills/alpha-skill".to_owned(),
                    tracked_branch: "main".to_owned(),
                },
                installed_revision: InstalledRevision {
                    fingerprint,
                    commit_oid: Some(oid.to_string()),
                },
                previous_revision: None,
                installations: Vec::new(),
            };
            let mut state = crate::state::AppState::default();
            state.packages.push(package.clone());
            StateStore::new(app_data.clone()).save(&state).unwrap();
            Self {
                _temp: temp,
                remote,
                app_data,
                package,
            }
        }

        fn commit(&self, body: &str, extra_frontmatter: &str) -> Oid {
            write_skill(&self.remote, "alpha-skill", body, extra_frontmatter);
            commit_all(&Repository::open(&self.remote).unwrap(), body)
        }
    }

    #[test]
    fn public_contract_rejects_non_https_and_requires_explicit_root() {
        let manager = GitSourceManager::default();
        let temp = TempDir::new().unwrap();
        let url_error = manager
            .plan_import(temp.path(), "ssh://example.com/repo", ".", "main")
            .unwrap_err();
        let subpath_error = manager
            .plan_import(temp.path(), "https://example.com/repo", "", "main")
            .unwrap_err();
        assert_eq!(url_error.code, SkillErrorCode::InvalidStructure);
        assert_eq!(subpath_error.code, SkillErrorCode::InvalidStructure);
        assert!(!temp.path().join("staging").exists());
    }

    #[test]
    fn imports_explicit_subpath_as_zero_installation_git_package() {
        let fixture = Fixture::new();
        let empty_app_data = fixture._temp.path().join("import-app-data");
        let coordinates = GitCoordinates {
            repository_url: fixture.remote.to_string_lossy().into_owned(),
            subpath: "skills/alpha-skill".to_owned(),
            tracked_branch: "main".to_owned(),
        };
        let manager = GitSourceManager::default();

        let plan = manager
            .plan_import_coordinates(&empty_app_data, coordinates)
            .unwrap();
        let package = manager.commit_import(&empty_app_data, &plan.id).unwrap();

        assert!(package.installations.is_empty());
        assert_eq!(package.installed_revision.commit_oid, Some(plan.commit_oid));
        assert!(matches!(package.source, SkillSource::Git { .. }));
        assert!(package.library_path.join("SKILL.md").is_file());
    }

    #[test]
    fn equal_fast_forward_and_commit_keep_one_previous_revision() {
        let fixture = Fixture::new();
        let manager = GitSourceManager::default();
        let equal = manager
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert_eq!(equal.status, GitUpdateStatus::Equal);

        let next_oid = fixture.commit("v2", "allowed-tools: Read\n");
        let update = manager
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert_eq!(update.status, GitUpdateStatus::FastForward);
        assert_eq!(update.remote_commit_oid, Some(next_oid.to_string()));
        let plan = update.plan.unwrap();
        assert_eq!(plan.changes.unknown_fields.added, ["allowed-tools"]);
        let result = manager.commit_update(&fixture.app_data, &plan.id).unwrap();
        assert_eq!(
            result.package.installed_revision.commit_oid,
            Some(next_oid.to_string())
        );
        assert_eq!(
            result.package.previous_revision.unwrap().commit_oid,
            fixture.package.installed_revision.commit_oid
        );
        assert!(!fixture.app_data.join("staging").join(plan.id).exists());
    }

    #[test]
    fn unrelated_installed_commit_is_diverged() {
        let fixture = Fixture::new();
        let unrelated = fixture._temp.path().join("unrelated");
        let repository = Repository::init(&unrelated).unwrap();
        repository.set_head("refs/heads/main").unwrap();
        write_skill(&unrelated, "other-skill", "other", "");
        let unrelated_oid = commit_all(&repository, "unrelated");
        let store = StateStore::new(fixture.app_data.clone());
        let mut state = store.load().unwrap().state.unwrap();
        state.packages[0].installed_revision.commit_oid = Some(unrelated_oid.to_string());
        store.save(&state).unwrap();
        let check = GitSourceManager::default()
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert_eq!(check.status, GitUpdateStatus::Diverged);
        assert!(check.plan.is_none());
    }

    #[test]
    fn deleted_subpath_is_source_missing_and_cleans_staging() {
        let fixture = Fixture::new();
        fs::remove_dir_all(fixture.remote.join("skills/alpha-skill")).unwrap();
        commit_all(
            &Repository::open(&fixture.remote).unwrap(),
            "remove subpath",
        );
        let check = GitSourceManager::default()
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap();
        assert_eq!(check.status, GitUpdateStatus::SourceMissing);
        assert!(fs::read_dir(fixture.app_data.join("staging"))
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true));
    }

    #[test]
    fn deleted_branch_is_source_missing() {
        let fixture = Fixture::new();
        Repository::open(&fixture.remote)
            .unwrap()
            .find_reference("refs/heads/main")
            .unwrap()
            .delete()
            .unwrap();

        let check = GitSourceManager::default()
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap();

        assert_eq!(check.status, GitUpdateStatus::SourceMissing);
    }

    #[test]
    fn commit_rejects_remote_head_that_changed_after_plan() {
        let fixture = Fixture::new();
        let manager = GitSourceManager::default();
        fixture.commit("v2", "");
        let plan = manager
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap()
            .plan
            .unwrap();
        fixture.commit("v3", "");

        let error = manager
            .commit_update(&fixture.app_data, &plan.id)
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::SourceChanged);
        let state = StateStore::new(fixture.app_data)
            .load()
            .unwrap()
            .state
            .unwrap();
        assert_eq!(state.packages[0], fixture.package);
    }

    #[test]
    fn changed_name_is_not_a_compatible_update() {
        let fixture = Fixture::new();
        fs::write(
            fixture.remote.join("skills/alpha-skill/SKILL.md"),
            "---\nname: beta-skill\ndescription: Fixture\n---\n",
        )
        .unwrap();
        commit_all(&Repository::open(&fixture.remote).unwrap(), "rename");
        let error = GitSourceManager::default()
            .check_update(&fixture.app_data, &fixture.package.id)
            .unwrap_err();
        assert_eq!(error.code, SkillErrorCode::InvalidMetadata);
        assert!(fs::read_dir(fixture.app_data.join("staging"))
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true));
    }

    #[test]
    fn repository_root_selection_excludes_git_metadata() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("alpha-skill");
        let repository = Repository::init(&root).unwrap();
        repository.set_head("refs/heads/main").unwrap();
        fs::write(
            root.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Fixture\n---\n",
        )
        .unwrap();
        commit_all(&repository, "root");
        let staged = temp.path().join("staging").join("alpha-skill");
        stage_selected_skill(&root, ".", &staged).unwrap();
        assert!(skill::validate_skill_dir(&staged).is_ok());
        assert!(!staged.join(".git").exists());
    }

    #[test]
    fn checkout_limit_reports_limit_and_observed() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("payload"), b"12345").unwrap();

        enforce_checkout_limit_with(temp.path(), 5).unwrap();
        let error = enforce_checkout_limit_with(temp.path(), 4).unwrap_err();

        assert_eq!(error.code, SkillErrorCode::ResourceLimitExceeded);
        assert_eq!(error.limit, Some(4));
        assert_eq!(error.observed, Some(5));
    }

    #[test]
    fn git_failures_and_transfer_abort_have_stable_errors() {
        let unreachable = map_git_error(git2::Error::from_str("offline"), Path::new("repo"), 0);
        let oversized = map_git_error(
            git2::Error::from_str("cancelled"),
            Path::new("repo"),
            TRANSFER_LIMIT + 1,
        );

        assert_eq!(unreachable.code, SkillErrorCode::SourceUnreachable);
        assert_eq!(oversized.code, SkillErrorCode::ResourceLimitExceeded);
        assert_eq!(oversized.limit, Some(TRANSFER_LIMIT));
        assert_eq!(oversized.observed, Some(TRANSFER_LIMIT + 1));
        assert!(transfer_within_limit(TRANSFER_LIMIT));
        assert!(!transfer_within_limit(TRANSFER_LIMIT + 1));
    }

    fn write_skill(root: &Path, name: &str, body: &str, extra_frontmatter: &str) {
        let path = root.join("skills").join(name);
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Fixture\n{extra_frontmatter}---\n{body}\n"),
        )
        .unwrap();
    }

    fn commit_all(repository: &Repository, message: &str) -> Oid {
        let mut index = repository.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.update_all(["*"], None).unwrap();
        index.write().unwrap();
        let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = Signature::now("Skill Deck Test", "noreply.invalid").unwrap();
        let parent = repository
            .head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|oid| repository.find_commit(oid).ok());
        let parents = parent.iter().collect::<Vec<_>>();
        repository
            .commit(
                Some("refs/heads/main"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )
            .unwrap()
    }
}
