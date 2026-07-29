//! Battle.net - Importa jogos instalados via Battle.net (Blizzard/Activision)

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games, trigger_enrichment_if_needed,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::battle_net::BattleNetSource;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn import_battle_net_games(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let source = BattleNetSource::new();
    let games = source.import_installed().await?;

    if games.is_empty() {
        return Ok(format_import_empty("Battle.net"));
    }

    let (inserted, updated, newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Battle.net", inserted, updated);
    info!("{}", message);

    trigger_enrichment_if_needed(&app, newly_imported);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
