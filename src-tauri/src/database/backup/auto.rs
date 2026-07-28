//! Módulo com a lógica de update automático

use crate::database::backup::export_queries::fetch_backup_data;
use crate::database::backup::models::BackupData;
use crate::database::{self, AppState};
use crate::errors::AppError;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn backup_if_major_update(
    app: &AppHandle,
    previous_version: &str,
    current_version: &str,
) -> Result<Option<PathBuf>, AppError> {
    // Parse das versões
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let (prev_major, _, _) = parse_version(previous_version);
    let (curr_major, _, _) = parse_version(current_version);

    // Se versão major mudou, faz backup
    if prev_major != curr_major && prev_major > 0 {
        tracing::info!(
            "Mudança de versão major detectada: v{} -> v{}",
            previous_version,
            current_version
        );
        let backup_path = backup_before_update(app, previous_version)?;
        Ok(Some(backup_path))
    } else {
        Ok(None)
    }
}

/// Cria backup automático antes de atualização de versão
///
/// Chamado automaticamente quando detecta mudança de versão major
pub fn backup_before_update(app: &AppHandle, previous_version: &str) -> Result<PathBuf, AppError> {
    tracing::info!("Criando backup automático antes da atualização...");

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::IoError(format!("Falha ao obter app_data_dir: {}", e)))?;

    let backups_dir = app_data_dir.join("backups");
    fs::create_dir_all(&backups_dir)?;

    // Nome do backup com timestamp e versão anterior
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("auto_backup_v{}_{}.json", previous_version, timestamp);
    let backup_path = backups_dir.join(backup_filename);

    // Reutiliza a função auxiliar de fetch
    let state: tauri::State<AppState> = app.state();
    let (
        games,
        game_details,
        wishlist_game,
        game_extras,
        system_requirements,
        game_data_paths,
        schema_version,
    ) = fetch_backup_data(&state)?;

    let backup = BackupData {
        version: schema_version,
        app_version: previous_version.to_string(),
        date: chrono::Local::now().to_rfc3339(),
        games,
        game_details,
        wishlist_game,
        game_extras,
        system_requirements,
        game_data_paths,
    };

    let json = serde_json::to_string_pretty(&backup)?;
    fs::write(&backup_path, json)?;

    // Atualiza timestamp do último backup automático
    let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    let now = Utc::now().to_rfc3339();
    database::configs::set_config(&cache_conn, "last_auto_backup_at", &now)?;

    tracing::info!("Backup automático criado: {:?}", backup_path);
    Ok(backup_path)
}
