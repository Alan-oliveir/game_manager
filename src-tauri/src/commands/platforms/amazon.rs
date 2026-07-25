//! Amazon Games - Login (registro de dispositivo) e importação de biblioteca completa

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, format_login_success, persist_source_games,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::amazon::AmazonSource;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

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
pub async fn import_amazon_games(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let source = AmazonSource::new(app.clone());

    let local_games = crate::sources::amazon::import_installed()?;

    // Sem login, mantém graceful degradation: só jogos instalados localmente.
    let mut games = if source.is_authenticated().unwrap_or(false) {
        source.fetch_library_detailed().await?
    } else {
        Vec::new()
    };

    crate::sources::amazon::merge_local_install_status(&mut games, local_games);

    if games.is_empty() {
        return Ok(format_import_empty("Amazon"));
    }

    let (inserted, updated, _newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Amazon", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
