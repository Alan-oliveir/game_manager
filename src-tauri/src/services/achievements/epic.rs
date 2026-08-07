//! Provider de conquistas da Epic Games Store.
//!
//! Mesmo que `sources::epic` já tenha um fluxo OAuth funcionando para
//! importar a biblioteca de jogos, isso NÃO dá acesso a conquistas: o
//! OAuth de conta Epic autentica o usuário para operações de conta
//! (biblioteca, perfil), mas as APIs de conquistas da Epic Online
//! Services (EOS) exigem um `client_id`/`client_secret` *específico de
//! cada jogo*, que só o desenvolvedor daquele jogo possui. Não existe
//! endpoint documentado que exponha "todas as conquistas do usuário em
//! todos os jogos" via conta Epic.
//!
//! Fica como stub por enquanto. Se decidirmos investigar depois um
//! endpoint não-documentado usado pelo próprio site epicgames.com (que
//! mostra conquistas na página de conta), isso seria engenharia reversa
//! sobre um endpoint privado — vale avaliar risco de ToS antes de ir
//! por esse caminho.

use crate::errors::AppError;
use crate::services::achievements::core::{AchievementProvider, DashboardAchievement, Platform};
use async_trait::async_trait;
use tauri::AppHandle;

pub struct EpicProvider;

#[async_trait]
impl AchievementProvider for EpicProvider {
    fn platform(&self) -> Platform {
        Platform::Epic
    }

    async fn is_configured(&self, _app: &AppHandle) -> bool {
        false
    }

    async fn fetch_recent_achievements(
        &self,
        _app: &AppHandle,
        _limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        Ok(vec![])
    }
}
