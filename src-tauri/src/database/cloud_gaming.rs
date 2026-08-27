//! Persistência dos catálogos de cloud gaming (GeForce NOW e Xbox Cloud Gaming).
//!
//! Camada pura de banco. O fetch/refresh (chamadas HTTP) fica em `providers/cloud_gaming/{gfn,xbox_cloud}.rs`.

use crate::providers::cloud_gaming::geforce_now::GfnAvailability;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

// === GEFORCE NOW ===

pub fn initialize_gfn_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS gfn_games (
            steam_app_id TEXT PRIMARY KEY,
            title        TEXT NOT NULL,
            store        TEXT NOT NULL,
            status       TEXT
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela gfn_games: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS gfn_games_meta (
            id         INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela gfn_games_meta: {e}"))?;

    Ok(())
}

pub fn get_gfn_last_fetched(conn: &Connection) -> anyhow::Result<Option<DateTime<Utc>>> {
    let ts: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM gfn_games_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    Ok(ts.and_then(|t| DateTime::from_timestamp(t, 0)))
}

/// Recebe `&mut Connection` porque abre transação — chame com o `MutexGuard`
/// já travado (ele faz deref mut automaticamente).
pub fn save_gfn_games(conn: &mut Connection, games: &[GfnAvailability]) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM gfn_games", [])?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO gfn_games (steam_app_id, title, store, status)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for g in games {
            stmt.execute(params![g.steam_app_id, g.title, g.store, g.status])?;
        }
    }
    tx.execute(
        "INSERT INTO gfn_games_meta (id, fetched_at) VALUES (1, ?1)
         ON CONFLICT(id) DO UPDATE SET fetched_at = ?1",
        params![Utc::now().timestamp()],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn find_gfn_availability(
    conn: &Connection,
    steam_app_id: &str,
) -> anyhow::Result<Option<GfnAvailability>> {
    Ok(conn
        .query_row(
            "SELECT steam_app_id, title, store, status FROM gfn_games WHERE steam_app_id = ?1 LIMIT 1",
            [steam_app_id],
            |row| {
                Ok(GfnAvailability {
                    steam_app_id: row.get(0)?,
                    title: row.get(1)?,
                    store: row.get(2)?,
                    status: row.get(3)?,
                })
            },
        )
        .optional()?)
}

// === XBOX CLOUD GAMING ===

pub fn initialize_xbox_cloud_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xbox_cloud_ids (store_id TEXT PRIMARY KEY)",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela xbox_cloud_ids: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS xbox_cloud_meta (
            id         INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela xbox_cloud_meta: {e}"))?;

    Ok(())
}

pub fn save_xbox_cloud_ids_cache(conn: &Connection, ids: &HashSet<String>) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM xbox_cloud_ids", [])
        .map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("INSERT OR REPLACE INTO xbox_cloud_ids (store_id) VALUES (?1)")
            .map_err(|e| e.to_string())?;
        for id in ids {
            stmt.execute(params![id]).map_err(|e| e.to_string())?;
        }
    }

    let now = Utc::now().timestamp();
    tx.execute(
        "INSERT OR REPLACE INTO xbox_cloud_meta (id, fetched_at) VALUES (1, ?1)",
        params![now],
    )
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())
}

pub fn xbox_cloud_cache_is_stale(conn: &Connection) -> Result<bool, String> {
    use crate::providers::cloud_gaming::xbox_cloud_gaming::XBOX_CLOUD_CACHE_TTL_DAYS;

    let fetched_at: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM xbox_cloud_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(match fetched_at {
        None => true,
        Some(ts) => Utc::now().timestamp() - ts > XBOX_CLOUD_CACHE_TTL_DAYS * 24 * 60 * 60,
    })
}

/// Verifica se um store_id também está disponível no Xbox Cloud Gaming.
pub fn is_available_on_xbox_cloud(conn: &Connection, store_id: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT 1 FROM xbox_cloud_ids WHERE store_id = ?1",
        [store_id],
        |_| Ok(()),
    )
        .optional()
        .map(|r| r.is_some())
        .map_err(|e| e.to_string())
}

/// Chamada única a partir de `database::core` — cria as tabelas de todos os
/// catálogos de cloud gaming de uma vez.
pub fn initialize_cloud_gaming_tables(conn: &Connection) -> Result<(), String> {
    initialize_gfn_tables(conn)?;
    initialize_xbox_cloud_tables(conn)?;
    Ok(())
}
