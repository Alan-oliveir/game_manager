//! Epic Games - Login OAuth e importação (biblioteca completa + instalados)

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, format_login_success, persist_source_games,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::epic::{merge_local_install_status, EpicSource};
use crate::sources::providers::OAuthGameSource;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

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
    state: State<'_, AppState>,
    wine_prefix: Option<String>,
) -> Result<String, AppError> {
    let prefix = wine_prefix
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let source = EpicSource::new(app.clone(), prefix);

    let local_games = source.import_installed().await?;

    // Sem login, mantém o comportamento atual: só jogos instalados.
    let mut games = if source.is_authenticated().unwrap_or(false) {
        source.fetch_library_detailed().await?
    } else {
        Vec::new()
    };

    merge_local_install_status(&mut games, local_games);

    if games.is_empty() {
        return Ok(format_import_empty("Epic"));
    }

    let (inserted, updated, _newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Epic", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
