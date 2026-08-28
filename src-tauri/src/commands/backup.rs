//! Módulo principal com comandos de backup

use crate::database::backup::export_queries::fetch_backup_data;
use crate::database::backup::import_queries::restore_backup_data;
use crate::database::backup::models::BackupData;
use crate::database::{self, current_schema_version, AppState};
use crate::errors::AppError;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn export_database(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), AppError> {
    let (
        games,
        game_details,
        game_descriptions,
        wishlist_game,
        game_extras,
        system_requirements,
        game_data_paths,
        schema_version,
    ) = fetch_backup_data(&state)?;

    let backup = BackupData {
        version: schema_version,
        app_version: app.package_info().version.to_string(),
        date: chrono::Local::now().to_rfc3339(),
        games,
        game_details,
        game_descriptions,
        wishlist_game,
        game_extras,
        system_requirements,
        game_data_paths,
    };

    let json = serde_json::to_string_pretty(&backup)?;
    fs::write(file_path, json)?;

    let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    let now = Utc::now().to_rfc3339();
    database::configs::set_config(&cache_conn, "last_backup_at", &now)?;

    Ok(())
}

#[tauri::command]
pub async fn import_database(
    _app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, AppError> {
    let content = fs::read_to_string(file_path)?;
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|_| AppError::ValidationError("Arquivo de backup inválido".to_string()))?;

    let current_version = {
        let conn = state.games_db.lock()?;
        current_schema_version(&conn)?
    };

    if backup.version > current_version {
        return Err(AppError::ValidationError(format!(
            "Backup incompatível: foi feito numa versão mais nova do schema (v{}) do que o app atual suporta (v{}). Atualize o Playlite antes de restaurar este backup.",
            backup.version, current_version
        )));
    }

    let conn = state.games_db.lock()?;
    restore_backup_data(&conn, &backup)
}
