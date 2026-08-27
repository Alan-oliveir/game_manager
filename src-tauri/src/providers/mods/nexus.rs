use crate::database::cache;
use crate::database::game_mods;
use crate::database::AppState;
use crate::utils::text::{normalize_for_matching, strip_edition_suffix};
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

// === STRUCTS ===

#[derive(Debug, Deserialize)]
pub struct NexusGame {
    pub id: i64,
    pub name: String,
    pub domain_name: String,
    pub genre: Option<String>,
    pub approved_date: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct NexusTrendingModRaw {
    name: String,
    author: String,
    summary: String,
    picture_url: String,
    domain_name: String,
    mod_id: i64,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendingMod {
    pub name: String,
    pub author: String,
    pub summary: String,
    pub picture_url: String,
    pub mod_page_url: String,
}

impl From<NexusTrendingModRaw> for TrendingMod {
    fn from(raw: NexusTrendingModRaw) -> Self {
        let mod_page_url = format!(
            "https://www.nexusmods.com/{}/mods/{}",
            raw.domain_name, raw.mod_id
        );
        TrendingMod {
            name: raw.name,
            author: raw.author,
            summary: raw.summary,
            picture_url: raw.picture_url,
            mod_page_url,
        }
    }
}

// === CLIENTE ===

pub async fn fetch_nexus_games(api_key: &str) -> Result<Vec<NexusGame>, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.nexusmods.com/v1/games.json")
        .header("accept", "application/json")
        .header("apikey", api_key)
        .header("User-Agent", "Playlite/1.0.0 (Windows_NT 10.0; x64)")
        .send()
        .await?
        .error_for_status()?;

    response.json::<Vec<NexusGame>>().await
}

pub async fn fetch_trending_mods(
    api_key: &str,
    domain: &str,
) -> Result<Vec<TrendingMod>, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.nexusmods.com/v1/games/{domain}/mods/trending.json");

    let response = client
        .get(&url)
        .header("accept", "application/json")
        .header("apikey", api_key)
        .header("User-Agent", "Playlite/1.0.0 (Windows_NT 10.0; x64)")
        .send()
        .await?
        .error_for_status()?;

    let raw: Vec<NexusTrendingModRaw> = response.json().await?;
    Ok(raw.into_iter().map(TrendingMod::from).collect())
}

/// Extrai o domain_name da URL salva em `external_links["nexus"]`.
pub fn extract_domain_from_nexus_url(url: &str) -> Option<&str> {
    url.trim_start_matches("https://www.nexusmods.com/")
        .split('/')
        .next()
        .filter(|s| !s.is_empty())
}

pub async fn fetch_trending_mods_cached(
    api_key: &str,
    domain: &str,
    cache_conn: &rusqlite::Connection,
) -> Result<Vec<TrendingMod>, String> {
    let cache_key = format!("trending_mods_{domain}");

    if let Some(cached) = cache::get_cached_api_data(cache_conn, "nexus", &cache_key) {
        if let Ok(mods) = serde_json::from_str::<Vec<TrendingMod>>(&cached) {
            return Ok(mods);
        }
    }

    let mods = fetch_trending_mods(api_key, domain)
        .await
        .map_err(|e| format!("Erro ao buscar mods em alta: {e}"))?;

    if let Ok(json) = serde_json::to_string(&mods) {
        let _ = cache::save_cached_api_data(cache_conn, "nexus", &cache_key, &json);
    }

    Ok(mods)
}

// === MATCHING ===

fn normalize_title(name: &str) -> String {
    normalize_for_matching(&strip_edition_suffix(name))
}

pub fn find_best_nexus_match<'a>(
    game_name: &str,
    nexus_games: &'a [NexusGame],
) -> Option<&'a NexusGame> {
    let normalized_target = normalize_title(game_name);

    if let Some(exact) = nexus_games
        .iter()
        .find(|g| normalize_title(&g.name) == normalized_target)
    {
        return Some(exact);
    }

    const SIMILARITY_THRESHOLD: f64 = 0.92;

    nexus_games
        .iter()
        .map(|g| {
            let score = strsim::jaro_winkler(&normalized_target, &normalize_title(&g.name));
            (g, score)
        })
        .filter(|(_, score)| *score >= SIMILARITY_THRESHOLD)
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(g, _)| g)
}

// === ORQUESTRAÇÃO ===

/// Verifica se o catálogo de jogos do Nexus está desatualizado e, se sim, busca e salva em cache.
pub async fn refresh_nexus_games_if_stale(app: &AppHandle) -> Result<(), String> {
    let state: tauri::State<AppState> = app.state();

    let needs_refresh = {
        let conn = state.games_db.lock().map_err(|_| "Falha DB Games Lock")?;
        game_mods::nexus_cache_is_stale(&conn).unwrap_or(true)
    };

    if !needs_refresh {
        return Ok(());
    }

    let api_key = match crate::database::get_secret(app, "nexus_api_key") {
        Ok(k) if !k.is_empty() => k,
        _ => return Ok(()),
    };

    let games = fetch_nexus_games(&api_key)
        .await
        .map_err(|e| format!("Erro ao buscar jogos do Nexus: {e}"))?;

    let conn = state.games_db.lock().map_err(|_| "Falha DB Games Lock")?;
    game_mods::save_nexus_games_cache(&conn, &games).map_err(|e| e.to_string())?;

    info!(
        "Catálogo Nexus atualizado: {} jogos salvos em cache",
        games.len()
    );
    Ok(())
}

/// Dispara o bootstrap em background.
pub fn spawn_nexus_games_bootstrap(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = refresh_nexus_games_if_stale(&app).await {
            warn!("Bootstrap Nexus: {}", e);
        }
    });
}
