//! Provider de conquistas da Steam.
//!
//! Estratégia: primeiro olha os jogos jogados nas últimas 2 semanas via
//! `get_recently_played_games` (mais provável de ter conquistas
//! novas). Se isso não preencher o `limit`, cai para `list_steam_games`
//! (biblioteca inteira), pra pegar conquistas mais antigas de jogos que
//! não foram jogados recentemente.

use crate::database;
use crate::errors::AppError;
use crate::services::achievements::core::{AchievementProvider, DashboardAchievement, Platform};
use crate::services::integration::steam_api::{self, SteamGame};
use async_trait::async_trait;
use std::collections::HashSet;
use tauri::AppHandle;

pub struct SteamProvider;

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
        let recent_games = steam_api::get_recently_played_games(&api_key, &steam_id)
            .await
            .map_err(AppError::ExternalApiError)?;
        collect_unlocked(&api_key, &steam_id, &recent_games, &mut all_achievements).await;

        // 2. Se não juntamos o suficiente, busca no restante da
        //    biblioteca (conquistas mais antigas de jogos parados).
        if all_achievements.len() < limit {
            let owned_games = steam_api::list_steam_games(&api_key, &steam_id)
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
        if let Ok(achievements) =
            steam_api::get_player_achievements(api_key, steam_id, game.appid).await
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
