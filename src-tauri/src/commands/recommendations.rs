use crate::database::AppState;
use crate::models::{Game, UserProfile};
use crate::services::recommendation;
use tauri::State;

#[tauri::command]
pub fn get_user_profile(state: State<AppState>) -> Result<UserProfile, String> {
    // Busca todos os jogos do banco (Database Layer)
    let games = {
        let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;
        let mut stmt = conn
            .prepare("SELECT id, name, genre, platform, cover_url, playtime, rating, favorite FROM games")
            .map_err(|e| e.to_string())?;

        let games_iter = stmt.query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
                genre: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                playtime: row.get(5)?,
                rating: row.get(6)?,
                favorite: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?;

        games_iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };

    // Passa para o Motor de Recomendação (Service Layer)
    let profile = recommendation::calculate_user_profile(&games);

    Ok(profile)
}