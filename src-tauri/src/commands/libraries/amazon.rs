use crate::commands::libraries::core::{format_login_success, spawn_import};
use crate::errors::AppError;
use crate::providers::libraries::amazon::AmazonSource;
use tauri::AppHandle;

#[tauri::command]
pub async fn amazon_login(app: AppHandle) -> Result<String, AppError> {
    let source = AmazonSource::new(app);
    source.login().await?;
    Ok(format_login_success("Amazon"))
}

#[tauri::command]
pub async fn amazon_logout(app: AppHandle) -> Result<(), AppError> {
    let source = AmazonSource::new(app);
    source.logout().await
}

#[tauri::command]
pub fn amazon_is_authenticated(app: AppHandle) -> Result<bool, AppError> {
    let source = AmazonSource::new(app);
    source.is_authenticated()
}

#[tauri::command]
pub async fn import_amazon_games(app: AppHandle) -> Result<(), AppError> {
    spawn_import(app, "Amazon", |app| async move {
        let source = AmazonSource::new(app.clone());
        let local_games = crate::providers::libraries::amazon::import_installed()?;

        let mut games = if source.is_authenticated().unwrap_or(false) {
            source.fetch_library_detailed().await?
        } else {
            Vec::new()
        };

        crate::providers::libraries::amazon::merge_local_install_status(&mut games, local_games);
        Ok(games)
    });
    Ok(())
}
