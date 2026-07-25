//! EA - Importa jogos instalados via EA Desktop (Electronic Arts)

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::ea::EaSource;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

/// Importa jogos instalados via EA App, escaneando a pasta de instalação informada.
///
/// - `ea_install_dir` — pasta onde o EA App instala os jogos (configurável no client EA Desktop).
/// **Observação:** Sem esse caminho, não há detecção possível.
#[tauri::command]
pub async fn import_ea_games(
    app: AppHandle,
    state: State<'_, AppState>,
    ea_install_dir: Option<String>,
) -> Result<String, AppError> {
    let install_dir = ea_install_dir
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let source = EaSource::new(install_dir);
    let games = source.import_installed().await?;

    if games.is_empty() {
        return Ok(format_import_empty("EA"));
    }

    let (inserted, updated, _newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("EA", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
