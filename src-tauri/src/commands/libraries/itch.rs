//! Itch.io - Importa jogos instalados ou biblioteca completa lendo o butler.db

use crate::database::libraries::persist_itch_games;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::itch::ItchioSource;
use crate::services::libraries::{spawn_import_custom, ImportOutcome};
use tauri::{AppHandle, Manager};

/// Importa jogos da Itch.io lendo o butler.db.
///
/// `full=false` (padrão): apenas jogos efetivamente instalados localmente.
/// `full=true`: biblioteca completa de posse do usuário na plataforma.
///
/// `butler_db_path`: caminho customizado opcional caso o usuário use o app itch de forma portátil ou em um local não-padrão.
#[tauri::command]
pub async fn import_itch_games(
    app: AppHandle,
    full: bool,
    butler_db_path: Option<String>,
) -> Result<(), AppError> {
    let db_path = butler_db_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import_custom(app, "Itch", move |app| async move {
        let state: tauri::State<AppState> = app.state();
        let source = ItchioSource::new(db_path);

        let games = if full {
            source.fetch_full_library_detailed().await?
        } else {
            source.fetch_installed_detailed().await?
        };

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = {
            let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
            persist_itch_games(&mut conn, games).map_err(AppError::DatabaseError)?
        };
        Ok(ImportOutcome::Persisted { inserted, updated, newly_imported })
    });

    Ok(())
}
