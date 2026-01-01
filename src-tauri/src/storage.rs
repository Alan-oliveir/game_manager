use crate::security;
use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "secrets.dat";

/// Função para salvar um valor encriptado no Store
pub fn set_secret(app: &AppHandle, key_name: &str, value: &str) -> Result<(), String> {
    let encrypted_value = security::encrypt(value);
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.set(key_name.to_string(), json!(encrypted_value));
    store.save().map_err(|e| format!("Erro ao salvar: {}", e))?;
    Ok(())
}

/// Função para ler e desencriptar um valor
pub fn get_secret(app: &AppHandle, key_name: &str) -> Result<String, String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;

    match store.get(key_name) {
        Some(value) => {
            let str_val = value.as_str().unwrap_or("").to_string();

            if str_val.is_empty() {
                return Ok(String::new());
            }

            match security::decrypt(&str_val) {
                Ok(decrypted) => Ok(decrypted),
                Err(_) => {
                    eprintln!(
                        "Aviso: Falha ao decriptar chave {}, retornando valor cru.",
                        key_name
                    );
                    Ok(str_val)
                }
            }
        }
        None => Ok(String::new()),
    }
}

/// Função para excluir um valor do Store
pub fn delete_secret(app: &AppHandle, key_name: &str) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.delete(key_name);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// Lista de chaves suportadas
pub fn list_supported_keys() -> Vec<&'static str> {
    vec!["steam_id", "steam_api_key", "rawg_api_key"]
}
