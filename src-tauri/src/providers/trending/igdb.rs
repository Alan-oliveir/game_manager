//! Jogos com lançamento futuro e em alta via IGDB.

use crate::database::AppState;
use crate::providers::metadata::igdb::client::igdb_request;
use crate::services::cache;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

const POPULARITY_TYPE_CACHE_KEY: &str = "igdb_popularity_type_steam_top_sellers";
const POPULARITY_TYPE_NAME: &str = "Global Top Sellers";

// === STRUCTS ===

#[derive(Debug, Deserialize)]
struct PopularityTypeRaw {
    id: i64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct PopularityPrimitiveRaw {
    game_id: i64,
}

#[derive(Debug, Deserialize)]
struct IgdbNamed {
    name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IgdbCover {
    pub(crate) image_id: String,
}

#[derive(Debug, Deserialize)]
struct KnownFreeToPlayEntry {
    #[allow(dead_code)]
    name: String,
    igdb_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IgdbTrendingRaw {
    id: i64,
    name: String,
    slug: String,
    cover: Option<IgdbCover>,
    #[serde(default)]
    genres: Vec<IgdbNamed>,
    #[serde(default)]
    collections: Vec<IgdbNamed>,
}

#[derive(Debug, Deserialize)]
struct IgdbUpcomingRaw {
    name: String,
    slug: String,
    first_release_date: Option<i64>,
    cover: Option<IgdbCover>,
    #[serde(default)]
    genres: Vec<IgdbNamed>,
    #[serde(default)]
    collections: Vec<IgdbNamed>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendingGame {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub cover_url: Option<String>,
    pub genres: Vec<String>,
    pub series: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingGame {
    pub name: String,
    pub slug: String,
    pub release_date: Option<String>,
    pub cover_url: Option<String>,
    pub genres: Vec<String>,
    pub series: Option<Vec<String>>,
}

// === HELPERS ===

fn cover_url(cover: Option<IgdbCover>) -> Option<String> {
    cover.map(|c| {
        format!(
            "https://images.igdb.com/igdb/image/upload/t_1080p/{}.jpg",
            c.image_id
        )
    })
}

fn genre_names(genres: Vec<IgdbNamed>) -> Vec<String> {
    genres.into_iter().map(|g| g.name).collect()
}

fn collection_names(collections: Vec<IgdbNamed>) -> Option<Vec<String>> {
    let names: Vec<String> = collections.into_iter().map(|c| c.name).collect();
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

fn save_cached<T>(app: &AppHandle, table: &str, cache_key: &str, data: &T)
where
    T: Serialize,
{
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Ok(json) = serde_json::to_string(data) {
            let _ = cache::save_cached_api_data(&conn, table, cache_key, &json);
        }
    }
}

/// Um fallback genérico para pegar dados do cache caso a API falhe.
fn fallback_stale<T>(
    app: &AppHandle,
    table: &str,
    cache_key: &str,
    err: String,
) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(payload) = cache::get_stale_api_data(&conn, table, cache_key) {
            if let Ok(games) = serde_json::from_str::<Vec<T>>(&payload) {
                return Ok(games);
            }
        }
    }
    Err(err)
}

// === FREE TO PLAY FILTER ===

static KNOWN_FREE_TO_PLAY: Lazy<Vec<KnownFreeToPlayEntry>> = Lazy::new(|| {
    let raw = include_str!("../../data/known_free_to_play_games.json");
    serde_json::from_str(raw).unwrap_or_else(|e| {
        eprintln!("Falha ao carregar known_free_to_play_games.json: {e}");
        Vec::new()
    })
});

fn is_known_free_to_play(slug: &str) -> bool {
    KNOWN_FREE_TO_PLAY.iter().any(|entry| {
        entry.igdb_slug.as_deref() == Some(slug)
    })
}

// === TRENDING ===

async fn resolve_popularity_type_id(app: &AppHandle) -> Result<i64, String> {
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(cached) =
            cache::get_cached_api_data(&conn, "igdb_popularity_type", POPULARITY_TYPE_CACHE_KEY)
        {
            if let Ok(id) = cached.parse::<i64>() {
                return Ok(id);
            }
        }
    }

