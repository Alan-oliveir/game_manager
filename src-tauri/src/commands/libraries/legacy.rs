//! Legacy - Importa jogos de Legacy Games Launcher

use crate::database::libraries::persist_legacy_games;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::legacy::LegacySource;
use crate::services::libraries::{spawn_import_custom, ImportOutcome};
use tauri::{AppHandle, Manager};

/// Importa a biblioteca de jogos da Legacy Games.
///
/// Lê o arquivo `app-state-bck.json` do launcher da Legacy Games,
/// cruza os jogos adquiridos com o catálogo embutido e persiste os dados
/// nas tabelas `games` e `game_details`.
///
/// `app_state_path` — (opcional) caminho customizado para o `app-state-bck.json`.
/// Se omitido, usa o caminho padrão do sistema operacional.
/// `wine_prefix` — (Linux) caminho do Wine prefix onde o Legacy Games Launcher está instalado.
/// No Windows o parâmetro é ignorado.
#[tauri::command]
pub async fn import_legacy_games(
    app: AppHandle,
    app_state_path: Option<String>,
    wine_prefix: Option<String>,
) -> Result<(), AppError> {
    let path = app_state_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);
    let prefix = wine_prefix
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import_custom(app, "LegacyGames", |app| async move {
        let state: tauri::State<AppState> = app.state();
        let source = LegacySource::new_with_wine(path, prefix);
        let games = source.fetch_games_detailed().await?;

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = {
            let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
            persist_legacy_games(&mut conn, games).map_err(AppError::DatabaseError)?
        };
        Ok(ImportOutcome::Persisted {
            inserted,
            updated,
            newly_imported,
        })
    });

    Ok(())
}
