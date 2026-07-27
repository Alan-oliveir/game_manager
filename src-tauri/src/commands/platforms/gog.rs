//! GOG GALAXY - Importa jogos da GOG

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, format_login_success,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::gog::GogSource;
use crate::sources::providers::OAuthGameSource;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};

// === GOG (OAuth) ===

/// Inicia o fluxo de login OAuth2 da conta GOG.
/// Abre uma janela de login; retorna quando o token é obtido e salvo com sucesso.
#[tauri::command]
pub async fn gog_login(app: AppHandle) -> Result<String, AppError> {
    let source = GogSource::new(app);
    source.login().await?;
    Ok(format_login_success("GOG"))
}

/// Remove o token OAuth salvo da conta GOG (logout).
#[tauri::command]
pub fn gog_logout(app: AppHandle) -> Result<(), AppError> {
    let source = GogSource::new(app);
    source.logout()
}

/// Verifica se existe uma conta GOG conectada (token salvo).
/// Não garante que o token ainda é válido — apenas que existe um login prévio.
#[tauri::command]
pub fn gog_is_authenticated(app: AppHandle) -> Result<bool, AppError> {
    let source = GogSource::new(app);
    source.is_authenticated()
}

// === GOG (Import Games) ===

/// Importa a biblioteca de jogos possuídos na conta GOG (requer login OAuth prévio).
#[tauri::command]
pub async fn import_gog_games(
    app: AppHandle,
    state: State<'_, AppState>,
    gog_games_dir: Option<String>,
) -> Result<String, AppError> {
    use crate::commands::platforms::core::persist_source_games;
    use crate::sources::gog::{detect_installed_games, GogSource};
    use std::path::Path;

    let source = GogSource::new(app.clone());
    let mut games = source.fetch_games_detailed().await?;

    if let Some(dir) = gog_games_dir.filter(|s| !s.trim().is_empty()) {
        let path = Path::new(&dir);
        if path.exists() && path.is_dir() {
            detect_installed_games(&mut games, path);
        } else {
            warn!("GOG games directory provided but not found: {}", dir);
        }
    }

    if games.is_empty() {
        return Ok(format_import_empty("GOG"));
    }

    let (inserted, updated, _newly_imported) = persist_source_games(&state, games).await?;
    let message = format_import_summary("GOG", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
