//! Configuração genérica de provedores OAuth2 e funções de troca/renovação de token.
//!
//! Cada plataforma (GOG, Battle.net, EA, Amazon Games) fornece um `OAuthProviderConfig`
//! e reaproveita estas funções em vez de reimplementar o fluxo.

use crate::errors::AppError;
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::core::PkceChallenge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

// === ENUMS ===

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenRequestMethod {
    Post,
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenAuthMethod {
    /// client_id/client_secret no corpo (form ou query) — GOG.
    Body,
    /// client_id:client_secret em Base64 no header `Authorization: Basic` — Epic.
    BasicHeader,
}

// === STRUCTS ===

/// Configuração de um provedor OAuth2.
#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub provider_id: &'static str,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub authorize_endpoint: String,
    pub token_endpoint: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub uses_pkce: bool,
    pub extra_params: Vec<(String, String)>,
    pub token_request_method: TokenRequestMethod,
    pub token_auth_method: TokenAuthMethod,
}

/// Resposta bruta do endpoint de token do provedor.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>, // segundos
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_scope")]
    pub scope: Option<String>,
}

/// Forma persistida do token (o que vai encriptado pro secrets.db).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // timestamp unix; None = expiração desconhecida
    pub scope: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, String>, // ex: "device_serial" para Amazon
}

impl From<TokenResponse> for OAuthToken {
    fn from(resp: TokenResponse) -> Self {
        let expires_at = resp.expires_in.map(|secs| now_unix() + secs as i64);
        Self {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at,
            scope: resp.scope,
            extra: HashMap::new(),
        }
    }
}

impl OAuthToken {
    /// Considera expirado 60s antes do prazo real, pra dar margem de segurança contra latência de rede durante uma chamada de API.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => now_unix() >= exp - 60,
            None => false,
        }
    }
}

// === HELPERS LOCAIS ===

pub(crate) fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Alguns provedores (GOG) retornam `scope` como string; outros (Epic) retornam
/// como array de strings, inclusive vazio (`[]`) quando não há escopos.
/// Normaliza os dois formatos: array vira string com join por espaço, array vazio vira `None`.
fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScopeField {
        Text(String),
        List(Vec<String>),
    }

    let opt = Option::<ScopeField>::deserialize(deserializer)?;
    Ok(match opt {
        Some(ScopeField::Text(s)) => Some(s),
        Some(ScopeField::List(list)) if list.is_empty() => None,
        Some(ScopeField::List(list)) => Some(list.join(" ")),
        None => None,
    })
}

async fn parse_token_response(
    response: reqwest::Response,
    provider_id: &str,
    stage: &str,
) -> Result<TokenResponse, AppError> {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(AppError::OAuthTokenExchangeError(format!(
            "[{provider_id}] falha em '{stage}': HTTP {status} - {body}"
        )));
    }

    serde_json::from_str::<TokenResponse>(&body).map_err(|e| {
        AppError::OAuthTokenExchangeError(format!(
            "[{provider_id}] resposta inesperada em '{stage}': {e}"
        ))
    })
}

/// Monta a requisição do endpoint de token, aplicando client_id/secret no corpo
/// ou via header `Authorization: Basic`, conforme `token_auth_method` do provedor.
fn build_token_request(
    config: &OAuthProviderConfig,
    mut params: Vec<(&'static str, String)>,
) -> reqwest::RequestBuilder {
    let mut request = match config.token_request_method {
        TokenRequestMethod::Get => HTTP_CLIENT.get(&config.token_endpoint),
        TokenRequestMethod::Post => HTTP_CLIENT.post(&config.token_endpoint),
    };

    match config.token_auth_method {
        TokenAuthMethod::Body => {
            params.push(("client_id", config.client_id.clone()));
            if let Some(secret) = &config.client_secret {
                params.push(("client_secret", secret.clone()));
            }
        }
        TokenAuthMethod::BasicHeader => {
            let secret = config.client_secret.clone().unwrap_or_default();
            request = request.basic_auth(&config.client_id, Some(secret));
        }
    }

    match config.token_request_method {
        TokenRequestMethod::Get => request.query(&params),
        TokenRequestMethod::Post => request.form(&params),
    }
}

// === FUNÇÕES ===

/// Monta a URL de autorização para abrir no navegador do sistema.
pub fn build_authorize_url(
    config: &OAuthProviderConfig,
    pkce: Option<&PkceChallenge>,
    state: &str,
) -> Result<Url, AppError> {
    let mut url = Url::parse(&config.authorize_endpoint)
        .map_err(|e| AppError::OAuthConfigError(format!("URL de autorização inválida: {e}")))?;

    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("state", state);

        if !config.scopes.is_empty() {
            query.append_pair("scope", &config.scopes.join(" "));
        }

        if config.uses_pkce {
            let pkce = pkce.ok_or_else(|| {
                AppError::OAuthConfigError(
                    "uses_pkce=true mas nenhum PkceChallenge foi fornecido".into(),
                )
            })?;
            query
                .append_pair("code_challenge", &pkce.challenge)
                .append_pair("code_challenge_method", "S256");
        }

        for (key, value) in &config.extra_params {
            query.append_pair(key, value);
        }
    }

    Ok(url)
}

/// Troca o `code` recebido no callback por um access_token.
pub async fn exchange_code_for_token(
    config: &OAuthProviderConfig,
    code: &str,
    pkce_verifier: Option<&str>,
) -> Result<TokenResponse, AppError> {
    let mut params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", config.redirect_uri.clone()),
    ];
    if let Some(verifier) = pkce_verifier {
        params.push(("code_verifier", verifier.to_string()));
    }

    let response = build_token_request(config, params).send().await?;
    parse_token_response(response, config.provider_id, "exchange").await
}

/// Renova o access_token usando o refresh_token salvo.
pub async fn refresh_access_token(
    config: &OAuthProviderConfig,
    refresh_token: &str,
) -> Result<TokenResponse, AppError> {
    let params = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
    ];

    let response = build_token_request(config, params).send().await?;
    parse_token_response(response, config.provider_id, "refresh").await
}

/// Obtém um App Access Token via Client Credentials Grant.
///
/// Usado por provedores sem interação do usuário (ex: IGDB/Twitch), onde
/// não existe `authorize_endpoint`, `redirect_uri` nem `refresh_token` —
/// a "renovação" é simplesmente pedir um token novo com as mesmas credenciais.
pub async fn fetch_app_access_token(
    config: &OAuthProviderConfig,
) -> Result<TokenResponse, AppError> {
    let params = vec![("grant_type", "client_credentials".to_string())];
    let response = build_token_request(config, params).send().await?;
    parse_token_response(response, config.provider_id, "client_credentials").await
}
