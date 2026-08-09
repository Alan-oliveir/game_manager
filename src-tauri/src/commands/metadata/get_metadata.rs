//! Preenchimento de campos de metadados faltantes via IGDB.
//!
//! Diferente do fluxo de enriquecimento inicial (`enrichment.rs`), este módulo
//! foca em jogos que já foram importados mas ainda têm lacunas nos metadados
//! (genres, developer, tags, description, etc.), usando RAWG apenas como fallback.
//!
//! Design notes:
//! - Prioriza IGDB e consulta RAWG em paralelo como fallback.
//! - Usa `COALESCE` via `save_game_details` — nunca sobrescreve campos existentes com NULL.
//! - Reutiliza `ProcessedGameDetails` e `save_game_details` de `enrichment.rs`.

use crate::commands::metadata::shared::{
    apply_hltb_metadata, fetch_hltb_metadata, fetch_rawg_metadata, fetch_steam_reviews,
    fetch_steam_store_data, resolve_steam_app_id, save_game_details, EnrichCompletePayload,
    EnrichProgress, ProcessedGameDetails,
};
use crate::constants::{RAWG_RATE_LIMIT_MS, RAWG_REQUISITIONS_PER_BATCH};
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::services::integration::nexus::{find_best_nexus_match, NexusGame};
use crate::services::integration::{igdb, steam_api};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::sleep;
use tracing::{info, warn};

struct MissingMetadataBatchItem {
    game_id: String,
    game_name: String,
    processed_data: ProcessedGameDetails,
    dlcs: Vec<igdb::core::IgdbDlc>,
}

/// Retorna um lote de jogos que possuem campos vazios, ignorando os IDs que já foram processados.
fn get_games_to_fill(
    conn: &Connection,
    processed_ids: &HashSet<String>,
    limit_val: u32,
) -> Vec<(String, String, String, Option<String>)> {
    let exclusions = if processed_ids.is_empty() {
        String::new()
    } else {
        let placeholders: Vec<String> = processed_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2)) // +2 porque ?1 = LIMIT
            .collect();
        format!(" AND g.id NOT IN ({})", placeholders.join(", "))
    };

    let sql = format!(
        "SELECT g.id, g.name, g.platform, g.platform_game_id
        FROM games g
        LEFT JOIN game_details gd ON g.id = gd.game_id
        LEFT JOIN game_descriptions gdesc ON g.id = gdesc.game_id
        WHERE (
            gd.game_id IS NULL
            OR gd.genres           IS NULL OR gd.genres           = '' OR gd.genres = '[]'
            OR gd.developer        IS NULL OR gd.developer        = ''
            OR gd.tags             IS NULL OR gd.tags             = '' OR gd.tags = '[]'
            OR gdesc.game_id IS NULL
            OR (gdesc.summary IS NULL AND gdesc.description IS NULL)
            OR gd.release_date     IS NULL OR gd.release_date     = ''
            OR gd.background_image IS NULL OR gd.background_image = ''
        ){}
        LIMIT ?1",
        exclusions
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let excluded_ids: Vec<String> = processed_ids.iter().cloned().collect();

    if excluded_ids.is_empty() {
        if let Ok(rows) = stmt.query_map(rusqlite::params![limit_val], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }) {
            rows.flatten().collect()
        } else {
            vec![]
        }
    } else {
        use rusqlite::types::ToSql;
        let mut bind: Vec<Box<dyn ToSql>> = vec![Box::new(limit_val)];
        for id in &excluded_ids {
            bind.push(Box::new(id.clone()));
        }
        let refs: Vec<&dyn ToSql> = bind.iter().map(|b| b.as_ref()).collect();
        if let Ok(rows) = stmt.query_map(refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }) {
            rows.flatten().collect()
        } else {
            vec![]
        }
    }
}

