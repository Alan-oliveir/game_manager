//! Módulo de cache para metadados de APIs externas
//!
//! Gerencia cache persistente em SQLite para respostas de RAWG e Steam,
//! reduzindo chamadas desnecessárias e melhorando performance.

use crate::constants::{CACHE_AMAZON_LUNA_TTL_DAYS, CACHE_DEFAULT_TTL_DAYS, CACHE_EA_PLAY_TTL_DAYS, CACHE_GAMEBRAIN_ID_TTL_DAYS, CACHE_GAMEBRAIN_MEDIA_TTL_DAYS, CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS, CACHE_GAMERPOWER_TTL_DAYS, CACHE_GAME_PASS_FULL_TTL_DAYS, CACHE_HLTB_TTL_DAYS, CACHE_NEXUS_TRENDING_MODS_TTL_DAYS, CACHE_PROTON_DB_TTL_DAYS, CACHE_RAWG_GAME_TTL_DAYS, CACHE_RAWG_LIST_TTL_DAYS, CACHE_STEAM_PLAYTIME_TTL_DAYS, CACHE_STEAM_RESOLVE_TTL_DAYS, CACHE_STEAM_REVIEWS_TTL_DAYS, CACHE_STEAM_STORE_TTL_DAYS, CACHE_UBISOFT_PLUS_TTL_DAYS, NEXUS_CACHE_TTL_DAYS};
use crate::errors::AppError;
use crate::providers::mods::nexus::NexusGame;
use rusqlite::{params, Connection, OptionalExtension};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Estrutura de estatísticas do cache
#[derive(Debug, serde::Serialize)]
pub struct CacheStats {
    pub total_entries: i32,
    pub rawg_entries: i32,
    pub gamebrain_entries: i32,
    pub steam_entries: i32,
    pub hltb_entries: i32,
    pub expired_entries: i32,
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

