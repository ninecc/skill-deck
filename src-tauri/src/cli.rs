use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    process::{Command, Output},
    sync::{Mutex, MutexGuard},
};

const MIN_NODE: (u64, u64, u64) = (22, 20, 0);
const MAX_INPUT: usize = 2_048;
const SEARCH_URL: &str = "https://skills.sh/api/search";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
    pub operation: Option<String>,
    pub exit_code: Option<i32>,
    pub diagnostics: Option<String>,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            operation: None,
            exit_code: None,
            diagnostics: None,
        }
    }

    fn command(operation: &str, output: &Output) -> Self {
        Self {
            code: "command_failed",
            message: format!("The Skills CLI {operation} command failed."),
            operation: Some(operation.into()),
            exit_code: output.status.code(),
            diagnostics: Some(diagnostic(output)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub ready: bool,
    pub version: Option<String>,
    pub node_version: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub name: String,
    pub path: String,
    pub scope: String,
    pub agents: Vec<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub name: String,
    pub slug: String,
    pub source: String,
    pub installs: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchResponse {
    skills: Vec<SearchItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchItem {
    id: String,
    name: String,
    source: String,
    installs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSettings {
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub copy: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub inventory: Vec<InstalledSkill>,
    pub changed_skills: Vec<String>,
    pub target_observed: Option<bool>,
    pub diagnostics: String,
}

enum ExpectedMutation<'a> {
    Add(Option<&'a str>),
    Remove(&'a str),
    Update,
}

#[derive(Debug, Clone)]
struct Session {
    version: String,
    node_version: String,
}

#[derive(Default)]
pub struct CliManager {
    session: Mutex<Option<Session>>,
    inventory: Mutex<BTreeMap<String, PathBuf>>,
    mutation: Mutex<()>,
}

impl CliManager {
    pub fn status(&self) -> Result<RuntimeStatus, CommandError> {
        match self.ensure_session() {
            Ok(session) => Ok(RuntimeStatus {
                ready: true,
                version: Some(session.version),
                node_version: Some(session.node_version),
                message: None,
            }),
            Err(error) => Ok(RuntimeStatus {
                ready: false,
                version: None,
                node_version: None,
                message: Some(error.message),
            }),
        }
    }

    pub fn retry(&self) -> Result<RuntimeStatus, CommandError> {
        *lock(&self.session)? = None;
        self.status()
    }

    pub fn list(&self) -> Result<Vec<InstalledSkill>, CommandError> {
        let session = self.ensure_session()?;
        self.list_with(&session)
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, CommandError> {
        self.ensure_session()?;
        validate("search query", query)?;
        let response: SearchResponse = reqwest::blocking::Client::new()
            .get(SEARCH_URL)
            .query(&[("q", query), ("limit", "20")])
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| {
                CommandError::new(
                    "search_unavailable",
                    format!("Search is unavailable: {error}"),
                )
            })?
            .json()
            .map_err(|error| {
                CommandError::new(
                    "incompatible_response",
                    format!("Search returned an incompatible response: {error}"),
                )
            })?;
        let mut items: Vec<_> = response
            .skills
            .into_iter()
            .map(|item| SearchResult {
                name: item.name,
                slug: item.id,
                source: item.source,
                installs: item.installs,
            })
            .collect();
        items.sort_by_key(|item| std::cmp::Reverse(item.installs));
        Ok(items)
    }

    pub fn add(
        &self,
        source: &str,
        skill: Option<&str>,
        settings: InstallSettings,
    ) -> Result<CommandResult, CommandError> {
        validate("source", source)?;
        if let Some(name) = skill {
            validate("skill name", name)?;
        }
        for agent in &settings.agents {
            validate("agent ID", agent)?;
        }
        let args = add_args(source, skill, settings);
        self.mutate("add", args, ExpectedMutation::Add(skill))
    }

    pub fn remove(&self, name: &str) -> Result<CommandResult, CommandError> {
        validate("skill name", name)?;
        self.mutate("remove", remove_args(name), ExpectedMutation::Remove(name))
    }

    pub fn update(&self, name: Option<&str>) -> Result<CommandResult, CommandError> {
        if let Some(name) = name {
            validate("skill name", name)?;
        }
        let args = update_args(name);
        self.mutate("update", args, ExpectedMutation::Update)
    }

    pub fn skill_root(&self, name: &str) -> Result<PathBuf, CommandError> {
        validate("skill name", name)?;
        lock(&self.inventory)?.get(name).cloned().ok_or_else(|| {
            CommandError::new(
                "skill_not_found",
                "The installed Skill is no longer in the current inventory.",
            )
        })
    }

    fn ensure_session(&self) -> Result<Session, CommandError> {
        let mut guard = lock(&self.session)?;
        if let Some(session) = guard.clone() {
            return Ok(session);
        }
        let node = run("node", &["--version"], "runtime probe")?;
        let node_version = utf8_stdout(&node, "node version")?
            .trim()
            .trim_start_matches('v')
            .to_owned();
        if parse_version(&node_version).is_none_or(|version| version < MIN_NODE) {
            return Err(CommandError::new(
                "node_too_old",
                format!("Node.js 22.20.0 or newer is required; found {node_version}. Install or upgrade Node.js, then retry."),
            ));
        }
        let output = run(
            "npx",
            &["--yes", "skills@latest", "--version"],
            "version resolution",
        )?;
        let version = utf8_stdout(&output, "Skills CLI version")?
            .trim()
            .trim_start_matches('v')
            .to_owned();
        if parse_version(&version).is_none_or(|value| value.0 != 1) {
            return Err(CommandError::new(
                "incompatible_cli",
                format!("skills@{version} is not compatible with this Skill Deck release. Upgrade Skill Deck."),
            ));
        }
        let session = Session {
            version,
            node_version,
        };
        self.list_with(&session)?;
        *guard = Some(session.clone());
        Ok(session)
    }

    fn list_with(&self, session: &Session) -> Result<Vec<InstalledSkill>, CommandError> {
        let package = format!("skills@{}", session.version);
        let output = run("npx", &["--yes", &package, "list", "-g", "--json"], "list")?;
        let stdout = utf8_stdout(&output, "Skills inventory")?;
        let inventory = decode_inventory(&session.version, stdout)?;
        let roots = inventory
            .iter()
            .map(|skill| (skill.name.clone(), PathBuf::from(&skill.path)))
            .collect();
        *lock(&self.inventory)? = roots;
        Ok(inventory)
    }

    fn mutate(
        &self,
        operation: &str,
        args: Vec<String>,
        expected: ExpectedMutation<'_>,
    ) -> Result<CommandResult, CommandError> {
        let _gate = self
            .mutation
            .try_lock()
            .map_err(|_| CommandError::new("busy", "Another Skill operation is in progress."))?;
        let session = self.ensure_session()?;
        let before = self.list_with(&session)?;
        let package = format!("skills@{}", session.version);
        let mut command_args = vec!["--yes".to_owned(), package];
        command_args.extend(args);
        let refs: Vec<_> = command_args.iter().map(String::as_str).collect();
        let output = run("npx", &refs, operation)?;
        let after = self.list_with(&session)?;
        let before_by_name: BTreeMap<_, _> = before
            .iter()
            .map(|skill| (skill.name.as_str(), skill))
            .collect();
        let after_by_name: BTreeMap<_, _> = after
            .iter()
            .map(|skill| (skill.name.as_str(), skill))
            .collect();
        let names: BTreeSet<_> = before_by_name
            .keys()
            .chain(after_by_name.keys())
            .copied()
            .collect();
        let changed_skills: Vec<_> = names
            .into_iter()
            .filter(|name| before_by_name.get(name) != after_by_name.get(name))
            .map(str::to_owned)
            .collect();
        let target_observed = match expected {
            ExpectedMutation::Add(Some(name)) => {
                Some(changed_skills.iter().any(|item| item == name))
            }
            ExpectedMutation::Add(None) => Some(!changed_skills.is_empty()),
            ExpectedMutation::Remove(name) => Some(!after.iter().any(|skill| skill.name == name)),
            ExpectedMutation::Update => None,
        };
        Ok(CommandResult {
            inventory: after,
            changed_skills,
            target_observed,
            diagnostics: diagnostic(&output),
        })
    }
}

fn add_args(source: &str, skill: Option<&str>, settings: InstallSettings) -> Vec<String> {
    let mut args = vec!["add".into(), source.into(), "-g".into(), "-y".into()];
    if let Some(name) = skill {
        args.extend(["--skill".into(), name.into()]);
    }
    if !settings.agents.is_empty() {
        args.push("--agent".into());
        args.extend(settings.agents);
    }
    if settings.copy {
        args.push("--copy".into());
    }
    args
}

fn remove_args(name: &str) -> Vec<String> {
    vec!["remove".into(), "-g".into(), "-y".into(), name.into()]
}

fn update_args(name: Option<&str>) -> Vec<String> {
    let mut args = vec!["update".into(), "-g".into(), "-y".into()];
    if let Some(name) = name {
        args.push(name.into());
    }
    args
}

fn decode_inventory(version: &str, stdout: &str) -> Result<Vec<InstalledSkill>, CommandError> {
    serde_json::from_str(stdout).map_err(|error| {
        CommandError::new(
            "incompatible_cli",
            format!(
                "skills@{version} returned incompatible list JSON ({error}). Upgrade Skill Deck."
            ),
        )
    })
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, CommandError> {
    mutex
        .lock()
        .map_err(|_| CommandError::new("internal", "Skill Deck's runtime state is unavailable."))
}

fn validate(label: &str, value: &str) -> Result<(), CommandError> {
    if value.trim().is_empty() || value.len() > MAX_INPUT || value.contains(['\0', '\n', '\r']) {
        return Err(CommandError::new(
            "invalid_input",
            format!("The {label} is empty or invalid."),
        ));
    }
    Ok(())
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let mut parts = value.split('.').map(|part| part.parse().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}

fn run(program: &str, args: &[&str], operation: &str) -> Result<Output, CommandError> {
    let output = cli_command(program, args).output().map_err(|error| {
        CommandError::new(
            "runtime_unavailable",
            format!("Could not run {program} from the app PATH: {error}"),
        )
    })?;
    if !output.status.success() {
        return Err(CommandError::command(operation, &output));
    }
    Ok(output)
}

fn cli_command(program: &str, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args).env("DO_NOT_TRACK", "1");
    command
}

fn utf8_stdout<'a>(output: &'a Output, label: &str) -> Result<&'a str, CommandError> {
    std::str::from_utf8(&output.stdout)
        .map_err(|_| CommandError::new("invalid_output", format!("{label} was not valid UTF-8.")))
}

fn diagnostic(output: &Output) -> String {
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    combined
        .chars()
        .filter(|character| *character == '\n' || *character == '\t' || !character.is_control())
        .take(8_000)
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open_agent_inventory_and_rejects_wrong_shape() {
        let json = r#"[{"name":"demo","path":"/tmp/demo","scope":"global","agents":["Gemini CLI","Future Agent"],"source":null,"sourceUrl":null,"sourceType":null}]"#;
        let skills: Vec<InstalledSkill> = serde_json::from_str(json).unwrap();
        assert_eq!(skills[0].agents, ["Gemini CLI", "Future Agent"]);
        assert!(serde_json::from_str::<Vec<InstalledSkill>>(r#"[{"name":"demo"}]"#).is_err());
    }

    #[test]
    fn add_uses_global_noninteractive_defaults_without_overrides() {
        let settings: InstallSettings = serde_json::from_str("{}").unwrap();
        assert!(settings.agents.is_empty());
        assert!(!settings.copy);
        assert_eq!(
            add_args("owner/repo", None, settings),
            ["add", "owner/repo", "-g", "-y"]
        );
        assert_eq!(parse_version("22.20.0"), Some(MIN_NODE));
    }

    #[test]
    fn mutation_argv_matches_the_upstream_noninteractive_contract() {
        assert_eq!(
            add_args(
                "owner/repo",
                Some("demo"),
                InstallSettings {
                    agents: vec!["codex".into(), "gemini-cli".into()],
                    copy: true,
                },
            ),
            [
                "add",
                "owner/repo",
                "-g",
                "-y",
                "--skill",
                "demo",
                "--agent",
                "codex",
                "gemini-cli",
                "--copy",
            ]
        );
        assert_eq!(remove_args("demo"), ["remove", "-g", "-y", "demo"]);
        assert_eq!(update_args(None), ["update", "-g", "-y"]);
        assert_eq!(update_args(Some("demo")), ["update", "-g", "-y", "demo"]);
    }

    #[test]
    fn malformed_inventory_fails_closed() {
        assert_eq!(
            decode_inventory("1.5.22", "not json").unwrap_err().code,
            "incompatible_cli"
        );
    }

    #[test]
    fn every_cli_process_disables_telemetry() {
        let command = cli_command("npx", &["--yes", "skills@1.5.22", "list"]);
        assert!(command
            .get_envs()
            .any(|(key, value)| key == "DO_NOT_TRACK" && value == Some(std::ffi::OsStr::new("1"))));
    }
}
