use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use atomicwrites::{AllowOverwrite, AtomicFile};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    install::RESTART_MESSAGE,
    inventory::{self, Agent, AgentRoots},
    skill::{SkillError, SkillErrorCode},
    state::{
        AppState, ConfigurationProvenance, Installation, ManagedSkillPackage, StateMode, StateStore,
    },
};

const CODEX_MARKER: &str = "# Managed by Skill Deck";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationPlan {
    pub id: String,
    pub package_id: String,
    pub agent: Agent,
    pub enabled: bool,
    pub current_enabled: bool,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationResult {
    pub package: ManagedSkillPackage,
    pub restart_message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationResolution {
    Reapply,
    Forget,
}

#[derive(Debug, Clone)]
struct PendingPlan {
    public: ConfigurationPlan,
    roots: AgentRoots,
    original: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct ConfigurationManager {
    mutation: Arc<Mutex<()>>,
    plans: Mutex<HashMap<String, PendingPlan>>,
}

impl ConfigurationManager {
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
        agent: Agent,
        enabled: bool,
    ) -> Result<ConfigurationPlan, SkillError> {
        self.plan_for_roots(
            app_data,
            package_id,
            agent,
            enabled,
            inventory::agent_roots()?,
        )
    }

    pub(crate) fn plan_for_roots(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
        enabled: bool,
        roots: AgentRoots,
    ) -> Result<ConfigurationPlan, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let state = load_writable_state(app_data)?;
        let package = find_package(&state, package_id)?;
        let installation = find_installation(package, agent)?;
        ensure_reconciliation_status(
            package,
            installation,
            inventory::ManagedInstallationStatus::Healthy,
        )?;
        let snapshot = snapshot(package, installation, &roots)?;
        ensure_writable(installation, &snapshot)?;

        let public = ConfigurationPlan {
            id: crate::library::next_id(),
            package_id: package_id.to_owned(),
            agent,
            enabled,
            current_enabled: snapshot.entry.unwrap_or(true),
            config_path: snapshot.path,
        };
        self.plans.lock().map_err(|_| lock_error())?.insert(
            public.id.clone(),
            PendingPlan {
                public: public.clone(),
                roots,
                original: snapshot.bytes,
            },
        );
        Ok(public)
    }

    pub fn commit(
        &self,
        app_data: &Path,
        plan_id: &str,
    ) -> Result<ConfigurationResult, SkillError> {
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
            .position(|package| package.id == plan.public.package_id)
            .ok_or_else(invalid_plan)?;
        let installation_index = state.packages[package_index]
            .installations
            .iter()
            .position(|installation| installation.agent == plan.public.agent)
            .ok_or_else(invalid_plan)?;
        let package = &state.packages[package_index];
        let installation = &package.installations[installation_index];
        ensure_reconciliation_status(
            package,
            installation,
            inventory::ManagedInstallationStatus::Healthy,
        )?;
        let current = snapshot(package, installation, &plan.roots)?;
        if current.bytes != plan.original {
            return Err(configuration_drift(current.path));
        }
        ensure_writable(installation, &current)?;

        let next = write_value(
            plan.public.agent,
            current.bytes.as_deref(),
            package,
            installation,
            plan.public.enabled,
            &current.path,
        )?;
        atomic_write(&current.path, &next)?;
        let installation = &mut state.packages[package_index].installations[installation_index];
        installation.enabled = plan.public.enabled;
        installation.configuration_provenance = ConfigurationProvenance::SkillDeck {
            path: current.path.clone(),
        };
        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            restore(&current.path, plan.original.as_deref())?;
            return Err(error);
        }
        Ok(ConfigurationResult {
            package: state.packages[package_index].clone(),
            restart_message: RESTART_MESSAGE,
        })
    }

    pub fn resolve(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
        resolution: ConfigurationResolution,
    ) -> Result<ConfigurationResult, SkillError> {
        self.resolve_for_roots(
            app_data,
            package_id,
            agent,
            resolution,
            inventory::agent_roots()?,
        )
    }

    pub(crate) fn resolve_for_roots(
        &self,
        app_data: &Path,
        package_id: &str,
        agent: Agent,
        resolution: ConfigurationResolution,
        roots: AgentRoots,
    ) -> Result<ConfigurationResult, SkillError> {
        let _mutation = self.mutation.try_lock().map_err(|_| busy_error())?;
        let mut state = load_writable_state(app_data)?;
        let package_index = state
            .packages
            .iter()
            .position(|package| package.id == package_id)
            .ok_or_else(invalid_plan)?;
        let installation_index = state.packages[package_index]
            .installations
            .iter()
            .position(|installation| installation.agent == agent)
            .ok_or_else(invalid_plan)?;
        let package = &state.packages[package_index];
        let installation = &package.installations[installation_index];
        ensure_reconciliation_status(
            package,
            installation,
            inventory::ManagedInstallationStatus::ConfigurationDrift,
        )?;
        if !matches!(
            installation.configuration_provenance,
            ConfigurationProvenance::SkillDeck { .. }
        ) {
            return Err(externally_controlled(config_path(agent, &roots)));
        }
        let current = snapshot(package, installation, &roots)?;
        if current.entry == Some(installation.enabled) && current.owned_shape {
            return Err(SkillError::new(
                SkillErrorCode::Conflict,
                "Agent configuration has no drift to resolve",
                Some(current.path),
            ));
        }

        let original = current.bytes;
        if resolution == ConfigurationResolution::Reapply {
            let next = write_value(
                agent,
                original.as_deref(),
                package,
                installation,
                installation.enabled,
                &current.path,
            )?;
            atomic_write(&current.path, &next)?;
        } else {
            state.packages[package_index].installations[installation_index]
                .configuration_provenance = ConfigurationProvenance::External {
                path: current.path.clone(),
            };
            if let Some(enabled) = current.entry {
                state.packages[package_index].installations[installation_index].enabled = enabled;
            }
        }

        if let Err(error) = StateStore::new(app_data.to_path_buf()).save(&state) {
            if resolution == ConfigurationResolution::Reapply {
                restore(&current.path, original.as_deref())?;
            }
            return Err(error);
        }
        Ok(ConfigurationResult {
            package: state.packages[package_index].clone(),
            restart_message: RESTART_MESSAGE,
        })
    }
}

