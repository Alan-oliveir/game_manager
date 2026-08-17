//! IndieGala - Importa jogos instalados ou biblioteca completa via IGClient

use crate::commands::libraries::core::{spawn_import_custom, ImportOutcome, NewlyImportedGame};
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::indiegala::IndiegalaSource;
use crate::utils::status_logic;
use chrono::Utc;
use rusqlite::params;
use tauri::{AppHandle, Manager};
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
    games: Vec<crate::providers::libraries::indiegala::IndiegalaGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), AppError> {
    let mut newly_imported = Vec::new();
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
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
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
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, 0, NULL, ?11, ?12)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    crate::models::PlaytimeSource::Store(crate::models::Library::Indiegala)
                        .as_db_str(),
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

            if let Some(tags) = &tags_json {
                tx.execute(
                    "INSERT OR IGNORE INTO game_details (game_id, tags) VALUES (?1, ?2)",
                    params![new_id, tags],
                )
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }

            if let Some(desc) = &indiegala_game.description {
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
                library: game.library.clone(),
                library_game_id: game.library_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                        installed       = ?1,
                        status          = ?2,
                        playtime        = COALESCE(?3, playtime),
                        playtime_source = CASE WHEN ?3 IS NOT NULL THEN ?4 ELSE playtime_source END,
                        install_path    = COALESCE(?5, install_path),
                        executable_path = COALESCE(?6, executable_path)
                    WHERE library = ?7 AND library_game_id = ?8",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes,
                    crate::models::PlaytimeSource::Store(crate::models::Library::Indiegala)
                        .as_db_str(),
                    game.install_path,
                    game.executable_path,
                    game.library,
                    game.library_game_id,
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
        let state: tauri::State<crate::database::AppState> = app.state();
        let source = IndiegalaSource::new(installed_path);

        let games = if full {
            source.fetch_full_library_detailed(config_path).await?
        } else {
            source.fetch_installed_detailed().await?
        };

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = persist_indiegala_games(&state, games).await?;
        Ok(ImportOutcome::Persisted {
            inserted,
            updated,
            newly_imported,
        })
    });

    Ok(())
}