/// Processa um único jogo priorizando IGDB, com RAWG e Steam como fallback.
async fn process_missing_metadata(
    app: &AppHandle,
    rawg_api_key: &str,
    game_id: &str,
    name: &str,
    platform: &str,
    platform_game_id: Option<String>,
    cache_conn: &Connection,
    nexus_games: &[NexusGame],
) -> (ProcessedGameDetails, Vec<String>, Vec<igdb::core::IgdbDlc>) {
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

    if let Some(nexus_match) = find_best_nexus_match(&name, nexus_games) {
        let url = format!("https://www.nexusmods.com/{}", nexus_match.domain_name);
        links_map.insert("nexus".to_string(), url);
    }

    // IGDB é a fonte principal. RAWG roda em paralelo como fallback — só é
    // usada campo a campo pro que o IGDB não trouxer (ou não achar o jogo).
    let igdb_future = igdb::fetch::search_and_resolve(app, name);
    let hltb_future = fetch_hltb_metadata(name, cache_conn);

    let rawg_future = async {
        if rawg_api_key.is_empty() {
            None
        } else {
            fetch_rawg_metadata(rawg_api_key, name, cache_conn).await
        }
    };

    let steam_future = async {
        let target_steam_id =
            resolve_steam_app_id(&name, &platform, platform_game_id.as_deref(), cache_conn)
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

    let (igdb_result, rawg_result, (target_steam_id, store_data, reviews), hltb_result) =
        tokio::join!(igdb_future, rawg_future, steam_future, hltb_future);

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

    // RAWG só preenche o que o IGDB deixou em branco (fallback campo a campo, não "tudo ou nada").
    if let Some(rawg_det) = rawg_result {
        if found_raw_tags.is_empty() {
            let raw_tag_slugs: Vec<String> = rawg_det.tags.iter().map(|t| t.slug.clone()).collect();
            found_raw_tags = raw_tag_slugs.clone();
            details.tags = crate::services::tags::classify_and_sort_tags(raw_tag_slugs, 10);
        }
        if details.description.summary.is_none() {
            details.description.description = rawg_det.description_raw;
        }
        if details.release_date.is_none() {
            details.release_date = rawg_det.released;
        }
        if details.genres.is_empty() {
            details.genres = rawg_det.genres.iter().map(|g| g.name.clone()).collect();
        }
        if details.developer.is_none() {
            details.developer = rawg_det.developers.first().map(|d| d.name.clone());
        }
        if details.publisher.is_none() {
            details.publisher = rawg_det.publishers.first().map(|p| p.name.clone());
        }
        if details.critic_score.is_none() {
            details.critic_score = rawg_det.metacritic;
        }
        if details.background_image.is_none() {
            details.background_image = rawg_det.background_image;
        }
        if details.esrb_rating.is_none() {
            details.esrb_rating = rawg_det.esrb_rating.as_ref().map(|r| r.name.clone());
        }
        if details.alternative_names.is_none() && !rawg_det.alternative_names.is_empty() {
            details.alternative_names = Some(rawg_det.alternative_names.clone());
        }

        if let Some(url) = &rawg_det.website {
            links_map
                .entry("website".to_string())
                .or_insert_with(|| url.clone());
        }
        if let Some(url) = &rawg_det.reddit_url {
            links_map
                .entry("reddit".to_string())
                .or_insert_with(|| url.clone());
        }
        if let Some(url) = &rawg_det.metacritic_url {
            links_map
                .entry("metacritic".to_string())
                .or_insert_with(|| url.clone());
        }
        links_map
            .entry("rawg".to_string())
            .or_insert_with(|| format!("https://rawg.io/games/{}", rawg_det.id));
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

    (details, found_raw_tags, igdb_dlcs)
}

#[tauri::command]
pub async fn fill_missing_metadata(app: AppHandle) -> Result<(), AppError> {
    let app_handle = app.clone();
    let rawg_api_key = database::get_secret(&app, "rawg_api_key").unwrap_or_default();

    tauri::async_runtime::spawn(async move {
        info!("Iniciando preenchimento de campos vazios (fresh RAWG)...");

        let state: State<AppState> = app_handle.state();
        let mut all_session_tags: HashSet<String> = HashSet::new();
        let mut processed_ids: HashSet<String> = HashSet::new();

        // Carrega o catálogo do Nexus do cache
        let nexus_games: Vec<NexusGame> = state
            .cache_db
            .lock()
            .ok()
            .and_then(|conn| crate::services::cache::get_cached_nexus_games(&conn).ok())
            .unwrap_or_default();

        loop {
            // 1. Busca os jogos usando nossa nova função de extração
            let games_to_fill = match state.games_db.lock() {
                Ok(conn) => get_games_to_fill(&conn, &processed_ids, RAWG_REQUISITIONS_PER_BATCH),
                Err(_) => break,
            };

            if games_to_fill.is_empty() {
                break;
            }

            let total_in_batch = games_to_fill.len();
            let mut batch_results: Vec<MissingMetadataBatchItem> = Vec::new();

            // 2. Processa cada jogo
            for (index, (game_id, name, platform, platform_game_id)) in
                games_to_fill.into_iter().enumerate()
            {
                processed_ids.insert(game_id.clone());

                let _ = app_handle.emit(
                    "enrich_progress",
                    EnrichProgress {
                        current: (index + 1) as i32,
                        total_found: total_in_batch as i32,
                        last_game: name.clone(),
                        status: "running".to_string(),
                        platform: None,
                    },
                );

                let cache_conn = match state.cache_db.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // Executa a requisição assíncrona usando o helper principal de IGDB
                let (processed_data, raw_tags, dlcs): (
                    ProcessedGameDetails,
                    Vec<String>,
                    Vec<igdb::core::IgdbDlc>,
                ) = tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        process_missing_metadata(
                            &app,
                            &rawg_api_key,
                            &game_id,
                            &name,
                            &platform,
                            platform_game_id,
                            &cache_conn,
                            &nexus_games,
                        )
                            .await
                    })
                });

                for tag in raw_tags {
                    all_session_tags.insert(tag);
                }
                batch_results.push(MissingMetadataBatchItem {
                    game_id,
                    game_name: name,
                    processed_data,
                    dlcs,
                });
            }

            // 3. Persiste o batch numa única transação usando save_game_details
            if let Ok(mut conn) = state.games_db.lock() {
                if let Ok(tx) = conn.transaction() {
                    let mut success_count = 0;
                    let mut error_count = 0;

                    for item in batch_results {
                        if let Err(e) = save_game_details(&tx, item.processed_data) {
                            warn!("fill_missing: erro ao salvar {}: {}", item.game_name, e);
                            error_count += 1;
                        } else {
                            let _ = igdb::core::save_game_dlcs(&tx, &item.game_id, &item.dlcs);
                            success_count += 1;
                        }
                    }

                    match tx.commit() {
                        Ok(_) => info!(
                            "fill_missing batch: {} ok, {} erros",
                            success_count, error_count
                        ),
                        Err(e) => warn!("fill_missing: commit falhou: {}", e),
                    }
                }
            }

            sleep(Duration::from_millis(RAWG_RATE_LIMIT_MS)).await;
        }

        let _ = crate::services::tags::generate_analysis_report(&app_handle, all_session_tags);
        let _ = app_handle.emit(
            "enrich_complete",
            EnrichCompletePayload {
                platform: None,
                message: "Campos vazios preenchidos!".to_string(),
            },
        );

        info!("fill_missing_metadata concluído.");
    });

    Ok(())
}
