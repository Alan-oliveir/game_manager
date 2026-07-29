//! Heroic - Importa jogos instalados do Heroic Games Launcher

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games, trigger_enrichment_if_needed,
};
use crate::database::AppState;
use crate::errors::AppError;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn import_heroic_games(
    app: AppHandle,
    state: State<'_, AppState>,
    heroic_config_path: Option<String>,
) -> Result<String, AppError> {
    use crate::sources::heroic::HeroicSource;

    let config_path = heroic_config_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let games = HeroicSource::import_installed(config_path).await?;

    if games.is_empty() {
        return Ok(format_import_empty("Heroic"));
    }

    let (inserted, updated, newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Heroic", inserted, updated);
    info!("{}", message);

    trigger_enrichment_if_needed(&app, newly_imported);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
