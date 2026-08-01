//! Módulo compartilhado para enriquecimento de metadados
//!
//! Contém estruturas e funções reutilizadas por enrichment e covers.

use crate::constants::NOT_FOUND_MARKER;
use crate::models::ImportConfidence;
use crate::services::cache;
use crate::services::integration::{rawg, steam_api};
use crate::utils::text::{is_likely_non_base_game, normalize_for_matching, strip_edition_suffix};
use tracing::warn;

// === ESTRUTURAS COMPARTILHADAS ===

/// Progresso de enriquecimento de metadados
#[derive(serde::Serialize, Clone)]
pub struct EnrichProgress {
    pub current: i32,
    pub total_found: i32,
    pub last_game: String,
    pub status: String,
}

/// Payload do evento `enrich_complete`.
///
/// `platform` é `Some` quando o enrichment é escopado a uma importação
/// específica (`enrich_newly_imported`), e `None` quando é uma varredura
/// geral da biblioteca (`update_metadata`, `fill_missing_metadata`).
#[derive(serde::Serialize, Clone)]
pub struct EnrichCompletePayload {
    pub platform: Option<String>,
    pub message: String,
}

/// Resultado da resolução de `steam_app_id` a partir do nome de um jogo.
pub struct SteamIdResolution {
    pub app_id: String,
    pub confidence: ImportConfidence,
}

// === HELPERS LOCAIS ===

fn rawg_cache_key(name: &str) -> String {
    format!("search_{}", name.to_lowercase())
}

