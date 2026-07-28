//! # App Initialization & Update Handler
//!
//! Gerencia o ciclo de vida da aplicação:
//! - Primeira instalação
//! - Atualizações de versão
//! - Backups automáticos
//! - Migrações de schema

use crate::database;
use crate::errors::AppError;
use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager};

/// Inicializa a aplicação após uma atualização
///
/// Verifica se houve mudança de versão e executa:
/// 1. Backup automático se versão major mudou
/// 2. Migração de schema se necessário
/// 3. Atualiza versão armazenada
///
/// Deve ser chamada durante o setup do Tauri
pub fn initialize_app(app: &AppHandle) -> Result<(), AppError> {
    let current_version = app.package_info().version.to_string();
    let previous_version = database::configs::get_stored_app_version(app)?;

    // Obtém acesso ao cache_db para configurações
    let state: tauri::State<database::AppState> = app.state();
    let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    // Verifica se é primeira instalação
    if database::configs::get_config(&cache_conn, "install_date")?.is_none() {
        let now = Utc::now().to_rfc3339();
        database::configs::set_config(&cache_conn, "install_date", &now)?;
        tracing::info!("Primeira execução detectada. Data salva: {}", now);
    }

    drop(cache_conn);

    tracing::info!(
        "Inicializando app - Versão anterior: {}, Atual: {}",
        previous_version,
        current_version
    );

    // Se é primeira execução ou versão mudou
    if previous_version != current_version {
        handle_version_update(app, &previous_version, &current_version)?;
    } else {
        tracing::info!("Nenhuma atualização detectada");
    }

    Ok(())
}

/// Processa atualização de versão
fn handle_version_update(
    app: &AppHandle,
    previous_version: &str,
    current_version: &str,
) -> Result<(), AppError> {
    // 1. Backup automático se major version mudou
    if let Some(backup_path) =
        database::backup::auto::backup_if_major_update(app, previous_version, current_version)?
    {
        tracing::info!("Backup automático criado em: {:?}", backup_path);

        // Emite evento para o frontend
        let backup_path_str = backup_path.to_string_lossy().to_string();
        let _ = app.emit("backup-created", backup_path_str);
    }

    // 2. Migração de schema
    let state: tauri::State<database::AppState> = app.state();
    let lib_conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    database::migrations::run_migrations(app, &lib_conn)?;
    drop(lib_conn);

    // 3. Atualiza versão armazenada
    database::configs::store_app_version(app, current_version)?;

    // 4. Armazena versão do schema
    let schema_version = app.package_info().version.major as u32;
    database::configs::store_schema_version(app, schema_version)?;

    // 5. Atualiza timestamp
    update_last_updated_timestamp(app)?;

    tracing::info!("App inicializado com sucesso na versão {}", current_version);

    Ok(())
}

/// Atualiza timestamp de última atualização
fn update_last_updated_timestamp(app: &AppHandle) -> Result<(), AppError> {
    let state: tauri::State<database::AppState> = app.state();
    let cache_conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    let now = Utc::now().to_rfc3339();
    database::configs::set_config(&cache_conn, "last_updated_at", &now)?;
    Ok(())
}
