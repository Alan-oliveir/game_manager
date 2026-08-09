//! Módulo de integração com a API IGDB (Twitch).
//!
//! Autenticação via Client Credentials Grant reaproveitando `utils/oauth`.
//! Client-ID do Twitch precisa ir em TODAS as chamadas, não só na troca do token.

use crate::constants::{IGDB_API_BASE, IGDB_TOKEN_ENDPOINT};
use crate::database;
use crate::errors::AppError;
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::config::{OAuthProviderConfig, TokenAuthMethod, TokenRequestMethod};
use crate::utils::oauth::token_store::get_valid_app_token;
use tauri::AppHandle;

/// Monta a config OAuth do IGDB a partir das credenciais salvas.
/// `authorize_endpoint`, `redirect_uri`, `scopes` ficam vazios de propósito:
/// client_credentials não usa nenhum deles.
fn build_igdb_config(client_id: String, client_secret: String) -> OAuthProviderConfig {
    OAuthProviderConfig {
        provider_id: "igdb",
        client_id,
        client_secret: Some(client_secret),
        authorize_endpoint: String::new(),
        token_endpoint: IGDB_TOKEN_ENDPOINT.to_string(),
        redirect_uri: String::new(),
        scopes: vec![],
        uses_pkce: false,
        extra_params: vec![],
        token_request_method: TokenRequestMethod::Post,
        token_auth_method: TokenAuthMethod::Body,
    }
}

fn load_igdb_config(app: &AppHandle) -> Result<OAuthProviderConfig, AppError> {
    let client_id = database::get_secret(app, "igdb_client_id")?;
    let client_secret = database::get_secret(app, "igdb_client_secret")?;

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::ValidationError(
            "IGDB: Client ID/Secret não configurados.".into(),
        ));
    }

    Ok(build_igdb_config(client_id, client_secret))
}

/// Executa uma requisição Apicalypse contra um endpoint do IGDB.
/// `query` é o corpo cru, ex: `search "Half-Life"; fields name; limit 1;`
pub async fn igdb_request(app: &AppHandle, endpoint: &str, query: &str) -> Result<String, String> {
    let config = load_igdb_config(app).map_err(|e| e.to_string())?;
    let access_token = get_valid_app_token(app, &config)
        .await
        .map_err(|e| e.to_string())?;

    crate::services::rate_limiter::IGDB_LIMITER
        .run(|| async {
            let url = format!("{IGDB_API_BASE}/{endpoint}");
            let res = HTTP_CLIENT
                .post(&url)
                .header("Client-ID", &config.client_id)
                .header("Authorization", format!("Bearer {access_token}"))
                .body(query.to_string())
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let status = res.status();
            let body = res.text().await.map_err(|e| e.to_string())?;

            if !status.is_success() {
                return Err(format!("Erro IGDB ({endpoint}): HTTP {status} - {body}"));
            }

            Ok(body)
        })
        .await
}

/// Teste isolado de autenticação — Confirma que token + Client-ID + query estão sendo aceitos.
pub async fn test_connection(app: &AppHandle) -> Result<String, String> {
    igdb_request(app, "games", r#"search "Half-Life"; fields name; limit 1;"#).await
}
