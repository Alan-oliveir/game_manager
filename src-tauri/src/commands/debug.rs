use crate::database;
use crate::errors::AppError;
use tauri::{AppHandle, Manager};

/// DEBUG — apenas para testes manuais via devtools.
/// Busca o catálogo do Nexus Mods e popula a tabela `nexus_games` do cache,
/// sem passar pelo fluxo de enriquecimento. Remover depois dos testes.
#[tauri::command]
pub async fn debug_populate_nexus_cache(app: AppHandle) -> Result<String, AppError> {
    let api_key = database::get_secret(&app, "nexus_api_key")?;
    if api_key.is_empty() {
        return Err(AppError::ValidationError(
            "Nexus API Key não configurada.".to_string(),
        ));
    }

    let games = crate::services::integration::nexus::fetch_nexus_games(&api_key)
        .await
        .map_err(|e| AppError::ValidationError(format!("Erro ao buscar dados do Nexus: {}", e)))?;

    let count = games.len();

    let state: tauri::State<database::AppState> = app.state();
    let cache_conn = state
        .cache_db
        .lock()
        .map_err(|_| AppError::ValidationError("Falha ao travar cache_db".to_string()))?;

    crate::services::cache::save_nexus_games_cache(&cache_conn, &games)
        .map_err(|e| AppError::ValidationError(format!("Erro ao salvar cache: {}", e)))?;

    Ok(format!("{} jogos do Nexus salvos no cache local.", count))
}
