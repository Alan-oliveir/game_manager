//! Scraper para o WineHQ AppDB.
//!
//! Utiliza `reqwest` e `scraper` para buscar aplicações no WineHQ AppDB.
//! A busca é feita através da página de aplicações do WineHQ e o resultado
//! contém o nome e a URL da aplicação encontrada.

use crate::errors::AppError;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// User-Agent utilizado nas requisições ao WineHQ.
const WINEHQ_USER_AGENT: &str = "Playlite/1.0 (playlite.app.dev@gmail.com)";

/// Delay mínimo entre requisições ao WineHQ.
const REQUEST_WINEHQ_DELAY_MS: u64 = 250;

/// Número máximo de tentativas para erros de rede.
const MAX_WINEHQ_RETRIES: u8 = 3;

/// Delay entre tentativas de uma requisição que falhou.
const RETRY_DELAY_SECONDS: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WineHqSummary {
    pub name: String,
    pub url: String,
    pub highest_rating: Option<String>,
}

/// Constrói o cliente HTTP utilizado pelo WineHQ.
pub(crate) fn build_http_client() -> Result<Client, AppError> {
    Client::builder()
        .user_agent(WINEHQ_USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| AppError::NetworkError(error.to_string()))
}

/// Busca um jogo no WineHQ AppDB por nome.
///
/// Retorna:
/// - `Ok(Some(...))` quando encontra uma aplicação;
/// - `Ok(None)` quando a busca funciona, mas nenhum jogo corresponde;
/// - `Err(...)` quando ocorre uma falha de rede ou parsing.
pub async fn fetch_winehq_data(
    game_name: &str,
) -> Result<Option<WineHqSummary>, AppError> {
    let client = build_http_client()?;

    let safe_name = game_name.trim();

    if safe_name.is_empty() {
        tracing::debug!("WineHQ: Nome do jogo vazio, ignorando busca");
        return Ok(None);
    }

    tracing::info!("WineHQ: Iniciando busca por '{}'", safe_name);

    let url = "https://appdb.winehq.org/objectManager.php";

    for attempt in 1..=MAX_WINEHQ_RETRIES {
        // Rate limit entre requisições.
        sleep(Duration::from_millis(REQUEST_WINEHQ_DELAY_MS)).await;

        tracing::debug!(
            "WineHQ: Tentativa {}/{} para '{}'",
            attempt,
            MAX_WINEHQ_RETRIES,
            safe_name
        );

        let response = match client
            .get(url)
            .query(&[
                ("sClass", "application"),
                ("sTitle", "Browse Applications"),
                ("sappNameOp", "sub"),
                ("sappNameData", safe_name),
            ])
            .send()
            .await
        {
            Ok(response) => response,

            Err(error) => {
                tracing::warn!(
                    "WineHQ: Erro de rede na tentativa {}/{} para '{}': {}",
                    attempt,
                    MAX_WINEHQ_RETRIES,
                    safe_name,
                    error
                );

                if attempt < MAX_WINEHQ_RETRIES {
                    sleep(Duration::from_secs(RETRY_DELAY_SECONDS)).await;
                    continue;
                }

                return Err(AppError::NetworkError(format!(
                    "WineHQ indisponível após {} tentativas: {}",
                    MAX_WINEHQ_RETRIES, error
                )));
            }
        };

        let status = response.status();

        tracing::debug!(
            "WineHQ: HTTP {} recebido para '{}'",
            status,
            safe_name
        );

        if !status.is_success() {
            return Err(AppError::NetworkError(format!(
                "WineHQ retornou HTTP {}",
                status
            )));
        }

        let html_content = response
            .text()
            .await
            .map_err(|error| AppError::ParseError(error.to_string()))?;

        return parse_winehq_search(&html_content, safe_name);
    }

    unreachable!("O loop de tentativas do WineHQ deveria sempre retornar");
}

/// Analisa o HTML retornado pela busca do WineHQ.
///
/// Esta função fica separada da camada HTTP para facilitar alterações
/// futuras caso a estrutura HTML do WineHQ seja modificada.
fn parse_winehq_search(
    html_content: &str,
    query: &str,
) -> Result<Option<WineHqSummary>, AppError> {
    let document = Html::parse_document(html_content);

    let link_selector = Selector::parse(
        "a[href*=\"iId=\"][href*=\"application\"]",
    )
        .map_err(|_| {
            AppError::ParseError("Seletor de link inválido".to_string())
        })?;

    let query_lower = query.to_lowercase();
    let mut found_links = 0;

    for a_tag in document.select(&link_selector) {
        found_links += 1;

        let name_clean = a_tag
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        if name_clean.is_empty() {
            continue;
        }

        let name_lower = name_clean.to_lowercase();

        tracing::debug!("WineHQ viu o link: '{}'", name_clean);

        if name_lower.contains(&query_lower)
            || query_lower.contains(&name_lower)
        {
            let href = a_tag
                .value()
                .attr("href")
                .unwrap_or_default();

            if href.is_empty() {
                tracing::debug!(
                    "WineHQ: Link encontrado para '{}', mas href está vazio",
                    name_clean
                );

                continue;
            }

            let final_url = if href.starts_with("http://")
                || href.starts_with("https://")
            {
                href.to_string()
            } else {
                format!(
                    "https://appdb.winehq.org/{}",
                    href.trim_start_matches('/')
                        .replace("&amp;", "&")
                )
            };

            tracing::info!(
                "WineHQ: Match encontrado! '{}' -> {}",
                name_clean,
                final_url
            );

            return Ok(Some(WineHqSummary {
                name: name_clean,
                url: final_url,
                highest_rating: None,
            }));
        }
    }

    tracing::debug!(
        "WineHQ: Nenhum jogo bateu com '{}' ({} links avaliados)",
        query,
        found_links
    );

    Ok(None)
}