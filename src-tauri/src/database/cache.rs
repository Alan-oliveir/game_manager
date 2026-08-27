//! Módulo de cache para metadados de APIs externas
//!
//! Gerencia cache persistente em SQLite para respostas de IGDB e Steam,
//! reduzindo chamadas desnecessárias e melhorando performance.

use crate::constants::{
    CACHE_AMAZON_LUNA_TTL_DAYS, CACHE_DEFAULT_TTL_DAYS, CACHE_EA_PLAY_TTL_DAYS,
    CACHE_GAMEBRAIN_ID_TTL_DAYS, CACHE_GAMEBRAIN_MEDIA_TTL_DAYS, CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS,
    CACHE_GAMERPOWER_TTL_DAYS, CACHE_GAME_PASS_FULL_TTL_DAYS, CACHE_HLTB_TTL_DAYS,
    CACHE_IGDB_UPCOMING_TTL_DAYS, CACHE_NEXUS_TRENDING_MODS_TTL_DAYS, CACHE_PROTON_DB_TTL_DAYS,
    CACHE_RAWG_GAME_TTL_DAYS, CACHE_RAWG_LIST_TTL_DAYS,
    CACHE_STEAM_ACHIEVEMENTS_CIRCUIT_BREAKER_TTL_DAYS, CACHE_STEAM_PLAYTIME_TTL_DAYS,
    CACHE_STEAM_RESOLVE_TTL_DAYS, CACHE_STEAM_REVIEWS_TTL_DAYS, CACHE_STEAM_STORE_TTL_DAYS,
    CACHE_STEAM_TRENDING_TTL_DAYS, CACHE_UBISOFT_PLUS_TTL_DAYS,
};
use crate::errors::AppError;
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Estatísticas detalhadas por tipo de cache — usado pela tela de configurações.
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

/// Obtém timestamp atual em segundos
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// === INICIALIZAÇÃO DB ===

/// Inicializa o banco de cache e cria o schema
pub fn initialize_cache_db(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_cache (
            source TEXT NOT NULL,
            external_id TEXT NOT NULL,
            payload TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (source, external_id)
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela api_cache: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cache_updated
         ON api_cache(source, updated_at)",
        [],
    )
        .map_err(|e| format!("Erro ao criar índice: {}", e))?;

    Ok(())
}

// === FUNÇÕES DE GERENCIAMENTO DE CACHE ===

