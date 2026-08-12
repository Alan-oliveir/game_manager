//! Legacy - Importa jogos de Legacy Games Launcher

use crate::commands::libraries::core::{spawn_import_custom, ImportOutcome, NewlyImportedGame};
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::legacy::LegacySource;
use crate::utils::status_logic;
use chrono::Utc;
use rusqlite::params;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

/// Persiste jogos da Legacy Games nas tabelas `games` e `game_details`.
///
/// Difere de `persist_source_games` por também gravar `cover_url` e
/// `description_raw` em `game_details`, que não fazem parte do `SourceGame` padrão.
async fn persist_legacy_games(
    state: &AppState,
    games: Vec<crate::providers::libraries::legacy::LegacyGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), AppError> {
    let mut newly_imported = Vec::new();
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for legacy_game in games {
        let game = &legacy_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND platform_game_id = ?2)",
                params![&game.platform, &game.platform_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            tx.execute(
                "INSERT INTO games (
                        id, name, slug, cover_url, platform, platform_game_id,
                        installed, status, playtime, playtime_source, last_played, added_at,
                        favorite, user_rating, install_path, executable_path
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, ?10, 0, NULL, ?11, ?12)",
                params![
                        new_id,
                        display_name,
                        slug,
                        legacy_game.cover_url,
                        game.platform,
                        game.platform_game_id,
                        game.installed,
                        status,
                        game.playtime_minutes.unwrap_or(0),
                        now,
                        game.install_path,
                        game.executable_path,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            // Insere metadados na tabela game_details
            if let Some(desc) = &legacy_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description) VALUES (?1, ?2)
                        ON CONFLICT(game_id) DO UPDATE SET description = COALESCE(game_descriptions.description, excluded.description)",
                    params![new_id, desc],
                )
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                platform: game.platform.clone(),
                platform_game_id: game.platform_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed   = ?1,
                    status      = ?2,
                    install_path     = COALESCE(?3, install_path),
                    executable_path  = COALESCE(?4, executable_path)
                 WHERE platform = ?5 AND platform_game_id = ?6",
                params![
                    game.installed,
                    status,
                    game.install_path,
                    game.executable_path,
                    game.platform,
                    game.platform_game_id,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            updated += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((inserted, updated, newly_imported))
}

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
        let state: tauri::State<crate::database::AppState> = app.state();
        let source = LegacySource::new_with_wine(path, prefix);
        let games = source.fetch_games_detailed().await?;

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = persist_legacy_games(&state, games).await?;
        Ok(ImportOutcome::Persisted {
            inserted,
            updated,
            newly_imported,
        })
    });

    Ok(())
}