struct Snapshot {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
    entry: Option<bool>,
    owned_shape: bool,
}

pub(crate) fn initial_configuration(
    agent: Agent,
    logical_path: &Path,
    package_name: &str,
    roots: &AgentRoots,
) -> Result<(bool, ConfigurationProvenance), SkillError> {
    let path = config_path(agent, roots);
    let bytes = read_optional(&path)?;
    let inspection = inspect(agent, bytes.as_deref(), logical_path, package_name, &path)?;
    Ok(match inspection.entry {
        Some(enabled) => (
            enabled,
            ConfigurationProvenance::External { path: path.clone() },
        ),
        None => (true, ConfigurationProvenance::None),
    })
}

#[derive(Debug)]
pub(crate) struct ConfigurationCleanup {
    path: PathBuf,
    original: Option<Vec<u8>>,
}

pub(crate) fn validate_owned_configuration(
    package_name: &str,
    installation: &Installation,
) -> Result<(), SkillError> {
    owned_configuration_snapshot(package_name, installation).map(|_| ())
}

pub(crate) fn cleanup_owned_configuration(
    package_name: &str,
    installation: &Installation,
) -> Result<Option<ConfigurationCleanup>, SkillError> {
    let Some(cleanup) = owned_configuration_snapshot(package_name, installation)? else {
        return Ok(None);
    };
    let next = remove_value(
        installation.agent,
        cleanup
            .original
            .as_deref()
            .expect("owned config entry has a file"),
        &installation.logical_path,
        package_name,
        &cleanup.path,
    )?;
    atomic_write(&cleanup.path, &next)?;
    Ok(Some(cleanup))
}

pub(crate) fn restore_configuration_cleanup(
    cleanup: Option<&ConfigurationCleanup>,
) -> Result<(), SkillError> {
    match cleanup {
        Some(cleanup) => restore(&cleanup.path, cleanup.original.as_deref()),
        None => Ok(()),
    }
}

