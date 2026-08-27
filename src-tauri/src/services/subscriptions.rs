//! Modulo para buscar games de serviços de assinatura e gerenciar quais serviços o usuário assina.

use crate::constants::{
    AMAZON_LUNA_CACHE_KEY, AMAZON_LUNA_CACHE_SOURCE, EA_PLAY_CACHE_KEY, EA_PLAY_CACHE_SOURCE,
    GAME_PASS_CACHE_SOURCE, GAME_PASS_FULL_CACHE_KEY, UBISOFT_PLUS_CACHE_KEY,
    UBISOFT_PLUS_CACHE_SOURCE,
};
use crate::database::cache;
use crate::database::AppState;
use crate::providers::subscriptions::amazon_luna::{fetch_amazon_luna_catalog, LunaGame};
use crate::providers::subscriptions::game_pass::{fetch_game_pass_pc_catalog, GamePassGame};
use crate::providers::subscriptions::ubisoft_plus::{fetch_ubisoft_plus_catalog, UbisoftGame};
use crate::providers::subscriptions::{fetch_ea_play_catalog, EAPlayGame};
use rusqlite::params;
use tauri::State;

/// Normaliza tags de idioma para formato padrão (ex: pt -> pt-BR, en -> en-US)
fn normalize_lang_tag(lang: &str) -> String {
    let lower = lang.to_lowercase();
    if lower.starts_with("pt") {
        "pt-BR".to_string()
    } else {
        "en-US".to_string() // fallback
    }
}

/// Mapeia idioma para o header prime-gaming-language (Amazon Luna)
fn amazon_luna_lang_tag(lang: &str) -> String {
    let normalized = normalize_lang_tag(lang);
    if normalized.starts_with("pt") {
        "pt-PT".to_string() // Amazon Luna usa pt-PT, não pt-BR
    } else {
        "en-US".to_string()
    }
}

/// Constrói chave de cache specific a idioma
fn catalog_cache_key(base_key: &str, lang: &str) -> String {
    let normalized = normalize_lang_tag(lang);
    format!("{}_{}", base_key, normalized)
}

/// Retorna catálogo do Amazon Luna (do cache ou scraping)
pub async fn get_amazon_luna_games(
    state: &State<'_, AppState>,
    lang: &str,
) -> Result<Vec<LunaGame>, String> {
    let _normalized_lang = normalize_lang_tag(lang);
    let cache_key = catalog_cache_key(AMAZON_LUNA_CACHE_KEY, lang);
    let luna_lang = amazon_luna_lang_tag(lang);

    let cached = {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::get_cached_api_data(&conn, AMAZON_LUNA_CACHE_SOURCE, &cache_key)
    };

    if let Some(cached) = cached {
        if let Ok(games) = serde_json::from_str::<Vec<LunaGame>>(&cached) {
            return Ok(games);
        }
    }

    let games = fetch_amazon_luna_catalog(&luna_lang).await?;

    {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        let payload = serde_json::to_string(&games).map_err(|e| e.to_string())?;
        cache::save_cached_api_data(&conn, AMAZON_LUNA_CACHE_SOURCE, &cache_key, &payload)?;
    }

    Ok(games)
}

/// Retorna catálogo do Game Pass PC (do cache ou scraping).
/// Quando `exclude_ea_play = true`, remove jogos EA Play do resultado.
pub async fn get_game_pass_games(
    state: &State<'_, AppState>,
    exclude_ea_play: bool,
    lang: &str,
) -> Result<Vec<GamePassGame>, String> {
    let normalized_lang = normalize_lang_tag(lang);
    let cache_key = catalog_cache_key(GAME_PASS_FULL_CACHE_KEY, lang);

    // Tenta cache primeiro — sempre armazena o catálogo COMPLETO
    let cached = {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::get_cached_api_data(&conn, GAME_PASS_CACHE_SOURCE, &cache_key)
    };

    let all_games: Vec<GamePassGame> = if let Some(data) = cached {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        // Cache miss — busca catálogo completo (`exclude_ea_play = false`).
        let games = fetch_game_pass_pc_catalog(false, &normalized_lang).await?;
        let payload = serde_json::to_string(&games).map_err(|e| e.to_string())?;
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::save_cached_api_data(&conn, GAME_PASS_CACHE_SOURCE, &cache_key, &payload)?;
        games
    };

    // O filtro final fica na camada de serviço para reaproveitar o mesmo cache.
    if exclude_ea_play {
        Ok(all_games.into_iter().filter(|g| !g.is_ea_play).collect())
    } else {
        Ok(all_games)
    }
}

/// Retorna catálogo do EA Play
pub async fn get_ea_play_games(
    state: &State<'_, AppState>,
    lang: &str,
) -> Result<Vec<EAPlayGame>, String> {
    let normalized_lang = normalize_lang_tag(lang);
    let cache_key = catalog_cache_key(EA_PLAY_CACHE_KEY, lang);

    let cached = {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::get_cached_api_data(&conn, EA_PLAY_CACHE_SOURCE, &cache_key)
    };

    let all_games: Vec<EAPlayGame> = if let Some(data) = cached {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let games = fetch_ea_play_catalog(&normalized_lang).await?;
        let payload = serde_json::to_string(&games).map_err(|e| e.to_string())?;
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::save_cached_api_data(&conn, EA_PLAY_CACHE_SOURCE, &cache_key, &payload)?;
        games
    };

    Ok(all_games)
}

/// Retorna catálogo do Ubisoft+ (do cache ou scraping)
pub async fn get_ubisoft_plus_games(
    state: &State<'_, AppState>,
) -> Result<Vec<UbisoftGame>, String> {
    let cached = {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        cache::get_cached_api_data(&conn, UBISOFT_PLUS_CACHE_SOURCE, UBISOFT_PLUS_CACHE_KEY)
    };

    if let Some(cached) = cached {
        if let Ok(games) = serde_json::from_str::<Vec<UbisoftGame>>(&cached) {
            return Ok(games);
        }
    }

    let games = fetch_ubisoft_plus_catalog().await?;

    {
        let conn = state.cache_db.lock().map_err(|e| e.to_string())?;
        let payload = serde_json::to_string(&games).map_err(|e| e.to_string())?;
        cache::save_cached_api_data(
            &conn,
            UBISOFT_PLUS_CACHE_SOURCE,
            UBISOFT_PLUS_CACHE_KEY,
            &payload,
        )?;
    }

    Ok(games)
}

/// Obtém a lista de serviços assinados pelo usuário (Amazon Luna, Game Pass, EA Play, Ubisoft+)
pub fn get_enabled_services(state: &State<'_, AppState>) -> Result<Vec<String>, String> {
    let conn = state.games_db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT service FROM subscriptions WHERE enabled = 1")
        .map_err(|e| e.to_string())?;

    let services = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(services)
}

/// Salva quais serviços o usuário assina
pub fn set_enabled_services(
    state: &State<'_, AppState>,
    services: Vec<String>,
) -> Result<(), String> {
    let conn = state.games_db.lock().map_err(|e| e.to_string())?;

    // Reseta todos para disabled
    conn.execute("UPDATE subscriptions SET enabled = 0", [])
        .map_err(|e| e.to_string())?;

    // Habilita os selecionados (upsert)
    for service in services {
        conn.execute(
            "INSERT INTO subscriptions (service, enabled)
             VALUES (?1, 1)
             ON CONFLICT(service) DO UPDATE SET enabled = 1",
            params![service],
        )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
