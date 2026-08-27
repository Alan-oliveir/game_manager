//! Módulo de atualização automática em background (Preços e Reviews).
//!
//! Executa sem travar a UI e falha silenciosamente em caso de erro.

use crate::constants::{
    BACKGROUND_TASK_INTERVAL_SECS, GAMERPOWER_CACHE_SOURCE, GAMERPOWER_LIST_ACTIVE_CACHE_KEY,
    STARTUP_DELAY_SECS,
};
use crate::database::cache;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::giveaways::gamerpower;
use crate::providers::metadata::steam::get_app_reviews;
use crate::providers::mods::nexus::refresh_nexus_games_if_stale;
use crate::providers::pricing::itad;
use crate::services::locale::get_or_detect_region;
use lazy_static::lazy_static;
use rusqlite::params;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{error, info, warn};

// Semaphore para evitar execuções duplicadas (previne race conditions)
// Arc permite compartilhamento seguro entre threads
lazy_static! {
    static ref BACKGROUND_REFRESH_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(1));
}

/// Comando disparado ao iniciar o app.
/// Roda numa thread separada (spawn) para não bloquear a inicialização.
/// Protegido contra execução duplicada (React Strict Mode chama useEffect 2x).
#[tauri::command]
pub async fn check_and_refresh_background(app: AppHandle) -> Result<(), AppError> {
    // Try acquire (non-blocking) - retorna erro se já está rodando
    let permit = match BACKGROUND_REFRESH_SEMAPHORE.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            // Já existe uma instância rodando, ignora esta chamada
            tracing::debug!("Background refresh já em execução, ignorando chamada duplicada");
            return Ok(());
        }
    };

    // Clone do app_handle para usar no spawn
    let app_clone = app.clone();

    // SPAWN: Isso garante que o Frontend continua fluido imediatamente
    tauri::async_runtime::spawn(async move {
        // Permit é dropped automaticamente ao final (RAII pattern)
        // Mesmo em caso de panic, o semaphore é liberado
        let _permit = permit;

        // Pequeno delay inicial para não competir com o boot do banco de dados
        sleep(Duration::from_secs(STARTUP_DELAY_SECS)).await;

        let state: State<AppState> = app_clone.state();

        // 1. Atualizar Reviews da Steam (Se cache > 7 dias)
        if let Err(e) = refresh_steam_reviews_background(&app_clone, &state).await {
            warn!("Falha ao atualizar reviews: {}", e);
        }

        // 2. Atualizar GamerPower (Se cache expirou)
        sleep(Duration::from_secs(BACKGROUND_TASK_INTERVAL_SECS)).await;

        if let Err(e) = refresh_gamerpower_background(&app_clone, &state).await {
            warn!("Falha ao atualizar GamerPower: {}", e);
        }

        // 3. Atualizar Preços da Wishlist (Se cache > 3 dias)
        sleep(Duration::from_secs(BACKGROUND_TASK_INTERVAL_SECS)).await;

        if let Err(e) = refresh_wishlist_prices_background(&app_clone, &state).await {
            warn!("Falha ao atualizar preços: {}", e);
        }

        // 4. Atualizar catálogo de jogos do Nexus (Se cache > 30 dias)
        sleep(Duration::from_secs(BACKGROUND_TASK_INTERVAL_SECS)).await;

        if let Err(e) = refresh_nexus_games_if_stale(&app_clone).await {
            warn!("Falha ao atualizar catálogo Nexus: {}", e);
        }

        // Avisa o frontend que o ciclo de background terminou
        let _ = app_clone.emit("background_refresh_complete", ());

        // _permit é automaticamente dropped aqui, liberando o semaphore
    });

    Ok(())
}

