use crate::database::AppState;
use crate::services::steam::{self, StoreSearchItem};
use rusqlite::params;
use std::time::Duration;
use tauri::State;
use tokio::time::sleep;

#[tauri::command]
pub async fn search_wishlist_game(query: String) -> Result<Vec<StoreSearchItem>, String> {
    steam::search_store(&query).await
}

#[tauri::command]
pub async fn refresh_prices(state: State<'_, AppState>) -> Result<String, String> {
    // Busca dados básicos do banco
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
        let mut current_app_id = steam_app_id;

        // 1. Se não tem AppID, tenta descobrir pelo nome (Auto-healing)
        if current_app_id.is_none() {
            // Nota: search_store retorna lista, pegamos o primeiro para auto-healing
            if let Ok(results) = steam::search_store(&name).await {
                if let Some(first) = results.first() {
                    current_app_id = Some(first.id as i32);
                    let conn = state.db.lock().map_err(|_| "Falha mutex")?;
                    let _ = conn.execute(
                        "UPDATE wishlist SET steam_app_id = ?1 WHERE id = ?2",
                        params![current_app_id, &id],
                    );
                }
            }
        }

        // 2. Busca Preço na Steam
        if let Some(app_id_val) = current_app_id {
            match steam::fetch_price(app_id_val as u32).await {
                Ok(Some(price)) => {
                    let conn = state.db.lock().map_err(|_| "Falha mutex")?;
                    let on_sale = price.discount_percent > 0;

                    // URL da loja Steam para o botão "Ir para Loja"
                    let store_url = format!("https://store.steampowered.com/app/{}/", app_id_val);

                    // Atualiza BRL
                    let _ = conn.execute(
                        "UPDATE wishlist
                            SET localized_price = ?1, localized_currency = ?2,
                                on_sale = ?3, store_url = ?4,
                                lowest_price = MIN(IFNULL(lowest_price, 9999), ?1)
                            WHERE id = ?5",
                        params![price.final_price, price.currency, on_sale, store_url, id],
                    );
                    updated_count += 1;
                }
                Ok(None) => {
                    // Opcional: Aqui você poderia implementar o fallback para USD
                    // chamando steam::fetch_price_usd se o BRL falhar.
                    println!("Jogo indisponível na loja BR: {}", name);
                }
                Err(_) => {}
            }
        }

        // Rate limit simples para não tomar ban da Steam
        sleep(Duration::from_millis(500)).await;
    }

    Ok(format!("Preços atualizados: {}/{}", updated_count, total))
}