fn owned_configuration_snapshot(
    package_name: &str,
    installation: &Installation,
) -> Result<Option<ConfigurationCleanup>, SkillError> {
    let ConfigurationProvenance::SkillDeck { path } = &installation.configuration_provenance else {
        return Ok(None);
    };
    let path = path.clone();
    let bytes = read_optional(&path)?;
    let inspection = inspect(
        installation.agent,
        bytes.as_deref(),
        &installation.logical_path,
        package_name,
        &path,
    )?;
    if inspection.entry != Some(installation.enabled) || !inspection.owned_shape {
        return Err(configuration_drift(path));
    }
    Ok(Some(ConfigurationCleanup {
        path,
        original: bytes,
    }))
}

fn snapshot(
    package: &ManagedSkillPackage,
    installation: &Installation,
    roots: &AgentRoots,
) -> Result<Snapshot, SkillError> {
    let path = match &installation.configuration_provenance {
        ConfigurationProvenance::SkillDeck { path }
        | ConfigurationProvenance::External { path } => path.clone(),
        ConfigurationProvenance::None => config_path(installation.agent, roots),
    };
    let bytes = read_optional(&path)?;
    let inspection = inspect(
        installation.agent,
        bytes.as_deref(),
        &installation.logical_path,
        &package.name,
        &path,
    )?;
    Ok(Snapshot {
        path,
        bytes,
        entry: inspection.entry,
        owned_shape: inspection.owned_shape,
    })
}

fn ensure_writable(installation: &Installation, snapshot: &Snapshot) -> Result<(), SkillError> {
    match &installation.configuration_provenance {
        ConfigurationProvenance::SkillDeck { .. } => {
            if snapshot.entry != Some(installation.enabled) || !snapshot.owned_shape {
                return Err(configuration_drift(snapshot.path.clone()));
            }
        }
        ConfigurationProvenance::External { .. } if snapshot.entry.is_some() => {
            return Err(externally_controlled(snapshot.path.clone()));
        }
        ConfigurationProvenance::External { .. } => {
            return Err(configuration_drift(snapshot.path.clone()));
        }
        ConfigurationProvenance::None if snapshot.entry.is_some() => {
            return Err(externally_controlled(snapshot.path.clone()));
        }
        ConfigurationProvenance::None => {}
    }
    Ok(())
}

struct Inspection {
    entry: Option<bool>,
    owned_shape: bool,
}

fn inspect(
    agent: Agent,
    bytes: Option<&[u8]>,
    logical_path: &Path,
    package_name: &str,
    path: &Path,
) -> Result<Inspection, SkillError> {
    match agent {
        Agent::Codex => inspect_codex(bytes, logical_path, path),
        Agent::Claude => inspect_claude(bytes, package_name, path),
    }
}

fn inspect_codex(
    bytes: Option<&[u8]>,
    logical_path: &Path,
    config_path: &Path,
) -> Result<Inspection, SkillError> {
    let Some(bytes) = bytes else {
        return Ok(Inspection {
            entry: None,
            owned_shape: false,
        });
    };
    let text = std::str::from_utf8(bytes).map_err(|error| invalid_config(config_path, error))?;
    let target = codex_target(logical_path, config_path)?;
    let mut found = None;
    for block in codex_blocks(text) {
        let Some(path) = codex_string_field(block.text, "path", config_path)? else {
            continue;
        };
        if path != target {
            continue;
        }
        if found.is_some() {
            return Err(SkillError::new(
                SkillErrorCode::ConfigurationExternallyControlled,
                "Multiple Codex configuration entries target this Skill",
                Some(config_path.to_path_buf()),
            ));
        }
        let enabled = codex_bool_field(block.text, "enabled", config_path)?.unwrap_or(true);
        found = Some((enabled, codex_owned_shape(block.text, &target, enabled)));
    }
    Ok(match found {
        Some((entry, owned_shape)) => Inspection {
            entry: Some(entry),
            owned_shape,
        },
        None => Inspection {
            entry: None,
            owned_shape: false,
        },
    })
}

