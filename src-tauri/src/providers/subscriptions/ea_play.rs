//! Scraper - EA Play

use crate::constants::GAME_PASS_BATCH_SIZE;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// IDs dos catálogos
const EA_PLAY_SIGL: &str = "b8900d09-a491-44cc-916e-32b5acae621b";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EAPlayGame {
    pub store_id: String,
    pub title: String,
    pub description: Option<String>,
    pub image_poster: Option<String>, // capa 2:3 para cards
    pub image_hero: Option<String>,   // banner 16:9 para carrossel
    pub categories: Vec<String>,
    pub developer: Option<String>,
    pub store_url: String,
}

fn parse_ea_play_products(products: &serde_json::Value) -> Vec<EAPlayGame> {
    let mut games = Vec::new();

    let Some(products) = products.as_object() else {
        return games;
    };

    for (store_id, product) in products {
        // Filtra não-jogos
        let product_type = product["ProductType"].as_str().unwrap_or("");
        if product_type != "Game" {
            continue;
        }

        // Categories
        let categories = product["Categories"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Build game
        let game = EAPlayGame {
            store_id: store_id.clone(),
            title: product["ProductTitle"].as_str().unwrap_or("?").to_string(),
            description: product["ProductDescription"]
                .as_str()
                .map(|s| s.to_string()),
            image_poster: product["ImagePoster"]["URI"]
                .as_str()
                .map(|s| s.to_string()),
            image_hero: product["ImageHero"]["URI"].as_str().map(|s| s.to_string()),
            categories,
            developer: product["DeveloperName"].as_str().map(|s| s.to_string()),
            store_url: format!("https://www.xbox.com/pt-BR/games/store/_/{}", store_id),
        };

        games.push(game);
    }

    games
}

pub async fn fetch_ea_play_catalog(language: &str) -> Result<Vec<EAPlayGame>, String> {
    let client = Client::new();

    // Etapa 1 — lista de IDs
    let url = format!(
        "https://catalog.gamepass.com/sigls/v2?id={}&language={}&market=BR",
        EA_PLAY_SIGL, language
    );

    let sigls: Vec<serde_json::Value> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // Extrai apenas os IDs (pula o primeiro objeto que é metadata do catálogo)
    let ids: Vec<String> = sigls
        .iter()
        .filter_map(|v| v["id"].as_str().map(|s| s.to_string()))
        .collect();

    // Etapa 2 — detalhes em batches de 20
    let mut games = Vec::new();

    for chunk in ids.chunks(GAME_PASS_BATCH_SIZE) {
        let body = serde_json::json!({ "Products": chunk });

        let response: serde_json::Value = client
            .post(format!(
                "https://catalog.gamepass.com/products?market=BR&language={}&hydration=MobileDetailsForConsole",
                language
            ))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        games.extend(parse_ea_play_products(&response["Products"]));

        // Delay entre batches para não sobrecarregar
        if !chunk.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    Ok(games)
}
