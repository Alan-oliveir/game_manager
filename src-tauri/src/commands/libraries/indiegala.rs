//! IndieGala - Importa jogos instalados ou biblioteca completa via IGClient

use crate::database::libraries::persist_indiegala_games;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::indiegala::IndiegalaSource;
use crate::services::libraries::{spawn_import_custom, ImportOutcome};
use tauri::{AppHandle, Manager};

/// Importa jogos da IndieGala via IGClient.
///
/// `full=false` (padrão): só jogos instalados no momento, via `installed.json`.
/// `full=true`: biblioteca completa de posse via `config.json` (`user_collection`),
/// cruzada com `installed.json` pra marcar o que está instalado e reaproveitar metadados desses casos.
///
/// `installed_json_path`/`config_json_path` — caminhos customizados opcionais.
/// Se omitidos, usam os caminhos padrão do Windows:
/// - `installed.json`: `%APPDATA%\IGClient\storage\installed.json`
/// - `config.json`: `%APPDATA%\IGClient\config.json`
#[tauri::command]
pub async fn import_indiegala_games(
    app: AppHandle,
    full: bool,
    installed_json_path: Option<String>,
    config_json_path: Option<String>,
) -> Result<(), AppError> {
    let installed_path = installed_json_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);
    let config_path = config_json_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import_custom(app, "Indiegala", move |app| async move {
        let state: tauri::State<AppState> = app.state();
        let source = IndiegalaSource::new(installed_path);

        let games = if full {
            source.fetch_full_library_detailed(config_path).await?
        } else {
            source.fetch_installed_detailed().await?
        };

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = {
            let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
            persist_indiegala_games(&mut conn, games).map_err(AppError::DatabaseError)?
        };
        Ok(ImportOutcome::Persisted {
            inserted,
            updated,
            newly_imported,
        })
    });

    Ok(())
}
