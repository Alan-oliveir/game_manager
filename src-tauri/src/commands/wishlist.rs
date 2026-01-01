use crate::database::AppState;
use crate::models::WishlistGame;
use crate::services::{cheapshark, steam};
use rusqlite::params;
use std::time::Duration;
use tauri::State;
use tokio::time::sleep;

#[tauri::command]
pub fn add_to_wishlist(
    state: State<AppState>,
    id: String,
    name: String,
    cover_url: Option<String>,
    store_url: Option<String>,
    current_price: Option<f64>,
    steam_app_id: Option<i32>,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "INSERT OR REPLACE INTO wishlist (id, name, cover_url, store_url, current_price, steam_app_id, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
        params![id, name, cover_url, store_url, current_price, steam_app_id],
    )
    .map_err(|e| e.to_string())?;

    Ok("Jogo adicionado à lista de desejos!".to_string())
}

#[tauri::command]
pub fn remove_from_wishlist(state: State<AppState>, id: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute("DELETE FROM wishlist WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok("Jogo removido da lista de desejos.".to_string())
}

#[tauri::command]
pub fn get_wishlist(state: State<AppState>) -> Result<Vec<WishlistGame>, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut stmt = conn
        .prepare("SELECT id, name, cover_url, store_url, current_price, lowest_price, on_sale, localized_price, localized_currency, steam_app_id, added_at FROM wishlist ORDER BY added_at DESC")
        .map_err(|e| e.to_string())?;

    let games_iter = stmt
        .query_map([], |row| {
            Ok(WishlistGame {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_url: row.get(2)?,
                store_url: row.get(3)?,
                current_price: row.get(4)?,
                lowest_price: row.get(5)?,
                on_sale: row.get(6)?,
                localized_price: row.get(7)?,
                localized_currency: row.get(8)?,
                steam_app_id: row.get(9)?,
                added_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut games = Vec::new();
    for game in games_iter {
        games.push(game.map_err(|e| e.to_string())?);
    }

    Ok(games)
}

#[tauri::command]
pub fn check_wishlist_status(state: State<AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(1) FROM wishlist WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(count > 0)
}

#[tauri::command]
pub async fn search_steam_app_id(game_name: String) -> Result<Option<u32>, String> {
    steam::search_steam_app_id(&game_name).await
}

#[tauri::command]
pub async fn refresh_prices(state: State<'_, AppState>) -> Result<String, String> {
    // Buscar todos os jogos da wishlist
    let games: Vec<(String, Option<i32>, String)> = {
        let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
        let mut stmt = conn
            .prepare("SELECT id, steam_app_id, name FROM wishlist")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let total = games.len();
    let mut updated_count = 0;

    for (id, steam_app_id, name) in games {
        let mut price_found = false;
        let mut current_app_id = steam_app_id;

        // 1. TENTATIVA DE BUSCAR ID SE NÃO TIVER
        if current_app_id.is_none() {
            match steam::search_steam_app_id(&name).await {
                Ok(Some(found_id)) => {
                    current_app_id = Some(found_id as i32);
                    // Salva o ID encontrado
                    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
                    let _ = conn.execute(
                        "UPDATE wishlist SET steam_app_id = ?1 WHERE id = ?2",
                        params![current_app_id, &id],
                    );
                    println!(
                        "Steam App ID encontrado via Busca para {}: {}",
                        name, found_id
                    );
                }
                _ => println!(
                    "Busca Steam falhou para {}, tentaremos via CheapShark...",
                    name
                ),
            }
        }

        // 2. BUSCA NO CHEAPSHARK (Seja para preço ou para tentar achar o ID)
        let cheapshark_deal = match cheapshark::find_best_price(&name).await {
            Ok(Some(deal)) => {
                if current_app_id.is_none() && deal.steam_app_id.is_some() {
                    current_app_id = deal.steam_app_id.map(|v| v as i32);
                    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
                    let _ = conn.execute(
                        "UPDATE wishlist SET steam_app_id = ?1 WHERE id = ?2",
                        params![current_app_id, &id],
                    );
                    println!(
                        "Steam App ID descoberto via CheapShark para {}: {:?}",
                        name, current_app_id
                    );
                }

                // Salva o preço em Dólar como fallback
                let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
                let _ = conn.execute(
                    "UPDATE wishlist
                        SET current_price = ?1, store_url = ?2, on_sale = ?3,
                            lowest_price = MIN(IFNULL(lowest_price, 9999), ?1)
                        WHERE id = ?4",
                    params![deal.price, deal.url, deal.on_sale, id],
                );
                Some(deal)
            }
            _ => None,
        };

        // 3. BUSCA PREÇO NA STEAM (EM REAIS)
        if let Some(app_id_value) = current_app_id {
            match steam::fetch_price(app_id_value as u32).await {
                Ok(Some(steam_price)) => {
                    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
                    let on_sale = steam_price.discount_percent > 0;

                    // Atualiza com preço da Steam em reais (PRIORIDADE NA VISUALIZAÇÃO)
                    let _ = conn.execute(
                        "UPDATE wishlist
                            SET localized_price = ?1, localized_currency = ?2, on_sale = ?3,
                                lowest_price = MIN(IFNULL(lowest_price, 9999), ?1)
                            WHERE id = ?4",
                        params![steam_price.final_price, steam_price.currency, on_sale, id],
                    );
                    updated_count += 1;
                    price_found = true;
                    println!(
                        "Preço BRL atualizado para {}: R$ {}",
                        name, steam_price.final_price
                    );
                }
                Ok(None) => println!("Preço Steam BRL não disponível para {}", name),
                Err(e) => println!("Erro Steam API para {}: {}", name, e),
            }
        }

        if !price_found && cheapshark_deal.is_some() {
            updated_count += 1;
            println!("Preço atualizado apenas via CheapShark (USD) para {}", name);
        }

        sleep(Duration::from_millis(500)).await;
    }

    Ok(format!(
        "Preços atualizados para {} de {} jogos.",
        updated_count, total
    ))
}
