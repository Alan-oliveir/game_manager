//! Comandos para enriquecimento automático de metadados
//!
//! Este módulo contém comandos Tauri para atualizar metadados de jogos na biblioteca
//! do usuário, buscando informações de APIs externas como RAWG e Steam.
//! Versão otimizada com cache SQLite e processamento em batch.
//!
//! Design notes:
//! - Cache persistente via SQLite (metadata.db)
//! - block_in_place usado para manter conexão SQLite durante awaits
//! - Itens compartilhados com covers estão no módulo shared

use super::shared::{
    fetch_rawg_metadata, fetch_steam_reviews, fetch_steam_store_data, resolve_steam_app_id,
    EnrichCompletePayload, EnrichProgress,
};
use crate::commands::platforms::core::NewlyImportedGame;
use crate::constants::RAWG_RATE_LIMIT_MS;
use crate::database;
use crate::database::AppState;
use crate::services::cache;
use crate::services::integration::nexus::{find_best_nexus_match, NexusGame};
use crate::services::integration::steam_api;
use crate::utils::series;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::sleep;
use tracing::{info, warn};

// === ESTRUTURAS DE DADOS ===

#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub success_count: i32,
    pub error_count: i32,
    pub total_processed: i32,
    pub message: String,
    pub errors: Vec<String>,
}

/// Estrutura intermediária
pub(in crate::commands::metadata) struct ProcessedGameDetails {
    pub(in crate::commands::metadata) game_id: String,
    pub(in crate::commands::metadata) description_raw: Option<String>,
    pub(in crate::commands::metadata) description_ptbr: Option<String>,
    pub(in crate::commands::metadata) release_date: Option<String>,
    pub(in crate::commands::metadata) genres: String,
    pub(in crate::commands::metadata) tags: Vec<crate::models::GameTag>,
    pub(in crate::commands::metadata) developer: Option<String>,
    pub(in crate::commands::metadata) publisher: Option<String>,
    pub(in crate::commands::metadata) critic_score: Option<i32>,
    pub(in crate::commands::metadata) background_image: Option<String>,
    pub(in crate::commands::metadata) series: Option<String>,
    pub(in crate::commands::metadata) steam_review_label: Option<String>,
    pub(in crate::commands::metadata) steam_review_count: Option<i32>,
    pub(in crate::commands::metadata) steam_review_score: Option<f32>,
    pub(in crate::commands::metadata) steam_review_updated_at: Option<String>,
    pub(in crate::commands::metadata) esrb_rating: Option<String>,
    pub(in crate::commands::metadata) is_adult: bool,
    pub(in crate::commands::metadata) adult_tags: Option<String>,
    pub(in crate::commands::metadata) external_links: Option<String>,
    pub(in crate::commands::metadata) steam_app_id: Option<String>,
    pub(in crate::commands::metadata) median_playtime: Option<i32>,
    pub(in crate::commands::metadata) estimated_playtime: Option<f32>,
}

// === LÓGICA CORE (REFATORADA) ===

