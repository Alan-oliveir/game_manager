//! Commands Tauri para conquistas: `get_recent_achievements` (card da
//! Home, top 5) e `get_all_achievements` (tela dedicada, lista
//! completa). Lógica de verdade em `providers::achievements::core`.

use crate::errors::AppError;
use crate::providers::achievements::core::{self, AchievementDetail, DashboardAchievement};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_recent_achievements(
    app: AppHandle,
) -> Result<Vec<DashboardAchievement>, AppError> {
    core::get_recent_achievements(&app).await
}

#[tauri::command]
pub async fn get_all_achievements(app: AppHandle) -> Result<Vec<AchievementDetail>, AppError> {
    core::list_all_achievements(&app).await
}
