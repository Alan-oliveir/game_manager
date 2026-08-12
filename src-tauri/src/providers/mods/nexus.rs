use crate::constants::NEXUS_CACHE_TTL_DAYS;
use crate::services::cache;
use crate::utils::text::{normalize_for_matching, strip_edition_suffix};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;

// === STRUCTS ===

#[derive(Debug, Deserialize)]
pub struct NexusGame {
    pub id: i64,
    pub name: String,
    pub domain_name: String,
    pub genre: Option<String>,
    pub approved_date: Option<i64>,
}

/// Mod retornado por GET /v1/games/{domain}/mods/trending.json.
/// A resposta é um array JSON puro (sem envelope "data"), e não traz
/// mod_page_url — construímos a partir de domain_name + mod_id.
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

/// Extrai o domain_name da URL salva em `external_links["nexus"]`
/// (ex: "https://www.nexusmods.com/skyrimspecialedition" -> "skyrimspecialedition").
/// Evita duplicar o domínio em dois lugares — a URL já é a fonte da verdade.
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

/// Normaliza nome de jogo para matching com o catálogo do Nexus, com as mesmas regras de usadas em séries/tags.
fn normalize_title(name: &str) -> String {
    normalize_for_matching(&strip_edition_suffix(name))
}

pub fn find_best_nexus_match<'a>(
    game_name: &str,
    nexus_games: &'a [NexusGame],
) -> Option<&'a NexusGame> {
    let normalized_target = normalize_title(game_name);

    // 1. Match exato normalizado primeiro
    if let Some(exact) = nexus_games
        .iter()
        .find(|g| normalize_title(&g.name) == normalized_target)
    {
        return Some(exact);
    }

    // 2. Fallback fuzzy — só aceita acima de um limiar alto
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

// === CACHE ===

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
