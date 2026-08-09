//! Provider de conquistas da Xbox Live.
//!
//! Utiliza `XboxLiveSource::ensure_valid_xsts` (fluxo: MSA → XAU → XSTS)
//!
//! Formato:
//!   - `titleAssociations[].id`: número (i64)
//!   - `progression.timeUnlocked`: RFC3339 completo, ex.: `"2026-02-26T00:18:12.4540000Z"`

use crate::constants::XBOX_ACHIEVEMENTS_BASE_URL;
use crate::database;
use crate::errors::AppError;
use crate::services::achievements::core::{AchievementProvider, DashboardAchievement, Platform};
use crate::services::integration::xbox_live::XboxLiveSource;
use crate::utils::http_client::HTTP_CLIENT;
use async_trait::async_trait;
use serde::Deserialize;
use tauri::AppHandle;

// === STRUCTS ===

#[derive(Deserialize)]
struct XboxAchievementsResponse {
    achievements: Vec<XboxAchievement>,
}

#[derive(Deserialize)]
struct XboxAchievement {
    name: String,
    #[serde(rename = "progressState")]
    progress_state: String,
    progression: XboxProgression,
    #[serde(rename = "titleAssociations")]
    title_associations: Vec<XboxTitleAssociation>,
}

#[derive(Deserialize)]
struct XboxProgression {
    #[serde(rename = "timeUnlocked", default)]
    time_unlocked: Option<String>,
}

#[derive(Deserialize)]
struct XboxTitleAssociation {
    name: String,
    id: i64,
}

pub struct XboxProvider;

#[async_trait]
impl AchievementProvider for XboxProvider {
    fn platform(&self) -> Platform {
        Platform::Xbox
    }

    async fn is_configured(&self, app: &AppHandle) -> bool {
        let Some(source) = build_source(app) else {
            return false;
        };
        source.is_authenticated().unwrap_or(false)
    }

    async fn fetch_recent_achievements(
        &self,
        app: &AppHandle,
        limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        let Some(source) = build_source(app) else {
            return Ok(vec![]);
        };

        // Mesmo token usado pra importar a biblioteca — se estiver
        // expirado, renova sozinho (inclui novo login MSA via refresh).
        let (user_hash, xsts_token, xuid) = source.ensure_valid_xsts().await?;
        let auth_header = format!("XBL3.0 x={user_hash};{xsts_token}");

        // Faz a requisição para buscar as conquistas recentes.
        let url = format!("{XBOX_ACHIEVEMENTS_BASE_URL}/users/xuid({xuid})/achievements");

        let response = HTTP_CLIENT
            .get(&url)
            .header("Authorization", &auth_header)
            .header("x-xbl-contract-version", "2")
            .query(&[
                ("maxItems", limit.to_string()),
                ("orderBy", "UnlockTime".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            // Não derruba o dashboard — só não mostra Xbox dessa vez.
            return Ok(vec![]);
        }

        let body: XboxAchievementsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ParseError(format!("Xbox achievements: {e}")))?;

        let achievements = body
            .achievements
            .into_iter()
            .filter(|a| a.progress_state == "Achieved")
            .filter_map(|a| {
                let title = a.title_associations.first()?;
                let unlock_time = parse_unlock_time(a.progression.time_unlocked.as_deref()?)?;

                Some(DashboardAchievement {
                    platform: Platform::Xbox,
                    game_name: title.name.clone(),
                    achievement_name: a.name,
                    unlock_time,
                    game_id: title.id.to_string(),
                })
            })
            .collect();

        Ok(achievements)
    }
}

// === HELPERS LOCAIS ===

fn build_source(app: &AppHandle) -> Option<XboxLiveSource> {
    let client_id = database::get_secret(app, "xbox_live_client_id").ok()?;
    let client_secret = database::get_secret(app, "xbox_live_client_secret").ok()?;

    if client_id.is_empty() || client_secret.is_empty() {
        return None;
    }

    Some(XboxLiveSource::new(app.clone(), client_id, client_secret))
}

/// RFC3339 (ex. "2026-02-26T00:18:12.4540000Z").
fn parse_unlock_time(raw: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.timestamp())
}
