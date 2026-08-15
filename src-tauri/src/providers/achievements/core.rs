//! Abstração para provedores de conquistas de diferentes lojas/launchers.
//!
//! Cada loja implementa `AchievementProvider`. `sync_achievements` bate nas
//! APIs externas e persiste no SQLite (chamado só pelo job em background).
//! `get_recent_achievements` (chamado pelo command Tauri do dashboard) só lê
//! do banco — nunca faz chamada de rede, então é instantâneo.

use crate::errors::AppError;
use crate::providers::achievements::epic::EpicProvider;
use crate::providers::achievements::steam::SteamProvider;
use crate::providers::achievements::xbox::XboxProvider;
use async_trait::async_trait;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::warn;

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

    /// Deve responder rápido (sem chamada de rede) se há credenciais suficientes para consultar essa plataforma.
    async fn is_configured(&self, app: &AppHandle) -> bool;

    /// Busca conquistas na API externa e persiste no SQLite via `database::achievements`. Retorna
    /// quantas linhas foram inseridas/atualizadas (só para log — não precisa ser exato).
    async fn sync_achievements(&self, app: &AppHandle) -> Result<usize, AppError>;
}

/// Chamado pelo job em background (nunca diretamente pela UI). Roda o sync de todas as plataformas
/// configuradas, uma de cada vez, e emite `achievements-updated` se algo novo foi salvo.
pub async fn sync_all_achievements(app: &AppHandle) -> Result<(), AppError> {
    let providers: Vec<Box<dyn AchievementProvider>> = vec![
        Box::new(SteamProvider),
        Box::new(EpicProvider),
        Box::new(XboxProvider),
    ];

    let mut total = 0;

    for provider in providers {
        if !provider.is_configured(app).await {
            continue;
        }

        match provider.sync_achievements(app).await {
            Ok(count) => total += count,
            Err(err) => {
                // Erro em uma plataforma não derruba as demais.
                warn!(
                    "Falha ao sincronizar conquistas de {:?}: {err}",
                    provider.platform()
                );
            }
        }
    }

    if total > 0 {
        let _ = app.emit("achievements-updated", ());
    }

    Ok(())
}

/// Chamado pelo command Tauri do dashboard. Só lê do cache local — rápido, nunca bloqueia esperando API externa.
pub fn get_recent_achievements(app: &AppHandle) -> Result<Vec<DashboardAchievement>, AppError> {
    crate::database::achievements::get_dashboard_achievements(app, DASHBOARD_LIMIT)
}
