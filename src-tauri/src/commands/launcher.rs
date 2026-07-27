use crate::commands::games::get_game_by_id;
use crate::database::AppState;
use crate::errors::AppError;
use crate::utils::launcher::{resolve_launch, LaunchResolution};
use serde::Serialize;
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

    match resolve_launch(&game, launcher_path_override.as_deref()) {
        LaunchResolution::Protocol(url) => {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| AppError::LaunchError(e.to_string()))?;
            Ok(LaunchOutcome::Launched)
        }
        LaunchResolution::Executable(path) => {
            Command::new(&path).spawn()?; // AppError::IoError via From, sem wrapper manual
            Ok(LaunchOutcome::Launched)
        }
        LaunchResolution::Launcher(path) => {
            Command::new(&path).spawn()?;
            Ok(LaunchOutcome::OpenedLauncher {
                installed: game.installed,
            })
        }
        LaunchResolution::Store(url) => {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| AppError::LaunchError(e.to_string()))?;
            Ok(LaunchOutcome::OpenedStore)
        }
        LaunchResolution::Unavailable => Ok(LaunchOutcome::Unavailable),
    }
}
