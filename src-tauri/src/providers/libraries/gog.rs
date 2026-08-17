//! Source para importar a biblioteca de jogos possuídos na conta GOG.

use crate::constants::{
    GOG_AUTH_ENDPOINT, GOG_CLIENT_ID, GOG_CLIENT_SECRET, GOG_FILTERED_PRODUCTS_ENDPOINT,
    GOG_REDIRECT_URI, GOG_TOKEN_ENDPOINT,
};
use crate::errors::AppError;
use crate::providers::libraries::providers::{GameSource, OAuthGameSource, SourceGame};
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::config::{
    build_authorize_url, exchange_code_for_token, OAuthProviderConfig, TokenAuthMethod,
    TokenRequestMethod,
};
use crate::utils::oauth::core::{generate_state, AuthCallbackResult};
use crate::utils::oauth::token_store::save_oauth_token;
use crate::utils::text::is_likely_non_base_game;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

// === STRUCTS ===

pub struct GogSource {
    app_handle: AppHandle,
    config: OAuthProviderConfig,
}

#[derive(Debug, Deserialize)]
struct GogProduct {
    id: i64,
    title: String,
}

#[derive(Debug, Deserialize)]
struct GogFilteredProductsResponse {
    #[serde(rename = "totalPages")]
    total_pages: u32,
    products: Vec<GogProduct>,
}

#[derive(Deserialize)]
struct GogPlayTask {
    #[serde(rename = "type")]
    task_type: String,
    category: Option<String>,
    path: Option<String>,
    #[serde(rename = "isPrimary", default)]
    is_primary: bool,
}

#[derive(Deserialize)]
struct GogGameInfo {
    #[serde(rename = "playTasks")]
    play_tasks: Vec<GogPlayTask>,
}

// === IMPLEMENTAÇÕES ===

impl GogSource {
    pub fn new(app_handle: AppHandle) -> Self {
        let config = OAuthProviderConfig {
            provider_id: "gog",
            client_id: GOG_CLIENT_ID.to_string(),
            client_secret: Some(GOG_CLIENT_SECRET.to_string()),
            authorize_endpoint: GOG_AUTH_ENDPOINT.to_string(),
            token_endpoint: GOG_TOKEN_ENDPOINT.to_string(),
            redirect_uri: GOG_REDIRECT_URI.to_string(),
            scopes: vec![],
            uses_pkce: false,
            extra_params: vec![("layout".into(), "galaxy".into())],
            token_request_method: TokenRequestMethod::Get,
            token_auth_method: TokenAuthMethod::Body,
        };

        Self { app_handle, config }
    }

    pub async fn fetch_games_detailed(&self) -> Result<Vec<SourceGame>, AppError> {
        let access_token = self.ensure_valid_token().await?;
        let products = fetch_all_owned_products(&access_token).await?;

        let games = products
            .into_iter()
            .filter(|p| !is_likely_non_base_game(&p.title))
            .map(|p| SourceGame {
                library: "GOG".to_string(),
                library_game_id: p.id.to_string(),
                name: Some(p.title),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
                source_label: None,
            })
            .collect();

        Ok(games)
    }
}

#[async_trait]
impl GameSource for GogSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        self.fetch_games_detailed().await
    }
}

#[async_trait]
impl OAuthGameSource for GogSource {
    fn oauth_config(&self) -> &OAuthProviderConfig {
        &self.config
    }

    fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }

    /// GOG não aceita redirect_uri para localhost. O redirect registrado (`embed.gog.com/on_login_success?origin=client`) é interceptado numa
    /// `WebviewWindow` própria antes de carregar, em vez de usar o servidor local (`wait_for_auth_code`) do fluxo default do trait.
    async fn login(&self) -> Result<(), AppError> {
        let config = self.oauth_config();
        let expected_state = generate_state();
        let auth_url = build_authorize_url(config, None, &expected_state)?;

        let (tx, rx) = mpsc::channel::<Result<AuthCallbackResult, String>>();
        let redirect_uri = config.redirect_uri.clone();

        let window = WebviewWindowBuilder::new(
            &self.app_handle,
            "gog_oauth_login",
            WebviewUrl::External(auth_url),
        )
            .title("Login GOG")
            .inner_size(480.0, 720.0)
            .on_navigation(move |url| {
                let url_str = url.as_str();
                if url_str.starts_with(&redirect_uri) {
                    let code = url
                        .query_pairs()
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.to_string());
                    let state = url
                        .query_pairs()
                        .find(|(k, _)| k == "state")
                        .map(|(_, v)| v.to_string());

                    let result = match code {
                        Some(code) => Ok(AuthCallbackResult { code, state }),
                        None => Err("Redirect alcançado, mas sem 'code' na URL".to_string()),
                    };
                    let _ = tx.send(result);
                    return false;
                }
                true
            })
            .build()
            .map_err(|e| {
                AppError::OAuthConfigError(format!("Falha ao abrir janela de login GOG: {e}"))
            })?;

        let callback = tokio::task::spawn_blocking(move || {
            rx.recv_timeout(Duration::from_secs(
                crate::constants::OAUTH_CALLBACK_TIMEOUT_SECS,
            ))
        })
            .await
            .map_err(|e| AppError::OAuthConfigError(format!("Task de callback falhou: {e}")))?
            .map_err(|_| AppError::OAuthConfigError("Tempo limite de login excedido".to_string()))?
            .map_err(AppError::OAuthConfigError)?;

        let _ = window.close();

        match &callback.state {
            Some(state) if state == &expected_state => {}
            _ => {
                return Err(AppError::OAuthConfigError(
                    "Parâmetro 'state' ausente ou não confere (possível CSRF)".into(),
                ))
            }
        }

        let token_response = exchange_code_for_token(config, &callback.code, None).await?;
        save_oauth_token(&self.app_handle, config.provider_id, &token_response.into())?;

        Ok(())
    }
}