/// Determina o TTL baseado no tipo de dado armazenado em cache.
///
/// Ponto único de verdade sobre TTL por tipo — usado tanto para decidir se
/// uma entrada ainda é válida (`is_cache_expired`) quanto para a limpeza
/// física (`cleanup_expired_cache`). Adicionar um novo tipo de cache aqui
/// já o cobre automaticamente nos dois fluxos.
fn get_ttl_for_cache_type(cache_key: &str) -> i64 {
    if cache_key.contains("_list_") {
        CACHE_RAWG_LIST_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("steam_trending") {
        CACHE_STEAM_TRENDING_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("achievements_circuit_breaker_") {
        CACHE_STEAM_ACHIEVEMENTS_CIRCUIT_BREAKER_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("igdb_upcoming") {
        CACHE_IGDB_UPCOMING_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("resolve_") {
        CACHE_STEAM_RESOLVE_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("protondb_") {
        CACHE_PROTON_DB_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("gamebrain_id:") {
        CACHE_GAMEBRAIN_ID_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("gamebrain_similar:") {
        CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("gamebrain_media:") {
        CACHE_GAMEBRAIN_MEDIA_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("catalog_amazon_luna") {
        CACHE_AMAZON_LUNA_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("gamerpower_list_active") {
        CACHE_GAMERPOWER_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("catalog_game_pass_full") {
        CACHE_GAME_PASS_FULL_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("catalog_ubisoft_plus") {
        CACHE_UBISOFT_PLUS_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("catalog_ea_play") {
        CACHE_EA_PLAY_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("rawg_") {
        CACHE_RAWG_GAME_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("store_") {
        CACHE_STEAM_STORE_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("reviews_") {
        CACHE_STEAM_REVIEWS_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("playtime_") {
        CACHE_STEAM_PLAYTIME_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("search_hltb_") {
        CACHE_HLTB_TTL_DAYS * 24 * 60 * 60
    } else if cache_key.starts_with("trending_mods_") {
        CACHE_NEXUS_TRENDING_MODS_TTL_DAYS * 24 * 60 * 60
    } else {
        CACHE_DEFAULT_TTL_DAYS * 24 * 60 * 60 // default 7 dias
    }
}

/// Verifica se o cache está expirado baseado no TTL do tipo de dado
fn is_cache_expired(cache_key: &str, updated_at: i64) -> bool {
    let now = current_timestamp();
    let ttl_seconds = get_ttl_for_cache_type(cache_key);
    (now - updated_at) > ttl_seconds
}

/// Busca dados em cache
///
/// Retorna None se:
/// - Dados não existem
/// - Cache expirou
pub fn get_cached_api_data(conn: &Connection, source: &str, external_id: &str) -> Option<String> {
    let result: Result<(String, i64), rusqlite::Error> = conn.query_row(
        "SELECT payload, updated_at FROM api_cache
         WHERE source = ?1 AND external_id = ?2",
        params![source, external_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match result {
        Ok((payload, updated_at)) => {
            if is_cache_expired(external_id, updated_at) {
                None
            } else {
                Some(payload)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            warn!("Erro ao buscar cache: {}", e);
            None
        }
    }
}

/// Salva dados no cache
pub fn save_cached_api_data(
    conn: &Connection,
    source: &str,
    external_id: &str,
    payload: &str,
) -> Result<(), String> {
    let now = current_timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO api_cache (source, external_id, payload, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![source, external_id, payload, now],
    )
        .map_err(|e| format!("Erro ao salvar cache: {}", e))?;

    Ok(())
}

/// Retorna as chaves (source, external_id) de todas as entradas atualmente
/// expiradas, segundo o TTL de cada tipo (`get_ttl_for_cache_type`).
fn list_expired_keys(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT source, external_id, updated_at FROM api_cache")
        .map_err(|e| e.to_string())?;

    let keys = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .filter(|(_, external_id, updated_at)| is_cache_expired(external_id, *updated_at))
        .map(|(source, external_id, _)| (source, external_id))
        .collect();

    Ok(keys)
}

/// Remove entradas expiradas do cache (limpeza granular).
///
/// Genérico sobre `get_ttl_for_cache_type` — não há lista de cutoffs
/// duplicada aqui, então um novo tipo de cache é automaticamente coberto
/// assim que ganha uma regra de TTL.
pub fn cleanup_expired_cache(conn: &Connection) -> Result<usize, String> {
    let stale_keys = list_expired_keys(conn)?;

    let mut deleted = 0;
    for (source, external_id) in stale_keys {
        deleted += conn
            .execute(
                "DELETE FROM api_cache WHERE source = ?1 AND external_id = ?2",
                params![source, external_id],
            )
            .map_err(|e| AppError::CacheCleanupError(e.to_string()).to_string())?;
    }

    if deleted > 0 {
        info!("Cache cleanup: {} entradas removidas", deleted);
    }

    Ok(deleted)
}

/// Busca dados em cache IGNORANDO a validade (para modo Offline)
///
/// Retorna Some(payload) se existir, independente da data.
pub fn get_stale_api_data(conn: &Connection, source: &str, external_id: &str) -> Option<String> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT payload FROM api_cache
         WHERE source = ?1 AND external_id = ?2",
        params![source, external_id],
        |row| row.get(0),
    );

    result.ok()
}

/// Remove TODAS as entradas do cache de API, indiscriminadamente.
pub fn clear_all_cache(conn: &Connection) -> Result<usize, String> {
    conn.execute("DELETE FROM api_cache", [])
        .map_err(|e| format!("Erro ao limpar cache: {}", e))
}

/// Remove uma entrada específica do cache, independente de estar expirada ou não.
/// Usado para marcadores de estado (ex: `app_state`/`enrichment_in_progress`) que
/// precisam ser limpos explicitamente ao final de um processo, não por TTL.
pub fn delete_cached_api_data(
    conn: &Connection,
    source: &str,
    external_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM api_cache WHERE source = ?1 AND external_id = ?2",
        params![source, external_id],
    )
        .map_err(|e| format!("Erro ao deletar cache: {}", e))?;
    Ok(())
}

/// Retorna estatísticas detalhadas, quebradas por tipo de dado em cache.
pub fn get_detailed_cache_stats(conn: &Connection) -> Result<DetailedCacheStats, String> {
    let count = |sql: &str| -> i32 { conn.query_row(sql, [], |row| row.get(0)).unwrap_or(0) };

    let total = count("SELECT COUNT(*) FROM api_cache");
    let rawg = count(
        "SELECT COUNT(*) FROM api_cache WHERE source = 'rawg' AND external_id LIKE 'search_%'",
    );
    let gamebrain = count("SELECT COUNT(*) FROM api_cache WHERE source = 'gamebrain'");
    let store = count(
        "SELECT COUNT(*) FROM api_cache WHERE source = 'steam' AND external_id LIKE 'store_%'",
    );
    let reviews = count(
        "SELECT COUNT(*) FROM api_cache WHERE source = 'steam' AND external_id LIKE 'reviews_%'",
    );
    let playtime = count(
        "SELECT COUNT(*) FROM api_cache WHERE source = 'steam' AND external_id LIKE 'playtime_%'",
    );
    let hltb = count(
        "SELECT COUNT(*) FROM api_cache WHERE source = 'hltb' AND external_id LIKE 'search_hltb_%'",
    );

    // Reaproveita a mesma lógica de expiração usada na limpeza real,
    // então este número sempre bate com o que `cleanup_cache` vai remover.
    let expired = list_expired_keys(conn)?.len() as i32;

    Ok(DetailedCacheStats {
        total,
        rawg_searches: rawg,
        gamebrain_entries: gamebrain,
        steam_store: store,
        steam_reviews: reviews,
        steam_playtime: playtime,
        hltb_searches: hltb,
        expired,
    })
}
