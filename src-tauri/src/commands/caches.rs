//! Comandos para gerenciar o cache de metadados
//!
//! Permite visualizar estatísticas, limpar cache expirado e invalidar entradas específicas
//! Expõe comandos Tauri para uso no frontend.

use crate::database::AppState;
use crate::errors::AppError;
use crate::services::cache;
use tauri::State;

/// Estatísticas detalhadas por tipo de cache
#[derive(serde::Serialize)]
pub struct DetailedCacheStats {
    pub total: i32,
    pub rawg_searches: i32,
    pub gamebrain_entries: i32,
    pub steam_store: i32,
    pub steam_reviews: i32,
    pub steam_playtime: i32,
    pub hltb_searches: i32,
    pub expired: i32,
}

/// Remove entradas expiradas do cache
#[tauri::command]
pub fn cleanup_cache(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.cache_db.lock()?;

    let deleted = cache::cleanup_expired_cache(&conn).map_err(AppError::DatabaseError)?;

    Ok(format!("{} entradas removidas", deleted))
}

/// Limpa TODO o cache (use com cuidado)
#[tauri::command]
pub fn clear_all_cache(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.cache_db.lock()?;

    let deleted = conn.execute("DELETE FROM api_cache", [])?;

    Ok(format!("Cache limpo: {} entradas removidas", deleted))
}

#[tauri::command]
pub fn get_detailed_cache_stats(state: State<AppState>) -> Result<DetailedCacheStats, AppError> {
    let conn = state.cache_db.lock()?;

    let total: i32 = conn
        .query_row("SELECT COUNT(*) FROM api_cache", [], |row| row.get(0))
        .unwrap_or(0);

    let rawg: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'rawg' AND external_id LIKE 'search_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let gamebrain: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'gamebrain'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let store: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'steam' AND external_id LIKE 'store_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let reviews: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'steam' AND external_id LIKE 'reviews_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let playtime: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'steam' AND external_id LIKE 'playtime_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let hltb: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE source = 'hltb' AND external_id LIKE 'search_hltb_%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let stats = cache::get_cache_stats(&conn).map_err(AppError::DatabaseError)?;

    Ok(DetailedCacheStats {
        total,
        rawg_searches: rawg,
        gamebrain_entries: gamebrain,
        steam_store: store,
        steam_reviews: reviews,
        steam_playtime: playtime,
        hltb_searches: hltb,
        expired: stats.expired_entries,
    })
}
