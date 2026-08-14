use crate::database::AppState;
use crate::providers::technical::anticheat::fetcher::ensure_fresh;
use crate::providers::technical::anticheat::lookup::find_anticheat_info;
use crate::providers::technical::anticheat::models::AnticheatInfo;
use crate::utils::http_client::HTTP_CLIENT;
use rusqlite::OptionalExtension;
use tauri::State;

#[tauri::command]
pub async fn get_anticheat_info(
    state: State<'_, AppState>,
    game_id: i64,
) -> Result<Option<AnticheatInfo>, String> {
    if !cfg!(target_os = "linux") {
        return Ok(None); // gate de plataforma: não faz busca de anticheat info em Windows
    }

    // ensure_fresh gerencia o próprio lock internamente — não segura MutexGuard durante o await.
    ensure_fresh(&state.games_db, &HTTP_CLIENT)
        .await
        .map_err(|e| e.to_string())?;

    let conn = state
        .games_db
        .lock()
        .map_err(|_| "Erro ao acessar o banco de dados".to_string())?;

    let game = conn
        .query_row(
            "SELECT platform_game_id, name, slug FROM games WHERE id = ?1",
            rusqlite::params![game_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let Some((platform_game_id, name, slug)) = game else {
        return Ok(None);
    };

    find_anticheat_info(&conn, Some(platform_game_id.as_str()), &name, &slug)
        .map_err(|e| e.to_string())
}