/// Atualiza reviews apenas se o cache estiver expirado.
/// Emite `reviews_refresh_complete` ao final (mesmo que nada tenha mudado).
async fn refresh_steam_reviews_background(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    // A. Ler IDs da Steam do banco (Leitura rápida)
    let steam_games: Vec<(u32, String)> = {
        let conn = state.games_db.lock().map_err(|_| "Falha DB Lock")?;

        conn.prepare("SELECT library_game_id, name FROM games WHERE library = 'Steam'")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    Ok((id_str.parse::<u32>().unwrap_or(0), name))
                })
                    .and_then(|mapped| mapped.collect::<Result<Vec<_>, _>>())
            })
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|(id, _)| *id > 0)
            .collect()
    };

    if steam_games.is_empty() {
        return Ok(());
    }

    let mut updated_count = 0;

    // B. Iterar jogos
    for (app_id, _title) in steam_games {
        let should_update = {
            match state.cache_db.lock() {
                Ok(cache_conn) => {
                    let cache_key = format!("reviews_{}", app_id);
                    // Verifica se o cache expirou
                    cache::get_cached_api_data(&cache_conn, "steam", &cache_key).is_none()
                }
                Err(_) => false, // Se erro ao acessar cache, pula atualização
            }
        };

        if should_update {
            let app_id_str = app_id.to_string();
            // C. Busca na API (Só se expirou)
            match get_app_reviews(&app_id_str).await {
                Ok(Some(summary)) => {
                    // D. Sucesso? Atualiza Library DB e Metadata Cache
                    {
                        // 1. Salva no Cache (para não buscar de novo por 7 dias)
                        if let Ok(cache_conn) = state.cache_db.lock() {
                            let cache_key = format!("reviews_{}", app_id);
                            if let Ok(json) = serde_json::to_string(&summary) {
                                let _ = cache::save_cached_api_data(
                                    &cache_conn,
                                    "steam",
                                    &cache_key,
                                    &json,
                                );
                            }
                        }
                    }

                    updated_count += 1;
                }
                Ok(None) => { /* Jogo não tem reviews ou erro 404, ignora */ }
                Err(e) => {
                    // E. Erro de API/Conexão? IGNORA. Mantém o dado velho.
                    warn!("Falha ao buscar review {}: {}", app_id, e);
                }
            }
            // Rate limit suave
            sleep(Duration::from_millis(200)).await;
        }
    }

    if updated_count > 0 {
        info!("{} avaliações atualizadas", updated_count);
        let _ = app.emit(
            "reviews_refresh_complete",
            format!(
                "{} avaliações Steam atualizadas em background.",
                updated_count
            ),
        );
    }

    Ok(())
}

/// Atualiza o cache do GamerPower apenas quando o cache estiver expirado.
async fn refresh_gamerpower_background(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let should_refresh = {
        let cache_conn = state.cache_db.lock().map_err(|_| "Falha DB Cache Lock")?;
        cache::get_cached_api_data(
            &cache_conn,
            GAMERPOWER_CACHE_SOURCE,
            GAMERPOWER_LIST_ACTIVE_CACHE_KEY,
        )
            .is_none()
    };

    if !should_refresh {
        return Ok(());
    }

    match gamerpower::fetch_giveaways(app).await {
        Ok(_) => {
            info!("Cache GamerPower atualizado em background");
            let _ = app.emit("gamerpower_refresh_complete", ());
            Ok(())
        }
        Err(e) => {
            warn!("Falha ao atualizar GamerPower em background: {}", e);
            Err(e)
        }
    }
}

