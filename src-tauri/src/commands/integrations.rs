use crate::constants;
use crate::database::AppState;
use crate::services::{rawg, steam};
use crate::storage;
use tracing::{info, error};
use rusqlite::params;
use std::time::Duration;
use tauri::{AppHandle, State};
use tokio::time::sleep;
use crate::constants::STEAM_RATE_LIMIT_MS;

#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub success_count: i32,
    pub error_count: i32,
    pub total_processed: i32,
    pub message: String,
    pub errors: Vec<String>, // Lista de nomes que falharam
}

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
pub async fn enrich_library(state: State<'_, AppState>) -> Result<ImportSummary, String> {
    info!("Iniciando processo de enriquecimento de biblioteca...");

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
        return Ok(ImportSummary {
            success_count: 0,
            error_count: 0,
            total_processed: 0,
            message: "Todos os jogos já estão atualizados.".to_string(),
            errors: vec![],
        });
    }

    info!("Encontrados {} jogos com metadados pendentes.", total);

    let mut batch_updates: Vec<(String, String)> = Vec::new();
    let mut success_count = 0;
    let mut failed_games = Vec::new();

    // Loop de processamento
    for (i, (id_str, name)) in games_to_update.iter().enumerate() {
        if let Ok(app_id) = id_str.parse::<u32>() {
            match steam::fetch_game_metadata(app_id).await {
                Ok(metadata) => {
                    batch_updates.push((id_str.clone(), metadata.genre.clone()));
                    // Log menos verboso no console, detalhado no arquivo
                    info!("Metadata OK ({}/{}): {} -> {}", i+1, total, name, metadata.genre);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Falha metadata ({}/{}): {} - Erro: {}", i+1, total, name, e);
                    failed_games.push(format!("{} ({})", name, e));
                }
            }

            sleep(Duration::from_millis(STEAM_RATE_LIMIT_MS)).await;
        }
    }

    // Salvar no Banco
    if !batch_updates.is_empty() {
        let conn = state.db.lock().map_err(|_| "Mutex error ao salvar")?;
        conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

        for (id, genre) in batch_updates {
            let _ = conn.execute(
                "UPDATE games SET genre = ?1 WHERE id = ?2",
                rusqlite::params![genre, id],
            );
        }

        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
        info!("Transação de atualização salva no banco com sucesso.");
    }

    let summary = ImportSummary {
        success_count,
        error_count: failed_games.len() as i32,
        total_processed: total as i32,
        message: format!("Processamento concluído. {} sucessos, {} falhas.", success_count, failed_games.len()),
        errors: failed_games,
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
