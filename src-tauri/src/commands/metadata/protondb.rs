use crate::database::cache;
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::technical::protondb;
use crate::providers::technical::protondb::{is_running_on_linux, ProtonDbSummary};
use tauri::State;

#[tauri::command]
pub async fn fetch_protondb_data(
    state: State<'_, AppState>,
    steam_app_id: String,
) -> Result<Option<ProtonDbSummary>, AppError> {
    if !is_running_on_linux() {
        return Ok(None); // segurança extra: mesmo se o frontend chamar por engano, não gasta requisição
    }

    let cache_key = format!("protondb_{}", steam_app_id);

    // 1. Tenta pegar do Cache
    let cached_summary = {
        let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
        cache::get_cached_api_data(&cache_conn, "protondb", &cache_key)
    };

    if let Some(cached) = cached_summary {
        if let Ok(summary) = serde_json::from_str(&cached) {
            return Ok(Some(summary));
        }
    }

    // 2. Se não tem cache, busca na API
    match protondb::get_compatibility_summary(&steam_app_id).await {
        Ok(Some(summary)) => {
            if let Ok(json) = serde_json::to_string(&summary) {
                let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
                // Salva o JSON stringificado usando "protondb" como source
                let _ = cache::save_cached_api_data(&cache_conn, "protondb", &cache_key, &json);
            }
            Ok(Some(summary))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::warn!("ProtonDB falhou para app_id '{}': {}", steam_app_id, e);
            Ok(None) // Falha silenciosa para o frontend
        }
    }
}
