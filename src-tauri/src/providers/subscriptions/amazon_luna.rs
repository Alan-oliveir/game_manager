//! Scraper - Amazon Luna
//!
//! - URL: https://luna.amazon.com/claims/home
//! - Descrição: Scraper para obter a lista de jogos gratuitos disponíveis na Amazon Luna,
//!   incluindo título, descrição, imagem, link de resgate e data de término da oferta.
//! - Método: Acesso à página de reivindicações da Amazon Luna para obter cookies de sessão e token CSRF,
//!   seguido por uma requisição POST GraphQL para a API interna da Amazon Luna, utilizando os cookies
//!   e token para autenticação.

use reqwest::{cookie::Jar, Client};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunaGame {
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub claim_url: String,
    pub end_time: Option<String>,
}
pub async fn fetch_amazon_luna_catalog(language: &str) -> Result<Vec<LunaGame>, String> {
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(jar)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let html = client
        .get("https://luna.amazon.com/claims/home")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let csrf_token = {
        let document = Html::parse_document(&html);
        let selector = Selector::parse("input[name='csrf-key']").map_err(|e| e.to_string())?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("value"))
            .ok_or("csrf-key não encontrado")?
            .to_string() // .to_string() move o valor para fora do escopo
    };

    // Payload GraphQL mínimo para solicitar os campos usados no parsing abaixo.
    let payload = serde_json::json!({
        "extensions": {},
        "operationName": "OffersContext_Offers_And_Items",
        "query": "query OffersContext_Offers_And_Items($pageSize: Int) {\n  games: items(collectionType: FREE_GAMES, pageSize: $pageSize) {\n    items {\n      id\n      assets {\n        title\n        externalClaimLink\n        shortformDescription\n        cardMedia { defaultMedia { src1x src2x __typename } __typename }\n        __typename\n      }\n      offers { id startTime endTime __typename }\n      __typename\n    }\n    __typename\n  }\n}",
        "variables": { "pageSize": 999 }
    });

    let response: serde_json::Value = client
        .post("https://luna.amazon.com/graphql")
        .header("Content-Type", "application/json")
        .header("Client-Id", "CarboniteApp")
        .header("Origin", "https://luna.amazon.com")
        .header("Referer", "https://luna.amazon.com/claims/home")
        .header("prime-gaming-language", language)
        .header("csrf-token", csrf_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // Parseia os jogos
    let items = response["data"]["games"]["items"]
        .as_array()
        .ok_or("Campo games não encontrado")?;

    let games = items
        .iter()
        .filter_map(|item| {
            let assets = item["assets"].as_object()?;
            let offer = item["offers"].as_array()?.first()?;

            let card_media = assets.get("cardMedia")?.as_object()?;
            let default_media = card_media.get("defaultMedia")?.as_object()?;

            Some(LunaGame {
                title: assets.get("title")?.as_str()?.to_string(),
                description: assets
                    .get("shortformDescription")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string()),
                image_url: default_media
                    .get("src2x")
                    .and_then(|value| value.as_str())
                    .or_else(|| default_media.get("src1x").and_then(|value| value.as_str()))
                    .map(|value| value.to_string()),
                claim_url: assets.get("externalClaimLink")?.as_str()?.to_string(),
                end_time: offer["endTime"].as_str().map(|s| s.to_string()),
            })
        })
        .collect();

    Ok(games)
}
