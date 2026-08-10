use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    skill::{self, SkillError, SkillErrorCode, ValidatedSkill},
    state::{ManagedSkillPackage, StateMode, StateStore},
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
pub struct InventoryDiagnostic {
    pub code: SkillErrorCode,
    pub message: String,
    pub path: Option<PathBuf>,
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
    pub managed_packages: Vec<ManagedSkillPackage>,
}

pub fn inventory(app_data: &std::path::Path) -> Result<Inventory, SkillError> {
    let roots = agent_roots()?;
    let mut inventory = inventory_for_roots([
        (Agent::Codex, roots.codex, false),
        (Agent::Claude, roots.claude, false),
        (Agent::Codex, roots.codex_legacy, true),
    ])?;
    let loaded = StateStore::new(app_data.to_path_buf()).load()?;
    if loaded.mode != StateMode::ReadOnlyRecovery {
        inventory.managed_packages = loaded
            .state
            .expect("writable state mode contains state")
            .packages;
        inventory.external_installations.retain(|external| {
            !inventory.managed_packages.iter().any(|package| {
                package.installations.iter().any(|installation| {
                    installation.logical_path == external.logical_path
                        && installation.agent == external.agent
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
            external_installations.push(inspect_entry(agent, entry.path(), legacy));
        }
    }

    external_installations.sort_by(|left, right| {
        left.logical_path
            .cmp(&right.logical_path)
            .then_with(|| agent_order(left.agent).cmp(&agent_order(right.agent)))
    });
    Ok(Inventory {
        targets,
        external_installations,
        managed_packages: Vec::new(),
    })
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
            if matches!(kind, InstallationKind::Link | InstallationKind::LegacyLink) {
                InstallationKind::BrokenLink
            } else {
                InstallationKind::Invalid
            },
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

    #[test]
    fn missing_roots_are_advisory() {
        let temp = TempDir::new().unwrap();
        let inventory =
            inventory_for_roots([(Agent::Codex, temp.path().join("missing"), false)]).unwrap();

        assert!(!inventory.targets[0].exists);
        assert!(inventory.external_installations.is_empty());
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

        assert_eq!(inventory.external_installations.len(), 2);
        assert!(inventory
            .external_installations
            .iter()
            .any(|installation| installation.kind == InstallationKind::Link));
        let broken = inventory
            .external_installations
            .iter()
            .find(|installation| installation.kind == InstallationKind::BrokenLink)
            .unwrap();
        assert_eq!(broken.resolved_target, Some(temp.path().join("absent")));
    }
}
