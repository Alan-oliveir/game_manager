use crate::database::AppState;
use crate::models::WishlistGame;
use crate::services::cheapshark;
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
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "INSERT OR REPLACE INTO wishlist (id, name, cover_url, store_url, current_price, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
        params![id, name, cover_url, store_url, current_price],
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
        .prepare("SELECT id, name, cover_url, store_url, current_price, lowest_price, on_sale, added_at FROM wishlist ORDER BY added_at DESC")
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
                added_at: row.get(7)?,
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
pub async fn refresh_prices(state: State<'_, AppState>) -> Result<String, String> {
    // Buscar todos os jogos da wishlist
    let games: Vec<(String, String)> = {
        let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
        let mut stmt = conn
            .prepare("SELECT id, name FROM wishlist")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let total = games.len();
    let mut updated_count = 0;

    // Para cada jogo, buscar preço na API
    for (id, name) in games {
        match cheapshark::find_best_price(&name).await {
            Ok(Some(deal)) => {
                let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

                // Atualiza no banco
                let _ = conn.execute(
                    "UPDATE wishlist
                     SET current_price = ?1, store_url = ?2, on_sale = ?3, lowest_price = MIN(IFNULL(lowest_price, 9999), ?1)
                     WHERE id = ?4",
                    params![deal.price, deal.url, deal.on_sale, id],
                );
                updated_count += 1;
            }
            Ok(None) => println!("Preço não encontrado para: {}", name),
            Err(e) => println!("Erro ao buscar {}: {}", name, e),
        }

        // Pausa para evitar rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    Ok(format!(
        "Preços atualizados para {} de {} jogos.",
        updated_count, total
    ))
}