fn inspect_claude(
    bytes: Option<&[u8]>,
    package_name: &str,
    config_path: &Path,
) -> Result<Inspection, SkillError> {
    let Some(bytes) = bytes else {
        return Ok(Inspection {
            entry: None,
            owned_shape: false,
        });
    };
    let root = parse_claude(bytes, config_path)?;
    let Some(overrides) = root.get("skillOverrides") else {
        return Ok(Inspection {
            entry: None,
            owned_shape: false,
        });
    };
    let overrides = overrides.as_object().ok_or_else(|| {
        SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Claude skillOverrides must be a JSON object",
            Some(config_path.to_path_buf()),
        )
    })?;
    let Some(value) = overrides.get(package_name) else {
        return Ok(Inspection {
            entry: None,
            owned_shape: false,
        });
    };
    let value = value.as_str().ok_or_else(|| {
        SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Claude Skill override must be a string",
            Some(config_path.to_path_buf()),
        )
    })?;
    let enabled = match value {
        "off" => false,
        "on" | "name-only" | "user-invocable-only" => true,
        _ => {
            return Err(SkillError::new(
                SkillErrorCode::InvalidStructure,
                format!("Unsupported Claude Skill override value {value:?}"),
                Some(config_path.to_path_buf()),
            ));
        }
    };
    Ok(Inspection {
        entry: Some(enabled),
        owned_shape: value == if enabled { "on" } else { "off" },
    })
}

fn write_value(
    agent: Agent,
    bytes: Option<&[u8]>,
    package: &ManagedSkillPackage,
    installation: &Installation,
    enabled: bool,
    config_path: &Path,
) -> Result<Vec<u8>, SkillError> {
    match agent {
        Agent::Codex => write_codex(bytes, &installation.logical_path, enabled, config_path),
        Agent::Claude => write_claude(bytes, &package.name, enabled, config_path),
    }
}

fn write_codex(
    bytes: Option<&[u8]>,
    logical_path: &Path,
    enabled: bool,
    config_path: &Path,
) -> Result<Vec<u8>, SkillError> {
    let text = match bytes {
        Some(bytes) => {
            std::str::from_utf8(bytes).map_err(|error| invalid_config(config_path, error))?
        }
        None => "",
    };
    let target = codex_target(logical_path, config_path)?;
    let replacement = codex_owned_block(&target, enabled);
    let matching = matching_codex_block(text, &target, config_path)?;
    let next = if let Some(block) = matching {
        format!(
            "{}{}{}",
            &text[..block.start],
            replacement,
            &text[block.end..]
        )
    } else {
        let separator = if text.is_empty() || text.ends_with("\n\n") {
            ""
        } else if text.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        format!("{text}{separator}{replacement}")
    };
    Ok(next.into_bytes())
}

fn write_claude(
    bytes: Option<&[u8]>,
    package_name: &str,
    enabled: bool,
    config_path: &Path,
) -> Result<Vec<u8>, SkillError> {
    let mut root = match bytes {
        Some(bytes) => parse_claude(bytes, config_path)?,
        None => Map::new(),
    };
    let overrides = root
        .entry("skillOverrides".to_owned())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| {
            SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Claude skillOverrides must be a JSON object",
                Some(config_path.to_path_buf()),
            )
        })?;
    overrides.insert(
        package_name.to_owned(),
        Value::String(if enabled { "on" } else { "off" }.to_owned()),
    );
    serialize_claude(root, config_path)
}

fn remove_value(
    agent: Agent,
    bytes: &[u8],
    logical_path: &Path,
    package_name: &str,
    config_path: &Path,
) -> Result<Vec<u8>, SkillError> {
    match agent {
        Agent::Codex => {
            let text =
                std::str::from_utf8(bytes).map_err(|error| invalid_config(config_path, error))?;
            let target = codex_target(logical_path, config_path)?;
            let block = matching_codex_block(text, &target, config_path)?
                .ok_or_else(|| configuration_drift(config_path.to_path_buf()))?;
            Ok(format!("{}{}", &text[..block.start], &text[block.end..]).into_bytes())
        }
        Agent::Claude => {
            let mut root = parse_claude(bytes, config_path)?;
            let overrides = root
                .get_mut("skillOverrides")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| configuration_drift(config_path.to_path_buf()))?;
            overrides.remove(package_name);
            if overrides.is_empty() {
                root.remove("skillOverrides");
            }
            serialize_claude(root, config_path)
        }
    }
}

