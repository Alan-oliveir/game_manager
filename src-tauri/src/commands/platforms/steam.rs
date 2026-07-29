//! Importa a biblioteca completa de jogos Steam do usuário.
//!
//! Obtém jogos de múltiplas fontes: instalados via arquivos VDF locais do Steam, não instalados
//! via librarycache do Steam e usa como fallback a API para jogos não encontrados localmente.

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, persist_source_games, trigger_enrichment_if_needed,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::steam;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn import_steam_library(
    app: AppHandle,
    state: State<'_, AppState>,
    api_key: String,
    steam_id: String,
    steam_root: String,
) -> Result<String, AppError> {
    use crate::sources::providers::GameSource; // Importa o Trait

    // 1. Instancia o provedor baseado no novo modelo de Trait
    let source = steam::SteamSource {
        steam_root,
        api_key,
        steam_id,
    };

    // 2. Busca os jogos (VDF + Cache + API)
    let games = source.fetch_games().await?;

    if games.is_empty() {
        return Ok(format_import_empty("Steam"));
    }

    // 3. Persiste usando a função genérica otimizada
    let (inserted, updated, newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("Steam", inserted, updated);
    info!("{}", message);
    
    let _ = app.emit("library_updated", ()); // Notifica o frontend

    // 4. Inicia a enriquecimento com metadados (RAWG)
    trigger_enrichment_if_needed(&app, newly_imported);

    Ok(message)
}
