//! Provider de conquistas e autenticação da Xbox Live.
//!
//! Para usar este módulo, precisa registrar seu próprio app no Azure AD
//! (Personal Microsoft accounts only, redirect URI `http://localhost/auth/callback`)
//! e fornecer client_id/client_secret.
//!
//! Fluxo de autenticação em 3 etapas, encadeadas:
//! 1. OAuth2 padrão com login.live.com (MSA) → access_token
//! 2. Troca do access_token por um "User Token" Xbox (XAU)
//! 3. Troca do User Token por um "XSTS Token" (usado em toda chamada de API)
//!
//! Endpoints e formato de payload documentados publicamente pela comunidade;
//! usados como referência o cliente open-source xbox-webapi-python (MIT), reescrito
//! do zero nesta implementação — não portado literalmente.
//!
//! Formato de conquistas:
//!   - `titleAssociations[].id`: número (i64)
//!   - `progression.timeUnlocked`: RFC3339 completo, ex.: `"2026-02-26T00:18:12.4540000Z"`

use crate::constants::{
    XBOX_ACHIEVEMENTS_BASE_URL, XBOX_MSA_AUTHORIZE_ENDPOINT, XBOX_MSA_REDIRECT_URI,
    XBOX_MSA_SCOPES, XBOX_MSA_TOKEN_ENDPOINT, XBOX_USER_AUTH_ENDPOINT, XBOX_XSTS_AUTH_ENDPOINT,
};
use crate::database;
use crate::database::achievements::AchievementRecord;
use crate::errors::AppError;
use crate::providers::achievements::core::{AchievementPlatform, AchievementProvider};
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::config::{now_unix, OAuthToken};
use crate::utils::oauth::core::wait_for_auth_code;
use crate::utils::oauth::token_store::{delete_oauth_token, load_oauth_token, save_oauth_token};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

const XBOX_PROVIDER_ID: &str = "xbox_live";

// === STRUCTS: MSA OAuth2 & XSTS ===

#[derive(Debug, Deserialize)]
struct MsaTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct XauResponse {
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Debug, Deserialize)]
struct XstsResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XstsDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XstsDisplayClaims {
    xui: Vec<XstsUserInfo>,
}

#[derive(Debug, Deserialize)]
struct XstsUserInfo {
    #[serde(rename = "uhs")]
    user_hash: String,
    #[serde(rename = "xid", default)]
    xuid: Option<String>,
}

// === STRUCTS: Achievements ===

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

// === SOURCE DE AUTENTICAÇÃO ===

pub struct XboxLiveSource {
    app_handle: AppHandle,
    client_id: String,
    client_secret: String,
}

impl XboxLiveSource {
    pub fn new(app_handle: AppHandle, client_id: String, client_secret: String) -> Self {
        Self {
            app_handle,
            client_id,
            client_secret,
        }
    }

    pub fn is_authenticated(&self) -> Result<bool, AppError> {
        Ok(load_oauth_token(&self.app_handle, XBOX_PROVIDER_ID)?.is_some())
    }

    pub async fn login(&self) -> Result<(), AppError> {
        let auth_url = build_msa_authorize_url(&self.client_id)?;

        self.app_handle
            .opener()
            .open_url(auth_url.as_str(), None::<String>)
            .map_err(|e| AppError::OAuthConfigError(format!("Falha ao abrir navegador: {e}")))?;

        let port = 8902;
        let callback = tokio::task::spawn_blocking(move || wait_for_auth_code(port))
            .await
            .map_err(|e| AppError::OAuthConfigError(format!("Task de callback falhou: {e}")))?
            .map_err(AppError::OAuthConfigError)?;

        let msa = self.request_msa_token(&callback.code).await?;
        let xau = self.request_user_token(&msa.access_token).await?;
        let xsts = self.request_xsts_token(&xau.token).await?;

        self.save_full_token(&msa, &xsts)?;

        Ok(())
    }

