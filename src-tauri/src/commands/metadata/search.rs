//! Comandos para busca de metadados externos
//!
//! Permite buscar detalhes de jogos, listas de tendências e giveaways
//! usando APIs externas como RAWG e GamerPower.

use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::models::Game;
use crate::providers::metadata::rawg;
use crate::providers::mods::nexus::TrendingMod;
use crate::services::cache;
use crate::services::integration::gamebrain;
use crate::services::integration::gamebrain::{GameMedia, SimilarGame};
use crate::services::integration::gamerpower::{self, Giveaway};
use crate::services::integration::hltb::{HltbClient, HltbEntry};
use crate::services::recommendation::core::calculate_game_weight;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
// === ESTRUTURAS ===

/// Jogo similar retornado para a seção de perfil em Trending.
/// Reutiliza SimilarGame do gamebrain, mas adiciona de qual jogo
/// âncora ele veio — útil para o frontend mostrar "Porque você jogou X".
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileSimilarGame {
    #[serde(flatten)]
    pub game: SimilarGame,
    /// Nome do jogo da biblioteca que originou esta sugestão.
    #[serde(rename = "becauseOf")]
    pub because_of: String,
}

#[derive(serde::Serialize)]
pub enum ModsAvailability {
    NoNexusMatch, // jogo não tem "nexus" em external_links
    Available,
}

#[derive(serde::Serialize)]
pub struct TrendingModsResult {
    pub availability: ModsAvailability,
    pub mods: Vec<TrendingMod>,
}

// === FUNÇÕES ===

/// Busca detalhes de um jogo na RAWG
#[tauri::command]
pub async fn fetch_game_details(
    app: AppHandle,
    query: String,
) -> Result<rawg::RawgGameDetails, AppError> {
    let api_key = database::get_secret(&app, "rawg_api_key")?;
    rawg::fetch_game_details(&api_key, query)
        .await
        .map_err(AppError::NetworkError)
}

/// Busca giveaways ativos na GamerPower
#[tauri::command]
pub async fn get_active_giveaways(app: AppHandle) -> Result<Vec<Giveaway>, AppError> {
    gamerpower::fetch_giveaways(&app)
        .await
        .map_err(AppError::NetworkError)
}

/// Busca jogos similares baseados nos 2 jogos com maior peso na biblioteca.
///
/// Fluxo:
///   1. Calcula peso de cada jogo via calculate_game_weight
///   2. Ordena desc, pega os 2 primeiros
///   3. Chama fetch_similar_games para cada um em paralelo (tokio::join!)
///   4. Filtra jogos que o usuário já possui (por nome, case-insensitive)
///   5. Deduplica por gamebrain_id
///   6. Retorna até 10 jogos (5 de cada âncora, intercalados)
#[tauri::command]
pub async fn get_profile_similar_games(
    app: AppHandle,
    // Jogos da biblioteca — frontend já tem, evita query extra ao banco
    user_games: Vec<Game>,
) -> Result<Vec<ProfileSimilarGame>, String> {
    if user_games.is_empty() {
        return Ok(vec![]);
    }

    // Etapa 1 e 2: pesos e top 2
    let mut weighted: Vec<(&Game, f32)> = user_games
        .iter()
        .map(|g| (g, calculate_game_weight(g)))
        .collect();

    weighted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let anchors: Vec<&Game> = weighted.iter().take(2).map(|(g, _)| *g).collect();

    // Etapa 3: chamadas em paralelo para os 2 âncoras
    let (result_a, result_b) = tokio::join!(
        gamebrain::fetch_similar_games(&app, &anchors[0].id, &anchors[0].name, Some(5)),
        // Se só tiver 1 jogo, usa o mesmo para não falhar
        gamebrain::fetch_similar_games(
            &app,
            &anchors.get(1).unwrap_or(&anchors[0]).id,
            &anchors.get(1).unwrap_or(&anchors[0]).name,
            Some(5),
        ),
    );

    // Normaliza nomes da biblioteca para filtro (lowercase, sem espaços extras)
    let library_names: std::collections::HashSet<String> = user_games
        .iter()
        .map(|g| g.name.trim().to_lowercase())
        .collect();

    // Etapa 4 e 5: filtra, deduplica e anota because_of
    let mut seen_ids = std::collections::HashSet::new();
    let mut results: Vec<ProfileSimilarGame> = Vec::new();

    let anchor_a_name = anchors[0].name.clone();
    let anchor_b_name = anchors.get(1).map(|g| g.name.clone()).unwrap_or_default();

    // Intercala A e B para variedade
    let games_a = result_a.unwrap_or_default();
    let games_b = result_b.unwrap_or_default();
    let max_len = games_a.len().max(games_b.len());

    for i in 0..max_len {
        for (games, because_of) in [(&games_a, &anchor_a_name), (&games_b, &anchor_b_name)] {
            if let Some(similar) = games.get(i) {
                // Filtra se já está na biblioteca
                if library_names.contains(&similar.name.trim().to_lowercase()) {
                    continue;
                }
                // Deduplica por ID
                if !seen_ids.insert(similar.id.clone()) {
                    continue;
                }
                results.push(ProfileSimilarGame {
                    game: similar.clone(),
                    because_of: because_of.clone(),
                });
            }
        }
    }

    Ok(results)
}

