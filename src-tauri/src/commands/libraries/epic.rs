use crate::commands::libraries::core::{format_login_success, spawn_import};
use crate::errors::AppError;
use crate::providers::libraries::epic::EpicSource;
use crate::providers::libraries::providers::OAuthGameSource;
use tauri::AppHandle;

#[tauri::command]
pub async fn epic_login(app: AppHandle) -> Result<String, AppError> {
    let source = EpicSource::new(app, None);
    source.login().await?;
    Ok(format_login_success("Epic"))
}

#[tauri::command]
pub fn epic_logout(app: AppHandle) -> Result<(), AppError> {
    let source = EpicSource::new(app, None);
    source.logout()
}

#[tauri::command]
pub fn epic_is_authenticated(app: AppHandle) -> Result<bool, AppError> {
    let source = EpicSource::new(app, None);
    source.is_authenticated()
}

#[tauri::command]
pub async fn import_epic_games(
    app: AppHandle,
    wine_prefix: Option<String>,
) -> Result<(), AppError> {
    let prefix = wine_prefix
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import(app, "Epic", |app| async move {
        let source = EpicSource::new(app, prefix);
        let local_games = source.import_installed().await?;

        let mut games = if source.is_authenticated().unwrap_or(false) {
            source.fetch_library_detailed().await?
        } else {
            Vec::new()
        };

        crate::providers::libraries::epic::merge_local_install_status(&mut games, local_games);
        Ok(games)
    });
    Ok(())
}
