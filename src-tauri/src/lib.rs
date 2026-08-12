mod cli;
mod preview;
mod translation;

#[tauri::command]
fn runtime_status(
    manager: tauri::State<'_, cli::CliManager>,
) -> Result<cli::RuntimeStatus, cli::CommandError> {
    manager.status()
}

#[tauri::command]
fn retry_runtime(
    manager: tauri::State<'_, cli::CliManager>,
) -> Result<cli::RuntimeStatus, cli::CommandError> {
    manager.retry()
}

#[tauri::command]
fn list_skills(
    manager: tauri::State<'_, cli::CliManager>,
) -> Result<Vec<cli::InstalledSkill>, cli::CommandError> {
    manager.list()
}

#[tauri::command]
fn search_skills(
    manager: tauri::State<'_, cli::CliManager>,
    query: String,
) -> Result<Vec<cli::SearchResult>, cli::CommandError> {
    manager.search(&query)
}

#[tauri::command]
fn add_skill(
    manager: tauri::State<'_, cli::CliManager>,
    source: String,
    skill: Option<String>,
    settings: cli::InstallSettings,
) -> Result<cli::CommandResult, cli::CommandError> {
    manager.add(&source, skill.as_deref(), settings)
}

#[tauri::command]
fn remove_skill(
    manager: tauri::State<'_, cli::CliManager>,
    name: String,
) -> Result<cli::CommandResult, cli::CommandError> {
    manager.remove(&name)
}

#[tauri::command]
fn update_skill(
    manager: tauri::State<'_, cli::CliManager>,
    name: Option<String>,
) -> Result<cli::CommandResult, cli::CommandError> {
    manager.update(name.as_deref())
}

#[tauri::command]
fn preview_tree(
    manager: tauri::State<'_, cli::CliManager>,
    skill: String,
) -> Result<Vec<preview::FileEntry>, cli::CommandError> {
    preview::tree(&manager, &skill)
}

#[tauri::command]
fn read_preview(
    manager: tauri::State<'_, cli::CliManager>,
    skill: String,
    path: String,
) -> Result<preview::FileContent, cli::CommandError> {
    preview::read(&manager, &skill, &path)
}

#[tauri::command]
fn reveal_path(
    manager: tauri::State<'_, cli::CliManager>,
    skill: String,
    path: Option<String>,
) -> Result<(), cli::CommandError> {
    preview::reveal(&manager, &skill, path.as_deref())
}

#[tauri::command]
fn translate_preview(
    manager: tauri::State<'_, cli::CliManager>,
    skill: String,
    path: String,
    target_language: String,
) -> Result<translation::TranslationResult, cli::CommandError> {
    translation::translate_installed(&manager, &skill, &path, &target_language)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(cli::CliManager::default())
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            retry_runtime,
            list_skills,
            search_skills,
            add_skill,
            remove_skill,
            update_skill,
            preview_tree,
            read_preview,
            reveal_path,
            translate_preview,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Skill Deck");
}
