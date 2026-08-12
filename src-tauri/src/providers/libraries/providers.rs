//! Este módulo define a estrutura `SourceGame` e a trait `GameSource`.
//!
//! - Representa jogos detectados de várias fontes (como Steam, Epic Games, GOG, etc.)
//! - Define a interface que cada fonte deve implementar para buscar os jogos instalados.

use crate::errors::AppError;
use crate::utils::oauth::config::{
    build_authorize_url, exchange_code_for_token, OAuthProviderConfig,
};
use crate::utils::oauth::core::{generate_state, wait_for_auth_code, PkceChallenge};
use crate::utils::oauth::token_store::{
    delete_oauth_token, get_valid_access_token, load_oauth_token, save_oauth_token,
};
use async_trait::async_trait;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Clone)]
pub struct SourceGame {
    pub platform: String,
    pub platform_game_id: String,
    pub name: Option<String>,
    pub installed: bool,
    pub executable_path: Option<String>,
    pub install_path: Option<String>,
    pub playtime_minutes: Option<u32>,
    pub last_played: Option<i64>, // Unix timestamp
    pub source_label: Option<String>,
}

#[async_trait]
pub trait GameSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError>;
}

/// Extensão de `GameSource` para fontes que dependem de login OAuth2 (biblioteca remota da conta do usuário).
#[async_trait]
pub trait OAuthGameSource: GameSource {
    /// Configuração do provedor (endpoints, client_id, escopos, etc.).
    fn oauth_config(&self) -> &OAuthProviderConfig;

    /// Handle da aplicação, necessário para acessar o secrets.db.
    fn app_handle(&self) -> &AppHandle;

    /// Verifica se existe um token salvo para este provedor.
    fn is_authenticated(&self) -> Result<bool, AppError> {
        let token = load_oauth_token(self.app_handle(), self.oauth_config().provider_id)?;
        Ok(token.is_some())
    }

    /// Retorna um access_token válido, renovando via refresh_token se necessário.
    async fn ensure_valid_token(&self) -> Result<String, AppError> {
        get_valid_access_token(self.app_handle(), self.oauth_config()).await
    }

    /// Executa o fluxo completo de login: abre o navegador, aguarda o callback local, valida o `state` (CSRF) e troca o `code` por um token, que é salvo automaticamente.
    async fn login(&self) -> Result<(), AppError> {
        let config = self.oauth_config();

        let pkce = config.uses_pkce.then(PkceChallenge::generate);
        let expected_state = generate_state();

        let auth_url = build_authorize_url(config, pkce.as_ref(), &expected_state)?;

        self.app_handle()
            .opener()
            .open_url(auth_url.as_str(), None::<String>)
            .map_err(|e| AppError::OAuthConfigError(format!("Falha ao abrir navegador: {e}")))?;

        let port = extract_port(&config.redirect_uri)?;

        // wait_for_auth_code é bloqueante (usa thread + recv_timeout), então roda numa thread separada do runtime async.
        let callback = tokio::task::spawn_blocking(move || wait_for_auth_code(port))
            .await
            .map_err(|e| AppError::OAuthConfigError(format!("Task de callback falhou: {e}")))?
            .map_err(AppError::OAuthConfigError)?;

        match &callback.state {
            Some(state) if state == &expected_state => {}
            _ => {
                return Err(AppError::OAuthConfigError(
                    "Parâmetro 'state' ausente ou não confere (possível CSRF)".into(),
                ))
            }
        }

        let verifier = pkce.as_ref().map(|p| p.verifier.as_str());
        let token_response = exchange_code_for_token(config, &callback.code, verifier).await?;

        save_oauth_token(
            self.app_handle(),
            config.provider_id,
            &token_response.into(),
        )?;

        Ok(())
    }

    /// Remove o token salvo (logout).
    fn logout(&self) -> Result<(), AppError> {
        delete_oauth_token(self.app_handle(), self.oauth_config().provider_id)
    }
}

/// Extrai a porta de um redirect_uri local (ex: "http://127.0.0.1:8901/callback" -> 8901).
fn extract_port(redirect_uri: &str) -> Result<u16, AppError> {
    url::Url::parse(redirect_uri)
        .ok()
        .and_then(|u| u.port())
        .ok_or_else(|| {
            AppError::OAuthConfigError(format!("redirect_uri sem porta válida: {redirect_uri}"))
        })
}
