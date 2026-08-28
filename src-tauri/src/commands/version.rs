//! Comandos relacionados a versionamento da aplicação
//!
//! Fornece informações sobre versões atual e anterior do app

use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

/// Estrutura que contém informações sobre a versão do aplicativo e do sistema.
#[derive(Serialize)]
pub struct AppSystemInfo {
    pub current_version: String,
    pub previous_version: Option<String>,
    pub schema_version: String,
    pub os_platform: Option<String>,
    pub install_date: Option<String>,
    pub last_updated: Option<String>,
    pub last_backup_at: Option<String>,
    pub last_auto_backup_at: Option<String>,
}

/// Retorna apenas a versão atual do aplicativo.
#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Retorna informações sobre a versão atual e anterior do aplicativo e outras infos do sistema.
#[tauri::command]
pub fn get_app_version_info(app: AppHandle) -> Result<AppSystemInfo, AppError> {
    let current_version = app.package_info().version.to_string();
    let previous_version = database::configs::get_stored_app_version(&app)?;

    let state: State<AppState> = app.state();

    let schema_version = {
        let games_conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
        database::current_schema_version(&games_conn)?
    };

    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    // Busca configurações genéricas
    let install_date = database::configs::get_config(&conn, "install_date")?;
    let last_updated = database::configs::get_config(&conn, "last_updated_at")?;
    let last_backup_at = database::configs::get_config(&conn, "last_backup_at")?;
    let last_auto_backup_at = database::configs::get_config(&conn, "last_auto_backup_at")?;
    let os_platform = database::configs::get_config(&conn, "os_platform")?;

    let previous = if previous_version == "0.0.0" {
        None
    } else {
        Some(previous_version)
    };

    Ok(AppSystemInfo {
        current_version,
        previous_version: previous,
        schema_version: schema_version.to_string(),
        os_platform,
        install_date,
        last_updated,
        last_backup_at,
        last_auto_backup_at,
    })
}

fn is_running_flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok()
}

#[tauri::command]
pub fn is_updater_enabled() -> bool {
    if cfg!(target_os = "linux") && is_running_flatpak() {
        return false;
    }
    true
}