    let body = igdb_request(app, "popularity_types", "fields id, name; limit 50;").await?;
    let types: Vec<PopularityTypeRaw> = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    let found = types
        .into_iter()
        .find(|t| t.name == POPULARITY_TYPE_NAME)
        .ok_or_else(|| format!("IGDB: popularity_type '{POPULARITY_TYPE_NAME}' não encontrado"))?;

    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        let _ = cache::save_cached_api_data(
            &conn,
            "igdb_popularity_type",
            POPULARITY_TYPE_CACHE_KEY,
            &found.id.to_string(),
        );
    }

    Ok(found.id)
}

pub async fn fetch_trending_games(app: &AppHandle) -> Result<Vec<TrendingGame>, String> {
    let table = "steam_trending";
    let cache_key = "steam_trending";

    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(cached) = cache::get_cached_api_data(&conn, table, cache_key) {
            if let Ok(games) = serde_json::from_str::<Vec<TrendingGame>>(&cached) {
                return Ok(games);
            }
        }
    }

    match fetch_trending_inner(app).await {
        Ok(games) => {
            save_cached(app, table, cache_key, &games);
            Ok(games)
        }
        Err(e) => fallback_stale(app, table, cache_key, e),
    }
}

pub async fn fetch_trending_inner(app: &AppHandle) -> Result<Vec<TrendingGame>, String> {
    let popularity_type_id = resolve_popularity_type_id(app).await?;

    let primitives_query = format!(
        "fields game_id; sort value desc; limit 50; where popularity_type = {popularity_type_id};"
    );
    let body = igdb_request(app, "popularity_primitives", &primitives_query).await?;
    let primitives: Vec<PopularityPrimitiveRaw> =
        serde_json::from_str(&body).map_err(|e| e.to_string())?;

    if primitives.is_empty() {
        return Ok(Vec::new());
    }

    let ordered_ids: Vec<i64> = primitives.iter().map(|p| p.game_id).collect();
    let ids_csv = ordered_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");

    // platforms = (6) -> PC (Microsoft Windows); exclui exclusivos de console.
    let games_query = format!(
        "fields name, slug, cover.image_id, genres.name, collections.name, external_games.url; \
         where id = ({ids_csv}) & game_type = 0 & platforms = (6); limit 30;"
    );
    let games_body = igdb_request(app, "games", &games_query).await?;
    let games_raw: Vec<IgdbTrendingRaw> =
        serde_json::from_str(&games_body).map_err(|e| e.to_string())?;

    let mut by_id: HashMap<i64, IgdbTrendingRaw> =
        games_raw.into_iter().map(|g| (g.id, g)).collect();

    let games: Vec<TrendingGame> = ordered_ids
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .map(|g| TrendingGame {
            id: g.id,
            name: g.name,
            slug: g.slug,
            cover_url: cover_url(g.cover),
            genres: genre_names(g.genres),
            series: collection_names(g.collections),
        })
        .filter(|g| !is_known_free_to_play(&g.slug))
        .take(15)
        .collect();

    Ok(games)
}

// === UPCOMING ===

pub async fn fetch_upcoming_games(app: &AppHandle) -> Result<Vec<UpcomingGame>, String> {
    let table = "igdb_upcoming";
    let cache_key = "igdb_upcoming";

    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(cached) = cache::get_cached_api_data(&conn, table, cache_key) {
            if let Ok(games) = serde_json::from_str::<Vec<UpcomingGame>>(&cached) {
                return Ok(games);
            }
        }
    }

    let now = chrono::Utc::now().timestamp();
    let next_year = now + 60 * 60 * 24 * 365;

    let query = format!(
        "fields name, slug, first_release_date, cover.image_id, genres.name, collections.name, hypes; \
        where first_release_date > {now} & first_release_date < {next_year} & game_type = 0 & platforms = (6); \
        sort hypes desc; limit 20;"
    );

    let body = match igdb_request(app, "games", &query).await {
        Ok(b) => b,
        Err(e) => return fallback_stale(app, table, cache_key, e),
    };

    let raw: Vec<IgdbUpcomingRaw> = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => return fallback_stale(app, table, cache_key, e.to_string()),
    };

    let games: Vec<UpcomingGame> = raw
        .into_iter()
        .map(|g| UpcomingGame {
            name: g.name,
            slug: g.slug,
            release_date: g.first_release_date.and_then(|ts| {
                chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
            }),
            cover_url: cover_url(g.cover),
            genres: genre_names(g.genres),
            series: collection_names(g.collections),
        })
        .collect();

    save_cached(app, table, cache_key, &games);

    Ok(games)
}
