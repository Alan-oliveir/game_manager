//! Comandos relacionados ao estado do banco de dados.

use crate::database::{self, AppState};
use tauri::State;

#[tauri::command]
pub fn init_db(state: State<AppState>) -> Result<String, String> {
    let conn = state
        .games_db
        .lock()
        .map_err(|_| "Falha ao bloquear mutex do games_db")?;

    Ok(database::db_status(&conn))
}
