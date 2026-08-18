use crate::commands::games::{get_game_by_id, get_library_game_details};
use crate::database::AppState;
use crate::models::Library;
use crate::providers::technical::anticheat::fetch::ensure_fresh;
use crate::providers::technical::anticheat::lookup::find_anticheat_info;
use crate::providers::technical::anticheat::models::AnticheatInfo;
use crate::utils::http_client::HTTP_CLIENT;
use tauri::State;

#[tauri::command]
pub async fn get_anticheat_info(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Option<AnticheatInfo>, String> {
    if !cfg!(target_os = "linux") {
        return Ok(None); // gate de plataforma
    }

    // tauri::State é apenas uma referência wrapada — Clone é barato (não clona a conexão).
    let game = get_game_by_id(state.clone(), game_id.clone())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Jogo não encontrado".to_string())?;

    let details = get_library_game_details(state.clone(), game_id).map_err(|e| e.to_string())?;

    // Prioridade: steam_app_id resolvido via metadados (cobre GOG/Epic com
    // versão Steam catalogada) > platform_game_id quando o jogo é da própria Steam.
    let steam_id = details
        .and_then(|d| d.steam_app_id)
        .or_else(|| match game.library {
            Library::Steam => Some(game.library_game_id.clone()),
            _ => None,
        });

    ensure_fresh(&state.games_db, &HTTP_CLIENT)
        .await
        .map_err(|e| e.to_string())?;

    let conn = state
        .games_db
        .lock()
        .map_err(|_| "Erro ao acessar o banco de dados".to_string())?;

    find_anticheat_info(&conn, steam_id.as_deref(), &game.name, &game.slug)
        .map_err(|e| e.to_string())
}
