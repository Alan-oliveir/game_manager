mod models;

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
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "INSERT INTO games (id, name, genre, platform) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, genre, platform],
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
            toggle_favorite
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