    pub fn logout(&self) -> Result<(), AppError> {
        delete_oauth_token(&self.app_handle, XBOX_PROVIDER_ID)
    }

    pub(crate) async fn ensure_valid_xsts(&self) -> Result<(String, String, String), AppError> {
        let stored = load_oauth_token(&self.app_handle, XBOX_PROVIDER_ID)?
            .ok_or_else(|| AppError::OAuthTokenNotFound(XBOX_PROVIDER_ID.to_string()))?;

        let user_hash = stored.extra.get("user_hash").cloned();
        let xsts_token = stored.extra.get("xsts_token").cloned();
        let xuid = stored.extra.get("xuid").cloned();

        if !stored.is_expired() {
            if let (Some(uhs), Some(xsts), Some(xuid)) = (user_hash, xsts_token, xuid) {
                return Ok((uhs, xsts, xuid));
            }
        }

        let refresh_token = stored.refresh_token.clone().ok_or_else(|| {
            AppError::OAuthTokenNotFound(
                "xbox_live: token expirado e sem refresh_token, é necessário novo login".to_string(),
            )
        })?;

        let msa = self.refresh_msa_token(&refresh_token).await?;
        let xau = self.request_user_token(&msa.access_token).await?;
        let xsts = self.request_xsts_token(&xau.token).await?;

        self.save_full_token(&msa, &xsts)?;

        let user_info = xsts.display_claims.xui.first().ok_or_else(|| {
            AppError::OAuthConfigError("Resposta XSTS sem DisplayClaims.xui".to_string())
        })?;
        let xuid = user_info.xuid.clone().ok_or_else(|| {
            AppError::OAuthConfigError("XSTS não retornou 'xid' (XUID)".to_string())
        })?;

        Ok((user_info.user_hash.clone(), xsts.token.clone(), xuid))
    }

    async fn request_msa_token(&self, code: &str) -> Result<MsaTokenResponse, AppError> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("scope", XBOX_MSA_SCOPES),
            ("redirect_uri", XBOX_MSA_REDIRECT_URI),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = HTTP_CLIENT
            .post(XBOX_MSA_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        parse_json_or_error(response, "MSA token exchange").await
    }

    async fn refresh_msa_token(&self, refresh_token: &str) -> Result<MsaTokenResponse, AppError> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", XBOX_MSA_SCOPES),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = HTTP_CLIENT
            .post(XBOX_MSA_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        parse_json_or_error(response, "MSA token refresh").await
    }

    async fn request_user_token(&self, msa_access_token: &str) -> Result<XauResponse, AppError> {
        let body = serde_json::json!({
            "RelyingParty": "https://auth.xboxlive.com",
            "TokenType": "JWT",
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={msa_access_token}"),
            }
        });

        let response = HTTP_CLIENT
            .post(XBOX_USER_AUTH_ENDPOINT)
            .header("x-xbl-contract-version", "1")
            .json(&body)
            .send()
            .await?;

        parse_json_or_error(response, "Xbox user token (XAU)").await
    }

    async fn request_xsts_token(&self, user_token: &str) -> Result<XstsResponse, AppError> {
        let body = serde_json::json!({
            "RelyingParty": "https://xboxlive.com",
            "TokenType": "JWT",
            "Properties": {
                "UserTokens": [user_token],
                "SandboxId": "RETAIL",
            }
        });

        let response = HTTP_CLIENT
            .post(XBOX_XSTS_AUTH_ENDPOINT)
            .header("x-xbl-contract-version", "1")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(AppError::OAuthConfigError(
                "Xbox Live recusou a autorização (401) — pode ser conta infantil (<18 anos) ou sandbox não elegível".to_string(),
            ));
        }

        parse_json_or_error(response, "Xbox XSTS token").await
    }

    fn save_full_token(&self, msa: &MsaTokenResponse, xsts: &XstsResponse) -> Result<(), AppError> {
        let user_info = xsts.display_claims.xui.first().ok_or_else(|| {
            AppError::OAuthConfigError("Resposta XSTS sem DisplayClaims.xui".to_string())
        })?;

        let mut extra = HashMap::new();
        extra.insert("user_hash".to_string(), user_info.user_hash.clone());
        extra.insert("xsts_token".to_string(), xsts.token.clone());
        if let Some(xuid) = &user_info.xuid {
            extra.insert("xuid".to_string(), xuid.clone());
        }

        let token = OAuthToken {
            access_token: msa.access_token.clone(),
            refresh_token: msa.refresh_token.clone(),
            expires_at: msa.expires_in.map(|secs| now_unix() + secs as i64),
            scope: None,
            extra,
        };

        save_oauth_token(&self.app_handle, XBOX_PROVIDER_ID, &token)
    }
}

