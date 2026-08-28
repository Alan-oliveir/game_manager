use crate::constants::{CACHE_GFN_TTL_DAYS, GFN_GAMES_URL};
use crate::database::cloud_gaming as cloud_db;
use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct GfnGameRaw {
    pub title: String,
    pub store: String,
    #[serde(default)]
    pub steam_url: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

impl GfnGameRaw {
    pub fn steam_app_id(&self) -> Option<String> {
        self.steam_url
            .as_deref()?
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .map(str::to_string)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GfnAvailability {
    pub steam_app_id: String,
    pub title: String,
    pub store: String,
    pub status: Option<String>,
}

// === REFRESH ===

pub async fn ensure_fresh(
    conn: &Mutex<Connection>,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let needs_refresh = {
        let c = conn
            .lock()
            .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
        match cloud_db::get_gfn_last_fetched(&c)? {
            Some(last) => {
                Utc::now().signed_duration_since(last) >= Duration::days(CACHE_GFN_TTL_DAYS)
            }
            None => true,
        }
    };

    if !needs_refresh {
        return Ok(());
    }

    refresh_dataset(conn, client).await
}

async fn refresh_dataset(conn: &Mutex<Connection>, client: &reqwest::Client) -> anyhow::Result<()> {
    let resp = client.get(GFN_GAMES_URL).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Lista da GFN retornou HTTP {}", resp.status());
    }

    let raw_games: Vec<GfnGameRaw> = resp.json().await?;

    tracing::info!("GFN raw_games recebidos: {}", raw_games.len());

    let normalized: Vec<GfnAvailability> = raw_games
        .iter()
        .filter_map(|g| {
            g.steam_app_id().map(|steam_app_id| GfnAvailability {
                steam_app_id,
                title: g.title.clone(),
                store: g.store.clone(),
                status: g.status.clone(),
            })
        })
        .collect();

    let mut c = conn
        .lock()
        .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
    cloud_db::save_gfn_games(&mut c, &normalized)?;

    tracing::info!(
        "Catálogo GeForce NOW atualizado: {} jogos com Steam App ID salvos em cache",
        normalized.len()
    );

    Ok(())
}

/// Busca disponibilidade na GFN por Steam App ID.
pub fn find_gfn_availability(
    conn: &Connection,
    steam_app_id: &str,
) -> anyhow::Result<Option<GfnAvailability>> {
    cloud_db::find_gfn_availability(conn, steam_app_id)
}
