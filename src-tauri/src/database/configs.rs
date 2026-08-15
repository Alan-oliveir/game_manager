//! Módulo para gerenciamento genérico de configurações da aplicação.
//!
//! Permite definir e buscar configurações chave-valor na tabela `app_config`.
//! A tabela é criada automaticamente se não existir.
//! As funções retornam erros apropriados em caso de falhas de banco de dados.

use crate::database::AppState;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager, State};

// === GERENCIAMENTO GENÉRICO DE CONFIGURAÇÃO (app_config) ===

/// Cria a tabela app_config se não existir (Idempotente)
pub fn ensure_config_table(conn: &Connection) -> Result<(), AppError> {
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
    let conn = state.config_db.lock().map_err(|_| AppError::MutexError)?;

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
    let conn = state.config_db.lock().map_err(|_| AppError::MutexError)?;

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

/// Armazena a versão do schema na tabela app_config em config.db
pub fn store_schema_version(app: &AppHandle, schema_version: u32) -> Result<(), AppError> {
    let state: State<AppState> = app.state();
    let conn = state.config_db.lock().map_err(|_| AppError::MutexError)?;

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
    let conn = state.config_db.lock().map_err(|_| AppError::MutexError)?;

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
