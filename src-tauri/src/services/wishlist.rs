//! Serviço de gerenciamento da wishlist.
//!
//! Orquestra: resolve IDs ITAD (com cache local), busca preços em lote,
//! calcula preço normal a partir do desconto, e persiste os resultados.

use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::metadata::igdb::client::igdb_request;
use crate::providers::metadata::igdb::models::IgdbCover;
use crate::providers::pricing::itad;
use crate::services::locale::get_or_detect_region;
use rusqlite::params;
use serde::Deserialize;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{error, info};

// === STRUCTS ===

#[derive(Debug, Deserialize)]
struct IgdbSearchRaw {
    id: i64,
    name: String,
    slug: String,
    cover: Option<IgdbCover>,
}

#[derive(Debug, Clone)]
pub struct IgdbSearchResult {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub cover_url: Option<String>,
}

/// Busca todos os jogos da wishlist local (id, nome, itad_id).
fn fetch_wishlist_games(
    state: &State<AppState>,
) -> Result<Vec<(String, String, Option<String>)>, AppError> {
    let conn = state.games_db.lock()?;
    let mut stmt = conn.prepare("SELECT id, name, itad_id FROM wishlist")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Resolve o ID ITAD de cada jogo (usa o cache local se existir, senão
/// busca na API e persiste pra não repetir a busca da próxima vez).
async fn resolve_itad_ids(
    state: &State<'_, AppState>,
    games: Vec<(String, String, Option<String>)>,
) -> Result<(Vec<String>, HashMap<String, (String, String)>), AppError> {
    let mut itad_ids = Vec::new();
    let mut game_map = HashMap::new(); // itad_id -> (local_id, name)

    for (local_id, name, current_itad_id) in games {
        let final_itad_id = match current_itad_id {
            Some(id) if !id.is_empty() => id,
            _ => match itad::find_game_id(&name).await {
                Ok(found_id) => {
                    let conn = state.games_db.lock()?;
                    let _ = conn.execute(
                        "UPDATE wishlist SET itad_id = ?1 WHERE id = ?2",
                        params![&found_id, &local_id],
                    );
                    found_id
                }
                Err(e) => {
                    error!("Jogo '{}' não encontrado na ITAD: {}", name, e);
                    continue;
                }
            },
        };
        itad_ids.push(final_itad_id.clone());
        game_map.insert(final_itad_id, (local_id, name));
    }

    Ok((itad_ids, game_map))
}

/// Persiste os preços retornados pela ITAD, calculando o preço normal
/// a partir do desconto quando aplicável.
fn persist_prices(
    conn: &rusqlite::Connection,
    overviews: Vec<itad::ItadGameOverview>,
    game_map: &HashMap<String, (String, String)>,
) -> u32 {
    let mut updated_count = 0;

    for game_data in overviews {
        let Some((local_id, _game_name)) = game_map.get(&game_data.id) else {
            error!("ITAD ID {} não encontrado no mapa local", game_data.id);
            continue;
        };

        let Some(deal) = game_data.current else {
            continue;
        };

        let lowest = game_data.lowest.map(|l| l.price).unwrap_or(deal.price);
        let cut = deal.cut.unwrap_or(0) as f64;
        let normal_price = if cut > 0.0 {
            deal.price / (1.0 - (cut / 100.0))
        } else {
            deal.price
        };

        match conn.execute(
            "UPDATE wishlist SET
                current_price = ?1,
                currency = ?2,
                lowest_price = ?3,
                store_platform = ?4,
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
            Err(e) => error!("Erro ao salvar preço: {}", e),
        }
    }

    updated_count
}

/// Atualiza os preços de todos os jogos na wishlist usando a API da ITAD,
/// respeitando a região configurada (ver `database::configs::get_or_detect_region`).
pub async fn refresh_prices(app: &AppHandle) -> Result<String, AppError> {
    let state: State<AppState> = app.state();

    let games_to_check = fetch_wishlist_games(&state)?;
    if games_to_check.is_empty() {
        return Ok("Lista de desejos vazia.".to_string());
    }

    let (itad_ids, game_map) = resolve_itad_ids(&state, games_to_check).await?;
    if itad_ids.is_empty() {
        return Ok("Nenhum jogo correspondente encontrado na ITAD.".to_string());
    }

    let region = get_or_detect_region(app)?;
    let overviews = itad::get_prices(itad_ids, &region)
        .await
        .map_err(AppError::NetworkError)?;

    let updated_count = {
        let conn = state.games_db.lock()?;
        persist_prices(&conn, overviews, &game_map)
    };

    if updated_count > 0 {
        info!("{} preços atualizados", updated_count);
    }

    Ok(format!("{} preços atualizados", updated_count))
}

fn cover_url(cover: Option<IgdbCover>) -> Option<String> {
    cover.map(|c| {
        format!(
            "https://images.igdb.com/igdb/image/upload/t_1080p/{}.jpg",
            c.image_id
        )
    })
}

/// Busca jogos por nome na IGDB. Usado pela busca manual e pelo preenchimento
/// de capas faltantes (`fill_missing_covers`).
pub async fn search_games(app: &AppHandle, query: String) -> Result<Vec<IgdbSearchResult>, String> {
    let sanitized = query.replace('"', "\\\"");
    let search_query = format!(
        "search \"{sanitized}\"; fields name, slug, cover.image_id; where game_type = 0; limit 10;"
    );

    let body = igdb_request(app, "games", &search_query).await?;
    let raw: Vec<IgdbSearchRaw> = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    Ok(raw
        .into_iter()
        .map(|g| IgdbSearchResult {
            id: g.id,
            name: g.name,
            slug: g.slug,
            cover_url: cover_url(g.cover),
        })
        .collect())
}

/// Busca no banco quais jogos da wishlist estão sem capa.
fn fetch_games_missing_covers(state: &State<AppState>) -> Result<Vec<(String, String)>, AppError> {
    let conn = state.games_db.lock()?;
    let mut stmt =
        conn.prepare("SELECT id, name FROM wishlist WHERE cover_url IS NULL OR cover_url = ''")?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Preenche capas faltantes na wishlist buscando na IGDB, uma por uma.
/// Emite o evento `wishlist_updated` no frontend ao final, se algo mudou.
async fn fill_missing_covers(app: &AppHandle) -> Result<(), AppError> {
    let state: State<AppState> = app.state();

    let missing_covers = fetch_games_missing_covers(&state)?;
    if missing_covers.is_empty() {
        return Ok(());
    }

    let mut updated_count = 0;

    for (id, name) in missing_covers {
        match search_games(app, name.clone()).await {
            Ok(results) => {
                if let Some(first_match) = results.iter().find(|g| g.cover_url.is_some()) {
                    if let Some(cover) = &first_match.cover_url {
                        let conn = state.games_db.lock()?;
                        if conn
                            .execute(
                                "UPDATE wishlist SET cover_url = ?1 WHERE id = ?2",
                                params![cover, id],
                            )
                            .is_ok()
                        {
                            updated_count += 1;
                        }
                    }
                }
            }
            Err(e) => error!("Erro IGDB para '{}': {}", name, e),
        }
    }

    if updated_count > 0 {
        info!("{} capas atualizadas", updated_count);
    }

    let _ = app.emit("wishlist_updated", ());
    Ok(())
}

/// Dispara o preenchimento de capas faltantes em background (não bloqueia
/// o command que chamou). Erros são logados internamente, não propagados.
pub fn spawn_fill_missing_covers(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = fill_missing_covers(&app).await {
            error!("Erro ao buscar capas faltantes: {}", e);
        }
    });
}