/// Busca jogos em tendência no momento na RAWG
#[tauri::command]
pub async fn get_trending_games(app: AppHandle) -> Result<Vec<rawg::RawgGame>, AppError> {
    let api_key = database::get_secret(&app, "rawg_api_key")?;
    rawg::fetch_trending_games(&app, &api_key)
        .await
        .map_err(AppError::NetworkError)
}

/// Busca jogos que serão lançados em breve na RAWG
#[tauri::command]
pub async fn get_upcoming_games(app: AppHandle) -> Result<Vec<rawg::RawgGame>, AppError> {
    let api_key = database::get_secret(&app, "rawg_api_key")?;
    rawg::fetch_upcoming_games(&app, &api_key)
        .await
        .map_err(AppError::NetworkError)
}

/// Busca jogos similares usando a API do GameBrain
#[tauri::command]
pub async fn get_similar_games(
    app: AppHandle,
    game_id: String,
    game_name: String,
) -> Result<Vec<SimilarGame>, String> {
    gamebrain::fetch_similar_games(&app, &game_id, &game_name, Some(12)).await
}

/// Busca midia de jogos (screenshots, trailers) usando a API do GameBrain
#[tauri::command]
pub async fn get_game_media(
    app: AppHandle,
    game_id: String,
    game_name: String,
) -> Result<GameMedia, String> {
    gamebrain::fetch_game_media(&app, &game_id, &game_name).await
}

/// Busca dados do HLTB para um jogo
#[tauri::command]
pub async fn search_hltb(app: AppHandle, game_name: String) -> Result<Vec<HltbEntry>, String> {
    let state = app.state::<AppState>();
    let cache_key = format!("search_hltb_{}", game_name.to_lowercase());

    if let Ok(cache_conn) = state.cache_db.lock() {
        if let Some(cached) = cache::get_cached_api_data(&cache_conn, "hltb", &cache_key) {
            if let Ok(entries) = serde_json::from_str::<Vec<HltbEntry>>(&cached) {
                return Ok(entries);
            }
        }
    }

    let client = HltbClient::new();
    let results = client.search(&game_name).await.map_err(|e| e.to_string())?;

    if let Ok(cache_conn) = state.cache_db.lock() {
        if let Ok(json) = serde_json::to_string(&results) {
            let _ = cache::save_cached_api_data(&cache_conn, "hltb", &cache_key, &json);
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_trending_mods(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: String,
) -> Result<TrendingModsResult, AppError> {
    let nexus_url: Option<String> = {
        let conn = state.games_db.lock()?;
        conn.query_row(
            "SELECT json_extract(external_links, '$.nexus') FROM game_details WHERE game_id = ?1",
            params![game_id],
            |row| row.get(0),
        )
            .optional()?
            .flatten()
    };

    let Some(url) = nexus_url.filter(|u| !u.is_empty()) else {
        return Ok(TrendingModsResult {
            availability: ModsAvailability::NoNexusMatch,
            mods: vec![],
        });
    };

    let domain = crate::providers::mods::nexus::extract_domain_from_nexus_url(&url)
        .ok_or_else(|| AppError::ValidationError("URL da Nexus inválida".into()))?;

    let api_key = database::get_secret(&app, "nexus_api_key")?;

    // Verifica cache antes de fazer await
    let cache_key = format!("trending_mods_{domain}");
    let cached_mods = {
        if let Ok(cache_conn) = state.cache_db.lock() {
            if let Some(cached) = cache::get_cached_api_data(&cache_conn, "nexus", &cache_key) {
                serde_json::from_str::<Vec<TrendingMod>>(&cached).ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(mods) = cached_mods {
        return Ok(TrendingModsResult {
            availability: ModsAvailability::Available,
            mods,
        });
    }

    // Faz o fetch sem manter o guard
    let mods = crate::providers::mods::nexus::fetch_trending_mods(&api_key, domain)
        .await
        .map_err(|e| AppError::ExternalApiError(e.to_string()))?;

    // Salva no cache
    if let Ok(cache_conn) = state.cache_db.lock() {
        if let Ok(json) = serde_json::to_string(&mods) {
            let _ = cache::save_cached_api_data(&cache_conn, "nexus", &cache_key, &json);
        }
    }

    Ok(TrendingModsResult {
        availability: ModsAvailability::Available,
        mods,
    })
}
