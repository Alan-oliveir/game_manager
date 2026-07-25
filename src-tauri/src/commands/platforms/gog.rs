//! GOG GALAXY - Importa jogos da GOG

use crate::commands::platforms::core::{
    format_import_empty, format_import_summary, format_login_success,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::gog::GogSource;
use crate::sources::providers::OAuthGameSource;
use crate::utils::status_logic;
use chrono::Utc;
use rusqlite::params;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};
use uuid::Uuid;

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

/// Persiste jogos da GOG nas tabelas `games` e `game_details`.
async fn persist_gog_games(
    state: &AppState,
    games: Vec<crate::sources::providers::SourceGame>,
) -> Result<(u32, u32), AppError> {
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for game in games {
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND platform_game_id = ?2)",
                params![&game.platform, &game.platform_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        if !exists {
            let new_id = Uuid::new_v4().to_string();

            tx.execute(
                "INSERT INTO games (
                        id, name, cover_url, platform, platform_game_id,
                        installed, status, playtime, last_played, added_at,
                        favorite, user_rating, install_path
                    ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, NULL, ?8, 0, NULL, ?9)",
                params![
                    new_id,
                    game.name.as_deref().unwrap_or("Unknown"),
                    game.platform,
                    game.platform_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    now,
                    game.install_path,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed = ?1,
                    status = ?2,
                    install_path = COALESCE(?3, install_path)
                WHERE platform = ?4 AND platform_game_id = ?5",
                params![
                    game.installed,
                    status,
                    game.install_path,
                    game.platform,
                    game.platform_game_id,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            updated += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((inserted, updated))
}

/// Importa a biblioteca de jogos possuídos na conta GOG (requer login OAuth prévio).
#[tauri::command]
pub async fn import_gog_games(
    app: AppHandle,
    state: State<'_, AppState>,
    gog_games_dir: Option<String>,
) -> Result<String, AppError> {
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

    let (inserted, updated) = persist_gog_games(&state, games).await?;
    let message = format_import_summary("GOG", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
