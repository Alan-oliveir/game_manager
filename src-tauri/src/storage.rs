use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.dat";

/// Função auxiliar para salvar um valor no Store
pub fn set_secret(app: &AppHandle, key_name: &str, value: &str) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.set(key_name.to_string(), json!(value));
    store
        .save()
        .map_err(|e| format!("Erro ao salvar no disco: {}", e))?;
    Ok(())
}

/// Função auxiliar para ler um valor
pub fn get_secret(app: &AppHandle, key_name: &str) -> Result<String, String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;

    match store.get(key_name) {
        Some(value) => {
            let str_val = value.as_str().unwrap_or("").to_string();
            Ok(str_val)
        }
        None => Ok(String::new()),
    }
}

// Opcional: deletar
pub fn delete_secret(app: &AppHandle, key_name: &str) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.delete(key_name);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_supported_keys() -> Vec<&'static str> {
    vec!["steam_id", "steam_api_key", "rawg_api_key"]
}
