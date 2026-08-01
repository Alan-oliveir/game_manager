//! Funções genéricas usadas na importação de bibliotecas de plataformas externas (Steam, Epic, GOG).
//!
//! Fornece comandos para salvar dados dos jogos nos bancos de dados e mensagens padronizadas.

use crate::constants;
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::providers::SourceGame;
use crate::sources::scanner::GameDiscovery;
use crate::utils::status_logic;
use chrono::{TimeZone, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};
use uuid::Uuid;

// === Estruturas de Dados ===

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanResult {
    pub success: bool,
    pub message: String,
    pub discoveries: Vec<GameDiscovery>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGameInput {
    pub name: String,
    pub executable_path: String,
    pub base_path: String,
}

/// Dados mínimos de um jogo recém-inserido, usados para disparar enriquecimento automático logo após a importação.
#[derive(Debug, Clone)]
pub struct NewlyImportedGame {
    pub game_id: String,
    pub name: String,
    pub platform: String,
    pub platform_game_id: String,
}

// === Funções Genéricas de Persistência ===

/// Persiste uma lista de jogos de uma fonte externa (como Steam) no banco de dados.
///
/// Retorna o número de jogos inseridos e atualizados.
pub(crate) async fn persist_source_games(
    state: &AppState,
    games: Vec<crate::sources::providers::SourceGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), AppError> {
    let mut conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;

    // Inicia uma transação única para todo o lote
    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut inserted = 0;
    let mut updated = 0;
    let mut newly_imported = Vec::new();
    let now = Utc::now().to_rfc3339();

    for game in games {
        // Verifica existência usando a transação
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND platform_game_id = ?2)",
                params![&game.platform, &game.platform_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        let last_played_iso = game.last_played.and_then(|ts| {
            if ts > 0 {
                Some(Utc.timestamp_opt(ts, 0).single().map(|dt| dt.to_rfc3339()))
            } else {
                None
            }
        });

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());

            // Define uma capa padrão da Steam se for essa a plataforma
            let cover_url = if game.platform == "Steam" {
                Some(format!(
                    "{}/{}",
                    constants::STEAM_CDN_URL,
                    constants::STEAM_LIBRARY_IMAGE_PATH.replace("{}", &game.platform_game_id)
                ))
            } else {
                None
            };

            tx.execute(
                "INSERT INTO games (
                    id, name, cover_url, platform, platform_game_id,
                    installed, status, playtime, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, NULL, ?11, ?12)",
                params![
                    new_id,
                    game.name.unwrap_or_else(|| "Unknown".to_string()),
                    cover_url,
                    game.platform,
                    game.platform_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    last_played_iso,
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                platform: game.platform.clone(),
                platform_game_id: game.platform_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed = ?1,
                    status = ?2,
                    playtime = ?3,
                    last_played = ?4,
                    install_path = COALESCE(?5, install_path),
                    executable_path = COALESCE(?6, executable_path)
                 WHERE platform = ?7 AND platform_game_id = ?8",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    last_played_iso,
                    game.install_path,
                    game.executable_path,
                    game.platform,
                    game.platform_game_id
                ],
            )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            updated += 1;
        }
    }

    // Finaliza a transação
    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((inserted, updated, newly_imported))
}

/// Dispara o enriquecimento de metadados em background para jogos recém-importados, se houver algum.
pub fn trigger_enrichment_if_needed(app: &AppHandle, newly_imported: Vec<NewlyImportedGame>) {
    if !newly_imported.is_empty() {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            crate::commands::metadata::enrichment::enrich_newly_imported(app_clone, newly_imported)
                .await;
        });
    }
}