/// Atualiza preços da Wishlist se o cache expirou
async fn refresh_wishlist_prices_background(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    // A. Ler Wishlist com itad_id
    let wishlist_items: Vec<(String, String, Option<String>)> = {
        let conn = state.games_db.lock().map_err(|_| "Falha DB")?;

        conn.prepare("SELECT id, name, itad_id FROM wishlist")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                })
                    .and_then(|mapped| mapped.collect::<Result<Vec<_>, _>>())
            })
            .map_err(|e| e.to_string())?
    };

    if wishlist_items.is_empty() {
        return Ok(());
    }

    // B. Coleta IDs da ITAD que precisam atualizar
    let mut itad_ids_to_fetch = Vec::new();
    let mut game_map = std::collections::HashMap::new();

    for (local_id, name, itad_id_opt) in wishlist_items {
        // Verifica se tem ITAD ID
        let itad_id = match itad_id_opt {
            Some(id) if !id.is_empty() => id,
            _ => {
                // Se não tem ITAD ID, tenta buscar
                match itad::find_game_id(&name).await {
                    Ok(found_id) => {
                        // Salva no banco para cachear
                        let conn = state.games_db.lock().unwrap();
                        let _ = conn.execute(
                            "UPDATE wishlist SET itad_id = ?1 WHERE id = ?2",
                            params![&found_id, &local_id],
                        );
                        found_id
                    }
                    Err(_) => {
                        continue; // Pula se não encontrou
                    }
                }
            }
        };

        // Verifica Cache
        let should_update = {
            let cache_conn = state.cache_db.lock().unwrap();
            let cache_key = format!("price_{}", itad_id);
            // Se não existe em cache ou expirou, precisa atualizar
            cache::get_cached_api_data(&cache_conn, "itad", &cache_key).is_none()
        };

        if should_update {
            itad_ids_to_fetch.push(itad_id.clone());
            game_map.insert(itad_id, (local_id, name));
        }
    }

    if itad_ids_to_fetch.is_empty() {
        return Ok(());
    }

    // C. Busca preços em lote da ITAD
    let region = get_or_detect_region(&app)?;
    let overviews = match itad::get_prices(itad_ids_to_fetch, &region).await {
        Ok(data) => data,
        Err(e) => {
            error!("Erro ao buscar preços da ITAD: {}", e);
            return Err(e);
        }
    };

    let mut updated_count = 0;

    // D. Atualiza banco e cache
    for game_data in overviews {
        if let Some((local_id, _game_name)) = game_map.get(&game_data.id) {
            // Salva no cache como um JSON simplificado
            {
                if let Ok(cache_conn) = state.cache_db.lock() {
                    let cache_key = format!("price_{}", game_data.id);

                    // Cria um JSON manual com os dados relevantes
                    let cache_data = serde_json::json!({
                        "id": game_data.id,
                        "current_price": game_data.current.as_ref().map(|d| d.price),
                        "currency": game_data.current.as_ref().map(|d| &d.currency),
                        "lowest_price": game_data.lowest.as_ref().map(|d| d.price),
                    });

                    let json = cache_data.to_string();
                    let _ = cache::save_cached_api_data(&cache_conn, "itad", &cache_key, &json);
                }
            }

            // Atualiza preços no banco de dados
            if let Some(deal) = game_data.current {
                let lowest = game_data.lowest.map(|l| l.price).unwrap_or(deal.price);

                let cut = deal.cut.unwrap_or(0) as f64;
                let normal_price = if cut > 0.0 {
                    deal.price / (1.0 - (cut / 100.0))
                } else {
                    deal.price
                };

                if let Ok(conn) = state.games_db.lock() {
                    match conn.execute(
                        "UPDATE wishlist SET
                            current_price = ?1,
                            currency = ?2,
                            lowest_price = ?3,
                            store = ?4,
                            store_url = ?5,
                            on_sale = ?6,
                            normal_price = ?7,
                            voucher = ?8
                         WHERE id = ?9",
                        params![
                            deal.price,
                            deal.currency,
                            lowest,
                            deal.shop.name,
                            deal.url,
                            deal.cut > Some(0),
                            normal_price,
                            deal.voucher,
                            local_id
                        ],
                    ) {
                        Ok(_) => updated_count += 1,
                        Err(e) => error!("Erro ao atualizar preço: {}", e),
                    }
                }
            }
        }
    }

    if updated_count > 0 {
        info!("{} preços atualizados", updated_count);
        let _ = app.emit(
            "wishlist_refresh_complete",
            format!(
                "{} preços da Wishlist atualizados em background.",
                updated_count
            ),
        );
    }

    Ok(())
}
