//! Scraper - Game Pass PC

use crate::constants::GAME_PASS_BATCH_SIZE;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// IDs dos catálogos
const GAME_PASS_PC_SIGL: &str = "fdd9e2a7-0fee-49f6-ad69-4354098401ff";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePassGame {
    pub store_id: String,
    pub title: String,
    pub description: Option<String>,
    pub image_poster: Option<String>, // capa 2:3 para cards
    pub image_hero: Option<String>,   // banner 16:9 para carrossel
    pub categories: Vec<String>,
    pub developer: Option<String>,
    pub is_ea_play: bool,
    pub store_url: String,
}

fn parse_game_pass_products(
    products: &serde_json::Value,
    exclude_ea_play: bool,
) -> Vec<GamePassGame> {
    let mut games = Vec::new();

    let Some(products) = products.as_object() else {
        return games;
    };

    for (store_id, product) in products {
        // Filtra não-jogos
        if product["ProductType"].as_str() != Some("Game") {
            continue;
        }

        // Filtra EA Play se solicitado
        let is_ea_play = product["IsEAPlay"].as_bool().unwrap_or(false);
        if exclude_ea_play && is_ea_play {
            continue;
        }

        // Confirma que roda no PC
        let platforms = product["AllowedPlatforms"]
            .as_array()
            .map(|p| p.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        if !platforms.contains(&"Windows.Desktop") {
            continue;
        }

        let categories = product["Categories"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        games.push(GamePassGame {
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
            is_ea_play,
            store_url: format!("https://www.xbox.com/pt-BR/games/store/_/{}", store_id),
        });
    }

    games
}

pub async fn fetch_game_pass_pc_catalog(
    exclude_ea_play: bool,
    language: &str,
) -> Result<Vec<GamePassGame>, String> {
    let client = Client::new();

    // Etapa 1 — lista de IDs
    let url = format!(
        "https://catalog.gamepass.com/sigls/v2?id={}&language={}&market=BR",
        GAME_PASS_PC_SIGL, language
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

        games.extend(parse_game_pass_products(
            &response["Products"],
            exclude_ea_play,
        ));

        // Delay entre batches para não sobrecarregar
        if !chunk.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    Ok(games)
}
