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
use crate::services::playtime::PlaytimeRegistry;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Define o estado global da aplicação com ambas as conexões
pub struct AppState {
    pub games_db: Mutex<Connection>,
    pub secrets_db: Mutex<Connection>,
    pub cache_db: Mutex<Connection>,
    pub config_db: Mutex<Connection>,
    pub playtime_registry: PlaytimeRegistry,
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

    // Conexão para games.db
    let games_path = app_data_dir.join(DB_FILENAME_GAMES);
    let mut games_conn = Connection::open(&games_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_GAMES, e))?;

    games_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| format!("Erro ao configurar WAL no games.db: {}", e))?;

    // Conexão para secrets.db
    let secrets_path = app_data_dir.join(DB_FILENAME_SECRETS);
    let secrets_conn = Connection::open(&secrets_path)
        .map_err(|e| format!("Erro ao abrir {}: {}", DB_FILENAME_SECRETS, e))?;

    secrets_conn
        .pragma_update(None, "journal_mode", DB_JOURNAL_MODE)
        .map_err(|e| format!("Erro ao configurar WAL no secrets.db: {}", e))?;

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
    crate::database::cache::initialize_cache_db(&cache_conn)?;

    // Executa migrations
    crate::database::migrations::run_migrations(&config_conn, &mut games_conn)?;

    Ok(AppState {
        config_db: Mutex::new(config_conn),
        games_db: Mutex::new(games_conn),
        secrets_db: Mutex::new(secrets_conn),
        cache_db: Mutex::new(cache_conn),
        playtime_registry: PlaytimeRegistry::default(),
    })
}

// === HELPERS ===

/// Versão atual do schema de games.db (nº de migrations aplicadas).
pub fn current_schema_version(conn: &Connection) -> Result<u32, AppError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);
    Ok(version.max(0) as u32)
}

/// Retorna uma mensagem de status com a versão atual do schema.
pub fn db_status(conn: &Connection) -> String {
    format!("Banco de dados OK (schema v{})", current_schema_version(conn).unwrap_or(0))
}

// === TAGS ===

// TODO: Alterar serialização/deserialização de tags (keywords) para novo formato da IGDB e colocar essas funções em util/services

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
