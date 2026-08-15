//! Módulo de gerenciamento de secrets (API keys, tokens) com criptografia.
//!
//! Acesso direto ao SQLite (secrets.db). Toda credencial é criptografada
//! com AES-256-GCM antes de ser persistida — a lógica de cifra/decifra em
//! si vive em `security`, este módulo só coordena leitura/escrita no banco.

use crate::database::AppState;
use crate::errors::AppError;
use crate::security;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager, State};

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
        "igdb_client_id",
        "igdb_client_secret",
        "xbox_live_client_id",
        "xbox_live_client_secret",
    ]
}
