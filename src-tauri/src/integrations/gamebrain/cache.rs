//! Caminho: src/services/integration/gamebrain/cache.rs

use super::models::SimilarGame;
use crate::database::cache;
use crate::database::AppState;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::{AppHandle, Manager};

pub fn gamebrain_id_cache_key(playlite_game_id: &str) -> String {
    format!("gamebrain_id:{}", playlite_game_id)
}

pub fn gamebrain_similar_cache_key(gamebrain_id: u64) -> String {
    format!("gamebrain_similar:{}", gamebrain_id)
}

pub fn gamebrain_media_cache_key(gamebrain_id: u64) -> String {
    format!("gamebrain_media:{}", gamebrain_id)
}

fn with_metadata_cache_conn<T>(
    app: &AppHandle,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let state = app.state::<AppState>();
    let conn = state
        .cache_db
        .lock()
        .map_err(|_| "Falha ao acessar o cache persistente do GameBrain".to_string())?;

    f(&conn)
}

pub fn read_cached_json<T: DeserializeOwned>(
    app: &AppHandle,
    source: &str,
    external_id: &str,
    stale: bool,
) -> Result<Option<T>, String> {
    with_metadata_cache_conn(app, |conn| {
        let payload = if stale {
            cache::get_stale_api_data(conn, source, external_id)
        } else {
            cache::get_cached_api_data(conn, source, external_id)
        };

        match payload {
            Some(payload) => serde_json::from_str::<T>(&payload)
                .map(Some)
                .map_err(|e| e.to_string()),
            None => Ok(None),
        }
    })
}

pub fn save_cached_json<T: Serialize>(
    app: &AppHandle,
    source: &str,
    external_id: &str,
    value: &T,
) -> Result<(), String> {
    with_metadata_cache_conn(app, |conn| {
        let payload = serde_json::to_string(value).map_err(|e| e.to_string())?;
        cache::save_cached_api_data(conn, source, external_id, &payload)
    })
}

pub fn take_similar_limit(
    results: Vec<SimilarGame>,
    requested_limit: u32,
) -> Vec<SimilarGame> {
    results.into_iter().take(requested_limit as usize).collect()
}

pub fn read_stale_gamebrain_id(
    app: &AppHandle,
    id_cache_key: &str,
    playlite_game_id: &str,
) -> Result<Option<u64>, String> {
    let stale_id = read_cached_json::<u64>(app, "gamebrain", id_cache_key, true)?;

    if let Some(id) = stale_id {
        tracing::debug!(
            "GameBrain ID fallback para cache antigo => game_id='{}' gamebrain_id={}",
            playlite_game_id,
            id
        );
    }

    Ok(stale_id)
}

pub fn read_stale_similar_games(
    app: &AppHandle,
    similar_cache_key: &str,
    gamebrain_id: u64,
    requested_limit: u32,
) -> Result<Option<Vec<SimilarGame>>, String> {
    let stale_results =
        read_cached_json::<Vec<SimilarGame>>(app, "gamebrain", similar_cache_key, true)?;

    if let Some(results) = stale_results {
        tracing::debug!(
            "GameBrain similar stale cache fallback => gamebrain_id={}",
            gamebrain_id
        );

        return Ok(Some(take_similar_limit(results, requested_limit)));
    }

    Ok(None)
}