/// Executa uma importação de plataforma em background, sem bloquear o comando Tauri.
///
/// `fetch` recebe uma cópia do `AppHandle` (necessário para fontes com OAuth, como Amazon/Epic/GOG)
/// e deve conter toda a parte lenta (rede, fetch, merge de fontes), retornando a lista final de
/// `SourceGame` pronta para persistência. O comando retorna imediatamente.
///
/// O resultado chega ao **frontend** via eventos:
///
/// - `import_started`   → payload: platform (string)
/// - `import_complete`  → payload: (platform, message)
/// - `import_error`     → payload: (platform, error)
/// - `library_updated`  → (mantido, sem payload, para compatibilidade com listeners existentes)
pub fn spawn_import<F, Fut>(app: AppHandle, platform: &'static str, fetch: F)
where
    F: FnOnce(AppHandle) -> Fut + Send + 'static,
    Fut: std::future::Future<Output=Result<Vec<SourceGame>, AppError>> + Send + 'static,
{
    let _ = app.emit("import_started", platform);
    let app_task = app.clone();
    let app_for_fetch = app.clone();

    tauri::async_runtime::spawn(async move {
        let games = match fetch(app_for_fetch).await {
            Ok(g) => g,
            Err(e) => {
                warn!("Import {} falhou: {}", platform, e);
                let _ = app_task.emit("import_error", (platform, e.to_string()));
                return;
            }
        };

        if games.is_empty() {
            let msg = format_import_empty(platform);
            let _ = app_task.emit("import_complete", (platform, msg));
            return;
        }

        let state: tauri::State<AppState> = app_task.state();
        let (inserted, updated, newly_imported) = match persist_source_games(&state, games).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Persist {} falhou: {}", platform, e);
                let _ = app_task.emit("import_error", (platform, e.to_string()));
                return;
            }
        };

        let message = format_import_summary(platform, inserted, updated);
        info!("{}", message);

        let _ = app_task.emit("library_updated", ());
        let _ = app_task.emit("import_complete", (platform, message));

        trigger_enrichment_if_needed(&app_task, newly_imported);
    });
}

/// Resultado de uma importação, usado por `spawn_import_custom` para fontes com persistência própria
/// (IndieGala, Itch.io, Legacy Games), que gravam campos extras (`description_raw`, `tags`, `cover_url`)
/// fora do `SourceGame` padrão e por isso não passam por `persist_source_games`.
pub enum ImportOutcome {
    Empty,
    Persisted {
        inserted: u32,
        updated: u32,
        newly_imported: Vec<NewlyImportedGame>,
    },
}

/// Como `spawn_import`, mas para fontes cuja persistência já está embutida em `run`
/// (fetch + persist_*_games próprio), retornando o resultado final e não uma lista crua de `SourceGame`.
pub fn spawn_import_custom<F, Fut>(app: AppHandle, platform: &'static str, run: F)
where
    F: FnOnce(AppHandle) -> Fut + Send + 'static,
    Fut: std::future::Future<Output=Result<ImportOutcome, AppError>> + Send + 'static,
{
    let _ = app.emit("import_started", platform);
    let app_task = app.clone();
    let app_for_run = app.clone();

    tauri::async_runtime::spawn(async move {
        match run(app_for_run).await {
            Ok(ImportOutcome::Empty) => {
                let msg = format_import_empty(platform);
                let _ = app_task.emit("import_complete", (platform, msg));
            }
            Ok(ImportOutcome::Persisted {
                   inserted,
                   updated,
                   newly_imported,
               }) => {
                let message = format_import_summary(platform, inserted, updated);
                info!("{}", message);
                let _ = app_task.emit("library_updated", ());
                let _ = app_task.emit("import_complete", (platform, message));
                trigger_enrichment_if_needed(&app_task, newly_imported);
            }
            Err(e) => {
                warn!("Import {} falhou: {}", platform, e);
                let _ = app_task.emit("import_error", (platform, e.to_string()));
            }
        }
    });
}

// === Funções padronizadas para mensagens ===

/// Mensagem padrão de sucesso: "<Plataforma>: X adicionados, Y atualizados".
pub fn format_import_summary(platform: &str, inserted: u32, updated: u32) -> String {
    format!("{platform}: {inserted} adicionados, {updated} atualizados")
}

/// Mensagem padrão de biblioteca vazia: "Nenhum jogo <plataforma> encontrado."
pub fn format_import_empty(platform: &str) -> String {
    format!("Nenhum jogo {platform} encontrado.")
}

/// Mensagem padrão de conexão bem sucedida: "Conta <plataforma> conectada com sucesso!"
pub fn format_login_success(platform: &str) -> String {
    format!("Conta {platform} conectada com sucesso!")
}
