//! Itch.io - Importa jogos instalados ou biblioteca completa lendo o butler.db

use crate::commands::platforms::core::{format_import_summary, trigger_enrichment_if_needed, NewlyImportedGame};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::itch::{ItchioGame, ItchioSource};
use crate::utils::status_logic;
use chrono::{TimeZone, Utc};
use rusqlite::params;
use tauri::{AppHandle, Emitter, State};
use tracing::info;
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
                "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND platform_game_id = ?2)",
                params![&game.platform, &game.platform_game_id],
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

            tx.execute(
                "INSERT INTO games (
                    id, name, cover_url, platform, platform_game_id,
                    installed, status, playtime, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, NULL, ?11, ?12)",
                params![
                    new_id,
                    display_name,
                    itchio_game.cover_url, // Capa oficial da API da itch
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

            // Salva a descrição (se existir) na tabela de detalhes
            if itchio_game.description_raw.is_some() {
                tx.execute(
                    "INSERT OR IGNORE INTO game_details (game_id, description_raw)
                     VALUES (?1, ?2)",
                    params![new_id, itchio_game.description_raw],
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
                    installed       = ?1,
                    status          = ?2,
                    playtime        = COALESCE(?3, playtime),
                    last_played     = COALESCE(?4, last_played),
                    install_path    = COALESCE(?5, install_path),
                    executable_path = COALESCE(?6, executable_path),
                    cover_url       = COALESCE(?7, cover_url)
                 WHERE platform = ?8 AND platform_game_id = ?9",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes,
                    last_played_iso,
                    game.install_path,
                    game.executable_path,
                    itchio_game.cover_url,
                    game.platform,
                    game.platform_game_id,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            // Atualiza a descrição caso o jogo já exista e não tenha (ou tenha mudado)
            if let Some(desc) = &itchio_game.description_raw {
                tx.execute(
                    "INSERT INTO game_details (game_id, description_raw)
                     VALUES ((SELECT id FROM games WHERE platform = ?1 AND platform_game_id = ?2), ?3)
                     ON CONFLICT(game_id) DO UPDATE SET description_raw = excluded.description_raw",
                    params![game.platform, game.platform_game_id, desc],
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
    state: State<'_, AppState>,
    full: bool,
    butler_db_path: Option<String>,
) -> Result<String, AppError> {
    let db_path = butler_db_path
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    let source = ItchioSource::new(db_path);

    let games = if full {
        source.fetch_full_library_detailed().await?
    } else {
        source.fetch_installed_detailed().await?
    };

    if games.is_empty() {
        let msg = if full {
            "Nenhum jogo Itch.io encontrado na biblioteca."
        } else {
            "Nenhum jogo Itch.io instalado encontrado."
        };
        return Ok(msg.to_string());
    }

    let (inserted, updated, newly_imported) = persist_itch_games(&state, games).await?;
    let message = format_import_summary("Itch.io", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    trigger_enrichment_if_needed(&app, newly_imported);

    Ok(message)
}
