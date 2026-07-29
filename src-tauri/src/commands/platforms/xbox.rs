//! Xbox / Microsoft Store - Importação de jogos instalados via Gaming Services

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games, trigger_enrichment_if_needed,
};
use crate::database::AppState;
use crate::errors::AppError;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn import_xbox_games(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let games = crate::sources::xbox::import_installed()?;

    if games.is_empty() {
        return Ok(format_import_empty("Xbox"));
    }

    let (inserted, updated, newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Xbox", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    trigger_enrichment_if_needed(&app, newly_imported);

    Ok(message)
}
