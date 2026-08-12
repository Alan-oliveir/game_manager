//! Cliente HTTP para a MediaWiki Action API do PCGamingWiki.
//!
//! Encapsula a construção do cliente `reqwest`, o helper de `cargoquery` com
//! rate limiting automático, e a busca de páginas por nome de jogo.

use crate::constants::{PCGW_API_BASE, REQUEST_PCGW_DELAY_MS};
use crate::errors::AppError;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// User-Agent obrigatorio pela politica da PCGW. Formato recomendado: "NomeDoApp/versao (contato)"
const PCGW_USER_AGENT: &str = "Playlite/1.0 (playlite.app.dev@gmail.com)";

// === ESTRUTURA PUBLICA ===

/// Resultado de busca por nome no PCGamingWiki.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PcgwSearchResult {
    #[serde(rename = "pageId")]
    pub page_id: String,
    #[serde(rename = "pageName")]
    pub page_name: String,
}

// === Helpers de HTTP ===

/// Constroi um cliente `reqwest` com User-Agent e timeout configurados.
pub(crate) fn build_http_client() -> Result<Client, AppError> {
    Client::builder()
        .user_agent(PCGW_USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::NetworkError(e.to_string()))
}

/// Faz uma cargoquery na PCGW API e retorna o array `cargoquery` do JSON.
///
/// Adiciona o delay de rate limiting antes de cada requisicao.
pub(crate) async fn cargo_query(
    client: &Client,
    tables: &str,
    fields: &str,
    where_clause: &str,
    limit: u32,
) -> Result<Vec<Value>, AppError> {
    sleep(Duration::from_millis(REQUEST_PCGW_DELAY_MS)).await;

    let response = client
        .get(PCGW_API_BASE)
        .query(&[
            ("action", "cargoquery"),
            ("tables", tables),
            ("fields", fields),
            ("where", where_clause),
            ("limit", &limit.to_string()),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(AppError::NetworkError(format!(
            "PCGW API retornou HTTP {}",
            status
        )));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    // Extrai o array de resultados; retorna vazio se ausente
    let rows = json
        .get("cargoquery")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(rows)
}

/// Busca a página via `action=query&list=search` por nome de jogo.
///
/// Usado como fallback quando o jogo não tem Steam AppID ou a busca por
/// AppID não retornar resultados. Retorna uma lista de candidatos para
/// que o usuário confirme qual é o correto.
pub async fn search_pcgw_by_name(game_name: &str) -> Result<Vec<PcgwSearchResult>, AppError> {
    let client = build_http_client()?;

    sleep(Duration::from_millis(REQUEST_PCGW_DELAY_MS)).await;

    let response = client
        .get(PCGW_API_BASE)
        .query(&[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", game_name),
            ("srnamespace", "0"),
            ("srlimit", "5"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json: Value = response
        .json()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    let results = json
        .pointer("/query/search")
        .and_then(|v| v.as_array())
        .map(|hits| {
            hits.iter()
                .filter_map(|hit| {
                    Some(PcgwSearchResult {
                        page_id: hit.get("pageid")?.as_u64()?.to_string(),
                        page_name: hit.get("title")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}
