use crate::commands::libraries::core::{format_login_success, spawn_import};
use crate::errors::AppError;
use crate::providers::libraries::gog::GogSource;
use crate::providers::libraries::providers::OAuthGameSource;
use tauri::AppHandle;
use tracing::warn;

#[tauri::command]
pub async fn gog_login(app: AppHandle) -> Result<String, AppError> {
    let source = GogSource::new(app);
    source.login().await?;
    Ok(format_login_success("GOG"))
}

#[tauri::command]
pub fn gog_logout(app: AppHandle) -> Result<(), AppError> {
    let source = GogSource::new(app);
    source.logout()
}

#[tauri::command]
pub fn gog_is_authenticated(app: AppHandle) -> Result<bool, AppError> {
    let source = GogSource::new(app);
    source.is_authenticated()
}

#[tauri::command]
pub async fn import_gog_games(
    app: AppHandle,
    gog_games_dir: Option<String>,
) -> Result<(), AppError> {
    use crate::providers::libraries::gog::{detect_installed_games, GogSource};
    use std::path::Path;

    spawn_import(app, "GOG", |app| async move {
        let source = GogSource::new(app);
        let mut games = source.fetch_games_detailed().await?;

        if let Some(dir) = gog_games_dir.filter(|s| !s.trim().is_empty()) {
            let path = Path::new(&dir);
            if path.exists() && path.is_dir() {
                detect_installed_games(&mut games, path);
            } else {
                warn!("GOG games directory provided but not found: {}", dir);
            }
        }

        Ok(games)
    });
    Ok(())
}
