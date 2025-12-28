use crate::constants;
use crate::database::AppState;
use crate::models;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn add_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Nome do jogo não pode ser vazio".to_string());
    }

    if name.len() > constants::MAX_NAME_LENGTH {
        return Err(format!(
            "Nome do jogo muito longo (máximo {} caracteres)",
            constants::MAX_NAME_LENGTH
        ));
    }

    if let Some(ref url) = cover_url {
        if url.len() > constants::MAX_URL_LENGTH {
            return Err(format!(
                "URL da capa muito longa (máximo {} caracteres)",
                constants::MAX_URL_LENGTH
            ));
        }
        if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL da capa deve começar com http:// ou https://".to_string());
        }
    }

    if let Some(ref g) = genre {
        if g.len() > constants::MAX_GENRE_LENGTH {
            return Err(format!(
                "Gênero muito longo (máximo {} caracteres)",
                constants::MAX_GENRE_LENGTH
            ));
        }
    }

    if let Some(ref p) = platform {
        if p.len() > constants::MAX_PLATFORM_LENGTH {
            return Err(format!(
                "Plataforma muito longa (máximo {} caracteres)",
                constants::MAX_PLATFORM_LENGTH
            ));
        }
    }

    if let Some(time) = playtime {
        if time < 0 {
            return Err("Tempo jogado não pode ser negativo".to_string());
        }
        if time > constants::MAX_PLAYTIME {
            return Err(format!(
                "Tempo jogado inválido (máximo {} horas)",
                constants::MAX_PLAYTIME
            ));
        }
    }

    if let Some(r) = rating {
        if r < constants::MIN_RATING || r > constants::MAX_RATING {
            return Err(format!(
                "Avaliação deve estar entre {} e {}",
                constants::MIN_RATING,
                constants::MAX_RATING
            ));
        }
    }

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Erro ao verificar duplicata: {}", e))?;

    if exists {
        return Err("Já existe um jogo com este ID".to_string());
    }

    conn.execute(
        "INSERT INTO games (id, name, genre, platform, cover_url, playtime, rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, genre, platform, cover_url, playtime, rating],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_games(state: State<AppState>) -> Result<Vec<models::Game>, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, genre, platform, cover_url, playtime, rating, favorite FROM games",
        )
        .map_err(|e| e.to_string())?;

    let games = stmt
        .query_map([], |row| {
            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                genre: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                playtime: row.get(5)?,
                rating: row.get(6)?,
                favorite: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(games)
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET favorite = NOT favorite WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_game(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute("DELETE FROM games WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Nome do jogo não pode ser vazio".to_string());
    }

    if name.len() > constants::MAX_NAME_LENGTH {
        return Err(format!(
            "Nome do jogo muito longo (máximo {} caracteres)",
            constants::MAX_NAME_LENGTH
        ));
    }

    if let Some(time) = playtime {
        if time < 0 {
            return Err("Tempo jogado não pode ser negativo".to_string());
        }
        if time > constants::MAX_PLAYTIME {
            return Err(format!(
                "Tempo jogado inválido (máximo {} horas)",
                constants::MAX_PLAYTIME
            ));
        }
    }

    if let Some(r) = rating {
        if r < constants::MIN_RATING || r > constants::MAX_RATING {
            return Err(format!(
                "Avaliação deve estar entre {} e {}",
                constants::MIN_RATING,
                constants::MAX_RATING
            ));
        }
    }

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET name = ?1, genre = ?2, platform = ?3, cover_url = ?4, playtime = ?5, rating = ?6 WHERE id = ?7",
        params![name, genre, platform, cover_url, playtime, rating, id],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}
