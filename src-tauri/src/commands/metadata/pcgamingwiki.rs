//! Comandos Tauri para obtenção e atualização de dados técnicos de jogos via PCGamingWiki.
//!
//! Implementa um fluxo **offline-first**: dados são servidos do cache SQLite sempre
//! que disponíveis, e buscados online apenas quando necessário. Falhas de rede são
//! tratadas graciosamente — o frontend recebe `None` em vez de um erro.

use crate::database::AppState;
use crate::errors::AppError;
use crate::models::{GameDataPath, GameExtras, SystemRequirements};
use crate::providers::technical::pcgamingwiki::client::{search_pcgw_by_name, PcgwSearchResult};
use crate::providers::technical::pcgamingwiki::db::{
    get_game_data_paths, get_pcgw_data, get_system_requirements, invalidate_pcgw_data,
    save_pcgw_data, save_scraped_data,
};
use crate::providers::technical::pcgamingwiki::fetch::fetch_pcgw_data;
use chrono::Utc;
use tracing::warn;

// === ESTRUTURAS ===

/// Dados completos do scraper para o frontend.
///
/// Retornados separadamente do `GameExtras` (Cargo) porque têm
/// cardinalidade N:1 — múltiplas linhas por jogo.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PcgwScrapedResponse {
    pub system_requirements: Vec<SystemRequirements>,
    pub config_paths: Vec<GameDataPath>,
    pub save_paths: Vec<GameDataPath>,
}

// === Comandos Tauri - API (fetch.rs) ===

