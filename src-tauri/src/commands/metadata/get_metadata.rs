//! Preenchimento de campos de metadados faltantes via RAWG.
//!
//! Diferente do fluxo de enriquecimento inicial (`enrichment.rs`), este módulo
//! foca em jogos que já foram importados mas ainda têm lacunas nos metadados
//! (genres, developer, tags, description, etc.).
//!
//! Design notes:
//! - Ignora o cache da RAWG (`fetch_rawg_metadata_fresh`) para garantir dados atualizados.
//! - Usa `COALESCE` via `save_game_details` — nunca sobrescreve campos existentes com NULL.
//! - Reutiliza `ProcessedGameDetails` e `save_game_details` de `enrichment.rs`.

use super::enrichment::{save_game_details, ProcessedGameDetails};
use super::shared::{
    fetch_rawg_metadata_fresh, fetch_steam_reviews, fetch_steam_store_data, resolve_steam_app_id,
    EnrichCompletePayload, EnrichProgress,
};
use crate::constants::{RAWG_RATE_LIMIT_MS, RAWG_REQUISITIONS_PER_BATCH};
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::services::integration::steam_api;
use crate::utils::series;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::sleep;
use tracing::{info, warn};

/// Preenche campos de metadados vazios consultando a RAWG (sem cache).
///
/// - Processa **todos** os jogos que ainda têm pelo menos um campo vazio:
///   `genres`, `developer`, `tags`, `description_raw`, `release_date` ou `background_image`.
/// - **Ignora o cache** da RAWG para garantir dados atualizados.
/// - **Nunca sobrescreve** campos que já têm valor — apenas preenche lacunas.
///
/// Ideal para rodar após importação parcial (ex: Legacy Games) ou quando a RAWG
/// atualizar o catálogo após a última sincronização.
#[tauri::command]
pub async fn fill_missing_metadata(app: AppHandle) -> Result<(), AppError> {
    let app_handle = app.clone();
    let api_key = database::get_secret(&app, "rawg_api_key")?;

    if api_key.is_empty() {
        return Err(AppError::ValidationError(
            "API Key da RAWG não configurada.".to_string(),
        ));
    }

    tauri::async_runtime::spawn(async move {
        info!("Iniciando preenchimento de campos vazios (fresh RAWG)...");

        let state: State<AppState> = app_handle.state();
        let mut all_session_tags: HashSet<String> = HashSet::new();
        // IDs já processados nesta sessão — evita re-processar jogos cujos dados simplesmente não existem na RAWG.
        let mut processed_ids: HashSet<String> = HashSet::new();

        loop {
            // 1. Seleciona jogos com ao menos um campo de metadados vazio, excluindo os que já foram tentados nesta sessão.
            let games_to_fill: Vec<(String, String, String, Option<String>)> = {
                let conn = match state.games_db.lock() {
                    Ok(c) => c,
                    Err(_) => break,
                };

                // Monta cláusula de exclusão dinâmica para os IDs já processados
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
                     WHERE (
                         gd.game_id IS NULL
                         OR gd.genres           IS NULL OR gd.genres           = ''
                         OR gd.developer        IS NULL OR gd.developer        = ''
                         OR gd.tags             IS NULL OR gd.tags             = '' OR gd.tags = '[]'
                         OR gd.description_raw  IS NULL OR gd.description_raw  = ''
                         OR gd.release_date     IS NULL OR gd.release_date     = ''
                         OR gd.background_image IS NULL OR gd.background_image = ''
                     ){}
                     LIMIT ?1",
                    exclusions
                );

                let mut stmt = match conn.prepare(&sql) {
                    Ok(s) => s,
                    Err(_) => break,
                };

                // Parâmetros: primeiro o LIMIT, depois os IDs excluídos
                let excluded_ids: Vec<String> = processed_ids.iter().cloned().collect();
                let limit_val = RAWG_REQUISITIONS_PER_BATCH;

                let result = if excluded_ids.is_empty() {
                    stmt.query_map(params![limit_val], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                    })
                        .unwrap()
                        .flatten()
                        .collect::<Vec<_>>()
                } else {
                    use rusqlite::types::ToSql;
                    let mut bind: Vec<Box<dyn ToSql>> = vec![Box::new(limit_val)];
                    for id in &excluded_ids {
                        bind.push(Box::new(id.clone()));
                    }
                    let refs: Vec<&dyn ToSql> = bind.iter().map(|b| b.as_ref()).collect();
                    stmt.query_map(refs.as_slice(), |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                    })
                        .unwrap()
                        .flatten()
                        .collect::<Vec<_>>()
                };

                result
            };

            if games_to_fill.is_empty() {
                break;
            }

            let total_in_batch = games_to_fill.len();
            let mut batch_results = Vec::new();

            // 2. Processa cada jogo do batch
            for (index, (game_id, name, platform, platform_game_id)) in
                games_to_fill.into_iter().enumerate()
            {
                // Marca como processado imediatamente — mesmo que a RAWRG retorne dados, este jogo não será selecionado novamente.
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

                let (processed_data, raw_tags) = {
                    let cache_conn = match state.cache_db.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    tokio::task::block_in_place(|| {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            let series_name = series::infer_series(&name);
                            let mut details = ProcessedGameDetails {
                                game_id: game_id.clone(),
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
                            let mut found_raw_tags: Vec<String> = Vec::new();

                            // A cadeia Steam (resolve ID → store_data + reviews em paralelo) não
                            // depende da RAWG, então roda inteira em paralelo com ela. Dentro da
                            // cadeia Steam, store_data e reviews só dependem do ID resolvido.
                            let rawg_future = fetch_rawg_metadata_fresh(&api_key, &name, &cache_conn);

                            let steam_future = async {
                                let target_steam_id = resolve_steam_app_id(
                                    &name,
                                    &platform,
                                    platform_game_id.as_deref(),
                                    &cache_conn,
                                )
                                    .await
                                    .map(|resolution| resolution.app_id);

                                match &target_steam_id {
                                    Some(steam_id) => {
                                        let (store_data, reviews) = tokio::join!(
                            fetch_steam_store_data(steam_id, &cache_conn),
                            fetch_steam_reviews(steam_id, &cache_conn)
                        );
                                        (target_steam_id, store_data, reviews)
                                    }
                                    None => (target_steam_id, None, None),
                                }
                            };

                            let (rawg_result, (target_steam_id, store_data, reviews)) =
                                tokio::join!(rawg_future, steam_future);

                            // 2a. Aplica resultado da RAWG (já resolvido acima)
                            if let Some(rawg_det) = rawg_result {
                                found_raw_tags = rawg_det.tags.iter().map(|t| t.slug.clone()).collect();
                                let raw_tag_slugs: Vec<String> =
                                    rawg_det.tags.iter().map(|t| t.slug.clone()).collect();

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
                            } else {
                                warn!(
                    game = %name,
                    "RAWG não retornou dados — jogo será ignorado nas próximas iterações"
                );
                            }

                            // 2b. Steam como fallback / complemento — já resolvido acima
                            if let Some(steam_id) = &target_steam_id {
                                if !links_map.contains_key("steam") {
                                    links_map.insert(
                                        "steam".to_string(),
                                        format!("https://store.steampowered.com/app/{}", steam_id),
                                    );
                                }
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

                            (details, found_raw_tags)
                        })
                    })
                };

                for tag in raw_tags {
                    all_session_tags.insert(tag);
                }
                batch_results.push((name.clone(), processed_data));
            }

            // 3. Persiste o batch numa única transação
            // save_game_details usa COALESCE → nunca sobrescreve campos existentes
            if let Ok(mut conn) = state.games_db.lock() {
                match conn.transaction() {
                    Ok(tx) => {
                        let mut success_count = 0;
                        let mut error_count = 0;

                        for (game_name, processed_data) in batch_results {
                            if let Err(e) = save_game_details(&tx, processed_data) {
                                warn!("fill_missing: erro ao salvar {}: {}", game_name, e);
                                error_count += 1;
                            } else {
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
                    Err(e) => warn!("fill_missing: transação falhou: {}", e),
                }
            }

            // 4. Rate limit entre batches
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
