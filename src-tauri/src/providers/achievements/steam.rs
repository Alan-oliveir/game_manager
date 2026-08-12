//! Provider de conquistas da Steam.
//!
//! Estratégia: primeiro olha os jogos jogados nas últimas 2 semanas via
//! `get_recently_played_games` (mais provável de ter conquistas
//! novas). Se isso não preencher o `limit`, cai para `list_steam_games`
//! (biblioteca inteira), pra pegar conquistas mais antigas de jogos que
//! não foram jogados recentemente.

use crate::database;
use crate::errors::AppError;
use crate::providers::achievements::core::{AchievementProvider, DashboardAchievement, Platform};
use crate::providers::libraries::steam::{list_steam_games, SteamGame};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::AppHandle;

// === STRUCTS ===

/// Estrutura auxiliar para obter conquistas de um jogo.
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

    async fn fetch_recent_achievements(
        &self,
        app: &AppHandle,
        limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        let api_key = database::get_secret(app, "steam_api_key")?;
        let steam_id = database::get_secret(app, "steam_id")?;

        if api_key.is_empty() || steam_id.is_empty() {
            return Ok(vec![]);
        }

        let mut all_achievements = Vec::new();

        // 1. Jogos jogados nas últimas 2 semanas.
        let recent_games = get_recently_played_games(&api_key, &steam_id)
            .await
            .map_err(AppError::ExternalApiError)?;
        collect_unlocked(&api_key, &steam_id, &recent_games, &mut all_achievements).await;

        // 2. Se não juntamos o suficiente, busca no restante da
        //    biblioteca (conquistas mais antigas de jogos parados).
        if all_achievements.len() < limit {
            let owned_games = list_steam_games(&api_key, &steam_id)
                .await
                .map_err(AppError::ExternalApiError)?;

            let already_checked: HashSet<_> = recent_games.iter().map(|g| g.appid).collect();
            let remaining: Vec<SteamGame> = owned_games
                .into_iter()
                .filter(|g| !already_checked.contains(&g.appid))
                .collect();

            collect_unlocked(&api_key, &steam_id, &remaining, &mut all_achievements).await;
        }

        all_achievements.sort_by(|a, b| b.unlock_time.cmp(&a.unlock_time));
        all_achievements.truncate(limit);

        Ok(all_achievements)
    }
}

async fn collect_unlocked(
    api_key: &str,
    steam_id: &str,
    games: &[SteamGame],
    out: &mut Vec<DashboardAchievement>,
) {
    for game in games {
        if let Ok(achievements) = get_player_achievements(api_key, steam_id, game.appid).await
        {
            for ach in achievements {
                if ach.achieved == 1 {
                    out.push(DashboardAchievement {
                        platform: Platform::Steam,
                        game_name: game.name.clone(),
                        achievement_name: ach.name.unwrap_or(ach.apiname),
                        unlock_time: ach.unlocktime,
                        game_id: game.appid.to_string(),
                    });
                }
            }
        }
    }
}

// === HELPERS LOCAIS ===

/// Busca jogos jogados nas últimas 2 semanas
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

/// Busca conquistas do jogador num jogo específico
async fn get_player_achievements(
    api_key: &str,
    steam_id: &str,
    app_id: u32,
) -> Result<Vec<SteamAchievement>, String> {
    // Usa l=brazilian para tentar obter os nomes traduzidos se disponíveis
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={}&key={}&steamid={}&l=brazilian",
        app_id, api_key, steam_id
    );

    let res = crate::utils::http_client::HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // Jogos sem conquistas retornam 400 ou erro, e são tratados como lista vazia
    if !res.status().is_success() {
        return Ok(vec![]);
    }

    let data: Result<PlayerStatsResponse, _> = res.json().await;
    match data {
        Ok(d) => Ok(d.playerstats.achievements.unwrap_or_default()),
        Err(_) => Ok(vec![]), // Falha no parse (jogo sem conquistas públicas)
    }
}