/// Retorna dados do PCGamingWiki para um jogo.
///
/// Fluxo offline-first:
/// 1. Se existe no banco → retorna imediatamente (sem rede)
/// 2. Se não existe → busca online e persiste
/// 3. Se offline e sem dados → retorna erro tratável pelo frontend
///
/// O frontend deve tratar `AppError::NetworkError` exibindo a seção de
/// dados técnicos como indisponível, sem quebrar o resto da view do jogo.
#[tauri::command]
pub async fn get_or_fetch_pcgw_data(
    steam_app_id: String,
    conn: tauri::State<'_, AppState>,
) -> Result<Option<GameExtras>, String> {
    {
        let db = conn.games_db.lock().map_err(|e| e.to_string())?;
        if let Some(data) = get_pcgw_data(&db, &steam_app_id) {
            return Ok(Some(data));
        }
    }

    match fetch_pcgw_data(&steam_app_id).await {
        Ok((data, scraped)) => {
            let db = conn.games_db.lock().map_err(|e| e.to_string())?;
            save_pcgw_data(&db, &data).map_err(|e| e.to_string())?;

            if let Some(scraped_data) = scraped {
                save_scraped_data(&db, &data.steam_app_id, &scraped_data)
                    .map_err(|e| e.to_string())?;
            }

            Ok(Some(data))
        }
        // Jogo não existe na PCGW — persiste linha vazia para não tentar novamente
        Err(AppError::NotFound(msg)) => {
            warn!(
                "pcgw_data: jogo {} não encontrado na PCGW — {}",
                steam_app_id, msg
            );
            let empty = GameExtras {
                steam_app_id: steam_app_id.clone(),
                fetched_at: Some(Utc::now().to_rfc3339()),
                ..Default::default()
            };
            let db = conn.games_db.lock().map_err(|e| e.to_string())?;
            save_pcgw_data(&db, &empty).map_err(|e| e.to_string())?;
            Ok(None)
        }
        // Falha de rede — não persiste nada, frontend trata graciosamente
        Err(AppError::NetworkError(msg)) => {
            warn!("pcgw_data: falha de rede para {} — {}", steam_app_id, msg);
            Ok(None)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Força a re-busca dos dados do PCGamingWiki para um jogo.
///
/// Invalida o cache existente e dispara nova busca online imediatamente.
/// Útil quando o usuário quer atualizar manualmente (ex.: após um patch grande).
#[tauri::command]
pub async fn refresh_pcgw_data(
    steam_app_id: String,
    conn: tauri::State<'_, AppState>,
) -> Result<Option<GameExtras>, String> {
    {
        let db = conn.games_db.lock().map_err(|e| e.to_string())?;
        invalidate_pcgw_data(&db, &steam_app_id).map_err(|e| e.to_string())?;
    }

    // Reutiliza o fluxo completo de busca
    get_or_fetch_pcgw_data(steam_app_id, conn).await
}

/// Busca candidatos no PCGamingWiki por nome de jogo.
///
/// Retorna até 5 resultados ordenados por relevância para o usuário confirmar
/// o jogo correto — útil quando o Steam AppID não está disponível ou a busca
/// por AppID não retornou resultados.
///
/// # Erros
/// Retorna `Err(String)` em caso de falha de rede ou resposta inesperada da API.
#[tauri::command]
pub async fn search_pcgw_games(game_name: String) -> Result<Vec<PcgwSearchResult>, String> {
    search_pcgw_by_name(&game_name)
        .await
        .map_err(|e| e.to_string())
}

// === Comandos Tauri - scraper.rs ===

/// Retorna os dados do scraper (requisitos e caminhos) para um jogo.
///
/// Fluxo:
///   1. Consulta SQLite — se existir, retorna imediatamente
///   2. Se não existir e `steam_app_id` estiver disponível → busca online
///      via `fetch_pcgw_data` (que já salva scraper + cargo juntos)
///   3. Retorna `None` se o jogo não foi encontrado na PCGW
///      ou se não há Steam AppID disponível
#[tauri::command]
pub async fn get_pcgw_scraped_data(
    steam_app_id: String,
    conn: tauri::State<'_, AppState>,
) -> Result<Option<PcgwScrapedResponse>, String> {
    if steam_app_id.trim().is_empty() {
        return Ok(None);
    }

    // 1. Verifica se há dados no banco (scraper já foi executado antes)
    {
        let db = conn.games_db.lock().map_err(|e| e.to_string())?;

        // Se pcgw_data existe, o fetch já ocorreu — retorna scraper do cache
        if get_pcgw_data(&db, &steam_app_id).is_some() {
            let sysreqs = get_system_requirements(&db, &steam_app_id);
            let all_paths = get_game_data_paths(&db, &steam_app_id);
            let (config_paths, save_paths) = split_paths(all_paths);
            return Ok(Some(PcgwScrapedResponse {
                system_requirements: sysreqs,
                config_paths,
                save_paths,
            }));
        }
    }

    // 2. Não há dados — dispara fetch completo (cargo + scraper). Reutiliza get_or_fetch_pcgw_data
    match fetch_pcgw_data(&steam_app_id).await {
        Ok((pcgw_data, scraped)) => {
            let db = conn.games_db.lock().map_err(|e| e.to_string())?;

            // Persiste cargo data (pode já existir — idempotente via REPLACE)
            save_pcgw_data(&db, &pcgw_data).map_err(|e| e.to_string())?;

            if let Some(scraped_data) = scraped {
                save_scraped_data(&db, &steam_app_id, &scraped_data).map_err(|e| e.to_string())?;

                let (config_paths, save_paths) = split_paths(scraped_data.game_data_paths);

                return Ok(Some(PcgwScrapedResponse {
                    system_requirements: scraped_data.system_requirements,
                    config_paths,
                    save_paths,
                }));
            }

            // fetch ok mas scraper retornou None (falha de rede no wikitext)
            Ok(None)
        }
        Err(AppError::NotFound(_)) => Ok(None),
        Err(AppError::NetworkError(msg)) => {
            warn!("get_pcgw_scraped_data: falha de rede — {}", msg);
            Ok(None)
        }
        Err(e) => Err(e.to_string()),
    }
}

// === HELPERS ===

/// Separa `Vec<GameDataPath>` em (config_paths, save_paths).
fn split_paths(paths: Vec<GameDataPath>) -> (Vec<GameDataPath>, Vec<GameDataPath>) {
    let config = paths
        .iter()
        .filter(|p| p.kind == "config")
        .cloned()
        .collect();
    let saves = paths
        .iter()
        .filter(|p| p.kind == "saves")
        .cloned()
        .collect();
    (config, saves)
}
