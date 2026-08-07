//! Provider de conquistas da Xbox Live / Xbox Network.
//!
//! A Microsoft tem uma API oficial (Xbox Live Services API) com
//! conquistas, mas o fluxo de autenticação tem várias etapas e NÃO é
//! um OAuth simples:
//!   1. Registrar um app no Azure AD / Microsoft Entra e habilitar o
//!      escopo Xbox Live (`Xboxlive.signin`).
//!   2. Login do usuário via Microsoft Identity Platform → access_token.
//!   3. Trocar esse token por um "user token" e depois por um
//!      "XSTS token" (endpoints próprios da Xbox Live, fora do padrão
//!      OAuth2 comum).
//!   4. Usar o XSTS token pra chamar
//!      `https://achievements.xboxlive.com/users/xuid({xuid})/achievements`.
//!
//! Os passos 1–3 ficam fora deste arquivo: eles deveriam acontecer numa
//! tela de "conectar conta Xbox" separada, que salva `xbox_xsts_token`
//! e `xbox_xuid` via `database::set_secret`, do mesmo jeito que você já
//! guarda `steam_api_key`. O XSTS token expira (geralmente em algumas
//! horas), então essa tela também precisa de um fluxo de refresh.
//!
//! Alternativa mais rápida pra prototipar: serviços como o OpenXBL
//! (openxbl.com) simplificam bastante esse fluxo de auth em troca de
//! uma API key própria deles. Isso reduz o trabalho de implementação,
//! mas adiciona uma dependência de um serviço de terceiros não-oficial
//! (rate limits e disponibilidade fora do seu controle) — avalie se
//! isso é aceitável pro seu app antes de usar em produção.

use crate::database;
use crate::errors::AppError;
use crate::services::achievements::core::{AchievementProvider, DashboardAchievement, Platform};
use async_trait::async_trait;
use tauri::AppHandle;

pub struct XboxProvider;

#[async_trait]
impl AchievementProvider for XboxProvider {
    fn platform(&self) -> Platform {
        Platform::Xbox
    }

    async fn is_configured(&self, app: &AppHandle) -> bool {
        let token = database::get_secret(app, "xbox_xsts_token").unwrap_or_default();
        let xuid = database::get_secret(app, "xbox_xuid").unwrap_or_default();
        !token.is_empty() && !xuid.is_empty()
    }

    async fn fetch_recent_achievements(
        &self,
        app: &AppHandle,
        limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        let token = database::get_secret(app, "xbox_xsts_token")?;
        let xuid = database::get_secret(app, "xbox_xuid")?;

        if token.is_empty() || xuid.is_empty() {
            return Ok(vec![]);
        }

        let client = reqwest::Client::new();
        let url = format!(
            "https://achievements.xboxlive.com/users/xuid({xuid})/achievements?maxItems={limit}&orderBy=UnlockTime"
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("XBL3.0 x={token}"))
            .header("x-xbl-contract-version", "2")
            .send()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;

        if !resp.status().is_success() {
            // Token expirado é o caso mais comum aqui — a tela de
            // "conectar conta Xbox" deveria pedir relogin quando isso
            // acontecer, em vez de simplesmente sumir do dashboard.
            return Ok(vec![]);
        }

        let body: XboxAchievementsResponse = resp
            .json()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;

        let achievements = body
            .achievements
            .into_iter()
            .filter(|a| a.progress_state == "Achieved")
            .filter_map(|a| {
                let title = a.title_associations.first()?;
                Some(DashboardAchievement {
                    platform: Platform::Xbox,
                    game_name: title.name.clone(),
                    achievement_name: a.name,
                    unlock_time: a.progression.time_unlocked_unix,
                    game_id: title.id.clone(),
                })
            })
            .collect();

        Ok(achievements)
    }
}

// Structs simplificadas — o payload real da Xbox Live API tem bem mais
// campos do que isso. Ajuste conforme a resposta que você receber.
#[derive(serde::Deserialize)]
struct XboxAchievementsResponse {
    achievements: Vec<XboxAchievement>,
}

#[derive(serde::Deserialize)]
struct XboxAchievement {
    name: String,
    #[serde(rename = "progressState")]
    progress_state: String,
    progression: XboxProgression,
    #[serde(rename = "titleAssociations")]
    title_associations: Vec<XboxTitleAssociation>,
}

#[derive(serde::Deserialize)]
struct XboxProgression {
    #[serde(rename = "timeUnlocked")]
    time_unlocked_unix: i64,
}

#[derive(serde::Deserialize)]
struct XboxTitleAssociation {
    name: String,
    id: String,
}
