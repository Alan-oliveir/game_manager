//! Funções genéricas usadas na importação de bibliotecas de plataformas externas (Steam, Epic, GOG).
//!
//! Fornece comandos para salvar dados dos jogos nos bancos de dados e mensagens padronizadas.

use crate::constants;
use crate::database::AppState;
use crate::errors::AppError;
use crate::sources::scanner::GameDiscovery;
use crate::utils::status_logic;
use chrono::{TimeZone, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
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
                    favorite, user_rating, install_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, NULL, ?11)",
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
                    game.install_path
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
                    install_path = COALESCE(?5, install_path)
                 WHERE platform = ?6 AND platform_game_id = ?7",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    last_played_iso,
                    game.install_path,
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
