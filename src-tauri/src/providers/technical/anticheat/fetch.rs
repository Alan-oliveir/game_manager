use crate::constants::{AWACY_URL, CACHE_AWACY_TTL_DAYS};
use crate::providers::technical::anticheat::models::AwacyGame;
use chrono::{DateTime, Duration, Utc};
use reqwest::header::{ETAG, IF_NONE_MATCH};
use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;

fn get_meta(conn: &Connection) -> anyhow::Result<Option<(Option<String>, DateTime<Utc>)>> {
    let row = conn
        .query_row(
            "SELECT etag, last_fetched FROM anticheat_meta WHERE id = 1",
            [],
            |row| {
                let etag: Option<String> = row.get(0)?;
                let last_fetched: String = row.get(1)?;
                Ok((etag, last_fetched))
            },
        )
        .optional()?;

    match row {
        Some((etag, last_fetched_str)) => {
            let last_fetched = DateTime::parse_from_rfc3339(&last_fetched_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now() - Duration::days(CACHE_AWACY_TTL_DAYS + 1));
            Ok(Some((etag, last_fetched)))
        }
        None => Ok(None),
    }
}

fn touch_last_fetched(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE anticheat_meta SET last_fetched = ?1 WHERE id = 1",
        rusqlite::params![Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

/// Garante que o dataset local está dentro do TTL de 90 dias.
///
/// Importante: recebe `&Mutex<Connection>`, não `&Connection` ou `&mut Connection` diretamente.
/// Isso permite que a função trave e destrave o mutex em pontos específicos, nunca segurando o
/// `MutexGuard` (que não é `Send`) através de um `.await`.
pub async fn ensure_fresh(
    conn: &Mutex<Connection>,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let (needs_refresh, cached_etag) = {
        let c = conn
            .lock()
            .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
        match get_meta(&c)? {
            Some((etag, last_fetched)) => {
                let age = Utc::now().signed_duration_since(last_fetched);
                if age < Duration::days(CACHE_AWACY_TTL_DAYS) {
                    (false, None)
                } else {
                    (true, etag)
                }
            }
            None => (true, None),
        }
        // `c` (o guard) é dropado aqui, antes de qualquer await
    };

    if !needs_refresh {
        return Ok(());
    }

    refresh_dataset(conn, client, cached_etag).await
}

async fn refresh_dataset(
    conn: &Mutex<Connection>,
    client: &reqwest::Client,
    cached_etag: Option<String>,
) -> anyhow::Result<()> {
    let mut req = client.get(AWACY_URL);
    if let Some(etag) = &cached_etag {
        req = req.header(IF_NONE_MATCH, etag);
    }
    let resp = req.send().await?; // nenhum lock seguro durante o await

    if resp.status() == 304 {
        let c = conn
            .lock()
            .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
        touch_last_fetched(&c)?;
        return Ok(());
    }

    if !resp.status().is_success() {
        anyhow::bail!("AWACY retornou HTTP {}", resp.status());
    }

    let new_etag = resp
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let games: Vec<AwacyGame> = resp.json().await?; // ainda sem lock

    // Só agora travamos, para a parte 100% síncrona de escrita
    let mut c = conn
        .lock()
        .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
    let tx = c.transaction()?;
    tx.execute("DELETE FROM anticheat_games", [])?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO anticheat_games
             (slug, name, status, anticheats, steam_id, epic_namespace, epic_slug, native, reference, date_changed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )?;
        for g in &games {
            stmt.execute(rusqlite::params![
                g.slug,
                g.name,
                g.status,
                serde_json::to_string(&g.anticheats)?,
                g.store_ids.steam,
                g.store_ids.epic.as_ref().map(|e| &e.namespace),
                g.store_ids.epic.as_ref().map(|e| &e.slug),
                g.native as i32,
                g.reference,
                g.date_changed,
            ])?;
        }
    }
    let game_count = i64::try_from(games.len())?;

    tx.execute(
        "INSERT INTO anticheat_meta (id, etag, last_fetched, game_count)
         VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET etag = ?1, last_fetched = ?2, game_count = ?3",
        rusqlite::params![new_etag, Utc::now().to_rfc3339(), game_count],
    )?;
    tx.commit()?;

    Ok(())
}
