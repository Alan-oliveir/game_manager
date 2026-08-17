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
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::achievements::core::{AchievementProvider, Library};
use crate::providers::libraries::steam::SteamGame;
use crate::services::cache;
use crate::services::rate_limiter::STEAM_LIMITER;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{AppHandle, Manager};
use tracing::warn;

const STEAM_RECHECK_TTL_SECS: i64 = 6 * 60 * 60; // 6h — jogos jogados recentemente
const STEAM_LIBRARY_RECHECK_TTL_SECS: i64 = 7 * 24 * 60 * 60; // 7 dias — resto da biblioteca
const STEAM_CIRCUIT_BREAKER_THRESHOLD: u32 = 1; // falhas consecutivas (retries esgotados) até abortar a rodada
const STEAM_CIRCUIT_BREAKER_CACHE_SOURCE: &str = "steam";
const STEAM_CIRCUIT_BREAKER_CACHE_KEY: &str = "achievements_circuit_breaker_steam";

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

/// Jogo a sincronizar — id + nome, independente de vir da API (recentes) ou da tabela `games` local (resto da biblioteca).
struct OwnedGame {
    id: String,
    name: String,
}

// === STEAM ACHIEVEMENTS PROVIDER ===

pub struct SteamProvider;

#[async_trait]
impl AchievementProvider for SteamProvider {
    fn library(&self) -> Library {
        Library::Steam
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

        if is_circuit_breaker_active(app) {
            warn!("Steam: pulando sync inteiro — circuit breaker ativo");
            return Ok(0);
        }

        let mut total = 0;

        // 1. Jogos jogados nas últimas 2 semanas — usa a API, porque é sobre atividade recente, não sobre biblioteca.
        let recent_games_raw = STEAM_LIMITER
            .run(|| get_recently_played_games(&api_key, &steam_id))
            .await
            .unwrap_or_else(|err| {
                warn!("Steam get_recently_played_games falhou, seguindo com fallback: {err}");
                if err.contains("403") {
                    trip_circuit_breaker(app);
                }
                Vec::new()
            });

        if is_circuit_breaker_active(app) {
            return Ok(total);
        }

        let recent_games: Vec<OwnedGame> = recent_games_raw
            .iter()
            .map(|g| OwnedGame {
                id: g.appid.to_string(),
                name: g.name.clone(),
            })
            .collect();

        let (recent_total, tripped) = sync_games(
            app,
            &api_key,
            &steam_id,
            &recent_games,
            STEAM_RECHECK_TTL_SECS,
        )
            .await;
        total += recent_total;

        if tripped {
            return Ok(total);
        }

        // 2. Resto da biblioteca — lido de `games` (já importado),sem nova chamada à API pra "GetOwnedGames".
        let owned_games = achievements::get_owned_games_by_library(app, "steam")?;

        let already_checked: HashSet<&str> = recent_games.iter().map(|g| g.id.as_str()).collect();
        let remaining: Vec<OwnedGame> = owned_games
            .into_iter()
            .filter(|(id, _)| !already_checked.contains(id.as_str()))
            .map(|(id, name)| OwnedGame { id, name })
            .collect();

        let (library_total, _tripped) = sync_games(
            app,
            &api_key,
            &steam_id,
            &remaining,
            STEAM_LIBRARY_RECHECK_TTL_SECS,
        )
            .await;
        total += library_total;

        Ok(total)
    }
}

async fn sync_games(
    app: &AppHandle,
    api_key: &str,
    steam_id: &str,
    games: &[OwnedGame],
    ttl_secs: i64,
) -> (usize, bool) {
    let mut total = 0;
    let mut consecutive_failures = 0u32;

    for game in games {
        if achievements::should_skip(app, Library::Steam, &game.id, ttl_secs) {
            continue;
        }

        let result = STEAM_LIMITER
            .run(|| get_player_achievements(api_key, steam_id, &game.id))
            .await;

        match result {
            Ok(steam_achievements) => {
                consecutive_failures = 0;

                let unlocked: Vec<AchievementRecord> = steam_achievements
                    .into_iter()
                    .filter(|a| a.achieved == 1)
                    .map(|a| AchievementRecord {
                        library: Library::Steam,
                        game_id: game.id.clone(),
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
                    warn!(
                        "Steam: falha ao salvar conquistas de {} ({}) no banco: {}",
                        game.name, game.id, e
                    );
                }

                if let Err(e) = achievements::mark_synced(app, Library::Steam, &game.id, true) {
                    warn!(
                        "Steam: falha ao marcar sync de {} ({}): {}",
                        game.name, game.id, e
                    );
                }
            }
            Err(e) => {
                if e.contains("400") {
                    if let Err(db_err) =
                        achievements::mark_synced(app, Library::Steam, &game.id, false)
                    {
                        warn!(
                            "Steam: falha ao marcar {} ({}) como sem conquistas: {}",
                            game.name, game.id, db_err
                        );
                    }
                } else {
                    if let Err(db_err) =
                        achievements::mark_synced(app, Library::Steam, &game.id, true)
                    {
                        warn!(
                            "Steam: falha ao marcar tentativa de {} ({}): {}",
                            game.name, game.id, db_err
                        );
                    }
                    consecutive_failures += 1;
                }

                warn!(
                    "Steam: falha ao buscar conquistas de {} ({}): {}",
                    game.name, game.id, e
                );

                if consecutive_failures >= STEAM_CIRCUIT_BREAKER_THRESHOLD {
                    warn!(
                        "Steam: {} falhas consecutivas após retries esgotados — disparando circuit breaker",
                        consecutive_failures
                    );
                    trip_circuit_breaker(app);
                    return (total, true);
                }
            }
        }
    }

    (total, false)
}

// === CIRCUIT BREAKER (via api_cache, não achievement_sync_state) ===

fn is_circuit_breaker_active(app: &AppHandle) -> bool {
    let state: tauri::State<AppState> = app.state();
    let conn = match state.cache_db.lock() {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "Steam: falha ao checar circuit breaker (mutex cache_db): {}",
                e
            );
            return false; // erro de lock não deve travar o sync indefinidamente
        }
    };
    cache::get_cached_api_data(
        &conn,
        STEAM_CIRCUIT_BREAKER_CACHE_SOURCE,
        STEAM_CIRCUIT_BREAKER_CACHE_KEY,
    )
        .is_some()
}

fn trip_circuit_breaker(app: &AppHandle) {
    let state: tauri::State<AppState> = app.state();
    let conn = match state.cache_db.lock() {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "Steam: falha ao registrar circuit breaker (mutex cache_db): {}",
                e
            );
            return;
        }
    };
    if let Err(e) = cache::save_cached_api_data(
        &conn,
        STEAM_CIRCUIT_BREAKER_CACHE_SOURCE,
        STEAM_CIRCUIT_BREAKER_CACHE_KEY,
        "blocked",
    ) {
        warn!("Steam: falha ao salvar circuit breaker no cache: {}", e);
    }
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
    app_id: &str,
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
