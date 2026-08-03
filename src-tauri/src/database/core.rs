//! Módulo de gerenciamento do banco de dados da aplicação.
//!
//! Gerencia a criação e inicialização do banco SQLite para a biblioteca de jogos e wishlist,
//! além do armazenamento seguro de secrets (API keys, tokens) com criptografia.
//!
//! **Bancos de Dados:**
//! - games.db: armazena jogos, wishlist, dados técnicos (pcgw_data) e subscriptions.
//! - secrets.db: armazena secrets encriptados com AES-256-GCM.
//! - cache.db: cache para respostas de APIs externas (RAWG, Steam).

use crate::constants::{
    DB_FILENAME_CACHE, DB_FILENAME_GAMES, DB_FILENAME_SECRETS, DB_JOURNAL_MODE,
};
use crate::errors::AppError;
use crate::security;
use crate::services::integration::pcgamingwiki::db::initialize_pcgamingwiki_tables;
use crate::services::playtime::PlaytimeRegistry;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use tauri::State;
use tauri::{AppHandle, Manager};

/// Define o estado global da aplicação com ambas as conexões
pub struct AppState {
    pub games_db: Mutex<Connection>,
    pub secrets_db: Mutex<Connection>,
    pub cache_db: Mutex<Connection>,
    pub playtime_registry: PlaytimeRegistry,
}

/// Retorna a versão atual do schema armazenada no banco
pub fn current_schema_version(conn: &Connection) -> Result<u32, AppError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(version.max(0) as u32)
}

/// Retorna a versão do schema esperada para esta versão do app
pub fn expected_schema_version(app: &AppHandle) -> u32 {
    // Usa o MAJOR da versão do app
    let version = app.package_info().version.clone();
    version.major as u32
}

// === INICIALIZAÇÃO CENTRALIZADA ===

/// Inicializa ambos os bancos de dados e retorna o estado da aplicação
///
/// **Erros:**
/// - Se não conseguir criar os diretórios
/// - Se não conseguir abrir as conexões
/// - Se falhar ao configurar WAL mode
pub fn initialize_databases(app: &AppHandle) -> Result<AppState, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Falha ao obter app_data_dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Falha ao criar diretório: {}", e))?;

    // Conexão para games.db
    let games_path = app_data_dir.join(DB_FILENAME_GAMES);
    let games_conn = Connection::open(&games_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_GAMES, e))?;

    games_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| format!("Erro ao configurar WAL no library.db: {}", e))?;

    // Conexão para cache.db
    let cache_path = app_data_dir.join(DB_FILENAME_CACHE);
    let cache_conn = Connection::open(&cache_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_CACHE, e))?;

    cache_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| {
            AppError::DatabaseWalConfigError("cache.db".to_string(), e.to_string()).to_string()
        })?;

    // Inicializa schema do cache
    crate::services::cache::initialize_cache_db(&cache_conn)?;

    // Conexão para secrets.db
    let secrets_path = app_data_dir.join(DB_FILENAME_SECRETS);
    let secrets_conn = Connection::open(&secrets_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_SECRETS, e))?;

    secrets_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| format!("Erro ao configurar WAL no secrets.db: {}", e))?;

    // Executa migrations
    crate::database::migrations::run_migrations(app, &games_conn)?;

    // Cria schema completo
    let schema_version = app.package_info().version.major as u32;
    create_schema(&games_conn, schema_version)?;

    Ok(AppState {
        games_db: Mutex::new(games_conn),
        secrets_db: Mutex::new(secrets_conn),
        cache_db: Mutex::new(cache_conn),
        playtime_registry: PlaytimeRegistry::default(),
    })
}

// === BANCO DE DADOS DE GERENCIAMENTO DE BIBLIOTECAS E WISHLIST  ===

/// Cria o schema completo do banco de dados (versão v4)
///
/// **Schema v4:**
/// - Campos HLTB removidos
/// - URLs legadas removidas (agora em external_links JSON)
/// - users_score removido (substituído por steam_review_*)
/// - Adicionadas as tabelas:
///     - subscriptions
///     - game_extras (detalhes técnicos)
///     - game_data_paths
///     - system_requirements
fn create_schema(conn: &Connection, schema_version: u32) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            cover_url TEXT,
            platform TEXT NOT NULL,
            platform_game_id TEXT NOT NULL,
            installed BOOLEAN DEFAULT 0,
            import_confidence TEXT,
            install_path TEXT,
            executable_path TEXT,
            launch_args TEXT,
            user_rating INTEGER,
            favorite BOOLEAN DEFAULT 0,
            status TEXT,
            playtime INTEGER,
            playtime_source TEXT,
            last_played TEXT,
            added_at TEXT NOT NULL
        )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_details (
            game_id TEXT PRIMARY KEY,
            steam_app_id TEXT,
            developer TEXT,
            publisher TEXT,
            release_date TEXT,
            genres TEXT,
            tags TEXT,
            series TEXT,
            description_raw TEXT,
            description_ptbr TEXT,
            background_image TEXT,
            critic_score INTEGER,
            steam_review_label TEXT,
            steam_review_count INTEGER,
            steam_review_score REAL,
            steam_review_updated_at TEXT,
            esrb_rating TEXT,
            is_adult BOOLEAN DEFAULT 0,
            adult_tags TEXT,
            external_links TEXT,
            median_playtime INTEGER,
            estimated_playtime REAL,
            FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
        )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wishlist (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            cover_url TEXT,
            store_url TEXT,
            store_platform TEXT,
            current_price REAL,
            normal_price REAL,
            lowest_price REAL,
            currency TEXT,
            on_sale BOOLEAN DEFAULT 0,
            voucher TEXT,
            added_at TEXT,
            itad_id TEXT
        )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS subscriptions (
        service TEXT PRIMARY KEY,   -- 'prime_gaming', 'game_pass', etc.
        enabled BOOLEAN DEFAULT 0,
        last_synced TEXT            -- ISO timestamp do último fetch
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    // Índices
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_name ON games(name COLLATE NOCASE)",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_platform ON games(platform)",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_favorite ON games(favorite)",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_status ON games(status)", [])
        .map_err(|e| e.to_string())?;

    // Tabelas extras - PCGamingWiki e scrapers relacionados
    initialize_pcgamingwiki_tables(conn).map_err(|e| e.to_string())?;

    // Marca versão do schema
    conn.pragma_update(None, "user_version", schema_version)
        .map_err(|e| format!("Erro ao definir versão do schema: {}", e))?;

    Ok(())
}

