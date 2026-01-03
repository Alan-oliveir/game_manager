use crate::security;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};
use crate::constants::DB_FILENAME_SECRETS;

/// Obtém a conexão com o banco de dados de secrets
fn db(app: &AppHandle) -> Result<Connection, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir não encontrado: {}", e))?;

    let conn = Connection::open(app_dir.join(DB_FILENAME_SECRETS)).map_err(|e| e.to_string())?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS encrypted_keys (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn)
}

/// Salva um segredo encriptado (API key, token, etc.)
pub fn set_secret(app: &AppHandle, key_name: &str, value: &str) -> Result<(), String> {
    let conn = db(app)?;
    let encrypted = security::encrypt(value);

    conn.execute(
        "INSERT OR REPLACE INTO encrypted_keys (key, value) VALUES (?1, ?2)",
        params![key_name, encrypted],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Recupera e decripta um segredo
pub fn get_secret(app: &AppHandle, key_name: &str) -> Result<String, String> {
    let conn = db(app)?;

    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM encrypted_keys WHERE key = ?1",
        params![key_name],
        |row| row.get(0),
    );

    match result {
        Ok(encrypted) => {
            let decrypted = security::decrypt(&encrypted)?;
            Ok(decrypted)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

/// Exclui um segredo do banco
pub fn delete_secret(app: &AppHandle, key_name: &str) -> Result<(), String> {
    let conn = db(app)?;

    conn.execute(
        "DELETE FROM encrypted_keys WHERE key = ?1",
        params![key_name],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Lista de chaves suportadas pela aplicação
pub fn list_supported_keys() -> Vec<&'static str> {
    vec!["steam_id", "steam_api_key", "rawg_api_key"]
}
