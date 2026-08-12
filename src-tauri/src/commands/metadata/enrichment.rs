//! Comandos para enriquecimento automático de metadados
//!
//! Este módulo contém comandos Tauri para atualizar metadados de jogos na biblioteca
//! do usuário, buscando informações de APIs externas como IGDB e Steam.
//! Versão otimizada com cache SQLite e processamento em batch.
//!
//! Design notes:
//! - Cache persistente via SQLite (metadata.db)
//! - block_in_place usado para manter conexão SQLite durante awaits
//! - Itens compartilhados com covers estão no módulo shared

use super::shared::{
    apply_hltb_metadata, fetch_hltb_metadata, fetch_steam_reviews, fetch_steam_store_data,
    resolve_steam_app_id, save_game_details, EnrichCompletePayload, EnrichProgress,
    ProcessedGameDetails,
};
use crate::commands::libraries::core::NewlyImportedGame;
use crate::database::AppState;
use crate::providers::metadata::igdb;
use crate::providers::metadata::steam::detect_adult_content;
use crate::providers::mods::nexus::{find_best_nexus_match, get_cached_nexus_games, NexusGame};
use crate::services::cache;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Emitter, Manager, State};
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

struct MissingMetadataBatchItem {
    game_id: String,
    game_name: String,
    processed_data: ProcessedGameDetails,
    dlcs: Vec<igdb::core::IgdbDlc>,
}

// === LÓGICA CORE (REFATORADA) ===

