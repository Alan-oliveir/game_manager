//! Provider de conquistas da Steam.
//!
//! `sync_achievements` roda em background (não é chamado pela UI diretamente):
//! 1. Jogos jogados nas últimas 2 semanas (via `get_recently_played_games`) —
//!    TTL curto (`STEAM_RECHECK_TTL_SECS`), porque é onde mais aparece
//!    conquista nova.
//! 2. Resto da biblioteca — TTL bem mais longo (`STEAM_LIBRARY_RECHECK_TTL_SECS`),
//!    só pra não deixar defasar de vez jogos parados.
//!
//! Cada jogo, antes de bater na API, passa por `achievements::should_skip`:
//! se já confirmamos (400 da Steam) que ele não tem stats de conquista, ou se
//! já foi sincronizado recentemente, pula sem gastar rate limit.

use crate::database;
use crate::database::achievements::{self, AchievementRecord};
use crate::errors::AppError;
use crate::providers::achievements::core::{AchievementProvider, Platform};
use crate::providers::libraries::steam::{list_steam_games, SteamGame};
use crate::services::rate_limiter::STEAM_LIMITER;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::AppHandle;
use tracing::warn;

const STEAM_RECHECK_TTL_SECS: i64 = 6 * 60 * 60; // 6h — jogos jogados recentemente
const STEAM_LIBRARY_RECHECK_TTL_SECS: i64 = 7 * 24 * 60 * 60; // 7 dias — resto da biblioteca
const STEAM_REQUEST_PACING_MS: u64 = 400; // pausa entre chamadas bem-sucedidas, evita rajada
const STEAM_CIRCUIT_BREAKER_THRESHOLD: u32 = 3; // falhas consecutivas (retries esgotados) até abortar a rodada

