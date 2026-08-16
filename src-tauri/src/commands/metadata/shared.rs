//! Módulo compartilhado para enriquecimento de metadados
//!
//! Contém estruturas e funções reutilizadas por enrichment e covers.

use crate::constants::NOT_FOUND_MARKER;
use crate::database;
use crate::models::{GameDescription, ImportConfidence};
use crate::providers::metadata::hltb;
use crate::providers::metadata::steam::{
    get_app_details, get_app_reviews, search_app_by_name, SteamReviewSummary, SteamStoreData,
};
use crate::services::cache;
use crate::utils::text::{is_likely_non_base_game, normalize_for_matching, strip_edition_suffix};
use rusqlite::params;
use std::collections::HashMap;
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
    pub description: GameDescription,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<crate::models::GameTag>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub critic_score: Option<i32>,
    pub series: Option<String>,
    pub steam_review_label: Option<String>,
    pub steam_review_count: Option<i32>,
    pub steam_review_score: Option<f32>,
    pub steam_review_updated_at: Option<String>,
    pub is_adult: bool,
    pub adult_tags: Option<String>,
    pub external_links: Option<String>,
    pub steam_app_id: Option<String>,
    pub hltb_main_story: Option<f64>,
    pub hltb_main_extra: Option<f64>,
    pub hltb_completionist: Option<f64>,
    pub hltb_coop_time: Option<f64>,
    pub alternative_names: Option<Vec<String>>,
    pub franchise: Option<Vec<String>>,
    pub game_modes: Option<Vec<String>>,
    pub player_perspectives: Option<Vec<String>>,
    pub themes: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>, // keywords crus, antes de virar GameTag
    pub age_ratings: Option<String>,   // JSON de HashMap<String,String>, como external_links
    pub display_name: Option<String>,  // nome canônico do IGDB
    pub updated_at: Option<String>,
}

/// Resultado da resolução de `steam_app_id` a partir do nome de um jogo.
pub struct SteamIdResolution {
    pub app_id: String,
    pub confidence: ImportConfidence,
}

/// Candidata a capa coletada durante o enrichment, ainda não persistida.
/// `priority` segue a convenção de `game_images`: menor valor = maior preferência.
pub struct CoverCandidate {
    pub source: &'static str, // "steamgriddb" | "igdb" | "steam"
    pub url: String,
    pub thumb_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub priority: i32,
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

