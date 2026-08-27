//! Abstração para provedores de conquistas de diferentes lojas/launchers.
//!
//! Cada loja implementa `AchievementProvider`. Steam/Xbox sincronizam em
//! background (`sync_achievements`, grava em `achievements` via
//! `database::achievements::upsert_achievements`); GOG lê direto do banco
//! local do Galaxy a cada chamada (`fetch_recent_achievements` /
//! `fetch_all_achievements`), sem precisar de sync/throttle.

use crate::errors::AppError;
use crate::providers::achievements::epic::EpicProvider;
use crate::providers::achievements::gog::GogProvider;
use crate::providers::achievements::steam::SteamProvider;
use crate::providers::achievements::xbox::XboxProvider;
use async_trait::async_trait;
use serde::Serialize;
use tauri::AppHandle;

const DASHBOARD_LIMIT: usize = 3;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AchievementPlatform {
    Steam,
    Epic,
    Gog,
    Xbox,
}

#[derive(Serialize)]
pub struct DashboardAchievement {
    pub source: AchievementPlatform,
    pub game_name: String,
    pub achievement_name: String,
    pub unlock_time: i64,
    pub game_id: String,
}

#[derive(Serialize)]
pub struct AchievementDetail {
    pub source: AchievementPlatform,
    pub game_id: String,
    pub game_name: String,
    pub achievement_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub rarity_percent: Option<f64>,
    pub rarity_slug: Option<String>,
    pub category: Option<String>,
    pub unlock_time: i64,
}

#[async_trait]
pub(crate) trait AchievementProvider: Send + Sync {
    fn library(&self) -> AchievementPlatform;

    async fn is_configured(&self, app: &AppHandle) -> bool;

    /// Sync em background: busca da API e grava em `achievements` local.
    /// Usado por plataformas com rate limit (Steam, Xbox). Default no-op.
    async fn sync_achievements(&self, _app: &AppHandle) -> Result<usize, AppError> {
        Ok(0)
    }

    /// Busca direto (sem passar pelo `achievements` local), pra fontes já
    /// locais/baratas (ex.: GOG). Default vazio.
    async fn fetch_recent_achievements(
        &self,
        _app: &AppHandle,
        _limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        Ok(vec![])
    }

    /// Igual acima, mas sem limite e com os metadados extras da tela
    /// dedicada. Default vazio.
    async fn fetch_all_achievements(
        &self,
        _app: &AppHandle,
    ) -> Result<Vec<AchievementDetail>, AppError> {
        Ok(vec![])
    }
}

fn all_providers() -> Vec<Box<dyn AchievementProvider>> {
    vec![
        Box::new(SteamProvider),
        Box::new(EpicProvider),
        Box::new(GogProvider),
        Box::new(XboxProvider),
    ]
}

/// Roda o sync de background de quem implementa `sync_achievements`
/// (hoje: Steam, Xbox). Chamado pelo scheduler, não pela UI.
pub async fn sync_all_achievements(app: &AppHandle) -> Result<usize, AppError> {
    let mut total = 0;
    for provider in all_providers() {
        if !provider.is_configured(app).await {
            continue;
        }
        match provider.sync_achievements(app).await {
            Ok(n) => total += n,
            Err(err) => log::warn!(
                "Falha ao sincronizar conquistas de {:?}: {err}",
                provider.library()
            ),
        }
    }
    Ok(total)
}

/// Chamado pelo command Tauri (card da Home). Combina o que já está
/// sincronizado localmente (Steam/Xbox) com o que as plataformas de
/// leitura direta (GOG) retornarem agora, ordena e trunca em `DASHBOARD_LIMIT`.
pub async fn get_recent_achievements(
    app: &AppHandle,
) -> Result<Vec<DashboardAchievement>, AppError> {
    let mut all_achievements =
        crate::database::achievements::get_dashboard_achievements(app, DASHBOARD_LIMIT)?;

    for provider in all_providers() {
        if !provider.is_configured(app).await {
            continue;
        }
        match provider.fetch_recent_achievements(app, DASHBOARD_LIMIT).await {
            Ok(achievements) => all_achievements.extend(achievements),
            Err(err) => log::warn!(
                "Falha ao buscar conquistas de {:?}: {err}",
                provider.library()
            ),
        }
    }

    all_achievements.sort_by(|a, b| b.unlock_time.cmp(&a.unlock_time));
    all_achievements.truncate(DASHBOARD_LIMIT);
    Ok(all_achievements)
}

/// Chamado pela tela dedicada. Agrega o que as plataformas de leitura
/// direta (GOG) retornarem, sem truncar.
///
/// NOTA: hoje isso não inclui o histórico completo do Steam/Xbox salvo
/// localmente — `database::achievements` só expõe `get_dashboard_achievements`
/// (com limite). Se quiser a lista completa deles aqui também, é só
/// adicionar um `get_all_achievements()` sem LIMIT lá e mapear pra
/// `AchievementDetail` (dá pra manter `description`/`rarity`/`category`
/// como `None`, já que a tabela `achievements` não guarda isso hoje).
pub async fn list_all_achievements(
    app: &AppHandle,
) -> Result<Vec<AchievementDetail>, AppError> {
    let mut all_achievements = Vec::new();

    for provider in all_providers() {
        if !provider.is_configured(app).await {
            continue;
        }
        match provider.fetch_all_achievements(app).await {
            Ok(achievements) => all_achievements.extend(achievements),
            Err(err) => log::warn!(
                "Falha ao buscar lista completa de conquistas de {:?}: {err}",
                provider.library()
            ),
        }
    }

    all_achievements.sort_by(|a, b| b.unlock_time.cmp(&a.unlock_time));
    Ok(all_achievements)
}
