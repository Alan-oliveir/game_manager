//! Comamdos para obter catálogo de jogos de serviços assinados.

use crate::database::AppState;
use crate::providers::subscriptions::amazon_luna::LunaGame;
use crate::providers::subscriptions::ea_play::EAPlayGame;
use crate::providers::subscriptions::game_pass::GamePassGame;
use crate::providers::subscriptions::ubisoft_plus::UbisoftGame;
use crate::services::subscriptions;
use tauri::State;

#[tauri::command]
pub async fn get_amazon_luna_catalog(
    state: State<'_, AppState>,
    lang: String,
) -> Result<Vec<LunaGame>, String> {
    subscriptions::get_amazon_luna_games(&state, &lang).await
}

#[tauri::command]
pub async fn get_game_pass_catalog(
    state: State<'_, AppState>,
    exclude_ea_play: bool,
    lang: String,
) -> Result<Vec<GamePassGame>, String> {
    subscriptions::get_game_pass_games(&state, exclude_ea_play, &lang).await
}

#[tauri::command]
pub async fn get_ea_play_catalog(
    state: State<'_, AppState>,
    lang: String,
) -> Result<Vec<EAPlayGame>, String> {
    subscriptions::get_ea_play_games(&state, &lang).await
}

#[tauri::command]
pub async fn get_ubisoft_plus_catalog(
    state: State<'_, AppState>,
) -> Result<Vec<UbisoftGame>, String> {
    subscriptions::get_ubisoft_plus_games(&state).await
}

#[tauri::command]
pub fn get_subscription_settings(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    subscriptions::get_enabled_services(&state)
}

#[tauri::command]
pub fn save_subscription_settings(
    state: State<'_, AppState>,
    services: Vec<String>,
) -> Result<(), String> {
    subscriptions::set_enabled_services(&state, services)
}
