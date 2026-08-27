use crate::database::cloud_gaming as cloud_db;
use crate::providers::subscriptions::GamePassGame;
use reqwest::Client;
use rusqlite::Connection;
use std::collections::HashSet;

const XBOX_CLOUD_SIGL: &str = "1bf84c2b-0643-4591-893f-d9edb703f692";

#[allow(dead_code)]
const XBOX_BUY_AND_STREAM_SIGL: &str = "e78d9a61-5ef4-43af-b400-edba1250b18e";

// pub(crate) porque database/cloud_gaming.rs precisa ler esse valor
pub(crate) const XBOX_CLOUD_CACHE_TTL_DAYS: i64 = 30;

// === CLIENTE ===

pub async fn fetch_xbox_cloud_ids(market: &str, language: &str) -> Result<HashSet<String>, String> {
    let client = Client::new();
    let url = format!(
        "https://catalog.gamepass.com/sigls/v3?id={}&market={}&language={}&subscriptionContext=none&platformContext=Cloud%3AXGPUWEB",
        XBOX_CLOUD_SIGL, market, language
    );

    let sigls: Vec<serde_json::Value> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(sigls
        .iter()
        .filter_map(|v| v["id"].as_str().map(str::to_string))
        .collect())
}

// === ORQUESTRAÇÃO ===

pub async fn refresh_xbox_cloud_ids_if_stale(
    conn: &std::sync::Mutex<Connection>,
    market: &str,
    language: &str,
) -> Result<(), String> {
    let needs_refresh = {
        let c = conn.lock().map_err(|_| "Falha DB Games Lock")?;
        cloud_db::xbox_cloud_cache_is_stale(&c)?
    };

    if !needs_refresh {
        return Ok(());
    }

    let ids = fetch_xbox_cloud_ids(market, language).await?;

    let c = conn.lock().map_err(|_| "Falha DB Games Lock")?;
    cloud_db::save_xbox_cloud_ids_cache(&c, &ids)?;

    tracing::info!(
        "Cache Xbox Cloud Gaming atualizado: {} IDs salvos",
        ids.len()
    );
    Ok(())
}

pub fn is_available_on_xbox_cloud(conn: &Connection, store_id: &str) -> Result<bool, String> {
    cloud_db::is_available_on_xbox_cloud(conn, store_id)
}

// === INTERSEÇÃO COM O CATÁLOGO PC GAME PASS ===

pub fn cloud_available_pc_games<'a>(
    pc_games: &'a [GamePassGame],
    cloud_ids: &HashSet<String>,
) -> Vec<&'a GamePassGame> {
    pc_games
        .iter()
        .filter(|g| cloud_ids.contains(&g.store_id))
        .collect()
}
