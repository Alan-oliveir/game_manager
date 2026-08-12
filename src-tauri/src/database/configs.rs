//! Módulo para gerenciamento genérico de configurações da aplicação.
//!
//! Permite definir e buscar configurações chave-valor na tabela `app_config`.
//! A tabela é criada automaticamente se não existir.
//! As funções retornam erros apropriados em caso de falhas de banco de dados.

use crate::database::AppState;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use sys_locale::get_locale;
use tauri::{AppHandle, Manager, State};

// === GERENCIAMENTO GENÉRICO DE CONFIGURAÇÃO (app_config) ===

/// Cria a tabela app_config se não existir (Idempotente)
fn ensure_config_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Define uma configuração (Upsert)
pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    ensure_config_table(conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Busca uma configuração (Option)
pub fn get_config(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    ensure_config_table(conn)?;
    let res: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        params![key],
        |row| row.get(0),
    );

    match res {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

// === STORAGE DE VERSÃO DA APLICAÇÃO ===

/// Armazena a versão atual da aplicação na tabela app_config em cache.db
pub fn store_app_version(app: &AppHandle, version: &str) -> Result<(), AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    ensure_config_table(&conn)?;

    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES ('app_version', ?1)",
        params![version],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Obtém a versão armazenada da aplicação
pub fn get_stored_app_version(app: &AppHandle) -> Result<String, AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    ensure_config_table(&conn)?;

    match conn.query_row(
        "SELECT value FROM app_config WHERE key = 'app_version'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(version) => Ok(version),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok("0.0.0".to_string()),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

// === STORAGE DE VERSÃO DO SCHEMA ===

/// Armazena a versão do schema na tabela app_config em cache.db
pub fn store_schema_version(app: &AppHandle, schema_version: u32) -> Result<(), AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    ensure_config_table(&conn)?;

    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES ('schema_version', ?1)",
        params![schema_version],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Obtém a versão do schema armazenada
pub fn get_stored_schema_version(app: &AppHandle) -> Result<u32, AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    ensure_config_table(&conn)?;

    match conn.query_row(
        "SELECT value FROM app_config WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(version_str) => version_str.parse::<u32>().map_err(|e| {
            AppError::DatabaseError(format!("Erro ao converter schema_version: {}", e))
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

// === REGIÃO (usado pela ITAD e futuramente outras APIs) ===

const CONFIG_KEY_REGION: &str = "region";

/// Detecta a região a partir do locale do sistema operacional (BCP 47).
/// Fallback "US" se não conseguir detectar ou vier um formato inesperado.
fn detect_region_from_system() -> String {
    get_locale()
        .and_then(|tag| {
            tag.split(['-', '_'])
                .nth(1)
                .map(|region| region.to_uppercase())
        })
        .filter(|r| r.len() == 2)
        .unwrap_or_else(|| "US".to_string())
}

/// Retorna a região configurada em app_config. Se ainda não existir,
/// detecta via sys-locale, persiste e retorna o valor detectado.
pub fn get_or_detect_region(app: &AppHandle) -> Result<String, AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    if let Some(region) = get_config(&conn, CONFIG_KEY_REGION)? {
        return Ok(region);
    }

    let detected = detect_region_from_system();
    set_config(&conn, CONFIG_KEY_REGION, &detected)?;
    Ok(detected)
}

/// Permite sobrescrever a região manualmente (futura tela de settings).
pub fn set_region(app: &AppHandle, region: &str) -> Result<(), AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    set_config(&conn, CONFIG_KEY_REGION, &region.to_uppercase())
}
