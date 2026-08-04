//! Módulo compartilhado para enriquecimento de metadados
//!
//! Contém estruturas e funções reutilizadas por enrichment e covers.

use crate::constants::NOT_FOUND_MARKER;
use crate::database;
use crate::models::ImportConfidence;
use crate::services::cache;
use crate::services::integration::{rawg, steam_api};
use crate::utils::text::{is_likely_non_base_game, normalize_for_matching, strip_edition_suffix};
use rusqlite::params;
use tracing::warn;

// === ESTRUTURAS COMPARTILHADAS ===

/// Progresso de enriquecimento de metadados
#[derive(serde::Serialize, Clone)]
pub struct EnrichProgress {
    pub current: i32,
    pub total_found: i32,
    pub last_game: String,
    pub status: String,
    pub platform: Option<String>,
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

/// Estrutura intermediária de metadados dos jogos
pub struct ProcessedGameDetails {
    pub game_id: String,
    pub description_raw: Option<String>,
    pub description_ptbr: Option<String>,
    pub release_date: Option<String>,
    pub genres: String,
    pub tags: Vec<crate::models::GameTag>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub critic_score: Option<i32>,
    pub background_image: Option<String>,
    pub series: Option<String>,
    pub steam_review_label: Option<String>,
    pub steam_review_count: Option<i32>,
    pub steam_review_score: Option<f32>,
    pub steam_review_updated_at: Option<String>,
    pub esrb_rating: Option<String>,
    pub is_adult: bool,
    pub adult_tags: Option<String>,
    pub external_links: Option<String>,
    pub steam_app_id: Option<String>,
    pub median_playtime: Option<i32>,
    pub estimated_playtime: Option<f32>,
    pub alternative_names: Option<Vec<String>>,
    pub updated_at: Option<String>,
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

async fn fetch_rawg_metadata_inner(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
    bypass_cache: bool,
) -> Option<rawg::RawgGameDetails> {
    let cache_key = rawg_cache_key(name);

    if !bypass_cache {
        if let Some(cached) = cache::get_cached_api_data(cache_conn, "rawg", &cache_key) {
            if cached == NOT_FOUND_MARKER {
                return None;
            }
            if let Ok(details) = serde_json::from_str::<rawg::RawgGameDetails>(&cached) {
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

// === FUNÇÕES COMPARTILHADAS - RAWG ===

/// Busca metadados RAWG com cache
///
/// Esta função é compartilhada entre enrichment e covers para buscar informações de jogos na API
/// RAWG com suporte a cache SQLite.
pub async fn fetch_rawg_metadata(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<rawg::RawgGameDetails> {
    fetch_rawg_metadata_inner(api_key, name, cache_conn, false).await
}

/// Variante que ignora o cache e sempre consulta a RAWG ao vivo.
///
/// Usada pelo comando `fill_missing_metadata` para garantir que dados possivelmente atualizados na
/// RAWG sejam buscados mesmo para jogos cujo cache ainda é válido.
pub async fn fetch_rawg_metadata_fresh(
    api_key: &str,
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<rawg::RawgGameDetails> {
    fetch_rawg_metadata_inner(api_key, name, cache_conn, true).await
}

// === FUNÇÕES COMPARTILHADAS - STEAM ===

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

// === PERSISTÊNCIA ===

/// Salva detalhes do jogo no banco. Aceita tanto Connection quanto Transaction (via Deref trait)
pub fn save_game_details<C>(conn: &C, d: ProcessedGameDetails) -> Result<(), rusqlite::Error>
where
    C: std::ops::Deref<Target=rusqlite::Connection>,
{
    let tags_json = database::serialize_tags(&d.tags).unwrap_or_else(|_| "[]".to_string());

    // Garante que a linha existe antes do UPDATE (para jogos que já têm
    // description_raw da Legacy Games, o INSERT OR IGNORE preserva o valor).
    conn.execute(
        "INSERT OR IGNORE INTO game_details (game_id) VALUES (?1)",
        params![d.game_id],
    )?;

    // Atualiza todos os campos usando COALESCE nos campos de texto para nunca
    // sobrescrever um valor existente com NULL vindo da RAWG.
    conn.execute(
        "UPDATE game_details SET
            description_raw     = COALESCE(?2,  description_raw),
            description_ptbr    = COALESCE(?3,  description_ptbr),
            release_date        = COALESCE(?4,  release_date),
            genres              = COALESCE(NULLIF(?5, ''), genres),
            tags                = COALESCE(NULLIF(?6, '[]'), tags),
            developer           = COALESCE(?7,  developer),
            publisher           = COALESCE(?8,  publisher),
            critic_score        = COALESCE(?9,  critic_score),
            background_image    = COALESCE(?10, background_image),
            series              = COALESCE(?11, series),
            steam_review_label  = COALESCE(?12, steam_review_label),
            steam_review_count  = COALESCE(?13, steam_review_count),
            steam_review_score  = COALESCE(?14, steam_review_score),
            steam_review_updated_at = COALESCE(?15, steam_review_updated_at),
            esrb_rating         = COALESCE(?16, esrb_rating),
            is_adult            = ?17,
            adult_tags          = COALESCE(?18, adult_tags),
            external_links      = COALESCE(?19, external_links),
            steam_app_id        = COALESCE(?20, steam_app_id),
            median_playtime     = COALESCE(?21, median_playtime),
            estimated_playtime  = COALESCE(?22, estimated_playtime),
            updated_at          = ?23
         WHERE game_id = ?1",
        params![
            d.game_id,
            d.description_raw,
            d.description_ptbr,
            d.release_date,
            d.genres,
            tags_json,
            d.developer,
            d.publisher,
            d.critic_score,
            d.background_image.clone(),
            d.series,
            d.steam_review_label,
            d.steam_review_count,
            d.steam_review_score,
            d.steam_review_updated_at,
            d.esrb_rating,
            d.is_adult,
            d.adult_tags,
            d.external_links,
            d.steam_app_id,
            d.median_playtime,
            d.estimated_playtime,
            d.updated_at
        ],
    )?;

    if let Some(img) = d.background_image {
        conn.execute(
            "UPDATE games SET cover_url = ?1 WHERE id = ?2 AND (cover_url IS NULL OR cover_url = '')",
            params![img, d.game_id],
        )?;
    }

    if let Some(alt_names) = d.alternative_names {
        if !alt_names.is_empty() {
            let alt_json = serde_json::to_string(&alt_names).unwrap_or_default();
            conn.execute(
                "UPDATE games SET alternative_names = COALESCE(?1, alternative_names) WHERE id = ?2",
                params![alt_json, d.game_id],
            )?;
        }
    }

    Ok(())
}
