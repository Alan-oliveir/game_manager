use super::shared::{EnrichCompletePayload, EnrichProgress};
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::media::steamgriddb::{self, SteamGridDbClient};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;

const CACHE_TTL_DAYS: i64 = 30;

#[tauri::command]
pub async fn fetch_missing_covers(app: AppHandle) -> Result<(), AppError> {
    let app_handle = app.clone();
    let api_key = database::get_secret(&app, "steamgriddb_api_key")?;
    if api_key.is_empty() {
        return Err(AppError::ValidationError(
            "API Key da SteamGridDB não configurada.".to_string(),
        ));
    }

    tauri::async_runtime::spawn(async move {
        info!("Iniciando retry de capas via SteamGridDB...");

        let state: State<AppState> = app_handle.state();
        let client = SteamGridDbClient::new(api_key);
        let mut total_updated = 0;
        let mut total_skipped = 0;

        // Jogos sem NENHUMA linha em game_images (independente da fonte)
        let games_without_cover: Vec<(String, String, Option<String>)> = {
            let conn = state.games_db.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT g.id, g.name, gd.steam_app_id
                 FROM games g
                 LEFT JOIN game_details gd ON g.id = gd.game_id
                 WHERE NOT EXISTS (
                     SELECT 1 FROM game_images gi WHERE gi.game_id = g.id AND gi.image_type = 'cover'
                 )",
            ).unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .flatten()
                .collect()
        };

        let count = games_without_cover.len();

        for (index, (game_id, name, steam_app_id)) in games_without_cover.into_iter().enumerate() {
            let _ = app_handle.emit(
                "enrich_progress",
                EnrichProgress {
                    current: (index + 1) as i32,
                    total_found: count as i32,
                    last_game: format!("Capa: {}", name),
                    status: "running".to_string(),
                    library: None,
                },
            );

            // TTL: pula se já tentamos recentemente e não achamos
            let skip = {
                let conn = state.games_db.lock().unwrap();
                match steamgriddb::db::get_cache_meta(&conn, &game_id) {
                    Ok(Some((checked_at, found))) if !found => {
                        chrono::DateTime::parse_from_rfc3339(&checked_at)
                            .map(|dt| {
                                (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_days()
                                    < CACHE_TTL_DAYS
                            })
                            .unwrap_or(false)
                    }
                    _ => false,
                }
            };
            if skip {
                total_skipped += 1;
                continue;
            }

            let appid_num = steam_app_id.as_deref().and_then(|s| s.parse::<u32>().ok());
            let result = steamgriddb::resolve_cover(&client, &name, appid_num).await;

            let conn = state.games_db.lock().unwrap();
            match result {
                Ok(Some(cover)) => {
                    let _ = steamgriddb::db::upsert_game_image(
                        &conn,
                        &game_id,
                        "steamgriddb",
                        &cover.url,
                        Some(&cover.thumb_url),
                        Some(cover.width),
                        Some(cover.height),
                        0,
                    );
                    let _ = steamgriddb::db::set_cache_meta(&conn, &game_id, true);
                    total_updated += 1;
                }
                Ok(None) => {
                    let _ = steamgriddb::db::set_cache_meta(&conn, &game_id, false);
                }
                Err(e) => {
                    tracing::warn!("SteamGridDB falhou para '{}': {}", name, e);
                }
            }
        }

        info!(
            "Retry de capas finalizado: {} atualizadas, {} puladas (TTL)",
            total_updated, total_skipped
        );
        let _ = app_handle.emit(
            "enrich_complete",
            EnrichCompletePayload {
                library: None,
                message: "Busca de capas finalizada.".to_string(),
            },
        );
    });

    Ok(())
}
