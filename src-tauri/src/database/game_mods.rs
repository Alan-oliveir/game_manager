//! Persistência do catálogo de mods de jogos provenientes do Nexus Mods (`nexus_games`).
//!
//! Camada pura de banco. A lógica de fetch, cache de mods em alta e matching
//! de nomes fica em `providers/mods/nexus.rs`.
//!
//! NOTA: `NexusGame` é um tipo de domínio definido em providers (não em
//! database), porque também é usado pra desserializar a resposta da API.

use crate::constants::NEXUS_CACHE_TTL_DAYS;
use crate::providers::mods::nexus::NexusGame;
use rusqlite::{params, Connection, OptionalExtension};

/// Cria as tabelas de catálogo do Nexus (domínio games, não cache de API).
pub fn initialize_nexus_tables(conn: &Connection) -> Result<(), String> {
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
            id         INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela nexus_games_cache_meta: {}", e))?;

    Ok(())
}

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
            Ok(now - ts > NEXUS_CACHE_TTL_DAYS * 24 * 60 * 60)
        }
    }
}

/// Carrega todos os jogos do cache local do Nexus (tabela nexus_games)
pub fn get_cached_nexus_games(conn: &Connection) -> Result<Vec<NexusGame>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT nexus_id, name, domain_name, genre, approved_date FROM nexus_games")?;

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
