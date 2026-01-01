use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

// Define o estado global da aplicação
pub struct AppState {
    pub db: Mutex<Connection>,
}

// Inicializa o banco de dados e cria as tabelas
#[tauri::command]
pub fn init_db(state: State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    // === TABELAS BÁSICAS ===

    // Tabela da Biblioteca de Jogos
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

    // Tabela da Lista de Desejos
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wishlist (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            cover_url TEXT,
            store_url TEXT,
            current_price REAL,
            lowest_price REAL,
            on_sale BOOLEAN DEFAULT 0,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Novas colunas (caso atualize o schema): ignoramos erro de coluna duplicada
    for stmt in [
        "ALTER TABLE wishlist ADD COLUMN localized_price REAL",
        "ALTER TABLE wishlist ADD COLUMN localized_currency TEXT",
        "ALTER TABLE wishlist ADD COLUMN steam_app_id INTEGER",
    ] {
        if let Err(err) = conn.execute(stmt, []) {
            let msg = err.to_string();
            if !msg.contains("duplicate column name") {
                return Err(msg);
            }
        }
    }

    // === ÍNDICES OTIMIZADOS ===

    // Índice para filtro de favoritos
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_favorite ON games(favorite)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Índice para busca case-insensitive por nome
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_name ON games(name COLLATE NOCASE)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Índice para filtro por plataforma
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_platform ON games(platform)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Índice para ordenação por data de adição na wishlist
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_wishlist_added ON wishlist(added_at)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok("Banco inicializado com sucesso!".to_string())
}