fn parse_claude(bytes: &[u8], path: &Path) -> Result<Map<String, Value>, SkillError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| invalid_config(path, error))?;
    value.as_object().cloned().ok_or_else(|| {
        SkillError::new(
            SkillErrorCode::InvalidStructure,
            "Claude settings must be a JSON object",
            Some(path.to_path_buf()),
        )
    })
}

fn serialize_claude(root: Map<String, Value>, path: &Path) -> Result<Vec<u8>, SkillError> {
    let mut bytes = serde_json::to_vec_pretty(&Value::Object(root))
        .map_err(|error| invalid_config(path, error))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Clone, Copy)]
struct CodexBlock<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

fn codex_blocks(text: &str) -> Vec<CodexBlock<'_>> {
    let mut headers = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.trim() == "[[skills.config]]" {
            headers.push(offset);
        }
        offset += line.len();
    }
    headers
        .iter()
        .enumerate()
        .map(|(index, &start)| {
            let end =
                next_toml_header(text, start + "[[skills.config]]".len()).unwrap_or(text.len());
            let end = headers
                .get(index + 1)
                .copied()
                .map_or(end, |next| end.min(next));
            CodexBlock {
                start,
                end,
                text: &text[start..end],
            }
        })
        .collect()
}

fn next_toml_header(text: &str, from: usize) -> Option<usize> {
    let mut offset = from;
    for line in text[from..].split_inclusive('\n') {
        if line.trim_start().starts_with('[') {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn matching_codex_block<'a>(
    text: &'a str,
    target: &str,
    path: &Path,
) -> Result<Option<CodexBlock<'a>>, SkillError> {
    let mut matching = None;
    for block in codex_blocks(text) {
        if codex_string_field(block.text, "path", path)?.as_deref() == Some(target) {
            if matching.is_some() {
                return Err(externally_controlled(path.to_path_buf()));
            }
            matching = Some(block);
        }
    }
    Ok(matching)
}

fn codex_string_field(text: &str, key: &str, path: &Path) -> Result<Option<String>, SkillError> {
    let Some(raw) = field(text, key) else {
        return Ok(None);
    };
    serde_json::from_str(raw)
        .map(Some)
        .map_err(|error| invalid_config(path, error))
}

fn codex_bool_field(text: &str, key: &str, path: &Path) -> Result<Option<bool>, SkillError> {
    let Some(raw) = field(text, key) else {
        return Ok(None);
    };
    raw.parse()
        .map(Some)
        .map_err(|error| invalid_config(path, error))
}

fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (left, right) = line.split_once('=')?;
        (left.trim() == key).then(|| right.trim())
    })
}

fn codex_owned_shape(text: &str, target: &str, enabled: bool) -> bool {
    text.trim_end() == codex_owned_block(target, enabled).trim_end()
}

fn codex_owned_block(target: &str, enabled: bool) -> String {
    format!(
        "[[skills.config]]\n{CODEX_MARKER}\npath = {}\nenabled = {enabled}\n",
        serde_json::to_string(target).expect("a path string always serializes")
    )
}

fn codex_target(logical_path: &Path, config_path: &Path) -> Result<String, SkillError> {
    logical_path
        .join("SKILL.md")
        .into_os_string()
        .into_string()
        .map_err(|_| {
            SkillError::new(
                SkillErrorCode::InvalidStructure,
                "Codex configuration cannot represent a non-Unicode Skill path",
                Some(config_path.to_path_buf()),
            )
        })
}