    let candidates = match search_app_by_name(name).await {
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
) -> Option<SteamStoreData> {
    let cache_key = format!("store_{}", steam_id);

    if let Some(cached) = cache::get_cached_api_data(cache_conn, "steam", &cache_key) {
        if let Ok(data) = serde_json::from_str::<SteamStoreData>(&cached) {
            return Some(data);
        }
    }

    match get_app_details(steam_id).await {
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
) -> Option<SteamReviewSummary> {
    let cache_key = format!("reviews_{}", steam_id);

    if let Some(cached) = cache::get_cached_api_data(cache_conn, "steam", &cache_key) {
        if let Ok(reviews) = serde_json::from_str::<SteamReviewSummary>(&cached) {
            return Some(reviews);
        }
    }

    match get_app_reviews(steam_id).await {
        Ok(Some(reviews)) => {
            if let Ok(json) = serde_json::to_string(&reviews) {
                let _ = cache::save_cached_api_data(cache_conn, "steam", &cache_key, &json);
            }
            Some(reviews)
        }
        _ => None,
    }
}

/// Busca dados do HLTB com cache e retorna o melhor resultado encontrado.
pub async fn fetch_hltb_metadata(
    name: &str,
    cache_conn: &rusqlite::Connection,
) -> Option<hltb::HltbEntry> {
    match hltb::HltbClient::search_hltb_with_cache(name, cache_conn).await {
        Ok(results) => results.first().cloned(),
        Err(e) => {
            warn!("search_hltb_with_cache falhou para '{}': {}", name, e);
            None
        }
    }
}

/// Aplica os campos do HLTB em `game_details`, preservando os links já existentes.
pub fn apply_hltb_metadata(details: &mut ProcessedGameDetails, entry: &hltb::HltbEntry) {
    details.hltb_main_story = entry.main_story;
    details.hltb_main_extra = entry.main_extra;
    details.hltb_completionist = entry.completionist;
    details.hltb_coop_time = entry.coop_time;

    let mut links_map: HashMap<String, String> = match details.external_links.as_ref() {
        Some(raw) => match serde_json::from_str(raw) {
            Ok(map) => map,
            Err(e) => {
                warn!(
                    "Falha ao ler external_links para anexar HLTB '{}': {}",
                    details.game_id, e
                );
                HashMap::new()
            }
        },
        None => HashMap::new(),
    };

    links_map
        .entry("hltb".to_string())
        .or_insert_with(|| entry.game_web_link.clone());

    details.external_links = serde_json::to_string(&links_map).ok();
}

// === PERSISTÊNCIA ===

/// Salva detalhes do jogo no banco. Aceita tanto Connection quanto Transaction (via Deref trait)
pub fn save_game_details<C>(conn: &C, d: ProcessedGameDetails) -> Result<(), rusqlite::Error>
where
    C: std::ops::Deref<Target=rusqlite::Connection>,
{
    let tags_json = database::serialize_tags(&d.tags).unwrap_or_else(|_| "[]".to_string());
    let genres_json = serde_json::to_string(&d.genres).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT OR IGNORE INTO game_details (game_id) VALUES (?1)",
        params![d.game_id],
    )?;

    conn.execute(
        "UPDATE game_details SET
            release_date        = COALESCE(?2,  release_date),
            genres              = COALESCE(NULLIF(?3, '[]'), genres),
            tags                = COALESCE(NULLIF(?4, '[]'), tags),
            developer           = COALESCE(?5,  developer),
            publisher           = COALESCE(?6,  publisher),
            critic_score        = COALESCE(?7,  critic_score),
            series              = COALESCE(?8,  series),
            steam_review_label  = COALESCE(?9, steam_review_label),
            steam_review_count  = COALESCE(?10, steam_review_count),
            steam_review_score  = COALESCE(?11, steam_review_score),
            steam_review_updated_at = COALESCE(?12, steam_review_updated_at),
            is_adult            = ?14,
            adult_tags          = COALESCE(?15, adult_tags),
            external_links      = COALESCE(?16, external_links),
            steam_app_id        = COALESCE(?17, steam_app_id),
            hltb_main_story     = COALESCE(?18, hltb_main_story),
            hltb_main_extra     = COALESCE(?19, hltb_main_extra),
            hltb_completionist  = COALESCE(?20, hltb_completionist),
            hltb_coop_time      = COALESCE(?21, hltb_coop_time),
            franchise           = COALESCE(?22, franchise),
            game_modes          = COALESCE(?23, game_modes),
            player_perspectives = COALESCE(?24, player_perspectives),
            themes              = COALESCE(?25, themes),
            keywords            = COALESCE(?26, keywords),
            age_ratings         = COALESCE(?27, age_ratings),
            display_name        = COALESCE(?28, display_name),
            updated_at          = ?29
         WHERE game_id = ?1",
        params![
            d.game_id,
            d.release_date,
            genres_json,
            tags_json,
            d.developer,
            d.publisher,
            d.critic_score,
            d.series,
            d.steam_review_label,
            d.steam_review_count,
            d.steam_review_score,
            d.steam_review_updated_at,
            d.is_adult,
            d.adult_tags,
            d.external_links,
            d.steam_app_id,
            d.hltb_main_story,
            d.hltb_main_extra,
            d.hltb_completionist,
            d.hltb_coop_time,
            d.franchise
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            d.game_modes
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            d.player_perspectives
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            d.themes
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            d.keywords
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            d.age_ratings,
            d.display_name,
            d.updated_at
        ],
    )?;

    save_game_description(conn, &d.game_id, &d.description)?;

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

pub(crate) fn save_game_description<C>(
    conn: &C,
    game_id: &str,
    d: &GameDescription,
) -> Result<(), rusqlite::Error>
where
    C: std::ops::Deref<Target=rusqlite::Connection>,
{
    conn.execute(
        "INSERT OR IGNORE INTO game_descriptions (game_id) VALUES (?1)",
        params![game_id],
    )?;
    conn.execute(
        "UPDATE game_descriptions SET
           summary                     = COALESCE(?2, summary),
           storyline                   = COALESCE(?3, storyline),
           short_description           = COALESCE(?4, short_description),
           description                 = COALESCE(?5, description),
           summary_translated          = COALESCE(?6, summary_translated),
           storyline_translated        = COALESCE(?7, storyline_translated),
           short_description_translated = COALESCE(?8, short_description_translated),
           description_translated      = COALESCE(?9, description_translated),
           translated_lang             = COALESCE(?10, translated_lang)
         WHERE game_id = ?1",
        params![
            game_id,
            d.summary,
            d.storyline,
            d.short_description,
            d.description,
            d.summary_translated,
            d.storyline_translated,
            d.short_description_translated,
            d.description_translated,
            d.translated_lang,
        ],
    )?;
    Ok(())
}
