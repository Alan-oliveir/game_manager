//! Importa jogos da Ubisoft a partir do diretório do Ubisoft Game Launcher.
//!
//! Lê os arquivos `.install` e o cache de configuração da biblioteca para detectar
//! jogos instalados e da biblioteca do usuário.
//!
//! `wine_prefix` — (Linux) caminho do Wine prefix onde o Ubisoft Game Launcher está instalado.
//! No Windows o parâmetro é ignorado.

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games, trigger_enrichment_if_needed,
};
use crate::database::AppState;
use crate::errors::AppError;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn import_ubisoft_games(
    app: AppHandle,
    state: State<'_, AppState>,
    wine_prefix: Option<String>,
) -> Result<String, AppError> {
    use crate::sources::providers::GameSource;
    use crate::sources::ubisoft::UbisoftSource;

    let prefix = wine_prefix
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let source = UbisoftSource::new(true, prefix);
    let games = source.fetch_games().await?;

    if games.is_empty() {
        return Ok(format_import_empty("Ubisoft"));
    }

    let (inserted, updated, newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Ubisoft", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    trigger_enrichment_if_needed(&app, newly_imported);

    Ok(message)
}