    // Índice para facilitar queries de limpeza por data
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cache_updated
         ON api_cache(source, updated_at)",
        [],
    )
        .map_err(|e| format!("Erro ao criar índice: {}", e))?;

    // Tabelas para o cache de jogos do Nexus
    conn.execute(
        "CREATE TABLE IF NOT EXISTS nexus_games (
        domain_name TEXT PRIMARY KEY,
        nexus_id    INTEGER NOT NULL,
        name        TEXT NOT NULL,
        genre       TEXT,
        approved_date INTEGER
    )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela nexus_games: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_nexus_games_name ON nexus_games(name)",
        [],
    )
        .map_err(|e| format!("Erro ao criar índice: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS nexus_games_cache_meta (
            id         INTEGER PRIMARY KEY CHECK (id = 1), -- singleton row
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela nexus_games_cache_meta: {}", e))?;

    Ok(())
}

// === FUNÇÕES DE GERENCIAMENTO DE CACHE ===

/// Determina o TTL baseado no tipo de dado armazenado em cache.
fn get_ttl_for_cache_type(cache_key: &str) -> i64 {
    // TTL de 1 dia para listas (Em Alta, Gratuitos, Lançamentos, etc.)
    if cache_key.contains("_list_") {
        CACHE_RAWG_LIST_TTL_DAYS * 24 * 60 * 60
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
            // Usa a chave completa (external_id) para determinar TTL
            let full_key = external_id;
            if is_cache_expired(full_key, updated_at) {
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

/// Remove entradas expiradas do cache (limpeza granular)
pub fn cleanup_expired_cache(conn: &Connection) -> Result<usize, String> {
    let now = current_timestamp();

    // Diferentes cutoffs para diferentes tipos
    let rawg_cutoff = now - (CACHE_RAWG_GAME_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_id_cutoff = now - (CACHE_GAMEBRAIN_ID_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_similar_cutoff = now - (CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_media_cutoff = now - (CACHE_GAMEBRAIN_MEDIA_TTL_DAYS * 24 * 60 * 60);
    let amazon_luna_cutoff = now - (CACHE_AMAZON_LUNA_TTL_DAYS * 24 * 60 * 60);
    let gamerpower_cutoff = now - (CACHE_GAMERPOWER_TTL_DAYS * 24 * 60 * 60);
    let game_pass_full_cutoff = now - (CACHE_GAME_PASS_FULL_TTL_DAYS * 24 * 60 * 60);
    let ubisoft_plus_cutoff = now - (CACHE_UBISOFT_PLUS_TTL_DAYS * 24 * 60 * 60);
    let ea_play_cutoff = now - (CACHE_EA_PLAY_TTL_DAYS * 24 * 60 * 60);
    let store_cutoff = now - (CACHE_STEAM_STORE_TTL_DAYS * 24 * 60 * 60);
    let reviews_cutoff = now - (CACHE_STEAM_REVIEWS_TTL_DAYS * 24 * 60 * 60);
    let playtime_cutoff = now - (CACHE_STEAM_PLAYTIME_TTL_DAYS * 24 * 60 * 60);
    let steam_resolve_cutoff = now - (CACHE_STEAM_RESOLVE_TTL_DAYS * 24 * 60 * 60);
    let protondb_cutoff = now - (CACHE_PROTON_DB_TTL_DAYS * 24 * 60 * 60);
    let hltb_cutoff = now - (CACHE_HLTB_TTL_DAYS * 24 * 60 * 60);

    let deleted = conn
        .execute(
            "DELETE FROM api_cache
             WHERE (source = 'rawg' AND external_id LIKE 'search_%' AND updated_at < ?1)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_id:%' AND updated_at < ?2)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_similar:%' AND updated_at < ?3)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_media:%' AND updated_at < ?4)
                OR (source = 'amazon_luna' AND external_id LIKE 'catalog_amazon_luna%' AND updated_at < ?5)
                OR (source = 'gamerpower' AND external_id LIKE 'gamerpower_list_active%' AND updated_at < ?6)
                OR (source = 'game_pass_pc' AND external_id LIKE 'catalog_game_pass_full%' AND updated_at < ?7)
                OR (source = 'ubisoft_plus' AND external_id LIKE 'catalog_ubisoft_plus%' AND updated_at < ?8)
                OR (source = 'ea_play' AND external_id LIKE 'catalog_ea_play%' AND updated_at < ?9)
                OR (source = 'steam' AND external_id LIKE 'store_%' AND updated_at < ?10)
                OR (source = 'steam' AND external_id LIKE 'reviews_%' AND updated_at < ?11)
                OR (source = 'steam' AND external_id LIKE 'playtime_%' AND updated_at < ?12)
                OR (source = 'steam_resolve' AND updated_at < ?13)
                OR (source = 'protondb' AND updated_at < ?14)
                OR (source = 'hltb' AND external_id LIKE 'search_hltb_%' AND updated_at < ?15)",
            params![
                rawg_cutoff,
                gamebrain_id_cutoff,
                gamebrain_similar_cutoff,
                gamebrain_media_cutoff,
                amazon_luna_cutoff,
                gamerpower_cutoff,
                game_pass_full_cutoff,
                ubisoft_plus_cutoff,
                ea_play_cutoff,
                store_cutoff,
                reviews_cutoff,
                playtime_cutoff,
                steam_resolve_cutoff,
                protondb_cutoff,
                hltb_cutoff,
            ],
        )
        .map_err(|e| AppError::CacheCleanupError(e.to_string()).to_string())?;

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

    result.ok() // Retorna o dado se existir, ou None se nunca foi salvo
}

/// Retorna estatísticas do cache
pub fn get_cache_stats(conn: &Connection) -> Result<CacheStats, String> {
    let total: i32 = conn
        .query_row("SELECT COUNT(*) FROM api_cache", [], |row| row.get(0))
        .unwrap_or(0);

    let rawg: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache WHERE source = 'rawg'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let gamebrain: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache WHERE source = 'gamebrain'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let steam: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache WHERE source = 'steam'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let hltb: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache WHERE source = 'hltb'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = current_timestamp();
    let rawg_cutoff = now - (CACHE_RAWG_GAME_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_id_cutoff = now - (CACHE_GAMEBRAIN_ID_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_similar_cutoff = now - (CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS * 24 * 60 * 60);
    let gamebrain_media_cutoff = now - (CACHE_GAMEBRAIN_MEDIA_TTL_DAYS * 24 * 60 * 60);
    let amazon_luna_cutoff = now - (CACHE_AMAZON_LUNA_TTL_DAYS * 24 * 60 * 60);
    let gamerpower_cutoff = now - (CACHE_GAMERPOWER_TTL_DAYS * 24 * 60 * 60);
    let game_pass_full_cutoff = now - (CACHE_GAME_PASS_FULL_TTL_DAYS * 24 * 60 * 60);
    let ubisoft_plus_cutoff = now - (CACHE_UBISOFT_PLUS_TTL_DAYS * 24 * 60 * 60);
    let ea_play_cutoff = now - (CACHE_EA_PLAY_TTL_DAYS * 24 * 60 * 60);
    let store_cutoff = now - (CACHE_STEAM_STORE_TTL_DAYS * 24 * 60 * 60);
    let reviews_cutoff = now - (CACHE_STEAM_REVIEWS_TTL_DAYS * 24 * 60 * 60);
    let playtime_cutoff = now - (CACHE_STEAM_PLAYTIME_TTL_DAYS * 24 * 60 * 60);
    let steam_resolve_cutoff = now - (CACHE_STEAM_RESOLVE_TTL_DAYS * 24 * 60 * 60);
    let protondb_cutoff = now - (CACHE_PROTON_DB_TTL_DAYS * 24 * 60 * 60);
    let hltb_cutoff = now - (CACHE_HLTB_TTL_DAYS * 24 * 60 * 60);
    let expired: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM api_cache
             WHERE (source = 'rawg' AND external_id LIKE 'search_%' AND updated_at < ?1)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_id:%' AND updated_at < ?2)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_similar:%' AND updated_at < ?3)
                OR (source = 'gamebrain' AND external_id LIKE 'gamebrain_media:%' AND updated_at < ?4)
                OR (source = 'amazon_luna' AND external_id LIKE 'catalog_amazon_luna%' AND updated_at < ?5)
                OR (source = 'gamerpower' AND external_id LIKE 'gamerpower_list_active%' AND updated_at < ?6)
                OR (source = 'game_pass_pc' AND external_id LIKE 'catalog_game_pass_full%' AND updated_at < ?7)
                OR (source = 'ubisoft_plus' AND external_id LIKE 'catalog_ubisoft_plus%' AND updated_at < ?8)
                OR (source = 'ea_play' AND external_id LIKE 'catalog_ea_play%' AND updated_at < ?9)
                OR (source = 'steam' AND external_id LIKE 'store_%' AND updated_at < ?10)
                OR (source = 'steam' AND external_id LIKE 'reviews_%' AND updated_at < ?11)
                OR (source = 'steam' AND external_id LIKE 'playtime_%' AND updated_at < ?12)
                OR (source = 'steam_resolve' AND updated_at < ?13)
                OR (source = 'protondb' AND updated_at < ?14)
                OR (source = 'hltb' AND external_id LIKE 'search_hltb_%' AND updated_at < ?15)",
            params![
                rawg_cutoff,
                gamebrain_id_cutoff,
                gamebrain_similar_cutoff,
                gamebrain_media_cutoff,
                amazon_luna_cutoff,
                gamerpower_cutoff,
                game_pass_full_cutoff,
                ubisoft_plus_cutoff,
                ea_play_cutoff,
                store_cutoff,
                reviews_cutoff,
                playtime_cutoff,
                steam_resolve_cutoff,
                protondb_cutoff,
                hltb_cutoff,
            ],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(CacheStats {
        total_entries: total,
        rawg_entries: rawg,
        gamebrain_entries: gamebrain,
        steam_entries: steam,
        hltb_entries: hltb,
        expired_entries: expired,
    })
}

// === NEXUS MODS (Cache) ===

pub fn save_nexus_games_cache(
    conn: &Connection,
    games: &[NexusGame],
) -> Result<(), rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;

    tx.execute("DELETE FROM nexus_games", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO nexus_games (domain_name, nexus_id, name, genre, approved_date)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for game in games {
            stmt.execute(params![
                game.domain_name,
                game.id,
                game.name,
                game.genre,
                game.approved_date,
            ])?;
        }
    }

    let now = chrono::Utc::now().timestamp();
    tx.execute(
        "INSERT OR REPLACE INTO nexus_games_cache_meta (id, fetched_at) VALUES (1, ?1)",
        params![now],
    )?;

    tx.commit()
}

pub fn nexus_cache_is_stale(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let fetched_at: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM nexus_games_cache_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    match fetched_at {
        None => Ok(true),
        Some(ts) => {
            let now = chrono::Utc::now().timestamp();
            Ok(now - ts > NEXUS_CACHE_TTL_DAYS * 24 * 60 * 60) // TTL expirado
        }
    }
}

/// Carrega todos os jogos do cache local do Nexus (tabela nexus_games)
pub fn get_cached_nexus_games(conn: &Connection) -> Result<Vec<NexusGame>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT nexus_id, name, domain_name, genre, approved_date FROM nexus_games",
    )?;

    let games = stmt
        .query_map([], |row| {
            Ok(NexusGame {
                id: row.get(0)?,
                name: row.get(1)?,
                domain_name: row.get(2)?,
                genre: row.get(3)?,
                approved_date: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(games)
}
