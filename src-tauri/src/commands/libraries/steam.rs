use crate::commands::libraries::core::spawn_import;
use crate::errors::AppError;
use crate::providers::libraries::steam;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_steam_library(
    app: AppHandle,
    api_key: String,
    steam_id: String,
    steam_root: String,
) -> Result<(), AppError> {
    use crate::providers::libraries::providers::GameSource;

    spawn_import(app, "Steam", |_app| async move {
        let source = steam::SteamSource {
            steam_root,
            api_key,
            steam_id,
        };
        source.fetch_games().await
    });
    Ok(())
}
