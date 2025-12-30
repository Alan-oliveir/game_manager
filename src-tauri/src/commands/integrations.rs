use crate::constants;
use crate::database::AppState;
use crate::services::{rawg, steam};
use crate::storage;

use rusqlite::params;
use std::time::Duration;
use tauri::{AppHandle, State};
use tokio::time::sleep;

#[tauri::command]
pub async fn import_steam_library(
    state: State<'_, AppState>,
    api_key: String,
    steam_id: String,
) -> Result<String, String> {
    let steam_games = steam::list_steam_games(&api_key, &steam_id).await?;

    if steam_games.is_empty() {
        return Ok("Nenhum jogo encontrado na sua biblioteca Steam.".to_string());
    }

    println!("{} jogos encontrados na Steam", steam_games.len());

    let mut games_to_insert = Vec::new();

    for game in steam_games {
        let cover_url = format!(
            "{}/steam/apps/{}/library_600x900.jpg",
            constants::STEAM_CDN_URL,
            game.appid
        );

        let playtime_hours = (game.playtime_forever as f32 / 60.0).round() as i32;

        games_to_insert.push((
            game.appid.to_string(),
            game.name.clone(),
            constants::DEFAULT_GENRE.to_string(),
            constants::DEFAULT_PLATFORM_STEAM.to_string(),
            cover_url,
            playtime_hours,
        ));
    }

    let count = {
        let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

        // Inicia transação
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Erro ao iniciar transação: {}", e))?;

        let mut inserted = 0;
        let mut skipped = 0;

        for (id, name, genre, platform, cover_url, playtime) in games_to_insert {
            match conn.execute(
                "INSERT OR IGNORE INTO games (id, name, genre, platform, cover_url, playtime, rating)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, name, genre, platform, cover_url, playtime, None::<i32>],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        inserted += rows;
                    } else {
                        skipped += 1;
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] Erro ao inserir jogo '{}': {}", name, e);
                }
            }
        }

        conn.execute("COMMIT", []).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Erro ao commitar transação: {}", e)
        })?;

        println!(
            "Import completado: {} inseridos, {} já existiam",
            inserted, skipped
        );

        inserted
    };

    Ok(format!(
        "Importação concluída! {} novos jogos adicionados.",
        count
    ))
}

#[tauri::command]
pub async fn enrich_library(state: State<'_, AppState>) -> Result<String, String> {
    let games_to_update = {
        let conn = state.db.lock().map_err(|_| "Mutex error")?;
        let mut stmt = conn
            .prepare("SELECT id, name FROM games WHERE genre = ?1 AND platform = ?2")
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query([constants::DEFAULT_GENRE, constants::DEFAULT_PLATFORM_STEAM])
            .map_err(|e| e.to_string())?;

        let mut games = Vec::new();
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let id: String = row.get(0).map_err(|e| e.to_string())?;
            let name: String = row.get(1).map_err(|e| e.to_string())?;
            games.push((id, name));
        }

        games
    };

    let total = games_to_update.len();
    if total == 0 {
        return Ok("Todos os jogos Steam já estão atualizados!".to_string());
    }

    println!("Iniciando enriquecimento de {} jogos...", total);

    let mut batch_updates: Vec<(String, String)> = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for (id_str, name) in games_to_update {
        if let Ok(app_id) = id_str.parse::<u32>() {
            match steam::fetch_game_metadata(app_id).await {
                Ok(metadata) => {
                    batch_updates.push((id_str.clone(), metadata.genre.clone()));
                    println!("Metadata obtida: {} -> {}", name, metadata.genre);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("[ERRO] Falha ao buscar metadata para {}: {}", name, e);
                    error_count += 1;
                }
            }

            sleep(Duration::from_millis(constants::STEAM_RATE_LIMIT_MS)).await;
        }
    }

    if !batch_updates.is_empty() {
        let conn = state.db.lock().map_err(|_| "Mutex error ao salvar batch")?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Erro ao iniciar transação: {}", e))?;

        let mut updates_applied = 0;
        for (id, genre) in batch_updates {
            match conn.execute(
                "UPDATE games SET genre = ?1 WHERE id = ?2",
                params![genre, id],
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        updates_applied += rows;
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] Erro ao atualizar jogo {}: {}", id, e);
                }
            }
        }

        conn.execute("COMMIT", []).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Erro ao commitar transação: {}", e)
        })?;
    }

    let summary = if error_count > 0 {
        format!(
            "Processo finalizado. {} de {} jogos atualizados com sucesso ({} erros).",
            success_count, total, error_count
        )
    } else {
        format!(
            "Processo finalizado. {} de {} jogos atualizados com sucesso!",
            success_count, total
        )
    };

    Ok(summary)
}

fn get_api_key(app_handle: &tauri::AppHandle) -> Result<String, String> {
    storage::get_secret(app_handle, "rawg_api_key")
}

#[tauri::command]
pub async fn fetch_game_details(
    app_handle: AppHandle,
    query: String,
) -> Result<rawg::GameDetails, String> {
    let api_key = get_api_key(&app_handle)?;

    if api_key.is_empty() {
        return Err("API Key da RAWG não configurada.".to_string());
    }

    rawg::fetch_game_details(&api_key, query).await
}

#[tauri::command]
pub async fn get_trending_games(app_handle: AppHandle) -> Result<Vec<rawg::RawgGame>, String> {
    let api_key = get_api_key(&app_handle)?;
    rawg::fetch_trending_games(&api_key).await
}

#[tauri::command]
pub async fn get_upcoming_games(api_key: String) -> Result<Vec<rawg::RawgGame>, String> {
    rawg::fetch_upcoming_games(&api_key).await
}
