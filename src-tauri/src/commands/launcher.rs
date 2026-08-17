use crate::commands::games::get_game_by_id;
use crate::database::AppState;
use crate::errors::AppError;
use crate::services::playtime::{has_official_playtime_source, watch_game};
use crate::utils::launcher::{resolve_launch, LaunchResolution};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchOutcome {
    Launched,
    OpenedLauncher { installed: bool },
    OpenedStore,
    Unavailable,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    game_id: String,
    launcher_path_override: Option<String>,
) -> Result<LaunchOutcome, AppError> {
    let state = app.state::<AppState>();
    let game = get_game_by_id(state, game_id.clone())?
        .ok_or_else(|| AppError::NotFound(format!("Jogo não encontrado: {game_id}")))?;

    let outcome = match resolve_launch(&game, launcher_path_override.as_deref()) {
        LaunchResolution::Protocol(url) => {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| AppError::LaunchError(e.to_string()))?;
            LaunchOutcome::Launched
        }
        LaunchResolution::Executable(path) => {
            Command::new(&path).spawn()?;
            LaunchOutcome::Launched
        }
        LaunchResolution::Launcher(path) => {
            Command::new(&path).spawn()?;
            LaunchOutcome::OpenedLauncher {
                installed: game.installed,
            }
        }
        LaunchResolution::Store(url) => {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| AppError::LaunchError(e.to_string()))?;
            LaunchOutcome::OpenedStore
        }
        LaunchResolution::Unavailable => LaunchOutcome::Unavailable,
    };

    let should_track = matches!(
        outcome,
        LaunchOutcome::Launched | LaunchOutcome::OpenedLauncher { installed: true }
    ) && !has_official_playtime_source(&game.library);

    if should_track {
        if let Some(exe_path) = game.executable_path.as_ref().map(PathBuf::from) {
            let install_path = game.install_path.as_ref().map(PathBuf::from);
            watch_game(app.clone(), game.id.clone(), exe_path, install_path);
        }
    }

    Ok(outcome)
}