// === STRUCTS ===

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SteamAchievement {
    pub apiname: String,
    pub achieved: i32,
    pub unlocktime: i64,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlayerStats {
    achievements: Option<Vec<SteamAchievement>>,
}

#[derive(Debug, Deserialize)]
struct PlayerStatsResponse {
    playerstats: PlayerStats,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RecentGamesResponse {
    response: RecentGamesData,
}

#[derive(Debug, Deserialize)]
struct RecentGamesData {
    games: Option<Vec<SteamGame>>,
}

pub struct SteamProvider;

// === STEAM ACHIEVEMENTS PROVIDER ===

#[async_trait]
impl AchievementProvider for SteamProvider {
    fn platform(&self) -> Platform {
        Platform::Steam
    }

    async fn is_configured(&self, app: &AppHandle) -> bool {
        let api_key = database::get_secret(app, "steam_api_key").unwrap_or_default();
        let steam_id = database::get_secret(app, "steam_id").unwrap_or_default();
        !api_key.is_empty() && !steam_id.is_empty()
    }

    async fn sync_achievements(&self, app: &AppHandle) -> Result<usize, AppError> {
        let api_key = database::get_secret(app, "steam_api_key")?;
        let steam_id = database::get_secret(app, "steam_id")?;

        if api_key.is_empty() || steam_id.is_empty() {
            return Ok(0);
        }

        let mut total = 0;

        // 1. Jogos jogados nas últimas 2 semanas.
        let recent_games = STEAM_LIMITER
            .run(|| get_recently_played_games(&api_key, &steam_id))
            .await
            .unwrap_or_else(|err| {
                warn!("Steam get_recently_played_games falhou, seguindo com fallback: {err}");
                Vec::new()
            });

        total += sync_games(
            app,
            &api_key,
            &steam_id,
            &recent_games,
            STEAM_RECHECK_TTL_SECS,
        )
            .await;

        // 2. Resto da biblioteca.
        let owned_games = list_steam_games(&api_key, &steam_id)
            .await
            .map_err(AppError::ExternalApiError)?;

        let already_checked: HashSet<_> = recent_games.iter().map(|g| g.appid).collect();
        let remaining: Vec<SteamGame> = owned_games
            .into_iter()
            .filter(|g| !already_checked.contains(&g.appid))
            .collect();

        total += sync_games(
            app,
            &api_key,
            &steam_id,
            &remaining,
            STEAM_LIBRARY_RECHECK_TTL_SECS,
        )
            .await;

        Ok(total)
    }
}

/// Sincroniza uma lista de jogos, pulando os que já foram checados dentro
/// do `ttl_secs` ou que já sabemos não terem conquistas públicas.
async fn sync_games(
    app: &AppHandle,
    api_key: &str,
    steam_id: &str,
    games: &[SteamGame],
    ttl_secs: i64,
) -> usize {
    let mut total = 0;
    let mut consecutive_failures = 0u32;

    for game in games {
        let game_id = game.appid.to_string();

        if achievements::should_skip(app, Platform::Steam, &game_id, ttl_secs) {
            continue;
        }

        let result = STEAM_LIMITER
            .run(|| get_player_achievements(api_key, steam_id, game.appid))
            .await;

        match result {
            Ok(steam_achievements) => {
                consecutive_failures = 0; // sucesso reseta o contador

                let unlocked: Vec<AchievementRecord> = steam_achievements
                    .into_iter()
                    .filter(|a| a.achieved == 1)
                    .map(|a| AchievementRecord {
                        platform: Platform::Steam,
                        game_id: game_id.clone(),
                        game_name: game.name.clone(),
                        achievement_key: a.apiname.clone(),
                        achievement_name: a.name.unwrap_or(a.apiname),
                        achievement_description: a.description,
                        unlocked_at: a.unlocktime,
                        icon_url: None,
                    })
                    .collect();

                total += unlocked.len();

                if let Err(e) = achievements::upsert_achievements(app, &unlocked) {
                    warn!("Steam: falha ao salvar conquistas de {} ({}) no banco: {}", game.name, game.appid, e);
                }

                if let Err(e) = achievements::mark_synced(app, Platform::Steam, &game_id, true) {
                    warn!("Steam: falha ao marcar sync de {} ({}): {}", game.name, game.appid, e);
                }

                // Pacing: só pausa após sucesso, pra não acelerar quando já está tudo dando 400/403.
                tokio::time::sleep(std::time::Duration::from_millis(STEAM_REQUEST_PACING_MS)).await;
            }
            Err(e) => {
                if e.contains("400") {
                    // Permanente — jogo sem stats de conquista. Não conta como falha de throttling.
                    if let Err(db_err) = achievements::mark_synced(app, Platform::Steam, &game_id, false) {
                        warn!("Steam: falha ao marcar {} ({}) como sem conquistas: {}", game.name, game.appid, db_err);
                    }
                } else {
                    // Retries já foram esgotados dentro do STEAM_LIMITER.run() antes de chegar aqui.
                    consecutive_failures += 1;
                }

                warn!("Steam: falha ao buscar conquistas de {} ({}): {}", game.name, game.appid, e);

                if consecutive_failures >= STEAM_CIRCUIT_BREAKER_THRESHOLD {
                    warn!(
                        "Steam: {} falhas consecutivas após retries esgotados — abortando o resto desta rodada de sync, tentando de novo na próxima",
                        consecutive_failures
                    );
                    break;
                }
            }
        }
    }

    total
}

// === HELPERS LOCAIS ===

async fn get_recently_played_games(
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<SteamGame>, String> {
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetRecentlyPlayedGames/v0001/?key={}&steamid={}&format=json&count=10",
        api_key, steam_id
    );

    let res = crate::utils::http_client::HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro Steam Recent Games: {}", res.status()));
    }

    let data: RecentGamesResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data.response.games.unwrap_or_default())
}

async fn get_player_achievements(
    api_key: &str,
    steam_id: &str,
    app_id: u32,
) -> Result<Vec<SteamAchievement>, String> {
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={}&key={}&steamid={}&l=brazilian",
        app_id, api_key, steam_id
    );

    let res = crate::utils::http_client::HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Steam Achievements Error: {}", res.status()));
    }

    let data: PlayerStatsResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data.playerstats.achievements.unwrap_or_default())
}
