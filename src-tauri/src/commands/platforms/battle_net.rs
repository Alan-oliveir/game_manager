//! Battle.net - Importa jogos instalados via Battle.net (Blizzard/Activision)

use crate::commands::platforms::core::{format_import_empty, format_import_summary};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::battle_net::{BattleNetGame, BattleNetSource};
use crate::utils::status_logic;
use chrono::Utc;
use rusqlite::params;
use tauri::{AppHandle, Emitter, State};
use tracing::info;
use uuid::Uuid;

/// Persiste jogos do Battle.net nas tabelas `games`.
///
/// Difere de `persist_source_games` por gravar `cover_url` na criação, vindo do `aggregate.json` (box_art/logo_art)
/// Usado como fallback até a RAWG encontrar (ou não) uma capa melhor no enriquecimento.
async fn persist_battle_net_games(
    state: &AppState,
    games: Vec<BattleNetGame>,
) -> Result<(u32, u32), AppError> {
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for bnet_game in games {
        let game = &bnet_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND platform_game_id = ?2)",
                params![&game.platform, &game.platform_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        let last_played_iso = game
            .last_played
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.to_rfc3339()));

        if !exists {
            let new_id = Uuid::new_v4().to_string();

            tx.execute(
                "INSERT INTO games (
                    id, name, cover_url, platform, platform_game_id,
                    installed, status, playtime, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, NULL, ?11, ?12)",
                params![
                    new_id,
                    game.name.as_deref().unwrap_or("Unknown"),
                    bnet_game.cover_url,
                    game.platform,
                    game.platform_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    last_played_iso,
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed       = ?1,
                    status          = ?2,
                    last_played     = COALESCE(?3, last_played),
                    install_path    = COALESCE(?4, install_path),
                    executable_path = COALESCE(?5, executable_path)
                 WHERE platform = ?6 AND platform_game_id = ?7",
                params![
                    game.installed,
                    status,
                    last_played_iso,
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

    Ok((inserted, updated))
}

/// Importa jogos instalados via Battle.net (lendo `product.db` + `aggregate.json`).
#[tauri::command]
pub async fn import_battle_net_games(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let source = BattleNetSource::new();
    let games = source.fetch_games_detailed().await?;

    if games.is_empty() {
        return Ok(format_import_empty("Battle.net"));
    }

    let (inserted, updated) = persist_battle_net_games(&state, games).await?;
    let message = format_import_summary("Battle.net", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
