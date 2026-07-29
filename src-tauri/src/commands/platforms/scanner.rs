//! Escaneia uma pasta local em busca de possíveis jogos.
//!
//! Retorna uma lista de descobertas encontradas.

use crate::commands::platforms::core::{
    format_import_summary, persist_source_games, trigger_enrichment_if_needed, ScanGameInput,
    ScanResult,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::scanner::scan_folder;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

#[tauri::command]
pub async fn scan_games_folder(folder_path: String) -> Result<ScanResult, String> {
    let path = Path::new(&folder_path);

    // Validações básicas
    if !path.exists() {
        return Ok(ScanResult {
            success: false,
            message: "Pasta não encontrada".to_string(),
            discoveries: vec![],
        });
    }

    if !path.is_dir() {
        return Ok(ScanResult {
            success: false,
            message: "Caminho não é uma pasta".to_string(),
            discoveries: vec![],
        });
    }

    // Executar scan
    let discoveries = scan_folder(path)?;

    let message = if discoveries.is_empty() {
        "Nenhum jogo encontrado nesta pasta".to_string()
    } else {
        format!("Encontrados {} possíveis jogos", discoveries.len())
    };

    Ok(ScanResult {
        success: true,
        message,
        discoveries,
    })
}

/// Adiciona um jogo descoberto pelo scan ao banco de dados.
#[tauri::command]
pub async fn add_game_from_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    executable_path: String,
    base_path: String,
) -> Result<String, AppError> {
    use crate::sources::providers::SourceGame;

    let game = SourceGame {
        platform: "Outra".to_string(),
        platform_game_id: executable_path.clone(),
        name: Some(name),
        installed: true,
        executable_path: Some(executable_path.clone()),
        install_path: Some(base_path),
        playtime_minutes: Some(0),
        last_played: None,
    };

    let (inserted, _, newly_imported) = persist_source_games(&state, vec![game]).await?;

    if inserted == 0 {
        return Err(AppError::ValidationError(
            "Este jogo já foi adicionado anteriormente.".to_string(),
        ));
    }

    trigger_enrichment_if_needed(&app, newly_imported);

    let _ = app.emit("library_updated", ());

    Ok("Jogo adicionado com sucesso.".to_string())
}

/// Adiciona múltiplos jogos descobertos pelo scan ao banco de dados.
#[tauri::command]
pub async fn add_games_from_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    games: Vec<ScanGameInput>,
) -> Result<String, AppError> {
    use crate::sources::providers::SourceGame;

    let source_games: Vec<SourceGame> = games
        .into_iter()
        .map(|g| SourceGame {
            platform: "Outra".to_string(),
            platform_game_id: g.executable_path.clone(),
            name: Some(g.name),
            installed: true,
            executable_path: Some(g.executable_path.clone()),
            install_path: Some(g.base_path),
            playtime_minutes: Some(0),
            last_played: None,
        })
        .collect();

    let (inserted, updated, newly_imported) = persist_source_games(&state, source_games).await?;
    let message = format_import_summary("Local", inserted, updated);
    info!("{}", message);

    trigger_enrichment_if_needed(&app, newly_imported);

    let _ = app.emit("library_updated", ());

    Ok(message)
}
