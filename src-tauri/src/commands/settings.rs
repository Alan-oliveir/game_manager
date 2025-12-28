use crate::storage;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
pub struct KeysBatch {
    pub steam_id: String,
    pub steam_api_key: String,
    pub rawg_api_key: String,
}

#[tauri::command]
pub fn get_secrets(app: AppHandle) -> Result<KeysBatch, String> {
    Ok(KeysBatch {
        steam_id: storage::get_secret(&app, "steam_id")?,
        steam_api_key: storage::get_secret(&app, "steam_api_key")?,
        rawg_api_key: storage::get_secret(&app, "rawg_api_key")?,
    })
}

#[tauri::command]
pub fn set_secrets(
    app: AppHandle,
    steam_id: Option<String>,
    steam_api_key: Option<String>,
    rawg_api_key: Option<String>,
) -> Result<(), String> {
    // Steam ID
    if let Some(id) = steam_id {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            storage::delete_secret(&app, "steam_id")?;
        } else {
            storage::set_secret(&app, "steam_id", trimmed)?;
        }
    }

    // Steam API Key
    if let Some(key) = steam_api_key {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            storage::delete_secret(&app, "steam_api_key")?;
        } else {
            storage::set_secret(&app, "steam_api_key", trimmed)?;
        }
    }

    // Rawg API Key
    if let Some(rawg) = rawg_api_key {
        let trimmed = rawg.trim();
        if trimmed.is_empty() {
            storage::delete_secret(&app, "rawg_api_key")?;
        } else {
            storage::set_secret(&app, "rawg_api_key", trimmed)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_secret(app: AppHandle, key_name: String, key_value: String) -> Result<(), String> {
    let trimmed_val = key_value.trim();

    if trimmed_val.is_empty() {
        return Err("Valor não pode ser vazio".to_string());
    }

    storage::set_secret(&app, &key_name, trimmed_val)
}

#[tauri::command]
pub fn get_secret(app: AppHandle, key_name: String) -> Result<String, String> {
    storage::get_secret(&app, &key_name)
}

#[tauri::command]
pub fn delete_secret(app: AppHandle, key_name: String) -> Result<(), String> {
    storage::delete_secret(&app, &key_name)
}

#[tauri::command]
pub fn list_secrets() -> Result<Vec<String>, String> {
    Ok(storage::list_supported_keys()
        .into_iter()
        .map(|s| s.to_string())
        .collect())
}
