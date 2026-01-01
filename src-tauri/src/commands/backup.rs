use crate::database::AppState;
use crate::models::{Game, WishlistGame};
use std::fs;
use tauri::{AppHandle, State};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BackupData {
    pub version: u32,
    pub date: String,
    pub games: Vec<Game>,
    pub wishlist_game: Vec<WishlistGame>
}

#[tauri::command]
pub async fn export_database(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String
) -> Result<(), String> {
    // Buscar dados em um único lock
    let (games, wishlist_game) = {
        let conn = state.db.lock().map_err(|_| "Falha no Mutex")?;

        // Inicia transação READ para consistência
        conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

        let games = fetch_games(&conn)?;
        let wishlist_game = fetch_wishlist(&conn)?;

        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

        (games, wishlist_game)
    }; // Lock liberado aqui

    let backup = BackupData {
        version: 1,
        date: chrono::Local::now().to_rfc3339(),
        games,
        wishlist_game,
    };

    let json = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    fs::write(file_path, json).map_err(|e| format!("Erro ao salvar arquivo: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn import_database(
    state: State<'_, AppState>,
    file_path: String
) -> Result<String, String> {
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|_| "Arquivo de backup inválido".to_string())?;

    // Validação de versão
    if backup.version != 1 {
        return Err(format!("Versão de backup incompatível: {}", backup.version));
    }

    let conn = state.db.lock().map_err(|_| "Falha no Mutex")?;

    // Transação única para todas as operações
    conn.execute("BEGIN IMMEDIATE TRANSACTION", []).map_err(|e| e.to_string())?;

    // Usa prepared statements para melhor performance
    let mut game_stmt = conn.prepare(
        "INSERT OR REPLACE INTO games (id, name, genre, platform, cover_url, playtime, rating, favorite)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    ).map_err(|e| e.to_string())?;

    let mut wishlist_stmt = conn.prepare(
        "INSERT OR REPLACE INTO wishlist (id, name, cover_url, store_url, current_price, lowest_price, on_sale, localized_price, localized_currency, steam_app_id, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    ).map_err(|e| e.to_string())?;

    for game in &backup.games {
        game_stmt.execute(rusqlite::params![
            game.id, game.name, game.genre, game.platform, game.cover_url,
            game.playtime, game.rating, game.favorite
        ]).map_err(|e| e.to_string())?;
    }

    for item in &backup.wishlist_game {
        wishlist_stmt.execute(rusqlite::params![
            item.id, item.name, item.cover_url, item.store_url,
            item.current_price, item.lowest_price, item.on_sale,
            item.localized_price, item.localized_currency,
            item.steam_app_id, item.added_at
        ]).map_err(|e| e.to_string())?;
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(format!("Backup restaurado! {} jogos e {} itens da lista de desejos.",
               backup.games.len(), backup.wishlist_game.len()))
}

// Funções auxiliares
fn fetch_games(conn: &rusqlite::Connection) -> Result<Vec<Game>, String> {
    let mut stmt = conn.prepare("SELECT * FROM games").map_err(|e| e.to_string())?;
    let game_iter = stmt.query_map([], |row| {
        Ok(Game {
            id: row.get("id")?,
            name: row.get("name")?,
            genre: row.get("genre")?,
            platform: row.get("platform")?,
            cover_url: row.get("cover_url")?,
            playtime: row.get("playtime")?,
            rating: row.get("rating").unwrap_or(None),
            favorite: row.get("favorite").unwrap_or(false),
        })
    }).map_err(|e| e.to_string())?;

    game_iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn fetch_wishlist(conn: &rusqlite::Connection) -> Result<Vec<WishlistGame>, String> {
    let mut stmt = conn.prepare("SELECT * FROM wishlist").map_err(|e| e.to_string())?;
    let wishlist_iter = stmt.query_map([], |row| {
        Ok(WishlistGame {
            id: row.get("id")?,
            name: row.get("name")?,
            cover_url: row.get("cover_url")?,
            store_url: row.get("store_url")?,
            current_price: row.get("current_price")?,
            lowest_price: row.get("lowest_price")?,
            on_sale: row.get("on_sale")?,
            localized_price: row.get("localized_price")?,
            localized_currency: row.get("localized_currency")?,
            steam_app_id: row.get("steam_app_id")?,
            added_at: row.get("added_at")?,
        })
    }).map_err(|e| e.to_string())?;

    wishlist_iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
