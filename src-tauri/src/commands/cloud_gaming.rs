//! Comando para disponibilidade em serviços de cloud gaming (GeForce NOW + Xbox Cloud Gaming).

use crate::database::AppState;
use crate::services::cloud_gaming::{self, CloudAvailability};
use tauri::State;

#[tauri::command]
pub async fn get_cloud_gaming_availability(
    state: State<'_, AppState>,
    game_name: String,
    library: String,
    library_game_id: String,
    steam_app_id: Option<String>,
) -> Result<CloudAvailability, String> {
    let xbox_store_id =
        cloud_gaming::resolve_xbox_store_id(&state, &game_name, &library, &library_game_id).await?;

    cloud_gaming::get_cloud_availability(&state, steam_app_id.as_deref(), xbox_store_id.as_deref())
}