pub async fn enrich_newly_imported(app: AppHandle, games: Vec<NewlyImportedGame>) {
    let platform_label = games.first().map(|g| g.platform.clone());

    let state: State<AppState> = app.state();
    let total = games.len();
    let mut all_session_tags: HashSet<String> = HashSet::new();

    let nexus_games: Vec<NexusGame> = state
        .cache_db
        .lock()
        .ok()
        .and_then(|conn| get_cached_nexus_games(&conn).ok())
        .unwrap_or_default();

    let batch_size = crate::constants::REQUISITIONS_PER_BATCH as usize;
    let mut processed_count = 0usize;
    let mut total_success = 0usize;
    let mut total_errors = 0usize;

    for chunk in games.chunks(batch_size) {
        let mut batch_results: Vec<MissingMetadataBatchItem> = Vec::new();

        for game in chunk {
            processed_count += 1;

            let _ = app.emit(
                "enrich_progress",
                EnrichProgress {
                    current: processed_count as i32,
                    total_found: total as i32,
                    last_game: game.name.clone(),
                    status: "running".to_string(),
                    platform: platform_label.clone(),
                },
            );

            let (processed_data, raw_tags, dlcs, _igdb_found) = {
                let cache_conn = match state.cache_db.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        enrich_game_metadata(
                            &app,
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
            batch_results.push(MissingMetadataBatchItem {
                game_id: game.game_id.clone(),
                game_name: game.name.clone(),
                processed_data,
                dlcs,
            });
        }

        // Persiste este lote antes de seguir pro próximo — se o app fechar
        // no meio, no máximo os jogos deste lote são reprocessados depois.
        if let Ok(mut conn) = state.games_db.lock() {
            if let Ok(tx) = conn.transaction() {
                let mut success = 0;
                let mut errors = 0;
                for item in batch_results {
                    if let Err(e) = save_game_details(&tx, item.processed_data) {
                        warn!(
                            "enrich_newly_imported: erro ao salvar {}: {}",
                            item.game_name, e
                        );
                        errors += 1;
                    } else {
                        let _ = igdb::core::save_game_dlcs(&tx, &item.game_id, &item.dlcs);
                        success += 1;
                    }
                }
                match tx.commit() {
                    Ok(_) => {
                        total_success += success;
                        total_errors += errors;
                        info!(
                            "enrich_newly_imported: lote salvo ({} ok, {} erros, {}/{} no total)",
                            success, errors, processed_count, total
                        );
                    }
                    Err(e) => {
                        warn!("enrich_newly_imported: commit do lote falhou: {}", e);
                        total_errors += success + errors;
                    }
                }
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

    info!(
        "enrich_newly_imported: {} ok, {} erros (total processado: {})",
        total_success, total_errors, processed_count
    );
}

/// Processa um único jogo com cache integrado (sem manter lock)
async fn enrich_game_metadata(
    app: &AppHandle,
    game_id: &str,
    name: &str,
    platform: &str,
    platform_game_id: Option<String>,
    cache_conn: &rusqlite::Connection,
    nexus_games: &[NexusGame],
) -> (
    ProcessedGameDetails,
    Vec<String>,
    Vec<igdb::core::IgdbDlc>,
    bool,
) {
    let mut details = ProcessedGameDetails {
        game_id: game_id.to_string(),
        description: crate::models::GameDescription::default(),
        release_date: None,
        genres: Vec::new(),
        tags: Vec::new(),
        developer: None,
        publisher: None,
        critic_score: None,
        background_image: None,
        series: None,
        steam_review_label: None,
        steam_review_count: None,
        steam_review_score: None,
        steam_review_updated_at: None,
        esrb_rating: None,
        is_adult: false,
        adult_tags: None,
        external_links: None,
        steam_app_id: None,
        hltb_main_story: None,
        hltb_main_extra: None,
        hltb_completionist: None,
        hltb_coop_time: None,
        alternative_names: None,
        franchise: None,
        game_modes: None,
        player_perspectives: None,
        themes: None,
        keywords: None,
        age_ratings: None,
        display_name: None,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let mut links_map: HashMap<String, String> = HashMap::new();

    if let Some(nexus_match) = find_best_nexus_match(name, nexus_games) {
        let url = format!("https://www.nexusmods.com/{}", nexus_match.domain_name);
        links_map.insert("nexus".to_string(), url);
    }

    // IGDB é a fonte principal. RAWG roda em paralelo como fallback — só é
    // usada campo a campo pro que o IGDB não trouxer (ou não achar o jogo).
    let igdb_future = igdb::fetch::search_and_resolve(app, name);
    let hltb_future = fetch_hltb_metadata(name, cache_conn);

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

    let (igdb_result, (target_steam_id, store_data, reviews), hltb_result) =
        tokio::join!(igdb_future, steam_future, hltb_future);

    let mut found_raw_tags: Vec<String> = Vec::new();
    let mut igdb_dlcs: Vec<igdb::core::IgdbDlc> = Vec::new();
    let mut igdb_found = false;

    match igdb_result {
        Ok(Some(game)) => {
            igdb_found = true;
            let mapped = igdb::core::map_igdb_game(&game, game_id);
            found_raw_tags = mapped.details.tags.iter().map(|t| t.slug.clone()).collect();
            igdb_dlcs = mapped.dlcs;

            if let Some(igdb_links_json) = &mapped.details.external_links {
                if let Ok(igdb_links) =
                    serde_json::from_str::<HashMap<String, String>>(igdb_links_json)
                {
                    for (k, v) in igdb_links {
                        links_map.entry(k).or_insert(v);
                    }
                }
            }

            details.description = mapped.details.description;
            details.release_date = mapped.details.release_date;
            details.genres = mapped.details.genres;
            details.tags = mapped.details.tags;
            details.developer = mapped.details.developer;
            details.publisher = mapped.details.publisher;
            details.critic_score = mapped.details.critic_score;
            details.background_image = mapped.details.background_image;
            details.series = mapped.details.series;
            details.esrb_rating = mapped.details.esrb_rating;
            details.alternative_names = mapped.details.alternative_names;
            details.franchise = mapped.details.franchise;
            details.game_modes = mapped.details.game_modes;
            details.player_perspectives = mapped.details.player_perspectives;
            details.themes = mapped.details.themes;
            details.keywords = mapped.details.keywords;
            details.age_ratings = mapped.details.age_ratings;
            details.display_name = mapped.details.display_name;
        }
        Ok(None) => warn!(
            "IGDB: nenhum resultado para '{}', usando RAWG como fallback",
            name
        ),
        Err(e) => warn!("IGDB search_and_resolve falhou para '{}': {}", name, e),
    }

    if let Some(steam_id) = &target_steam_id {
        links_map
            .entry("steam".to_string())
            .or_insert_with(|| format!("https://store.steampowered.com/app/{}", steam_id));
        details.steam_app_id = Some(steam_id.clone());

        if let Some(store_data) = store_data {
            let (detected_adult, flags) = detect_adult_content(&store_data);
            details.is_adult = detected_adult;
            if !flags.is_empty() {
                details.adult_tags = serde_json::to_string(&flags).ok();
            }
            details
                .description
                .short_description
                .get_or_insert(store_data.short_description);
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

    if let Some(hltb_det) = hltb_result {
        apply_hltb_metadata(&mut details, &hltb_det);
    }

    (details, found_raw_tags, igdb_dlcs, igdb_found)
}
