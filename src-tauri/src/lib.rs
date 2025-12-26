mod models;
mod steam_service;

use rusqlite::{params, Connection};
use std::sync::Mutex;
use tauri::State;

// Estado da aplicação
pub struct AppState {
    db: Mutex<Connection>,
}

// Comando para inicializar o banco
#[tauri::command]
fn init_db(state: State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            genre TEXT,
            platform TEXT,
            cover_url TEXT,
            playtime INTEGER DEFAULT 0,
            rating INTEGER,
            favorite BOOLEAN DEFAULT FALSE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok("Banco inicializado com sucesso!".to_string())
}

// Comando para adicionar um jogo
#[tauri::command]
fn add_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "INSERT INTO games (id, name, genre, platform, cover_url, playtime, rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, genre, platform, cover_url, playtime, rating],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para listar todos os jogos
#[tauri::command]
fn get_games(state: State<AppState>) -> Result<Vec<models::Game>, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, genre, platform, cover_url, playtime, rating, favorite FROM games",
        )
        .map_err(|e| e.to_string())?;

    let games = stmt
        .query_map([], |row| {
            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                genre: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                playtime: row.get(5)?,
                rating: row.get(6)?,
                favorite: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(games)
}

// Comando para marcar/desmarcar favorito
#[tauri::command]
fn toggle_favorite(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET favorite = NOT favorite WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para deletar um jogo
#[tauri::command]
fn delete_game(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute("DELETE FROM games WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para atualizar informações de um jogo
#[tauri::command]
fn update_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET name = ?1, genre = ?2, platform = ?3, cover_url = ?4, playtime = ?5, rating = ?6 WHERE id = ?7",
        params![name, genre, platform, cover_url, playtime, rating, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para importar jogos da biblioteca Steam
#[tauri::command]
async fn import_steam_library(
    state: State<'_, AppState>,
    api_key: String,
    steam_id: String,
) -> Result<String, String> {
    // Busca os jogos na API da Steam
    let steam_games = steam_service::list_steam_games(&api_key, &steam_id).await?;

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut count = 0;

    // Salva cada jogo no banco (se não existir)
    for game in steam_games {
        // Monta a URL da capa (formato padrão da Steam)
        let cover_url = format!(
            "https://steamcdn-a.akamaihd.net/steam/apps/{}/library_600x900_2x.jpg",
            game.appid
        );

        // Tenta inserir. O "OR IGNORE" pula se o ID já existir.
        let result = conn.execute(
            "INSERT OR IGNORE INTO games (id, name, genre, platform, cover_url, playtime, rating)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                game.appid.to_string(), // Usamos o ID da Steam como ID do jogo
                game.name,
                "Desconhecido", // API básica não dá gênero do jogo
                "Steam",
                cover_url,
                game.playtime_forever / 60, // Converte minutos para horas
                None::<i32>                 // Sem avaliação inicial
            ],
        );

        if let Ok(rows) = result {
            count += rows;
        }
    }

    Ok(format!(
        "Importação concluída! {} novos jogos adicionados.",
        count
    ))
}

// Função principal que configura o Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializa conexão com arquivo local
    let conn = Connection::open("library.db").expect("Erro ao abrir banco");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            init_db,
            add_game,
            get_games,
            toggle_favorite,
            delete_game,
            update_game,
            import_steam_library
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
