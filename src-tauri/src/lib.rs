use serde::Serialize;

mod adoption;
mod configuration;
mod diagnostics;
mod git_source;
mod install;
mod inventory;
mod library;
mod lifecycle;
mod revision;
mod skill;
mod state;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Skill Deck",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
fn validate_local_skill(path: String) -> Result<skill::ValidatedSkill, skill::SkillError> {
    skill::validate_skill_dir(std::path::Path::new(&path))
}

#[tauri::command]
fn plan_git_import(
    app: tauri::AppHandle,
    manager: tauri::State<'_, git_source::GitSourceManager>,
    repository_url: String,
    subpath: String,
    tracked_branch: String,
) -> Result<git_source::GitImportPlan, skill::SkillError> {
    manager.plan_import(&app_data(&app)?, &repository_url, &subpath, &tracked_branch)
}

#[tauri::command]
fn commit_git_import(
    app: tauri::AppHandle,
    manager: tauri::State<'_, git_source::GitSourceManager>,
    plan_id: String,
) -> Result<state::ManagedSkillPackage, skill::SkillError> {
    manager.commit_import(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn check_git_update(
    app: tauri::AppHandle,
    manager: tauri::State<'_, git_source::GitSourceManager>,
    package_id: String,
) -> Result<git_source::GitUpdateCheck, skill::SkillError> {
    manager.check_update(&app_data(&app)?, &package_id)
}

#[tauri::command]
fn commit_git_update(
    app: tauri::AppHandle,
    manager: tauri::State<'_, git_source::GitSourceManager>,
    plan_id: String,
) -> Result<revision::RevisionResult, skill::SkillError> {
    manager.commit_update(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn inventory(app: tauri::AppHandle) -> Result<inventory::Inventory, skill::SkillError> {
    inventory::inventory(&app_data(&app)?)
}

#[tauri::command]
fn plan_diagnostics_export(
    app: tauri::AppHandle,
    manager: tauri::State<'_, diagnostics::DiagnosticsManager>,
    destination: String,
) -> Result<diagnostics::DiagnosticsExportPlan, skill::SkillError> {
    manager.plan_export(&app_data(&app)?, std::path::Path::new(&destination))
}

#[tauri::command]
fn commit_diagnostics_export(
    app: tauri::AppHandle,
    manager: tauri::State<'_, diagnostics::DiagnosticsManager>,
    plan_id: String,
) -> Result<diagnostics::DiagnosticsExportResult, skill::SkillError> {
    manager.commit_export(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn cancel_staging(
    app: tauri::AppHandle,
    manager: tauri::State<'_, diagnostics::DiagnosticsManager>,
) -> Result<(), skill::SkillError> {
    manager.cancel_staging(&app_data(&app)?)
}

#[tauri::command]
fn plan_install(
    app: tauri::AppHandle,
    manager: tauri::State<'_, install::InstallManager>,
    package_id: String,
    targets: Vec<inventory::Agent>,
    create_missing_roots: bool,
) -> Result<install::InstallPlan, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.plan(&app_data, &package_id, targets, create_missing_roots)
}

#[tauri::command]
fn commit_install(
    app: tauri::AppHandle,
    manager: tauri::State<'_, install::InstallManager>,
    plan_id: String,
    confirm_copy_fallback: bool,
) -> Result<install::InstallResult, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.commit(&app_data, &plan_id, confirm_copy_fallback)
}

#[tauri::command]
fn plan_adoption(
    app: tauri::AppHandle,
    manager: tauri::State<'_, adoption::AdoptionManager>,
    installations: Vec<adoption::ExternalInstallationIdentity>,
) -> Result<adoption::AdoptionPlan, skill::SkillError> {
    manager.plan_adoption(&app_data(&app)?, installations)
}

#[tauri::command]
fn commit_adoption(
    app: tauri::AppHandle,
    manager: tauri::State<'_, adoption::AdoptionManager>,
    plan_id: String,
    confirm_copy_fallback: bool,
) -> Result<adoption::AdoptionResult, skill::SkillError> {
    manager.commit_adoption(&app_data(&app)?, &plan_id, confirm_copy_fallback)
}

#[tauri::command]
fn plan_legacy_migration(
    app: tauri::AppHandle,
    manager: tauri::State<'_, adoption::AdoptionManager>,
    logical_path: String,
) -> Result<adoption::LegacyMigrationPlan, skill::SkillError> {
    manager.plan_legacy_migration(&app_data(&app)?, logical_path.into())
}

#[tauri::command]
fn commit_legacy_migration(
    app: tauri::AppHandle,
    manager: tauri::State<'_, adoption::AdoptionManager>,
    plan_id: String,
) -> Result<adoption::AdoptionResult, skill::SkillError> {
    manager.commit_legacy_migration(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_configuration(
    app: tauri::AppHandle,
    manager: tauri::State<'_, configuration::ConfigurationManager>,
    package_id: String,
    agent: inventory::Agent,
    enabled: bool,
) -> Result<configuration::ConfigurationPlan, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.plan(&app_data, &package_id, agent, enabled)
}

#[tauri::command]
fn commit_configuration(
    app: tauri::AppHandle,
    manager: tauri::State<'_, configuration::ConfigurationManager>,
    plan_id: String,
) -> Result<configuration::ConfigurationResult, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.commit(&app_data, &plan_id)
}

#[tauri::command]
fn resolve_configuration(
    app: tauri::AppHandle,
    manager: tauri::State<'_, configuration::ConfigurationManager>,
    package_id: String,
    agent: inventory::Agent,
    resolution: configuration::ConfigurationResolution,
) -> Result<configuration::ConfigurationResult, skill::SkillError> {
    manager.resolve(&app_data(&app)?, &package_id, agent, resolution)
}

fn app_data(app: &tauri::AppHandle) -> Result<std::path::PathBuf, skill::SkillError> {
    use tauri::Manager;

    app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })
}

#[tauri::command]
fn plan_uninstall(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    package_id: String,
    agent: inventory::Agent,
) -> Result<lifecycle::UninstallPlan, skill::SkillError> {
    manager.plan_uninstall(&app_data(&app)?, &package_id, agent)
}

#[tauri::command]
fn commit_uninstall(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    plan_id: String,
) -> Result<lifecycle::LifecycleResult, skill::SkillError> {
    manager.commit_uninstall(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_detach(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    package_id: String,
    agent: inventory::Agent,
) -> Result<lifecycle::DetachPlan, skill::SkillError> {
    manager.plan_detach(&app_data(&app)?, &package_id, agent)
}

#[tauri::command]
fn commit_detach(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    plan_id: String,
) -> Result<lifecycle::LifecycleResult, skill::SkillError> {
    manager.commit_detach(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_forget_installation(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    package_id: String,
    agent: inventory::Agent,
) -> Result<lifecycle::ForgetInstallationPlan, skill::SkillError> {
    manager.plan_forget_installation(&app_data(&app)?, &package_id, agent)
}

#[tauri::command]
fn commit_forget_installation(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    plan_id: String,
) -> Result<lifecycle::LifecycleResult, skill::SkillError> {
    manager.commit_forget_installation(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_remove_library(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    package_id: String,
) -> Result<lifecycle::RemoveLibraryPlan, skill::SkillError> {
    manager.plan_remove_library(&app_data(&app)?, &package_id)
}

#[tauri::command]
fn commit_remove_library(
    app: tauri::AppHandle,
    manager: tauri::State<'_, lifecycle::LifecycleManager>,
    plan_id: String,
    confirmation_name: String,
) -> Result<lifecycle::LifecycleResult, skill::SkillError> {
    manager.commit_remove_library(&app_data(&app)?, &plan_id, &confirmation_name)
}

#[tauri::command]
fn plan_replace_local_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    package_id: String,
    path: String,
) -> Result<revision::ReplaceLocalPlan, skill::SkillError> {
    manager.plan_replace_local(&app_data(&app)?, &package_id, std::path::Path::new(&path))
}

#[tauri::command]
fn commit_replace_local_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    plan_id: String,
) -> Result<revision::RevisionResult, skill::SkillError> {
    manager.commit_replace_local(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_rollback_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    package_id: String,
) -> Result<revision::RollbackRevisionPlan, skill::SkillError> {
    manager.plan_rollback(&app_data(&app)?, &package_id)
}

#[tauri::command]
fn commit_rollback_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    plan_id: String,
) -> Result<revision::RevisionResult, skill::SkillError> {
    manager.commit_rollback(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_export_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    package_id: String,
    destination: String,
) -> Result<revision::ExportRevisionPlan, skill::SkillError> {
    manager.plan_export(
        &app_data(&app)?,
        &package_id,
        std::path::Path::new(&destination),
    )
}

#[tauri::command]
fn commit_export_revision(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    plan_id: String,
) -> Result<revision::ExportRevisionResult, skill::SkillError> {
    manager.commit_export(&app_data(&app)?, &plan_id)
}

#[tauri::command]
fn plan_restore_installation(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    package_id: String,
    agent: inventory::Agent,
) -> Result<revision::RestoreInstallationPlan, skill::SkillError> {
    manager.plan_restore(&app_data(&app)?, &package_id, agent)
}

#[tauri::command]
fn commit_restore_installation(
    app: tauri::AppHandle,
    manager: tauri::State<'_, revision::RevisionManager>,
    plan_id: String,
    confirm_overwrite: bool,
    confirm_create_root: bool,
) -> Result<revision::RevisionResult, skill::SkillError> {
    manager.commit_restore_with_root_confirmation(
        &app_data(&app)?,
        &plan_id,
        confirm_overwrite,
        confirm_create_root,
    )
}

#[tauri::command]
fn plan_add_local_skill(
    app: tauri::AppHandle,
    manager: tauri::State<'_, library::LibraryManager>,
    path: String,
) -> Result<library::AddToLibraryPlan, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.plan_local(&app_data, std::path::Path::new(&path))
}

#[tauri::command]
fn commit_add_local_skill(
    app: tauri::AppHandle,
    manager: tauri::State<'_, library::LibraryManager>,
    plan_id: String,
) -> Result<state::ManagedSkillPackage, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    manager.commit_local(&app_data, &plan_id)
}

#[tauri::command]
fn state_status(app: tauri::AppHandle) -> Result<state::StateLoad, skill::SkillError> {
    use tauri::Manager;

    let app_data = app.path().app_data_dir().map_err(|error| {
        skill::SkillError::new(
            skill::SkillErrorCode::Io,
            format!("Could not resolve Skill Deck app-data: {error}"),
            None,
        )
    })?;
    let store = state::StateStore::new(app_data);
    let loaded = store.load()?;
    if loaded.mode == state::StateMode::RecoveredBackup {
        store.save(
            loaded
                .state
                .as_ref()
                .expect("recovered mode has valid state"),
        )?;
    }
    Ok(loaded)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mutation = std::sync::Arc::new(std::sync::Mutex::new(()));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;

            let app_data = app.path().app_data_dir()?;
            diagnostics::cleanup_stale_staging(&app_data)
                .map_err(|error| std::io::Error::other(error.message))?;
            Ok(())
        })
        .manage(adoption::AdoptionManager::new(mutation.clone()))
        .manage(configuration::ConfigurationManager::new(mutation.clone()))
        .manage(diagnostics::DiagnosticsManager::new(mutation.clone()))
        .manage(git_source::GitSourceManager::new(mutation.clone()))
        .manage(install::InstallManager::new(mutation.clone()))
        .manage(library::LibraryManager::new(mutation.clone()))
        .manage(lifecycle::LifecycleManager::new(mutation.clone()))
        .manage(revision::RevisionManager::new(mutation))
        .invoke_handler(tauri::generate_handler![
            app_info,
            cancel_staging,
            check_git_update,
            commit_add_local_skill,
            commit_adoption,
            commit_configuration,
            commit_diagnostics_export,
            commit_install,
            commit_git_import,
            commit_git_update,
            commit_legacy_migration,
            commit_detach,
            commit_forget_installation,
            commit_remove_library,
            commit_replace_local_revision,
            commit_restore_installation,
            commit_rollback_revision,
            commit_export_revision,
            commit_uninstall,
            inventory,
            plan_add_local_skill,
            plan_adoption,
            plan_configuration,
            plan_diagnostics_export,
            plan_install,
            plan_git_import,
            plan_legacy_migration,
            plan_detach,
            plan_forget_installation,
            plan_remove_library,
            plan_replace_local_revision,
            plan_restore_installation,
            plan_rollback_revision,
            plan_export_revision,
            plan_uninstall,
            resolve_configuration,
            state_status,
            validate_local_skill
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Skill Deck");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_uses_package_metadata() {
        let info = app_info();
        assert_eq!(info.name, "Skill Deck");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }
}