// === FUNÇÕES AUXILIARES ===

/// Busca todos os jogos possuídos, já com nome, percorrendo a paginação de `/account/getFilteredProducts`.
async fn fetch_all_owned_products(access_token: &str) -> Result<Vec<GogProduct>, AppError> {
    let mut all_products = Vec::new();
    let mut page = 1u32;

    loop {
        let response = HTTP_CLIENT
            .get(GOG_FILTERED_PRODUCTS_ENDPOINT)
            .bearer_auth(access_token)
            .query(&[("mediaType", "1"), ("page", &page.to_string())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::NetworkError(format!(
                "GOG getFilteredProducts retornou HTTP {status}: {body}"
            )));
        }

        let parsed: GogFilteredProductsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ParseError(format!("Falha ao parsear produtos GOG: {e}")))?;

        let total_pages = parsed.total_pages;
        all_products.extend(parsed.products);

        log::debug!("GOG getFilteredProducts: página {page}/{total_pages}");

        if page >= total_pages {
            break;
        }
        page += 1;
    }

    Ok(all_products)
}

/// Marca como instalados os jogos cuja pasta existe dentro do diretório `Games` configurado do GOG Galaxy.
///
/// **Heurística:** o GOG Galaxy cria uma subpasta por jogo com o nome do produto exatamente como aparece na loja.
/// **Exemplo:** "Tomb Raider I-III Remastered"
///
/// **Observação:** Alguns jogos na loja têm sufixos extras no nome (ex: "Leap of Faith + Official Walkthrough"),
/// enquanto a pasta instalada usa apenas o nome base do jogo. Por isso o match verifica se o nome da
/// loja *começa com* o nome da pasta, e não por igualdade exata.
pub fn detect_installed_games(games: &mut [SourceGame], gog_games_dir: &Path) {
    let Ok(entries) = fs::read_dir(gog_games_dir) else {
        log::warn!("Não foi possível ler diretório de jogos GOG: {gog_games_dir:?}");
        return;
    };

    let installed_folders: Vec<(String, std::path::PathBuf)> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().trim().to_lowercase();
            if name.is_empty() {
                return None;
            }
            Some((name, e.path()))
        })
        .collect();

    for game in games.iter_mut() {
        let Some(name) = &game.name else {
            continue;
        };
        let normalized = name.trim().to_lowercase();

        // Verifica se o nome da loja começa com o nome de alguma pasta instalada, garantindo
        // que o caractere seguinte ao prefixo (se houver) não seja alfanumérico.
        let matched = installed_folders.iter().find(|(folder, _)| {
            if !normalized.starts_with(folder.as_str()) {
                return false;
            }
            match normalized.as_bytes().get(folder.len()) {
                None => true,
                Some(&b) => !(b as char).is_alphanumeric(),
            }
        });

        if let Some((_, path)) = matched {
            game.installed = true;
            game.install_path = Some(path.to_string_lossy().to_string());
            game.executable_path = resolve_gog_executable(path, &game.library_game_id)
                .map(|p| p.to_string_lossy().to_string());
        }
    }
}

fn resolve_gog_executable(install_path: &Path, product_id: &str) -> Option<PathBuf> {
    let info_path = install_path.join(format!("goggame-{product_id}.info"));
    let content = fs::read_to_string(&info_path).ok()?;
    let info: GogGameInfo = serde_json::from_str(&content).ok()?;

    let is_game_file_task =
        |t: &GogPlayTask| t.task_type == "FileTask" && t.category.as_deref() == Some("game");

    // Preferência: task marcado como primário. Fallback: primeiro FileTask de categoria "game",
    // já que alguns goggame-*.info não marcam isPrimary em nenhum task.
    info.play_tasks
        .iter()
        .find(|t| is_game_file_task(t) && t.is_primary)
        .or_else(|| info.play_tasks.iter().find(|t| is_game_file_task(t)))
        .and_then(|t| t.path.as_ref())
        .map(|rel| install_path.join(rel))
}
