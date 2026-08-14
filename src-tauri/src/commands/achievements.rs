//! Command Tauri para o dashboard: conquistas recentes de todas as
//! plataformas configuradas. Lógica de verdade em `services::achievements`.

use crate::providers::achievements::core::DashboardAchievement;
use tauri::AppHandle;

#[tauri::command]
pub fn get_recent_achievements(app: AppHandle) -> Result<Vec<DashboardAchievement>, String> {
    crate::providers::achievements::core::get_recent_achievements(&app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_achievements_now(app: AppHandle) -> Result<(), String> {
    crate::providers::achievements::core::sync_all_achievements(&app)
        .await
        .map_err(|e| e.to_string())
}
