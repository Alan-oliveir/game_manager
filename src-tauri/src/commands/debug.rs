use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use rusqlite::params;
use tauri::State;
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

/// Comando temporário: popula `slug` para jogos já existentes no banco
/// (antes da coluna existir ou antes de rodar o slugify no import).
/// Chamar uma vez pela devtools e remover depois.
#[tauri::command]
pub fn backfill_slug_names(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM games WHERE slug = ''")
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .filter_map(Result::ok)
        .collect();

    let count = rows.len();

    for (id, name) in rows {
        let slug = crate::utils::text::slugify(&name);
        conn.execute(
            "UPDATE games SET slug = ?1 WHERE id = ?2",
            params![slug, id],
        )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }

    Ok(format!("{} jogos atualizados com slug", count))
}