fn config_path(agent: Agent, roots: &AgentRoots) -> PathBuf {
    match agent {
        Agent::Codex => roots
            .codex_legacy
            .parent()
            .expect("Codex legacy root has a state root")
            .join("config.toml"),
        Agent::Claude => roots
            .claude
            .parent()
            .expect("Claude Skill root has a personal root")
            .join("settings.json"),
    }
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, SkillError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(SkillError::io(path, error)),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), SkillError> {
    let parent = path.parent().expect("config path has a parent");
    fs::create_dir_all(parent).map_err(|error| SkillError::io(parent, error))?;
    AtomicFile::new(path, AllowOverwrite)
        .write(|file| {
            file.write_all(bytes)?;
            file.sync_all()
        })
        .map_err(|error| {
            SkillError::new(
                SkillErrorCode::Io,
                format!("Could not atomically write Agent configuration: {error}"),
                Some(path.to_path_buf()),
            )
        })
}

fn restore(path: &Path, original: Option<&[u8]>) -> Result<(), SkillError> {
    if let Some(bytes) = original {
        atomic_write(path, bytes)
    } else {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SkillError::io(path, error)),
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
        .ok_or_else(invalid_plan)
}

fn find_installation(
    package: &ManagedSkillPackage,
    agent: Agent,
) -> Result<&Installation, SkillError> {
    package
        .installations
        .iter()
        .find(|installation| installation.agent == agent)
        .ok_or_else(invalid_plan)
}

fn ensure_reconciliation_status(
    package: &ManagedSkillPackage,
    installation: &Installation,
    expected: inventory::ManagedInstallationStatus,
) -> Result<(), SkillError> {
    let reconciliation = inventory::reconcile_installation(package, installation);
    if reconciliation.status == expected {
        return Ok(());
    }
    Err(reconciliation
        .diagnostic
        .map(|diagnostic| SkillError::new(diagnostic.code, diagnostic.message, diagnostic.path))
        .unwrap_or_else(|| {
            SkillError::new(
                SkillErrorCode::Conflict,
                "Configuration action is unavailable for the current Installation status",
                Some(installation.logical_path.clone()),
            )
        }))
}

fn invalid_config(path: &Path, error: impl std::fmt::Display) -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidStructure,
        format!("Could not parse Agent configuration: {error}"),
        Some(path.to_path_buf()),
    )
}

fn externally_controlled(path: PathBuf) -> SkillError {
    SkillError::new(
        SkillErrorCode::ConfigurationExternallyControlled,
        "This Skill is controlled by user or third-party Agent configuration",
        Some(path),
    )
}

fn configuration_drift(path: PathBuf) -> SkillError {
    SkillError::new(
        SkillErrorCode::ConfigurationDrift,
        "Skill Deck-owned Agent configuration changed outside Skill Deck",
        Some(path),
    )
}

fn busy_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Busy,
        "Another configuration mutation is already running",
        None,
    )
}

fn lock_error() -> SkillError {
    SkillError::new(
        SkillErrorCode::Io,
        "The in-memory configuration plan store is unavailable",
        None,
    )
}

