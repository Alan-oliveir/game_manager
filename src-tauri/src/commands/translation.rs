//! Comando Tauri para traduzir descrições de jogos.

use crate::errors::AppError;
use crate::models::GameDescription;
use crate::services::translation;
use tauri::AppHandle;

#[tauri::command]
pub async fn translate_description(
    app: AppHandle,
    game_id: String,
    target_lang: Option<String>,
) -> Result<GameDescription, AppError> {
    translation::translate_description(&app, game_id, target_lang).await
}
