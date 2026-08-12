//! Serviço para interagir com a API da IsThereAnyDeal (ITAD)
//!
//! Fornece funções para buscar IDs de jogos e obter informações de preços.
//! A ITAD é uma plataforma que agrega ofertas de jogos de várias lojas digitais.

use crate::constants::ITAD_API_URL;
use crate::security;
use crate::utils::http_client::HTTP_CLIENT;
use serde::{Deserialize, Serialize};

// === ESTRUTURAS PÚBLICAS ===

#[derive(Debug, Serialize, Deserialize)]
pub struct ItadLookupResult {
    pub found: bool,
    pub id: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItadShop {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItadPrice {
    pub price: f64,
    pub currency: String,
    pub cut: Option<i32>,
    pub shop: ItadShop,
    pub url: Option<String>,
    pub voucher: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItadGameOverview {
    pub id: String,
    pub title: Option<String>,
    pub current: Option<ItadPrice>,
    pub lowest: Option<ItadPrice>,
}

// === ESTRUTURAS INTERNAS (Espelho exato do JSON da API) ===

#[derive(Debug, Deserialize)]
struct RawItadResponse {
    prices: Vec<RawItadGame>,
}

#[derive(Debug, Deserialize)]
struct RawItadGame {
    id: String,
    current: Option<RawItadDeal>,
    lowest: Option<RawItadDeal>,
}

#[derive(Debug, Deserialize)]
struct RawItadDeal {
    shop: ItadShop,
    price: RawPriceValue,
    cut: Option<i32>,
    url: Option<String>,
    voucher: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawPriceValue {
    amount: f64,
    currency: String,
}

// === IMPLEMENTAÇÃO ===

/// Busca o ID do jogo na ITAD pelo título
pub async fn find_game_id(title: &str) -> Result<String, String> {
    let key = security::get_itad_api_key();
    if key.is_empty() {
        return Err("API Key da ITAD não configurada".into());
    }

    let url = format!(
        "{}/games/lookup/v1?key={}&title={}",
        ITAD_API_URL,
        key,
        urlencoding::encode(title)
    );

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro ITAD Lookup: {}", res.status()));
    }

    let response_text = res.text().await.map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct LookupResponse {
        game: Option<GameInfo>,
    }
    #[derive(Deserialize)]
    struct GameInfo {
        id: String,
    }

    let response: LookupResponse = serde_json::from_str(&response_text).map_err(|e| {
        tracing::error!("Erro JSON Lookup: {}", e);
        format!("Erro de JSON: {}", e)
    })?;

    match response.game {
        Some(game_info) => Ok(game_info.id),
        None => Err("Jogo não encontrado na ITAD".into()),
    }
}

/// Busca informações de preço para uma lista de IDs da ITAD
pub async fn get_prices(itad_ids: Vec<String>) -> Result<Vec<ItadGameOverview>, String> {
    let key = security::get_itad_api_key();
    if key.is_empty() {
        return Err("API Key ausente".into());
    }
    if itad_ids.is_empty() {
        return Ok(vec![]);
    }

    let url = format!("{}/games/overview/v2?key={}&country=BR", ITAD_API_URL, key);

    tracing::debug!("ITAD Prices Request: {} items", itad_ids.len());

    let res = HTTP_CLIENT
        .post(&url)
        .json(&itad_ids)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();

    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        tracing::error!("Erro ITAD Prices: {}", body);
        return Err(format!("Erro ITAD Prices: {}", status));
    }

    let response_text = res.text().await.map_err(|e| e.to_string())?;

    // 1. Deserializa para a estrutura bruta (Raw) que corresponde ao JSON
    let raw_response: RawItadResponse = serde_json::from_str(&response_text).map_err(|e| {
        tracing::error!("Erro ao parsear JSON da ITAD: {}", e);
        // Dica de debug: mostra onde quebrou
        format!(
            "Erro de JSON (linha: {}, col: {}): {}",
            e.line(),
            e.column(),
            e
        )
    })?;

    // 2. Converte para a estrutura limpa (Public)
    let clean_overview: Vec<ItadGameOverview> = raw_response
        .prices
        .into_iter()
        .map(|raw| {
            ItadGameOverview {
                id: raw.id,
                title: None, // A API de preços não retorna título, o wishlist.rs já tem o nome
                current: raw.current.map(convert_deal),
                lowest: raw.lowest.map(convert_deal),
            }
        })
        .collect();

    Ok(clean_overview)
}

// Helper para converter o deal bruto para o limpo (achatando o preço)
fn convert_deal(raw: RawItadDeal) -> ItadPrice {
    ItadPrice {
        price: raw.price.amount,
        currency: raw.price.currency,
        cut: raw.cut,
        shop: raw.shop,
        url: raw.url,
        voucher: raw.voucher,
    }
}
