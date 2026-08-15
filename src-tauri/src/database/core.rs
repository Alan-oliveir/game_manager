//! Módulo de gerenciamento do banco de dados da aplicação.
//!
//! Gerencia a criação e inicialização do banco SQLite para a biblioteca de jogos e wishlist,
//! além do armazenamento seguro de secrets (API keys, tokens) com criptografia.
//!
//! **Bancos de Dados:**
//! - games.db: armazena jogos, wishlist, dados técnicos (pcgw_data) e subscriptions.
//! - secrets.db: armazena secrets encriptados com AES-256-GCM.
//! - cache.db: cache para respostas de APIs externas (RAWG, Steam).
//! - config.db: configurações da aplicação.

use crate::constants::{
    DB_FILENAME_CACHE, DB_FILENAME_CONFIG, DB_FILENAME_GAMES, DB_FILENAME_SECRETS, DB_JOURNAL_MODE,
};
use crate::errors::AppError;
use crate::providers::mods::nexus::initialize_nexus_tables;
use crate::providers::technical::pcgamingwiki::db::initialize_pcgamingwiki_tables;
use crate::services::playtime::PlaytimeRegistry;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;
use tauri::{AppHandle, Manager};

/// Define o estado global da aplicação com ambas as conexões
pub struct AppState {
    pub games_db: Mutex<Connection>,
    pub secrets_db: Mutex<Connection>,
    pub cache_db: Mutex<Connection>,
    pub config_db: Mutex<Connection>,
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

    // Conexão para config.db
    let config_path = app_data_dir.join(DB_FILENAME_CONFIG);
    let config_conn = Connection::open(&config_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_CONFIG, e))?;

    config_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| {
            AppError::DatabaseWalConfigError("config.db".to_string(), e.to_string()).to_string()
        })?;

    crate::database::configs::ensure_config_table(&config_conn)?;

    // Executa migrations
    crate::database::migrations::run_migrations(app, &games_conn)?;

    // Cria schema completo
    let schema_version = app.package_info().version.major as u32;
    create_schema(&games_conn, schema_version)?;

    Ok(AppState {
        games_db: Mutex::new(games_conn),
        secrets_db: Mutex::new(secrets_conn),
        cache_db: Mutex::new(cache_conn),
        config_db: Mutex::new(config_conn),
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
            slug TEXT NOT NULL DEFAULT '',
            cover_url TEXT,
            platform TEXT NOT NULL,
            source_label TEXT,
            platform_game_id TEXT NOT NULL,
            alternative_names TEXT,
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
            hltb_main_story REAL,
            hltb_main_extra REAL,
            hltb_completionist REAL,
            hltb_coop_time REAL,
            franchise TEXT,
            game_modes TEXT,
            player_perspectives TEXT,
            themes TEXT,
            keywords TEXT,
            age_ratings TEXT,
            display_name TEXT,
            updated_at TEXT,
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_dlcs (
        game_id TEXT NOT NULL,     -- FK para games.id (jogo base na biblioteca)
        igdb_id INTEGER NOT NULL,  -- id do DLC/expansion no IGDB
        name TEXT NOT NULL,
        slug TEXT,
        cover_image_id TEXT,
        kind TEXT NOT NULL,        -- 'expansion' | 'standalone_expansion'
        owned INTEGER NOT NULL DEFAULT 0, -- se o standalone já foi importado como jogo próprio
        PRIMARY KEY (game_id, igdb_id)
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_descriptions (
        game_id TEXT PRIMARY KEY,
        summary TEXT,
        storyline TEXT,
        short_description TEXT,
        description TEXT,
        summary_translated TEXT,
        storyline_translated TEXT,
        short_description_translated TEXT,
        description_translated TEXT,
        translated_lang TEXT,
        FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_sources (
        id TEXT PRIMARY KEY,
        folder_path TEXT NOT NULL UNIQUE,
        label TEXT NOT NULL,
        created_at TEXT NOT NULL,
        last_scanned_at TEXT
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    // Metadados do dataset inteiro (controla o TTL global, não por jogo). Ex: etag, última atualização, contagem de jogos
    conn.execute(
        "CREATE TABLE IF NOT EXISTS anticheat_meta (
        id INTEGER PRIMARY KEY CHECK (id = 1), -- singleton row
        etag TEXT,
        last_fetched TEXT NOT NULL,
        game_count INTEGER NOT NULL DEFAULT 0
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    // Snapshot local do games.json do AWACY
    conn.execute(
        "CREATE TABLE IF NOT EXISTS anticheat_games (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        slug TEXT NOT NULL,
        name TEXT NOT NULL,
        status TEXT NOT NULL,
        anticheats TEXT NOT NULL,
        steam_id TEXT,
        epic_namespace TEXT,
        epic_slug TEXT,
        native INTEGER NOT NULL DEFAULT 0,
        reference TEXT,
        date_changed TEXT
    )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS achievements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT NOT NULL,
            game_id TEXT NOT NULL,
            game_name TEXT NOT NULL,
            achievement_key TEXT NOT NULL,
            achievement_name TEXT NOT NULL,
            achievement_description TEXT,
            unlocked_at INTEGER NOT NULL,
            icon_url TEXT,
            UNIQUE(platform, game_id, achievement_key)
        )",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_achievements_unlocked_at ON achievements(unlocked_at DESC)",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS achievement_sync_state (
            platform TEXT NOT NULL,
            game_id TEXT NOT NULL,
            last_synced_at INTEGER NOT NULL,
            has_achievements INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (platform, game_id)
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

    conn.execute("CREATE INDEX IF NOT EXISTS idx_slug ON games(slug)", [])
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_anticheat_slug ON anticheat_games(slug)",
        [],
    )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_anticheat_steam ON anticheat_games(steam_id)",
        [],
    )
        .map_err(|e| e.to_string())?;

    // Tabelas extras - PCGamingWiki, Nexus e relacionadas
    initialize_pcgamingwiki_tables(conn).map_err(|e| e.to_string())?;
    initialize_nexus_tables(conn).map_err(|e| e.to_string())?;

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
