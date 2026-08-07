//! Abstração para provedores de conquistas de diferentes lojas/launchers.
//!
//! Cada loja implementa `AchievementProvider`. `get_recent_achievements`
//! (chamado por `commands::achievements::get_recent_achievements`) instancia todos os providers,
//! pula os que não estão configurados, e agrega o resultado dos demais.

use crate::errors::AppError;
use crate::services::achievements::epic::EpicProvider;
use crate::services::achievements::steam::SteamProvider;
use crate::services::achievements::xbox::XboxProvider;
use async_trait::async_trait;
use serde::Serialize;

const DASHBOARD_LIMIT: usize = 5;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Steam,
    Epic,
    Gog,
    Xbox,
}

#[derive(Serialize)]
pub struct DashboardAchievement {
    pub platform: Platform,
    pub game_name: String,
    pub achievement_name: String,
    pub unlock_time: i64,
    pub game_id: String,
}

#[async_trait]
pub(crate) trait AchievementProvider: Send + Sync {
    fn platform(&self) -> Platform;

    /// Deve responder rápido (sem chamada de rede) se há credenciais
    /// suficientes para consultar essa plataforma.
    async fn is_configured(&self, app: &tauri::AppHandle) -> bool;

    /// Busca até `limit` conquistas, priorizando as mais recentes e
    /// completando com conquistas mais antigas se necessário.
    async fn fetch_recent_achievements(
        &self,
        app: &tauri::AppHandle,
        limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError>;
}

/// Chamado pelo command Tauri. Agrega conquistas de todas as
/// plataformas configuradas, ordena por `unlock_time` e retorna só as
/// `DASHBOARD_LIMIT` mais recentes.
pub async fn get_recent_achievements(
    app: &tauri::AppHandle,
) -> Result<Vec<DashboardAchievement>, AppError> {
    let providers: Vec<Box<dyn AchievementProvider>> = vec![
        Box::new(SteamProvider),
        Box::new(EpicProvider),
        Box::new(XboxProvider),
    ];

    let mut all_achievements = Vec::new();

    for provider in providers {
        if !provider.is_configured(app).await {
            continue;
        }

        match provider
            .fetch_recent_achievements(app, DASHBOARD_LIMIT)
            .await
        {
            Ok(achievements) => all_achievements.extend(achievements),
            Err(err) => {
                // Erro em uma plataforma não derruba as demais.
                log::warn!(
                    "Falha ao buscar conquistas de {:?}: {err}",
                    provider.platform()
                );
            }
        }
    }

    all_achievements.sort_by(|a, b| b.unlock_time.cmp(&a.unlock_time));
    all_achievements.truncate(DASHBOARD_LIMIT);

    Ok(all_achievements)
}
