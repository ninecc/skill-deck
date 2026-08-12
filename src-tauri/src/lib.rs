mod cli;
mod preview;
mod translation;

use std::sync::Arc;

async fn blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, cli::CommandError> + Send + 'static,
) -> Result<T, cli::CommandError> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|_| {
            cli::CommandError::new("internal", "The background operation could not complete.")
        })?
}

#[tauri::command]
async fn runtime_status(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
) -> Result<cli::RuntimeStatus, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.status()).await
}

#[tauri::command]
async fn retry_runtime(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
) -> Result<cli::RuntimeStatus, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.retry()).await
}

#[tauri::command]
async fn list_skills(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
) -> Result<Vec<cli::InstalledSkill>, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.list()).await
}

#[tauri::command]
async fn search_skills(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    query: String,
) -> Result<Vec<cli::SearchResult>, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.search(&query)).await
}

#[tauri::command]
async fn add_skill(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    source: String,
    skill: Option<String>,
    settings: cli::InstallSettings,
) -> Result<cli::CommandResult, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.add(&source, skill.as_deref(), settings)).await
}

#[tauri::command]
async fn remove_skill(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    name: String,
) -> Result<cli::CommandResult, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.remove(&name)).await
}

#[tauri::command]
async fn update_skill(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    name: Option<String>,
) -> Result<cli::CommandResult, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || manager.update(name.as_deref())).await
}

#[tauri::command]
async fn preview_tree(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    skill: String,
) -> Result<Vec<preview::FileEntry>, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || preview::tree(&manager, &skill)).await
}

#[tauri::command]
async fn read_preview(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    skill: String,
    path: String,
) -> Result<preview::FileContent, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || preview::read(&manager, &skill, &path)).await
}

#[tauri::command]
async fn reveal_path(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    skill: String,
    path: Option<String>,
) -> Result<(), cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || preview::reveal(&manager, &skill, path.as_deref())).await
}

#[tauri::command]
async fn translate_preview(
    manager: tauri::State<'_, Arc<cli::CliManager>>,
    skill: String,
    path: String,
    target_language: String,
    translation_proxy: String,
) -> Result<translation::TranslationResult, cli::CommandError> {
    let manager = Arc::clone(manager.inner());
    blocking(move || {
        translation::translate_installed(
            &manager,
            &skill,
            &path,
            &target_language,
            &translation_proxy,
        )
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(cli::CliManager::default()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        pin::pin,
        sync::mpsc,
        task::{Context, Poll, Waker},
    };

    #[test]
    fn blocking_bridge_yields_while_work_is_pending() {
        let (release, wait) = mpsc::channel();
        let future = blocking(move || {
            wait.recv().unwrap();
            Ok(())
        });
        let mut future = pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(future.as_mut().poll(&mut context), Poll::Pending));
        release.send(()).unwrap();
        tauri::async_runtime::block_on(future).unwrap();
    }
}
