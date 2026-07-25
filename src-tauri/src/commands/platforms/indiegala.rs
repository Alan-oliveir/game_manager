//! IndieGala - Importa jogos instalados ou biblioteca completa via IGClient

use crate::commands::platforms::core::format_import_summary;
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::indiegala::IndiegalaSource;
use crate::utils::status_logic;
use chrono::Utc;
use rusqlite::params;
use tauri::{AppHandle, Emitter, State};
use tracing::info;
use uuid::Uuid;

/// Persiste jogos da IndieGala nas tabelas `games` e `game_details`.
///
/// Difere de `persist_source_games` por também gravar `description_raw` e `tags`
/// em `game_details`. Diferente da Legacy Games, aqui não há `cover_url`.
///
/// `playtime_minutes` é passado como `Option` (não `unwrap_or(0)`) pro `UPDATE`
/// porque no modo `full`, jogos possuídos mas que não foram instalados não têm playtime
/// conhecido — usar `COALESCE` preserva o valor real já salvo de uma importação anterior
/// (ex: jogo que foi desinstalado depois de já ter sido jogado) em vez de zerar por engano.
async fn persist_indiegala_games(
    state: &AppState,
    games: Vec<crate::sources::indiegala::IndiegalaGame>,
) -> Result<(u32, u32), AppError> {
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for indiegala_game in games {
        let game = &indiegala_game.source;

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

            tx.execute(
                "INSERT INTO games (
                    id, name, cover_url, platform, platform_game_id,
                    installed, status, playtime, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, NULL, ?8, 0, NULL, ?9, ?10)",
                params![
                    new_id,
                    game.name.as_deref().unwrap_or("Unknown"),
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

            let tags_json = indiegala_game
                .tags
                .as_ref()
                .and_then(|tags| crate::database::serialize_tags(tags).ok());

            if indiegala_game.description_raw.is_some() || tags_json.is_some() {
                tx.execute(
                    "INSERT OR IGNORE INTO game_details (game_id, description_raw, tags)
                     VALUES (?1, ?2, ?3)",
                    params![new_id, indiegala_game.description_raw, tags_json],
                )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed       = ?1,
                    status          = ?2,
                    playtime        = COALESCE(?3, playtime),
                    install_path    = COALESCE(?4, install_path),
                    executable_path = COALESCE(?5, executable_path)
                 WHERE platform = ?6 AND platform_game_id = ?7",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes, // Option<u32> cru — sem unwrap_or(0)
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
    state: State<'_, AppState>,
    full: bool,
    installed_json_path: Option<String>,
    config_json_path: Option<String>,
) -> Result<String, AppError> {
    let installed_path = installed_json_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);
    let config_path = config_json_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let source = IndiegalaSource::new(installed_path);

    let games = if full {
        source.fetch_full_library_detailed(config_path).await?
    } else {
        source.fetch_installed_detailed().await?
    };

    if games.is_empty() {
        let msg = if full {
            "Nenhum jogo IndieGala encontrado na biblioteca."
        } else {
            "Nenhum jogo IndieGala instalado encontrado."
        };
        return Ok(msg.to_string());
    }

    let (inserted, updated) = persist_indiegala_games(&state, games).await?;
    let message = format_import_summary("IndieGala", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