// === PROVIDER DE CONQUISTAS ===

pub struct XboxProvider;

#[async_trait]
impl AchievementProvider for XboxProvider {
    fn library(&self) -> AchievementPlatform {
        AchievementPlatform::Xbox
    }

    async fn is_configured(&self, app: &AppHandle) -> bool {
        let Some(source) = build_source(app) else {
            return false;
        };
        source.is_authenticated().unwrap_or(false)
    }

    async fn sync_achievements(&self, app: &AppHandle) -> Result<usize, AppError> {
        let Some(source) = build_source(app) else {
            return Ok(0);
        };

        let (user_hash, xsts_token, xuid) = source.ensure_valid_xsts().await?;
        let auth_header = format!("XBL3.0 x={user_hash};{xsts_token}");

        let url = format!("{XBOX_ACHIEVEMENTS_BASE_URL}/users/xuid({xuid})/achievements");

        let response = HTTP_CLIENT
            .get(&url)
            .header("Authorization", &auth_header)
            .header("x-xbl-contract-version", "2")
            .query(&[
                ("maxItems", "200".to_string()),
                ("orderBy", "UnlockTime".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            // Mantém o comportamento atual (conhecidamente com bug) — não derruba o sync das outras plataformas.
            return Ok(0);
        }

        let body: XboxAchievementsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ParseError(format!("Xbox achievements: {e}")))?;

        let records: Vec<AchievementRecord> = body
            .achievements
            .into_iter()
            .filter(|a| a.progress_state == "Achieved")
            .filter_map(|a| {
                let title = a.title_associations.first()?;
                let unlock_time = parse_unlock_time(a.progression.time_unlocked.as_deref()?)?;

                Some(AchievementRecord {
                    library: AchievementPlatform::Xbox,
                    game_id: title.id.to_string(),
                    game_name: title.name.clone(),
                    // Xbox não expõe aqui um id estável separado do nome da conquista.
                    achievement_key: a.name.clone(),
                    achievement_name: a.name,
                    achievement_description: None,
                    unlocked_at: unlock_time,
                    icon_url: None,
                })
            })
            .collect();

        crate::database::achievements::upsert_achievements(app, &records)
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

fn build_msa_authorize_url(client_id: &str) -> Result<url::Url, AppError> {
    let mut url = url::Url::parse(XBOX_MSA_AUTHORIZE_ENDPOINT)
        .map_err(|e| AppError::OAuthConfigError(format!("URL de autorização MSA inválida: {e}")))?;

    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("approval_prompt", "auto")
        .append_pair("scope", XBOX_MSA_SCOPES)
        .append_pair("redirect_uri", XBOX_MSA_REDIRECT_URI);

    Ok(url)
}

/// RFC3339 (ex. "2026-02-26T00:18:12.4540000Z").
fn parse_unlock_time(raw: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.timestamp())
}

async fn parse_json_or_error<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    stage: &str,
) -> Result<T, AppError> {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(AppError::NetworkError(format!(
            "{stage} retornou HTTP {status}: {body}"
        )));
    }

    serde_json::from_str(&body)
        .map_err(|e| AppError::ParseError(format!("Falha ao parsear {stage}: {e} — corpo: {body}")))
}
