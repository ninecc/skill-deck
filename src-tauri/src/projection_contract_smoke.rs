use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde_json::Value;
use tempfile::TempDir;

use crate::{
    configuration::{ConfigurationManager, ConfigurationResolution},
    install::{create_preferred_link, remove_created_entry, InstallManager},
    inventory::{inventory_for_agent_roots, Agent, AgentRoots, ManagedInstallationStatus},
    library::LibraryManager,
    lifecycle::LifecycleManager,
    revision::RevisionManager,
};

struct Fixture {
    _temp: TempDir,
    app_data: PathBuf,
    source: PathBuf,
    roots: AgentRoots,
    gate: Arc<Mutex<()>>,
}

impl Fixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source/alpha-skill");
        let roots = AgentRoots {
            codex: temp.path().join("home/.agents/skills"),
            codex_legacy: temp.path().join("codex-home/skills"),
            claude: temp.path().join("claude-home/skills"),
        };
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("SKILL.md"),
            "---\nname: alpha-skill\ndescription: Projection smoke fixture\n---\nbody\n",
        )
        .unwrap();
        for root in [&roots.codex, &roots.codex_legacy, &roots.claude] {
            fs::create_dir_all(root).unwrap();
        }
        Self {
            app_data: temp.path().join("app-data"),
            source,
            roots,
            gate: Arc::new(Mutex::new(())),
            _temp: temp,
        }
    }

    fn config_path(&self, agent: Agent) -> PathBuf {
        match agent {
            Agent::Codex => self
                .roots
                .codex_legacy
                .parent()
                .unwrap()
                .join("config.toml"),
            Agent::Claude => self.roots.claude.parent().unwrap().join("settings.json"),
        }
    }

    fn logical_path(&self, agent: Agent) -> PathBuf {
        match agent {
            Agent::Codex => self.roots.codex.join("alpha-skill"),
            Agent::Claude => self.roots.claude.join("alpha-skill"),
        }
    }
}

#[test]
fn codex_projection_contract_smoke() {
    projection_round_trip(Agent::Codex);
}

#[test]
fn claude_projection_contract_smoke() {
    projection_round_trip(Agent::Claude);
}

fn projection_round_trip(agent: Agent) {
    let fixture = Fixture::new();
    seed_unrelated_configuration(&fixture, agent);
    let library = LibraryManager::new(fixture.gate.clone());
    let installer = InstallManager::new(fixture.gate.clone());
    let configuration = ConfigurationManager::new(fixture.gate.clone());
    let revision = RevisionManager::new(fixture.gate.clone());

    let add = library
        .plan_local(&fixture.app_data, &fixture.source)
        .unwrap();
    let package = library.commit_local(&fixture.app_data, &add.id).unwrap();
    let install = installer
        .plan_for_roots(
            &fixture.app_data,
            &package.id,
            vec![agent],
            false,
            fixture.roots.clone(),
        )
        .unwrap();
    let installed = installer
        .commit(&fixture.app_data, &install.id, false)
        .unwrap()
        .package;
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::Healthy
    );
    assert_eq!(
        installed.installations[0].deployment_mode,
        crate::install::preferred_mode()
    );

    let disable = configuration
        .plan_for_roots(
            &fixture.app_data,
            &installed.id,
            agent,
            false,
            fixture.roots.clone(),
        )
        .unwrap();
    configuration
        .commit(&fixture.app_data, &disable.id)
        .unwrap();
    assert_configuration(&fixture, agent, false);
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::Healthy
    );

    drift_configuration(&fixture, agent);
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::ConfigurationDrift
    );
    configuration
        .resolve_for_roots(
            &fixture.app_data,
            &installed.id,
            agent,
            ConfigurationResolution::Reapply,
            fixture.roots.clone(),
        )
        .unwrap();
    assert_configuration(&fixture, agent, false);
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::Healthy
    );

    remove_created_entry(&fixture.logical_path(agent)).unwrap();
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::Missing
    );
    let restore = revision
        .plan_restore(&fixture.app_data, &installed.id, agent)
        .unwrap();
    revision
        .commit_restore_with_root_confirmation(&fixture.app_data, &restore.id, false, false)
        .unwrap();
    assert_configuration(&fixture, agent, false);
    assert_eq!(
        status(&fixture, &installed.id, agent),
        ManagedInstallationStatus::Healthy
    );
}

