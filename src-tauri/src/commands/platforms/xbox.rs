use crate::commands::platforms::core::spawn_import;
use crate::errors::AppError;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_xbox_games(app: AppHandle) -> Result<(), AppError> {
    spawn_import(app, "Xbox", |_app| async move {
        crate::sources::xbox::import_installed()
    });
    Ok(())
}
