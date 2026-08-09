//! Armazenamento persistente de tokens OAuth2 por provedor.
//!
//! Reaproveita a infraestrutura de secrets já existente (`secrets.db` +
//! AES-256-GCM via `security::encrypt`/`decrypt`), guardando cada token
//! serializado em JSON sob a chave `oauth_<provider_id>` na tabela
//! `encrypted_keys`. Nenhuma tabela ou schema novo é necessário.

use crate::database::core::{delete_secret, get_secret, set_secret};
use crate::errors::AppError;
use crate::utils::oauth::config::{
    fetch_app_access_token, refresh_access_token, OAuthProviderConfig, OAuthToken,
};
use tauri::AppHandle;

/// Prefixo usado para não colidir com as demais chaves de `encrypted_keys` (steam_api_key, rawg_api_key, etc.).
fn secret_key_for(provider_id: &str) -> String {
    format!("oauth_{provider_id}")
}

/// Salva (ou substitui) o token OAuth de um provedor.
pub fn save_oauth_token(
    app: &AppHandle,
    provider_id: &str,
    token: &OAuthToken,
) -> Result<(), AppError> {
    let json = serde_json::to_string(token)?;
    set_secret(app, &secret_key_for(provider_id), &json)
}

/// Carrega o token OAuth salvo de um provedor, se existir.
///
/// **Observação:**
/// - `get_secret` retorna string vazia quando a chave não existe (em vez de erro)
/// -  Sendo então tratado como "nenhum token salvo" -> `Ok(None)`.
pub fn load_oauth_token(
    app: &AppHandle,
    provider_id: &str,
) -> Result<Option<OAuthToken>, AppError> {
    let raw = get_secret(app, &secret_key_for(provider_id))?;

    if raw.is_empty() {
        return Ok(None);
    }

    let token: OAuthToken = serde_json::from_str(&raw)?;
    Ok(Some(token))
}

/// Remove o token OAuth salvo de um provedor (ex: logout).
pub fn delete_oauth_token(app: &AppHandle, provider_id: &str) -> Result<(), AppError> {
    delete_secret(app, &secret_key_for(provider_id))
}

/// Retorna um `access_token` válido para o provedor, renovando automaticamente via `refresh_token` se o token salvo estiver expirado.
///
/// **Erros:**
/// - `OAuthTokenNotFound`: nenhum token salvo (usuário nunca fez login, ou fez logout).
/// - `OAuthTokenNotFound`: token expirado e sem `refresh_token` disponível — precisa de novo login.
/// - `OAuthRefreshError`: a renovação falhou (refresh_token revogado/expirado no provedor).
pub async fn get_valid_access_token(
    app: &AppHandle,
    config: &OAuthProviderConfig,
) -> Result<String, AppError> {
    let stored = load_oauth_token(app, config.provider_id)?
        .ok_or_else(|| AppError::OAuthTokenNotFound(config.provider_id.to_string()))?;

    if !stored.is_expired() {
        return Ok(stored.access_token);
    }

    let refresh_token = stored.refresh_token.as_ref().ok_or_else(|| {
        AppError::OAuthTokenNotFound(format!(
            "{}: token expirado e sem refresh_token, é necessário novo login",
            config.provider_id
        ))
    })?;

    let refreshed = refresh_access_token(config, refresh_token).await?;
    let mut new_token: OAuthToken = refreshed.into();

    // Alguns provedores não reenviam o refresh_token na renovação (esperam que o cliente mantenha o mesmo até expirar de fato).
    // Se a resposta não trouxer um novo, preserva o que já estava salvo.
    if new_token.refresh_token.is_none() {
        new_token.refresh_token = stored.refresh_token.clone();
    }

    save_oauth_token(app, config.provider_id, &new_token)?;

    Ok(new_token.access_token)
}

/// Retorna um `access_token` válido para provedores Client Credentials (sem
/// refresh_token). Se não houver token salvo, ou se o salvo estiver expirado,
/// busca um novo do zero.
pub async fn get_valid_app_token(
    app: &AppHandle,
    config: &OAuthProviderConfig,
) -> Result<String, AppError> {
    if let Some(stored) = load_oauth_token(app, config.provider_id)? {
        if !stored.is_expired() {
            return Ok(stored.access_token);
        }
    }

    let fetched = fetch_app_access_token(config).await?;
    let new_token: OAuthToken = fetched.into();
    save_oauth_token(app, config.provider_id, &new_token)?;

    Ok(new_token.access_token)
}
