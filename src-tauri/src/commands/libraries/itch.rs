//! Itch.io - Importa jogos instalados ou biblioteca completa lendo o butler.db

use crate::commands::libraries::core::{spawn_import_custom, ImportOutcome, NewlyImportedGame};
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::itch::{ItchioGame, ItchioSource};
use crate::utils::status_logic;
use chrono::{TimeZone, Utc};
use rusqlite::{params, OptionalExtension};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

/// Persiste jogos da Itch.io nas tabelas `games` e `game_details`.
///
/// Grava `cover_url` diretamente na tabela `games` e envia `description_raw` para a tabela `game_details`.
async fn persist_itch_games(
    state: &AppState,
    games: Vec<ItchioGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), AppError> {
    let mut newly_imported = Vec::new();
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for itchio_game in games {
        let game = &itchio_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        // O app itch salva o timestamp em unix (segundos). Convertendo para ISO 8601
        let last_played_iso = game.last_played.and_then(|ts| {
            if ts > 0 {
                Utc.timestamp_opt(ts, 0).single().map(|dt| dt.to_rfc3339())
            } else {
                None
            }
        });

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            tx.execute(
                "INSERT INTO games (
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, NULL, ?12, ?13)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    crate::models::PlaytimeSource::Store(crate::models::Library::Itch)
                        .as_db_str(),
                    last_played_iso,
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            if let Some(url) = &itchio_game.cover_url {
                crate::providers::media::steamgriddb::db::upsert_game_image(
                    &tx, &new_id, "itch", url, None, None, None, 2,
                )
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }

            // Salva a descrição (se existir) na tabela de detalhes
            if let Some(desc) = &itchio_game.description {
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
                    last_played     = COALESCE(?5, last_played),
                    install_path    = COALESCE(?6, install_path),
                    executable_path = COALESCE(?7, executable_path)
                WHERE library = ?8 AND library_game_id = ?9",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes,
                    crate::models::PlaytimeSource::Store(crate::models::Library::Itch)
                        .as_db_str(),
                    last_played_iso,
                    game.install_path,
                    game.executable_path,
                    game.library,
                    game.library_game_id,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            if let Some(url) = &itchio_game.cover_url {
                let existing_id: Option<String> = tx
                    .query_row(
                        "SELECT id FROM games WHERE library = ?1 AND library_game_id = ?2",
                        params![game.library, game.library_game_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                if let Some(id) = existing_id {
                    crate::providers::media::steamgriddb::db::upsert_game_image(
                        &tx, &id, "itch", url, None, None, None, 2,
                    )
                        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
                }
            }

            // Atualiza a descrição caso o jogo já exista e não tenha (ou tenha mudado)
            if let Some(desc) = &itchio_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description)
                        VALUES ((SELECT id FROM games WHERE library = ?1 AND library_game_id = ?2), ?3)
                        ON CONFLICT(game_id) DO UPDATE SET description = excluded.description",
                    params![game.library, game.library_game_id, desc],
                ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }

            updated += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((inserted, updated, newly_imported))
}

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
        let state: tauri::State<crate::database::AppState> = app.state();
        let source = ItchioSource::new(db_path);

        let games = if full {
            source.fetch_full_library_detailed().await?
        } else {
            source.fetch_installed_detailed().await?
        };

        if games.is_empty() {
            return Ok(ImportOutcome::Empty);
        }

        let (inserted, updated, newly_imported) = persist_itch_games(&state, games).await?;
        Ok(ImportOutcome::Persisted {
            inserted,
            updated,
            newly_imported,
        })
    });

    Ok(())
}