#[test]
fn retargeted_projection_can_be_forgotten_without_touching_external_content() {
    let fixture = Fixture::new();
    let library = LibraryManager::new(fixture.gate.clone());
    let installer = InstallManager::new(fixture.gate.clone());
    let lifecycle = LifecycleManager::new(fixture.gate.clone());
    let add = library
        .plan_local(&fixture.app_data, &fixture.source)
        .unwrap();
    let package = library.commit_local(&fixture.app_data, &add.id).unwrap();
    let install = installer
        .plan_for_roots(
            &fixture.app_data,
            &package.id,
            vec![Agent::Codex],
            false,
            fixture.roots.clone(),
        )
        .unwrap();
    installer
        .commit(&fixture.app_data, &install.id, false)
        .unwrap();

    let external = fixture._temp.path().join("retarget/alpha-skill");
    fs::create_dir_all(&external).unwrap();
    let external_skill = external.join("SKILL.md");
    let original = b"---\nname: alpha-skill\ndescription: External fixture\n---\nkeep me\n";
    fs::write(&external_skill, original).unwrap();
    let logical = fixture.logical_path(Agent::Codex);
    remove_created_entry(&logical).unwrap();
    create_preferred_link(&external, &logical).unwrap();
    assert_eq!(
        status(&fixture, &package.id, Agent::Codex),
        ManagedInstallationStatus::Retargeted
    );
    assert_eq!(fs::read(&external_skill).unwrap(), original);

    let forget = lifecycle
        .plan_forget_installation(&fixture.app_data, &package.id, Agent::Codex)
        .unwrap();
    lifecycle
        .commit_forget_installation(&fixture.app_data, &forget.id)
        .unwrap();
    let inventory = inventory_for_agent_roots(&fixture.app_data, &fixture.roots).unwrap();
    assert!(inventory.managed_installation_statuses.is_empty());
    assert!(inventory
        .external_installations
        .iter()
        .any(|entry| entry.logical_path == logical));
    assert_eq!(fs::read(external_skill).unwrap(), original);
    assert_eq!(
        logical.canonicalize().unwrap(),
        external.canonicalize().unwrap()
    );
}

fn status(fixture: &Fixture, package_id: &str, agent: Agent) -> ManagedInstallationStatus {
    inventory_for_agent_roots(&fixture.app_data, &fixture.roots)
        .unwrap()
        .managed_installation_statuses
        .into_iter()
        .find(|entry| entry.package_id == package_id && entry.agent == agent)
        .unwrap()
        .status
}

fn seed_unrelated_configuration(fixture: &Fixture, agent: Agent) {
    let path = fixture.config_path(agent);
    match agent {
        Agent::Codex => fs::write(path, "model = \"gpt-5\"\n").unwrap(),
        Agent::Claude => {
            fs::write(path, r#"{"theme":"dark","permissions":{"allow":["Read"]}}"#).unwrap()
        }
    }
}

fn assert_configuration(fixture: &Fixture, agent: Agent, enabled: bool) {
    let path = fixture.config_path(agent);
    match agent {
        Agent::Codex => {
            let text = fs::read_to_string(path).unwrap();
            let target = serde_json::to_string(
                &fixture
                    .logical_path(agent)
                    .join("SKILL.md")
                    .to_string_lossy(),
            )
            .unwrap();
            assert!(text.contains("model = \"gpt-5\""));
            assert!(text.contains("[[skills.config]]\n# Managed by Skill Deck"));
            assert!(text.contains(&format!("path = {target}")));
            assert!(text.contains(&format!("enabled = {enabled}")));
        }
        Agent::Claude => {
            let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            assert_eq!(value["theme"], "dark");
            assert_eq!(value["permissions"]["allow"][0], "Read");
            assert_eq!(
                value["skillOverrides"]["alpha-skill"],
                if enabled { "on" } else { "off" }
            );
        }
    }
}

fn drift_configuration(fixture: &Fixture, agent: Agent) {
    let path = fixture.config_path(agent);
    match agent {
        Agent::Codex => {
            let text = fs::read_to_string(&path).unwrap();
            fs::write(path, text.replace("enabled = false", "enabled = true")).unwrap();
        }
        Agent::Claude => {
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["skillOverrides"]["alpha-skill"] = Value::String("on".to_owned());
            fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
    }
}