/// Inicializa o banco de dados e verifica a versão do schema.
///
/// Se o banco estiver desatualizado, retorna erro com instruções para o usuário.
#[tauri::command]
pub fn init_db(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let conn = state
        .games_db
        .lock()
        .map_err(|_| "Falha ao bloquear mutex do games_db")?;

    let current_version = current_schema_version(&conn).unwrap_or(0) as i32;
    let expected_version = expected_schema_version(&app) as i32;

    if current_version == 0 {
        let schema_version = expected_schema_version(&app);
        return Ok(format!("Banco de dados novo criado (v{})", schema_version));
    }

    if current_version != expected_version {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Falha ao obter app_data_dir: {}", e))?;

        return Err(format!(
            "Banco desatualizado: Schema atual: v{}, esperado: v{}. Faça backup, exclua o diretório da aplicação em: {:?} e reinicie para recriar o banco.",
            current_version, expected_version, app_data_dir
        ));
    }

    Ok(format!("Banco de dados OK (v{})", current_version))
}

/// Serializa tags para salvar no banco
pub fn serialize_tags(tags: &[crate::models::GameTag]) -> Result<String, String> {
    serde_json::to_string(tags).map_err(|e| e.to_string())
}

/// Deserializa tags do banco (com fallback para formato antigo)
pub fn deserialize_tags(tags_json: &str) -> Vec<crate::models::GameTag> {
    use crate::utils::tag_utils::{TagCategory, TagRole};

    // Tenta deserializar como novo formato
    if let Ok(tags) = serde_json::from_str::<Vec<crate::models::GameTag>>(tags_json) {
        return tags;
    }

    // Fallback: formato antigo (string separada por vírgulas)
    tags_json
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|slug| crate::models::GameTag {
            slug: slug.to_string(),
            name: slug.to_string(),
            category: TagCategory::Meta,
            role: TagRole::Context,
            relevance: 5.0,
        })
        .collect()
}

// === BANCO DE DADOS PARA GERENCIAMENTO DE API KEYS (secrets.db) ===

/// Obtém conexão com o banco de secrets a partir do AppState.
/// Cria automaticamente a tabela `encrypted_keys` se não existir.
fn get_secrets_connection<'a>(
    state: &'a State<AppState>,
) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    let conn = state
        .secrets_db
        .lock()
        .map_err(|_| "Falha ao bloquear mutex do secrets_db".to_string())?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS encrypted_keys (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
        [],
    )
        .map_err(|e: rusqlite::Error| e.to_string())?;

    Ok(conn)
}

/// Salva um secret encriptado no banco. Se a chave já existir, o valor é substituído (upsert).
pub fn set_secret(app: &AppHandle, key_name: &str, value: &str) -> Result<(), AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = get_secrets_connection(&state)?;

    let encrypted = security::encrypt(app, value)?;

    conn.execute(
        "INSERT OR REPLACE INTO encrypted_keys (key, value) VALUES (?1, ?2)",
        params![key_name, encrypted],
    )?;

    Ok(())
}

/// Recupera e decripta um secret do banco. Se a chave não existir, retorna string vazia ao invés de erro.
pub fn get_secret(app: &AppHandle, key_name: &str) -> Result<String, AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = get_secrets_connection(&state)?;

    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM encrypted_keys WHERE key = ?1",
        params![key_name],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(encrypted) => {
            let decrypted = security::decrypt(app, &encrypted)?;
            Ok(decrypted)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

/// Remove um secret do banco permanentemente.
pub fn delete_secret(app: &AppHandle, key_name: &str) -> Result<(), AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = get_secrets_connection(&state)?;

    conn.execute(
        "DELETE FROM encrypted_keys WHERE key = ?1",
        params![key_name],
    )?;

    Ok(())
}

/// Retorna lista de chaves de secrets suportadas pela aplicação.
pub fn list_supported_keys() -> Vec<&'static str> {
    vec![
        "steam_id",
        "steam_api_key",
        "rawg_api_key",
        "gemini_api_key",
        "gamebrain_api_key",
        "nexus_api_key",
    ]
}
