//! Comandos relacionados ao enriquecimento de capas do jogo
//!
//! Permite buscar capas faltantes para jogos na biblioteca usando a API RAWG, com cache de metadados.

use super::shared::{fetch_rawg_metadata, EnrichCompletePayload, EnrichProgress};
use crate::constants::RAWG_RATE_LIMIT_MS;
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use rusqlite::params;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::sleep;
use tracing::info;

/// Busca capas faltantes via RAWG (COM CACHE)
#[tauri::command]
pub async fn fetch_missing_covers(app: AppHandle) -> Result<(), AppError> {
    let app_handle = app.clone();
    let api_key = database::get_secret(&app, "rawg_api_key")?;
    if api_key.is_empty() {
        return Err(AppError::ValidationError(
            "API Key da RAWG não configurada.".to_string(),
        ));
    }

    tauri::async_runtime::spawn(async move {
        info!("Iniciando busca de capas faltantes com cache...");

        let state: State<AppState> = app_handle.state();
        let mut total_updated = 0;
        let mut total_failed = 0;

        let games_without_cover: Vec<(String, String)> = {
            let conn = state.games_db.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name FROM games WHERE cover_url IS NULL OR cover_url = ''")
                .unwrap();

            stmt.query_map([], |row| -> rusqlite::Result<(String, String)> {
                Ok((row.get(0)?, row.get(1)?))
            })
                .unwrap()
                .flatten()
                .collect()
        };

        if !games_without_cover.is_empty() {
            let count = games_without_cover.len();

            for (index, (game_id, name)) in games_without_cover.into_iter().enumerate() {
                let _ = app_handle.emit(
                    "enrich_progress",
                    EnrichProgress {
                        current: (index + 1) as i32,
                        total_found: count as i32,
                        last_game: format!("Capa: {}", name),
                        status: "running".to_string(),
                        platform: None,
                    },
                );

                // Busca com cache usando block_in_place
                let img_url = {
                    let cache_conn = state.cache_db.lock().unwrap();
                    let result = tokio::task::block_in_place(|| {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            fetch_rawg_metadata(&api_key, &name, &cache_conn)
                                .await
                                .and_then(|d| d.background_image)
                        })
                    });
                    result
                };

                if let Some(img) = img_url {
                    let conn = state.games_db.lock().unwrap();
                    if conn
                        .execute(
                            "UPDATE games SET cover_url = ?1 WHERE id = ?2",
                            params![img, game_id],
                        )
                        .is_ok()
                    {
                        total_updated += 1;
                    } else {
                        total_failed += 1;
                    }
                } else {
                    total_failed += 1;
                }

                sleep(Duration::from_millis(RAWG_RATE_LIMIT_MS)).await;
            }
        }

        info!(
            "Busca de capas finalizada: {} sucesso, {} falhas",
            total_updated, total_failed
        );
        let _ = app_handle.emit(
            "enrich_complete",
            EnrichCompletePayload {
                platform: None,
                message: "Busca de capas finalizada.".to_string(),
            },
        );
    });

    Ok(())
}
