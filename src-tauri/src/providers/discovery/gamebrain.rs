//! Integracao com GameBrain API.
//!
//! Responsavel por busca semantica de jogos, descoberta e recomendacoes.
//!
//! Atualmente implementa:
//! - Busca por caracteristicas/texto
//! - Filtros (plataforma, genero, modo de jogo, preco, etc.)

use crate::constants::GAMEBRAIN_SIMILAR_REQUEST_LIMIT;
use crate::database;
use crate::integrations::gamebrain::cache::{
    gamebrain_similar_cache_key, read_cached_json, read_stale_similar_games, save_cached_json,
    take_similar_limit,
};
use crate::integrations::gamebrain::client::GameBrainClient;
pub(crate) use crate::integrations::gamebrain::models::{
    GameBrainFilter, GameBrainFilterValue, GameBrainSearchParams, GameBrainSearchResult,
    SimilarGame,
};
use crate::integrations::gamebrain::raw::{RawSearchResponse, RawSimilarResponse};
use crate::providers::translation::gemini;
use crate::utils::http_client::HTTP_CLIENT;
use std::collections::HashSet;
use tauri::AppHandle;

/// Busca jogos similares ao jogo identificado pelo UUID interno do Playlite.
pub async fn fetch_similar_games(
    app: &AppHandle,
    playlite_game_id: &str,
    game_name: &str,
    limit: Option<u32>,
) -> Result<Vec<SimilarGame>, String> {
    let api_key = GameBrainClient::get_api_key(app)?;
    let requested_limit = limit.unwrap_or(10);

    // Etapa 1: Resolver o gamebrain_id (Usa o cache compartilhado com a aba Mídia automaticamente!)
    let gamebrain_id = GameBrainClient::resolve_id(app, playlite_game_id, game_name).await?;

    // Etapa 2: Buscar similares usando o gamebrain_id
    let similar_cache_key = gamebrain_similar_cache_key(gamebrain_id);

    if let Some(cached_results) =
        read_cached_json::<Vec<SimilarGame>>(app, "gamebrain", &similar_cache_key, false)?
    {
        tracing::debug!(
            "GameBrain similar cache hit => gamebrain_id={}",
            gamebrain_id
        );
        return Ok(take_similar_limit(cached_results, requested_limit));
    }

    // Cache miss: chama a API
    tracing::debug!(
        "GameBrain similar cache miss => chamando /similar para gamebrain_id={}",
        gamebrain_id
    );

    let url = format!("https://api.gamebrain.co/v1/games/{}/similar", gamebrain_id);
    let mut request = HTTP_CLIENT.get(&url).header("x-api-key", &api_key);
    request = request.query(&[("limit", GAMEBRAIN_SIMILAR_REQUEST_LIMIT.to_string())]);

    let response = match request.send().await {
        Ok(r) => r,
        Err(err) => {
            tracing::error!("GameBrain similar request error: {}", err);
            if let Some(cached_results) =
                read_stale_similar_games(app, &similar_cache_key, gamebrain_id, requested_limit)?
            {
                return Ok(cached_results);
            }
            return Err(err.to_string());
        }
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "GameBrain similar HTTP Error => status={} body={}",
            status,
            body
        );

        if let Some(cached_results) =
            read_stale_similar_games(app, &similar_cache_key, gamebrain_id, requested_limit)?
        {
            return Ok(cached_results);
        }
        return Err(format!("Erro GameBrain similar: {}", status));
    }

    let text = match response.text().await {
        Ok(text) => text,
        Err(err) => {
            if let Some(cached_results) =
                read_stale_similar_games(app, &similar_cache_key, gamebrain_id, requested_limit)?
            {
                return Ok(cached_results);
            }
            return Err(err.to_string());
        }
    };

    let raw: RawSimilarResponse =
        match serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&text)) {
            Ok(raw) => raw,
            Err(e) => {
                tracing::error!(
                    "GameBrain similar JSON parse error em '{}': {}",
                    e.path(),
                    e.inner()
                );
                if let Some(cached_results) = read_stale_similar_games(
                    app,
                    &similar_cache_key,
                    gamebrain_id,
                    requested_limit,
                )? {
                    return Ok(cached_results);
                }
                return Err(format!("Erro JSON GameBrain similar: {}", e));
            }
        };

    let results: Vec<SimilarGame> = raw
        .results
        .into_iter()
        .filter_map(|g| {
            // Usa o helper do Client para converter o ID
            let raw_id = GameBrainClient::parse_gamebrain_id(&g.id)?;
            let name = g.name?;

            Some(SimilarGame {
                id: format!("gamebrain:{}", raw_id),
                name,
                cover_url: g.image,
                genre: g.genre,
                year: g.year.map(|y| y as u32),
                rating: g.rating.and_then(|r| r.mean).map(|m| (m * 100.0).round()),
                link: g.link,
                screenshots: g.screenshots.unwrap_or_default(), // Tratamento do Option que criamos
                micro_trailer: g.micro_trailer,
                adult_only: g.adult_only,
            })
        })
        .collect();

    let _ = save_cached_json(app, "gamebrain", &similar_cache_key, &results);

    Ok(take_similar_limit(results, requested_limit))
}