pub async fn enrich_newly_imported(app: AppHandle, games: Vec<NewlyImportedGame>) {
    let api_key = match database::get_secret(&app, "rawg_api_key") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            warn!("Enrichment pós-import ignorado: API Key da RAWG não configurada.");
            return;
        }
    };

    // Todos os jogos do lote vêm da mesma importação — extrai a plataforma antes de consumir a lista no loop.
    let platform_label = games.first().map(|g| g.platform.clone());

    let state: State<AppState> = app.state();
    let total = games.len();
    let mut all_session_tags: HashSet<String> = HashSet::new();
    let mut batch_results = Vec::new();

    // Carrega o catálogo do Nexus uma única vez, direto do cache local
    let nexus_games: Vec<NexusGame> = state
        .cache_db
        .lock()
        .ok()
        .and_then(|conn| cache::get_cached_nexus_games(&conn).ok())
        .unwrap_or_default();

    // loop de processamento
    for (index, game) in games.into_iter().enumerate() {
        let _ = app.emit(
            "enrich_progress",
            EnrichProgress {
                current: (index + 1) as i32,
                total_found: total as i32,
                last_game: game.name.clone(),
                status: "running".to_string(),
                platform: platform_label.clone(),
            },
        );

        let (processed_data, raw_tags, _rawg_found) = {
            let cache_conn = match state.cache_db.lock() {
                Ok(c) => c,
                Err(_) => continue,
            };
            tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    enrich_game_metadata(
                        &api_key,
                        &game.game_id,
                        &game.name,
                        &game.platform,
                        Some(game.platform_game_id.clone()),
                        &cache_conn,
                        &nexus_games,
                    )
                        .await
                })
            })
        };

        for tag in raw_tags {
            all_session_tags.insert(tag);
        }
        batch_results.push((game.name, processed_data));

        sleep(Duration::from_millis(RAWG_RATE_LIMIT_MS)).await;
    }

    // bloco de persistência/transação
    if let Ok(mut conn) = state.games_db.lock() {
        if let Ok(tx) = conn.transaction() {
            let mut success = 0;
            let mut errors = 0;
            for (name, data) in batch_results {
                if let Err(e) = save_game_details(&tx, data) {
                    warn!("enrich_newly_imported: erro ao salvar {}: {}", name, e);
                    errors += 1;
                } else {
                    success += 1;
                }
            }
            match tx.commit() {
                Ok(_) => info!("enrich_newly_imported: {} ok, {} erros", success, errors),
                Err(e) => warn!("enrich_newly_imported: commit falhou: {}", e),
            }
        }
    }

    let _ = crate::services::tags::generate_analysis_report(&app, all_session_tags);

    let message = match &platform_label {
        Some(platform) => format!("{}: enriquecimento concluído.", platform),
        None => "Enriquecimento pós-import concluído.".to_string(),
    };
    let _ = app.emit(
        "enrich_complete",
        EnrichCompletePayload {
            platform: platform_label,
            message,
        },
    );
}

