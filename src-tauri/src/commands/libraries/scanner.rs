//! Escaneia uma pasta local em busca de possíveis jogos.
//!
//! Retorna uma lista de descobertas encontradas.

use crate::commands::libraries::core::{
    format_import_summary, persist_source_games, trigger_enrichment_if_needed, ScanGameInput,
    ScanResult,
};
use crate::errors::AppError;
use crate::providers::libraries::scanner::scan_folder;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tracing::info;

// === STRUCTS ===

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSourceInfo {
    pub id: String,
    pub folder_path: String,
    pub label: String,
    pub created_at: String,
    pub last_scanned_at: Option<String>,
    pub game_count: i64, // quantos jogos na biblioteca têm esse label
}

use crate::database::AppState;
use std::collections::HashSet;
use tauri::State;

#[tauri::command]
pub async fn scan_games_folder(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<ScanResult, String> {
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
    let mut discoveries = scan_folder(path).map_err(|e| e.to_string())?;

    // Marca quem já está na biblioteca (compara pela pasta base do jogo, não pelo executável escolhido)
    if let Ok(conn) = state.games_db.lock() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT install_path FROM games WHERE platform = 'Outra' AND install_path IS NOT NULL",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                let existing: HashSet<String> = rows.flatten().collect();
                for d in discoveries.iter_mut() {
                    d.already_imported = existing.contains(&d.base_path);
                }
            }
        }
    }

    let _ = register_scan_source(&state, &folder_path);

    let new_count = discoveries.iter().filter(|d| !d.already_imported).count();
    let message = if discoveries.is_empty() {
        "Nenhum jogo encontrado nesta pasta".to_string()
    } else {
        format!(
            "Encontrados {} jogos ({} novos)",
            discoveries.len(),
            new_count
        )
    };

    Ok(ScanResult {
        success: true,
        message,
        discoveries,
    })
}

/// Garante que a pasta raiz esteja salva como fonte, pra o label ficar disponível na importação (add_games_from_scan).
fn register_scan_source(state: &State<'_, AppState>, folder_path: &str) -> Result<(), AppError> {
    let conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let label = Path::new(folder_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Pasta Local")
        .to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO scan_sources (id, folder_path, label, created_at, last_scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(folder_path) DO UPDATE SET last_scanned_at = ?4",
        rusqlite::params![uuid::Uuid::new_v4().to_string(), folder_path, label, now],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

fn get_scan_source_label(state: &State<'_, AppState>, folder_path: &str) -> Result<String, AppError> {
    let conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;
    let label: Option<String> = conn
        .query_row(
            "SELECT label FROM scan_sources WHERE folder_path = ?1",
            params![folder_path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(label.unwrap_or_else(|| {
        Path::new(folder_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Pasta Local")
            .to_string()
    }))
}

/// Adiciona um jogo descoberto pelo scan ao banco de dados.
#[tauri::command]
pub async fn add_game_from_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    root_folder_path: String,
    name: String,
    executable_path: String,
    base_path: String,
) -> Result<String, AppError> {
    use crate::providers::libraries::providers::SourceGame;

    let label = get_scan_source_label(&state, &root_folder_path)?;
    let game = SourceGame {
        platform: "Outra".to_string(),
        platform_game_id: executable_path.clone(),
        name: Some(name),
        installed: true,
        executable_path: Some(executable_path.clone()),
        install_path: Some(base_path),
        playtime_minutes: Some(0),
        last_played: None,
        source_label: Some(label),
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
    root_folder_path: String,
    games: Vec<ScanGameInput>,
) -> Result<String, AppError> {
    use crate::providers::libraries::providers::SourceGame;

    let label = get_scan_source_label(&state, &root_folder_path)?;
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
            source_label: Some(label.clone()),
        })
        .collect();

    let (inserted, updated, newly_imported) = persist_source_games(&state, source_games).await?;
    let message = format_import_summary("Local", inserted, updated);
    info!("{}", message);

    let _ = app.emit("library_updated", ());
    trigger_enrichment_if_needed(&app, newly_imported);
    Ok(message)
}

#[tauri::command]
pub fn list_scan_sources(state: State<'_, AppState>) -> Result<Vec<ScanSourceInfo>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
            s.id, s.folder_path, s.label, s.created_at, s.last_scanned_at,
            (SELECT COUNT(*) FROM games g WHERE g.source_label = s.label AND g.platform = 'Outra')
         FROM scan_sources s
         ORDER BY s.label ASC",
    )?;

    let sources = stmt
        .query_map([], |row| {
            Ok(ScanSourceInfo {
                id: row.get(0)?,
                folder_path: row.get(1)?,
                label: row.get(2)?,
                created_at: row.get(3)?,
                last_scanned_at: row.get(4)?,
                game_count: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sources)
}

#[tauri::command]
pub fn rename_scan_source(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    new_label: String,
) -> Result<(), AppError> {
    let new_label = new_label.trim().to_string();
    if new_label.is_empty() {
        return Err(AppError::ValidationError(
            "O nome da fonte não pode ser vazio".to_string(),
        ));
    }

    let mut conn = state.games_db.lock()?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pega o label atual pra saber o que atualizar nos jogos
    let old_label: Option<String> = tx
        .query_row(
            "SELECT label FROM scan_sources WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;

    let old_label = old_label
        .ok_or_else(|| AppError::ValidationError("Fonte de scan não encontrada".to_string()))?;

    tx.execute(
        "UPDATE scan_sources SET label = ?1 WHERE id = ?2",
        params![new_label, id],
    )?;

    tx.execute(
        "UPDATE games SET source_label = ?1 WHERE source_label = ?2 AND platform = 'Outra'",
        params![new_label, old_label],
    )?;

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let _ = app.emit("library_updated", ());
    Ok(())
}

#[tauri::command]
pub fn delete_scan_source(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    remove_games: bool,
) -> Result<(), AppError> {
    let mut conn = state.games_db.lock()?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let label: Option<String> = tx
        .query_row(
            "SELECT label FROM scan_sources WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;

    let label = label
        .ok_or_else(|| AppError::ValidationError("Fonte de scan não encontrada".to_string()))?;

    if remove_games {
        tx.execute(
            "DELETE FROM game_details WHERE game_id IN (
                SELECT id FROM games WHERE source_label = ?1 AND platform = 'Outra'
            )",
            params![label],
        )?;
        tx.execute(
            "DELETE FROM games WHERE source_label = ?1 AND platform = 'Outra'",
            params![label],
        )?;
    } else {
        tx.execute(
            "UPDATE games SET source_label = NULL WHERE source_label = ?1 AND platform = 'Outra'",
            params![label],
        )?;
    }

    tx.execute("DELETE FROM scan_sources WHERE id = ?1", params![id])?;

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let _ = app.emit("library_updated", ());
    Ok(())
}