/// Busca jogos por descrição/características aplicando os filtros fornecidos.
pub async fn search_games_by_features(
    app: &AppHandle,
    query: &str,
    params: GameBrainSearchParams,
) -> Result<Vec<GameBrainSearchResult>, String> {
    let api_key = GameBrainClient::get_api_key(app)?;
    let cleaned_query = query.trim();

    if cleaned_query.is_empty() {
        return Ok(vec![]);
    }

    let english_query = match database::get_secret(app, "gemini_api_key") {
        Ok(gemini_key) if !gemini_key.trim().is_empty() => {
            gemini::translate_query_to_english(&gemini_key, cleaned_query)
                .await
                .unwrap_or_else(|_| cleaned_query.to_string())
        }
        _ => cleaned_query.to_string(),
    };

    let url = "https://api.gamebrain.co/v1/games";
    let mut request = HTTP_CLIENT
        .get(url)
        .header("x-api-key", api_key)
        .query(&[("query", english_query.as_str())]);

    if !params.filters.is_empty() {
        let filters_json = serde_json::to_string(&params.filters).map_err(|e| e.to_string())?;
        request = request.query(&[("filters", filters_json)]);
    }
    if let Some(sort) = &params.sort {
        request = request.query(&[("sort", sort.as_str())]);
    }
    if let Some(order) = &params.sort_order {
        request = request.query(&[("sort-order", order.as_str())]);
    }
    if let Some(limit) = params.limit {
        request = request.query(&[("limit", limit.to_string())]);
    }
    if let Some(offset) = params.offset {
        request = request.query(&[("offset", offset.to_string())]);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("Erro GameBrain: {}", status));
    }

    let response_text = response.text().await.map_err(|e| e.to_string())?;
    let raw_response: RawSearchResponse =
        serde_json::from_str(&response_text).map_err(|e| format!("Erro JSON GameBrain: {}", e))?;

    let results: Vec<GameBrainSearchResult> = raw_response
        .results
        .into_iter()
        .map(|game| {
            let cover_url = game
                .image
                .or_else(|| game.cover.filter(|c| !c.url.is_empty()).map(|c| c.url));
            let platforms: Vec<String> = {
                let mut seen = HashSet::new();
                game.platforms
                    .into_iter()
                    .filter(|p| !p.name.is_empty() && seen.insert(p.name.clone()))
                    .map(|p| p.name)
                    .collect()
            };

            GameBrainSearchResult {
                id: format!("gamebrain:{}", game.id),
                name: game.name,
                cover_url,
                genre: game.genre,
                year: game.year.map(|y| y as u32),
                rating: game
                    .rating
                    .and_then(|r| r.mean)
                    .map(|m| (m * 100.0).round()),
                platforms,
                link: game.link,
            }
        })
        .collect();

    Ok(results)
}

/// Atalho para focar apenas em jogos de PC (adiciona filtro `platform: pc`).
pub async fn search_pc_games_by_features(
    app: &AppHandle,
    query: &str,
    mut params: GameBrainSearchParams,
) -> Result<Vec<GameBrainSearchResult>, String> {
    if !params.filters.iter().any(|f| f.key == "platform") {
        params.filters.push(GameBrainFilter {
            key: "platform".into(),
            values: vec![GameBrainFilterValue { value: "pc".into() }],
            connection: Some("OR".into()),
        });
    }
    search_games_by_features(app, query, params).await
}
