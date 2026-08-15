//! Módulo de gerenciamento de configurações e secrets.
//!
//! Fornece interface para armazenar e recuperar credenciais
//! sensíveis (API keys, IDs) de forma segura usando criptografia.
//! Todos os secrets são criptografados usando AES-256-GCM antes de serem salvas no banco de dados.
//! A chave de criptografia é derivada de características únicas via `security::init_security()`.

use crate::database;
use crate::errors::AppError;
use crate::services::locale::{get_or_detect_language, get_or_detect_region, set_language, set_region};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Lote de todas as API keys configuradas no sistema.
///
/// Usado para retornar múltiplos secrets de uma vez para o frontend.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeysBatch {
    pub steam_id: String,
    pub steam_api_key: String,
    pub rawg_api_key: String,
    pub gemini_api_key: String,
    pub gamebrain_api_key: String,
    pub nexus_api_key: String,
    pub igdb_client_id: String,
    pub igdb_client_secret: String,
    pub xbox_live_client_id: String,
    pub xbox_live_client_secret: String,
}

/// Recupera todos os secrets configurados em lote.
///
/// Busca e descriptografa todas as credenciais armazenadas,
/// retornando strings vazias para secrets não configurados.
#[tauri::command]
pub fn get_secrets(app: AppHandle) -> Result<KeysBatch, AppError> {
    Ok(KeysBatch {
        steam_id: database::get_secret(&app, "steam_id")?,
        steam_api_key: database::get_secret(&app, "steam_api_key")?,
        rawg_api_key: database::get_secret(&app, "rawg_api_key")?,
        gemini_api_key: database::get_secret(&app, "gemini_api_key")?,
        gamebrain_api_key: database::get_secret(&app, "gamebrain_api_key")?,
        nexus_api_key: database::get_secret(&app, "nexus_api_key")?,
        igdb_client_id: database::get_secret(&app, "igdb_client_id")?,
        igdb_client_secret: database::get_secret(&app, "igdb_client_secret")?,
        xbox_live_client_id: database::get_secret(&app, "xbox_live_client_id")?,
        xbox_live_client_secret: database::get_secret(&app, "xbox_live_client_secret")?,
    })
}

/// Configura múltiplos secrets numa única operação.
///
/// Permite atualizar várias credenciais simultaneamente.
/// Mantém a compatibilidade com o objeto JSON enviado pelo frontend.
#[tauri::command]
pub fn set_secrets(
    app: AppHandle,
    steam_id: Option<String>,
    steam_api_key: Option<String>,
    rawg_api_key: Option<String>,
    gemini_api_key: Option<String>,
    gamebrain_api_key: Option<String>,
    igdb_client_id: Option<String>,
    igdb_client_secret: Option<String>,
    xbox_live_client_id: Option<String>,
    xbox_live_client_secret: Option<String>,
    nexus_api_key: Option<String>,
) -> Result<(), AppError> {
    // Helper para salvar ou deletar baseado no valor
    let save_or_delete = |key: &str, value: Option<String>| -> Result<(), AppError> {
        if let Some(v) = value {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                database::delete_secret(&app, key)?;
            } else {
                database::set_secret(&app, key, trimmed)?;
            }
        }
        Ok(())
    };

    save_or_delete("steam_id", steam_id)?;
    save_or_delete("steam_api_key", steam_api_key)?;
    save_or_delete("rawg_api_key", rawg_api_key)?;
    save_or_delete("gemini_api_key", gemini_api_key)?;
    save_or_delete("gamebrain_api_key", gamebrain_api_key)?;
    save_or_delete("igdb_client_id", igdb_client_id)?;
    save_or_delete("igdb_client_secret", igdb_client_secret)?;
    save_or_delete("xbox_live_client_id", xbox_live_client_id)?;
    save_or_delete("xbox_live_client_secret", xbox_live_client_secret)?;

    let nexus_key_provided = nexus_api_key
        .as_ref()
        .is_some_and(|v| !v.trim().is_empty());
    save_or_delete("nexus_api_key", nexus_api_key)?;

    if nexus_key_provided {
        crate::providers::mods::nexus::spawn_nexus_games_bootstrap(&app);
    }

    Ok(())
}

/// Define um secret individual por nome de chave.
///
/// Versão genérica que permite configurar qualquer secret suportado fornecendo nome e valor.
#[tauri::command]
pub fn set_secret(app: AppHandle, key_name: String, key_value: String) -> Result<(), AppError> {
    let trimmed_val = key_value.trim();

    if trimmed_val.is_empty() {
        return Err(AppError::ValidationError(
            "Valor não pode ser vazio".to_string(),
        ));
    }

    database::set_secret(&app, &key_name, trimmed_val)?;

    if key_name == "nexus_api_key" {
        crate::providers::mods::nexus::spawn_nexus_games_bootstrap(&app);
    }

    Ok(())
}

/// Recupera um secret individual por nome de chave.
///
/// Busca e descriptografa uma credencial específica do banco.
#[tauri::command]
pub fn get_secret(app: AppHandle, key_name: String) -> Result<String, AppError> {
    database::get_secret(&app, &key_name)
}

/// Remove permanentemente um secret do banco.
///
/// Exclui a credencial criptografada do banco de dados. Operação irreversível.
#[tauri::command]
pub fn delete_secret(app: AppHandle, key_name: String) -> Result<(), AppError> {
    database::delete_secret(&app, &key_name)
}

/// Lista todos os nomes de secrets suportados pelo sistema.
///
/// Retorna lista com os nomes de todas as credenciais que podem
/// ser configuradas, útil para validação no frontend.
#[tauri::command]
pub fn list_secrets() -> Vec<String> {
    database::list_supported_keys()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

#[tauri::command]
pub fn get_app_region(app: AppHandle) -> Result<String, AppError> {
    get_or_detect_region(&app)
}

#[tauri::command]
pub fn set_app_region(app: AppHandle, region: String) -> Result<(), AppError> {
    set_region(&app, &region)
}

#[tauri::command]
pub fn get_app_language(app: AppHandle) -> Result<String, AppError> {
    get_or_detect_language(&app)
}

#[tauri::command]
pub fn set_app_language(app: AppHandle, language: String) -> Result<(), AppError> {
    set_language(&app, &language)
}
