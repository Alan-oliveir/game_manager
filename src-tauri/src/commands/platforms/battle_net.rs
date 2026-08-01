use crate::commands::platforms::core::spawn_import;
use crate::errors::AppError;
use crate::sources::battle_net::BattleNetSource;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_battle_net_games(app: AppHandle) -> Result<(), AppError> {
    spawn_import(app, "BattleNet", |_app| async move {
        BattleNetSource::new().import_installed().await
    });
    Ok(())
}
