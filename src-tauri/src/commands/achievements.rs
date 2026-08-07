//! Command Tauri para o dashboard: conquistas recentes de todas as
//! plataformas configuradas. Lógica de verdade em
//! `services::achievements`.

use crate::errors::AppError;
use crate::services::achievements::core::DashboardAchievement;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_recent_achievements(
    app: AppHandle,
) -> Result<Vec<DashboardAchievement>, AppError> {
    crate::services::achievements::core::get_recent_achievements(&app).await
}
