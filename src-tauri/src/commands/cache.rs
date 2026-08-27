//! Comandos para gerenciar o cache de metadados
//!
//! Expõe comandos Tauri para uso no frontend. Toda a lógica de acesso
//! e agregação vive em `services::cache` — este módulo só traduz
//! Result<_, String> em AppError e formata mensagens de retorno.

use crate::database::cache::{self, DetailedCacheStats};
use crate::database::AppState;
use crate::errors::AppError;
use tauri::State;

/// Remove entradas expiradas do cache
#[tauri::command]
pub fn cleanup_cache(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.cache_db.lock()?;
    let deleted = cache::cleanup_expired_cache(&conn).map_err(AppError::DatabaseError)?;
    Ok(format!("{} entradas removidas", deleted))
}

/// Limpa TODO o cache (use com cuidado)
#[tauri::command]
pub fn clear_all_cache(state: State<AppState>) -> Result<String, AppError> {
    let conn = state.cache_db.lock()?;
    let deleted = cache::clear_all_cache(&conn).map_err(AppError::DatabaseError)?;
    Ok(format!("Cache limpo: {} entradas removidas", deleted))
}

#[tauri::command]
pub fn get_detailed_cache_stats(state: State<AppState>) -> Result<DetailedCacheStats, AppError> {
    let conn = state.cache_db.lock()?;
    cache::get_detailed_cache_stats(&conn).map_err(AppError::DatabaseError)
}
