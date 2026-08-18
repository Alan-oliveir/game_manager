use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::technical::winehq::{fetch_winehq_data, WineHqSummary};
use crate::services::cache;
use tauri::State;

#[tauri::command]
pub async fn get_winehq_data(
    state: State<'_, AppState>,
    game_name: String,
) -> Result<Option<WineHqSummary>, AppError> {
    // Normaliza o nome para servir de chave de cache.
    //
    // Exemplos:
    // "The Witcher 3"   -> "the_witcher_3"
    // "Hollow  Knight"  -> "hollow_knight"
    // " Hollow Knight " -> "hollow_knight"
    let normalized_name = game_name
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");

    if normalized_name.is_empty() {
        return Ok(None);
    }

    let cache_key = format!("winehq_{}", normalized_name);

    // 1. Tenta obter os dados do cache.
    let cached_summary = {
        let cache_conn = state
            .cache_db
            .lock()
            .map_err(|_| AppError::MutexError)?;

        cache::get_cached_api_data(&cache_conn, "winehq", &cache_key)
    };

    if let Some(cached) = cached_summary {
        match serde_json::from_str::<WineHqSummary>(&cached) {
            Ok(summary) => {
                tracing::info!("WineHQ: Retornando cache para '{}'", game_name);
                return Ok(Some(summary));
            }
            Err(error) => {
                tracing::warn!(
                    "WineHQ: Cache inválido para '{}': {}",
                    game_name,
                    error
                );
            }
        }
    }

    // 2. Sem cache: busca no WineHQ.
    match fetch_winehq_data(&game_name).await {
        Ok(Some(summary)) => {
            if let Ok(json) = serde_json::to_string(&summary) {
                let cache_conn = state
                    .cache_db
                    .lock()
                    .map_err(|_| AppError::MutexError)?;

                if let Err(error) = cache::save_cached_api_data(
                    &cache_conn,
                    "winehq",
                    &cache_key,
                    &json,
                ) {
                    tracing::warn!(
                        "WineHQ: Falha ao salvar cache para '{}': {}",
                        game_name,
                        error
                    );
                }
            }

            Ok(Some(summary))
        }

        Ok(None) => Ok(None),

        Err(error) => {
            tracing::warn!(
                "WineHQ scraper falhou para '{}': {}",
                game_name,
                error
            );

            // Mantém o comportamento atual do Playlite:
            // uma falha do WineHQ não impede o carregamento dos metadados.
            Ok(None)
        }
    }
}