fn invalid_plan() -> SkillError {
    SkillError::new(
        SkillErrorCode::InvalidPlan,
        "The configuration plan is missing or no longer valid",
        None,
    )
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::state::{DeploymentMode, InstalledRevision, SkillSource};

    #[test]
    fn codex_disable_enable_and_cleanup_preserve_unrelated_toml() {
        let fixture = fixture(Agent::Codex, ConfigurationProvenance::None, true);
        let config_path = config_path(Agent::Codex, &fixture.roots);
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "model = \"gpt-5\"\n").unwrap();
        let manager = ConfigurationManager::default();

        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Codex,
                false,
                fixture.roots.clone(),
            )
            .unwrap();
        let result = manager.commit(&fixture.app_data, &plan.id).unwrap();
        assert!(!result.package.installations[0].enabled);
        assert_eq!(
            result.package.installations[0].configuration_provenance,
            ConfigurationProvenance::SkillDeck {
                path: config_path.clone()
            }
        );
        let disabled = fs::read_to_string(&config_path).unwrap();
        assert!(disabled.contains("model = \"gpt-5\""));
        assert!(disabled.contains("enabled = false"));

        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Codex,
                true,
                fixture.roots.clone(),
            )
            .unwrap();
        let result = manager.commit(&fixture.app_data, &plan.id).unwrap();
        cleanup_owned_configuration("alpha-skill", &result.package.installations[0]).unwrap();
        let cleaned = fs::read_to_string(config_path).unwrap();
        assert_eq!(cleaned.trim(), "model = \"gpt-5\"");
    }

    #[test]
    fn claude_round_trip_preserves_unrelated_json() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        let config_path = config_path(Agent::Claude, &fixture.roots);
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(
            &config_path,
            r#"{"theme":"dark","permissions":{"allow":["Read"]}}"#,
        )
        .unwrap();
        let manager = ConfigurationManager::default();

        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                false,
                fixture.roots.clone(),
            )
            .unwrap();
        manager.commit(&fixture.app_data, &plan.id).unwrap();

        let value: Value = serde_json::from_slice(&fs::read(config_path).unwrap()).unwrap();
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["permissions"]["allow"][0], "Read");
        assert_eq!(value["skillOverrides"]["alpha-skill"], "off");
    }

    #[test]
    fn configuration_plan_uses_the_frontend_camel_case_contract() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        let plan = ConfigurationManager::default()
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                false,
                fixture.roots,
            )
            .unwrap();
        let json = serde_json::to_value(plan).unwrap();

        assert_eq!(json["packageId"], "package-1");
        assert_eq!(json["agent"], "claude");
        assert_eq!(json["enabled"], false);
        assert_eq!(json["currentEnabled"], true);
        assert!(json.get("configPath").is_some());
        assert!(json.get("package_id").is_none());
    }

    #[test]
    fn owned_provenance_keeps_using_original_path_after_root_change() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        let original_path = config_path(Agent::Claude, &fixture.roots);
        let manager = ConfigurationManager::default();
        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                false,
                fixture.roots.clone(),
            )
            .unwrap();
        manager.commit(&fixture.app_data, &plan.id).unwrap();
        let mut changed_roots = fixture.roots;
        changed_roots.claude = fixture._temp.path().join("different/skills");

        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                true,
                changed_roots,
            )
            .unwrap();

        assert_eq!(plan.config_path, original_path);
    }

    #[test]
    fn existing_external_configuration_locks_toggle() {
        let fixture = fixture(Agent::Codex, ConfigurationProvenance::None, true);
        let config_path = config_path(Agent::Codex, &fixture.roots);
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let skill_path = fixture.package.installations[0]
            .logical_path
            .join("SKILL.md");
        fs::write(
            &config_path,
            format!(
                "[[skills.config]]\npath = {}\nenabled = false\n",
                serde_json::to_string(&skill_path.to_string_lossy()).unwrap()
            ),
        )
        .unwrap();

        let error = ConfigurationManager::default()
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Codex,
                true,
                fixture.roots,
            )
            .unwrap_err();

        assert_eq!(
            error.code,
            SkillErrorCode::ConfigurationExternallyControlled
        );
        assert_eq!(error.path, Some(config_path));
    }

    #[test]
    fn skill_deck_configuration_drift_blocks_toggle() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        let manager = ConfigurationManager::default();
        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                false,
                fixture.roots.clone(),
            )
            .unwrap();
        manager.commit(&fixture.app_data, &plan.id).unwrap();
        let config_path = config_path(Agent::Claude, &fixture.roots);
        fs::write(&config_path, r#"{"skillOverrides":{"alpha-skill":"on"}}"#).unwrap();

        let error = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                true,
                fixture.roots,
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::ConfigurationDrift);
        assert_eq!(error.path, Some(config_path));
    }

    #[test]
    fn explicit_reapply_and_forget_resolve_owned_configuration_drift() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        let manager = ConfigurationManager::default();
        let plan = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                false,
                fixture.roots.clone(),
            )
            .unwrap();
        manager.commit(&fixture.app_data, &plan.id).unwrap();
        let config_path = config_path(Agent::Claude, &fixture.roots);
        fs::write(&config_path, r#"{"skillOverrides":{"alpha-skill":"on"}}"#).unwrap();

        let reapplied = manager
            .resolve(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                ConfigurationResolution::Reapply,
            )
            .unwrap();
        assert!(!reapplied.package.installations[0].enabled);
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&config_path).unwrap()).unwrap()
                ["skillOverrides"]["alpha-skill"],
            "off"
        );

        fs::write(&config_path, r#"{"skillOverrides":{"alpha-skill":"on"}}"#).unwrap();
        let forgotten = manager
            .resolve(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                ConfigurationResolution::Forget,
            )
            .unwrap();
        assert!(forgotten.package.installations[0].enabled);
        assert_eq!(
            forgotten.package.installations[0].configuration_provenance,
            ConfigurationProvenance::External { path: config_path }
        );
    }

    #[test]
    fn missing_owned_configuration_is_drift() {
        let temp = TempDir::new().unwrap();
        let owned_path = temp.path().join("original/settings.json");
        let fixture = fixture(
            Agent::Claude,
            ConfigurationProvenance::SkillDeck {
                path: owned_path.clone(),
            },
            false,
        );

        let error = ConfigurationManager::default()
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                true,
                fixture.roots.clone(),
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::ConfigurationDrift);
        assert_eq!(error.path, Some(owned_path));
    }

    #[test]
    fn missing_installation_blocks_configuration_mutation() {
        let fixture = fixture(Agent::Claude, ConfigurationProvenance::None, true);
        fs::remove_dir_all(&fixture.package.installations[0].logical_path).unwrap();

        assert_eq!(
            ConfigurationManager::default()
                .plan_for_roots(
                    &fixture.app_data,
                    "package-1",
                    Agent::Claude,
                    false,
                    fixture.roots,
                )
                .unwrap_err()
                .code,
            SkillErrorCode::InstallationMissing
        );
    }

    #[test]
    fn removed_external_configuration_is_drift() {
        let fixture = fixture(
            Agent::Claude,
            ConfigurationProvenance::External {
                path: PathBuf::from("old-settings.json"),
            },
            false,
        );
        let manager = ConfigurationManager::default();

        let error = manager
            .plan_for_roots(
                &fixture.app_data,
                "package-1",
                Agent::Claude,
                true,
                fixture.roots,
            )
            .unwrap_err();

        assert_eq!(error.code, SkillErrorCode::ConfigurationDrift);
    }

    struct Fixture {
        _temp: TempDir,
        app_data: PathBuf,
        roots: AgentRoots,
        package: ManagedSkillPackage,
    }

    fn fixture(agent: Agent, provenance: ConfigurationProvenance, enabled: bool) -> Fixture {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        let roots = AgentRoots {
            codex: temp.path().join("home/.agents/skills"),
            claude: temp.path().join("home/.claude/skills"),
            codex_legacy: temp.path().join("home/.codex/skills"),
        };
        let logical_path = match agent {
            Agent::Codex => roots.codex.join("alpha-skill"),
            Agent::Claude => roots.claude.join("alpha-skill"),
        };
        let library_path = app_data.join("library/alpha-skill/current");
        fs::create_dir_all(&library_path).unwrap();
        fs::write(
            library_path.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Configuration fixture\n---\n",
        )
        .unwrap();
        let fingerprint = crate::skill::validate_installed_revision(&library_path, "alpha-skill")
            .unwrap()
            .fingerprint;
        crate::library::copy_directory(&library_path, &logical_path).unwrap();
        let package = ManagedSkillPackage {
            id: "package-1".to_owned(),
            name: "alpha-skill".to_owned(),
            library_path,
            source: SkillSource::LocalSnapshot,
            installed_revision: InstalledRevision {
                fingerprint: fingerprint.clone(),
                commit_oid: None,
            },
            previous_revision: None,
            installations: vec![Installation {
                agent,
                logical_path: logical_path.clone(),
                resolved_target: logical_path,
                deployment_mode: DeploymentMode::CopyFallback,
                enabled,
                last_known_fingerprint: fingerprint,
                configuration_provenance: provenance,
            }],
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
            roots,
            package,
        }
    }
}