/// Processa um único jogo com cache integrado (sem manter lock)
async fn enrich_game_metadata(
    api_key: &str,
    game_id: &str,
    name: &str,
    platform: &str,
    platform_game_id: Option<String>,
    cache_conn: &rusqlite::Connection,
    nexus_games: &[NexusGame],
) -> (ProcessedGameDetails, Vec<String>, bool) {
    let series_name = series::infer_series(name);
    let mut details = ProcessedGameDetails {
        game_id: game_id.to_string(),
        description_raw: None,
        description_ptbr: None,
        release_date: None,
        genres: String::new(),
        tags: Vec::new(),
        developer: None,
        publisher: None,
        critic_score: None,
        background_image: None,
        series: series_name,
        steam_review_label: None,
        steam_review_count: None,
        steam_review_score: None,
        steam_review_updated_at: None,
        esrb_rating: None,
        is_adult: false,
        adult_tags: None,
        external_links: None,
        steam_app_id: None,
        median_playtime: None,
        estimated_playtime: None,
    };

    let mut links_map: HashMap<String, String> = HashMap::new();

    if let Some(nexus_match) = find_best_nexus_match(name, nexus_games) {
        let url = format!("https://www.nexusmods.com/{}", nexus_match.domain_name);
        links_map.insert("nexus".to_string(), url);
    }

    // A cadeia Steam (resolve ID → store_data + reviews em paralelo) não depende da RAWG,
    // então roda inteira em paralelo com ela. Dentro da cadeia Steam, store_data e reviews
    // só dependem do ID resolvido, não uma da outra.
    let rawg_future = fetch_rawg_metadata(api_key, name, cache_conn);

    let steam_future = async {
        let target_steam_id =
            resolve_steam_app_id(name, platform, platform_game_id.as_deref(), cache_conn)
                .await
                .map(|resolution| resolution.app_id);

        match &target_steam_id {
            Some(steam_id) => {
                let (store_data, reviews) = tokio::join!(
                    fetch_steam_store_data(steam_id, cache_conn),
                    fetch_steam_reviews(steam_id, cache_conn)
                );
                (target_steam_id, store_data, reviews)
            }
            None => (target_steam_id, None, None),
        }
    };

    let (rawg_result, (target_steam_id, store_data, reviews)) =
        tokio::join!(rawg_future, steam_future);

    let mut found_raw_tags: Vec<String> = Vec::new();
    let mut rawg_found = false;

    if let Some(rawg_det) = rawg_result {
        rawg_found = true;
        found_raw_tags = rawg_det.tags.iter().map(|t| t.slug.clone()).collect();

        let raw_tag_slugs: Vec<String> = rawg_det.tags.iter().map(|t| t.slug.clone()).collect();

        details.description_raw = rawg_det.description_raw;
        details.release_date = rawg_det.released;
        details.genres = rawg_det
            .genres
            .iter()
            .map(|g| g.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        details.tags = crate::services::tags::classify_and_sort_tags(raw_tag_slugs, 10);
        details.developer = rawg_det.developers.first().map(|d| d.name.clone());
        details.publisher = rawg_det.publishers.first().map(|p| p.name.clone());
        details.critic_score = rawg_det.metacritic;
        details.background_image = rawg_det.background_image;
        details.esrb_rating = rawg_det.esrb_rating.as_ref().map(|r| r.name.clone());

        if let Some(url) = &rawg_det.website {
            links_map.insert("website".to_string(), url.clone());
        }
        if let Some(url) = &rawg_det.reddit_url {
            links_map.insert("reddit".to_string(), url.clone());
        }
        if let Some(url) = &rawg_det.metacritic_url {
            links_map.insert("metacritic".to_string(), url.clone());
        }
        links_map.insert(
            "rawg".to_string(),
            format!("https://rawg.io/games/{}", rawg_det.id),
        );
    }

    if let Some(steam_id) = &target_steam_id {
        links_map
            .entry("steam".to_string())
            .or_insert_with(|| format!("https://store.steampowered.com/app/{}", steam_id));
        details.steam_app_id = Some(steam_id.clone());

        if let Some(store_data) = store_data {
            let (detected_adult, flags) = steam_api::detect_adult_content(&store_data);
            details.is_adult = detected_adult;
            if !flags.is_empty() {
                details.adult_tags = serde_json::to_string(&flags).ok();
            }
            if details.description_raw.is_none() {
                details.description_raw = Some(store_data.short_description);
            }
            if details.release_date.is_none() {
                details.release_date = store_data.release_date;
            }
            if details.background_image.is_none() {
                details.background_image = Some(store_data.header_image);
            }
        }

        if let Some(reviews) = reviews {
            details.steam_review_label = Some(reviews.review_score_desc);
            details.steam_review_count = Some(reviews.total_reviews as i32);
            let total = reviews.total_positive + reviews.total_negative;
            if total > 0 {
                details.steam_review_score =
                    Some((reviews.total_positive as f32 / total as f32) * 100.0);
            }
            details.steam_review_updated_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    if !links_map.is_empty() {
        details.external_links = serde_json::to_string(&links_map).ok();
    }

    (details, found_raw_tags, rawg_found)
}

// === PERSISTÊNCIA ===

/// Salva detalhes do jogo no banco. Aceita tanto Connection quanto Transaction (via Deref trait)
pub(in crate::commands::metadata) fn save_game_details<C>(
    conn: &C,
    d: ProcessedGameDetails,
) -> Result<(), rusqlite::Error>
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
            estimated_playtime  = COALESCE(?22, estimated_playtime)
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
            d.estimated_playtime
        ],
    )?;

    if let Some(img) = d.background_image {
        conn.execute(
            "UPDATE games SET cover_url = ?1 WHERE id = ?2 AND (cover_url IS NULL OR cover_url = '')",
            params![img, d.game_id],
        )?;
    }

    Ok(())
}
