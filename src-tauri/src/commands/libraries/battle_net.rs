use crate::errors::AppError;
use crate::providers::libraries::battle_net::BattleNetSource;
use crate::services::libraries::spawn_import;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_battle_net_games(app: AppHandle) -> Result<(), AppError> {
    spawn_import(app, "BattleNet", |_app| async move {
        BattleNetSource::new().import_installed().await
    });
    Ok(())
}