pub(in crate::commands::metadata) fn rawg_not_found_cached(
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> bool {
    let cache_key = rawg_cache_key(name);
    cache::get_cached_api_data(cache_conn, "rawg", &cache_key)
        .is_some_and(|cached| cached == NOT_FOUND_MARKER)
}

async fn fetch_rawg_metadata_inner(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
    bypass_cache: bool,
) -> Option<rawg::GameDetails> {
    let cache_key = rawg_cache_key(name);

    if !bypass_cache {
        if let Some(cached) = cache::get_cached_api_data(cache_conn, "rawg", &cache_key) {
            if cached == NOT_FOUND_MARKER {
                return None;
            }
            if let Ok(details) = serde_json::from_str::<rawg::GameDetails>(&cached) {
                return Some(details);
            }
        }
    }

    let search_result = crate::services::rate_limiter::RAWG_LIMITER
        .run(|| rawg::search_games(api_key, name))
        .await;

    match search_result {
        Ok(results) => {
            if let Some(best_match) = results.first() {
                let details_result = crate::services::rate_limiter::RAWG_LIMITER
                    .run(|| rawg::fetch_game_details(api_key, best_match.id.to_string()))
                    .await;

                match details_result {
                    Ok(details) => {
                        if let Ok(json) = serde_json::to_string(&details) {
                            let _ =
                                cache::save_cached_api_data(cache_conn, "rawg", &cache_key, &json);
                        }
                        Some(details)
                    }
                    Err(err) => {
                        // Só marca como NOT_FOUND em 404 real — 429 já foi
                        // tentado via backoff e não deve ser confundido com ausência.
                        if err.contains("não encontrado") || err.contains("404") {
                            let _ = cache::save_cached_api_data(
                                cache_conn,
                                "rawg",
                                &cache_key,
                                NOT_FOUND_MARKER,
                            );
                        } else {
                            warn!(
                                "RAWG fetch_game_details falhou (não-404) para '{}': {}",
                                name, err
                            );
                        }
                        None
                    }
                }
            } else {
                let _ =
                    cache::save_cached_api_data(cache_conn, "rawg", &cache_key, NOT_FOUND_MARKER);
                None
            }
        }
        Err(e) => {
            warn!("RAWG search_games falhou para '{}': {}", name, e);
            None // erro de rede/429 esgotado — NÃO cacheia como not-found
        }
    }
}

// === FUNÇÕES COMPARTILHADAS ===

/// Busca metadados RAWG com cache
///
/// Esta função é compartilhada entre enrichment e covers
/// para buscar informações de jogos na API RAWG com suporte a cache SQLite.
pub async fn fetch_rawg_metadata(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<rawg::GameDetails> {
    fetch_rawg_metadata_inner(api_key, name, cache_conn, false).await
}

/// Variante que ignora o cache e sempre consulta a RAWG ao vivo.
///
/// Usada pelo comando `fill_missing_metadata` para garantir que dados
/// possivelmente atualizados na RAWG sejam buscados mesmo para jogos
/// cujo cache ainda é válido.
pub async fn fetch_rawg_metadata_fresh(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<rawg::GameDetails> {
    fetch_rawg_metadata_inner(api_key, name, cache_conn, true).await
}

pub async fn resolve_steam_app_id(
    name: &str,
    platform: &str,
    platform_game_id: Option<&str>,
    cache_conn: &rusqlite::Connection,
) -> Option<SteamIdResolution> {
    if platform.to_lowercase() == "steam" {
        if let Some(id) = platform_game_id {
            return Some(SteamIdResolution {
                app_id: id.to_string(),
                confidence: ImportConfidence::High,
            });
        }
    }

    let cache_key = format!("resolve_{}", normalize_for_matching(name));
    if cache::get_cached_api_data(cache_conn, "steam_resolve", &cache_key)
        .is_some_and(|v| v == NOT_FOUND_MARKER)
    {
        return None;
    }

    let candidates = match steam_api::search_app_by_name(name).await {
        Ok(c) => c,
        Err(e) => {
            warn!("search_app_by_name falhou para '{}': {}", name, e);
            return None;
        }
    };

    let target = normalize_for_matching(name);

    // 1. Match exato de nome normalizado → confiança alta
    let resolution = candidates
        .iter()
        .find(|item| normalize_for_matching(&item.name) == target)
        .map(|item| SteamIdResolution {
            app_id: item.id.to_string(),
            confidence: ImportConfidence::High,
        })
        // 2. Nome sem sufixo de edição, contra candidatos também sem sufixo
        .or_else(|| {
            let stripped_target = normalize_for_matching(&strip_edition_suffix(name));
            candidates
                .iter()
                .find(|item| {
                    normalize_for_matching(&strip_edition_suffix(&item.name)) == stripped_target
                })
                .map(|item| SteamIdResolution {
                    app_id: item.id.to_string(),
                    confidence: ImportConfidence::Medium,
                })
        })
        // 3. Sem match exato: pega o primeiro candidato que não pareça DLC/edição/trilha sonora.
        // Evita cair numa correlação claramente errada quando o nome divergir entre plataformas (subtítulo, edição regional etc.)
        .or_else(|| {
            candidates
                .iter()
                .find(|item| !is_likely_non_base_game(&item.name))
                .map(|item| SteamIdResolution {
                    app_id: item.id.to_string(),
                    confidence: ImportConfidence::Low,
                })
        });

    if resolution.is_none() {
        let _ =
            cache::save_cached_api_data(cache_conn, "steam_resolve", &cache_key, NOT_FOUND_MARKER);
    }

    resolution
}

/// Busca dados Steam Store com cache
pub(crate) async fn fetch_steam_store_data(
    steam_id: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<steam_api::SteamStoreData> {
    let cache_key = format!("store_{}", steam_id);

    if let Some(cached) = cache::get_cached_api_data(cache_conn, "steam", &cache_key) {
        if let Ok(data) = serde_json::from_str::<steam_api::SteamStoreData>(&cached) {
            return Some(data);
        }
    }

    match steam_api::get_app_details(steam_id).await {
        Ok(Some(data)) => {
            if let Ok(json) = serde_json::to_string(&data) {
                let _ = cache::save_cached_api_data(cache_conn, "steam", &cache_key, &json);
            }
            Some(data)
        }
        _ => None,
    }
}

/// Busca reviews Steam com cache
pub(crate) async fn fetch_steam_reviews(
    steam_id: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<steam_api::SteamReviewSummary> {
    let cache_key = format!("reviews_{}", steam_id);

    if let Some(cached) = cache::get_cached_api_data(cache_conn, "steam", &cache_key) {
        if let Ok(reviews) = serde_json::from_str::<steam_api::SteamReviewSummary>(&cached) {
            return Some(reviews);
        }
    }

    match steam_api::get_app_reviews(steam_id).await {
        Ok(Some(reviews)) => {
            if let Ok(json) = serde_json::to_string(&reviews) {
                let _ = cache::save_cached_api_data(cache_conn, "steam", &cache_key, &json);
            }
            Some(reviews)
        }
        _ => None,
    }
}
