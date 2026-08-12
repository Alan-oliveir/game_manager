//! Integração com GamerPower API para buscar jogos grátis.
//!
//! Fornece funcionalidades para obter ofertas ativas de jogos gratuitos para PC.
//! A GamerPower é uma plataforma que agrega ofertas de jogos gratuitos de várias fontes.

use crate::constants::{GAMERPOWER_CACHE_SOURCE, GAMERPOWER_LIST_ACTIVE_CACHE_KEY};
use crate::database::AppState;
use crate::services::cache;
use crate::utils::http_client::HTTP_CLIENT;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Giveaway {
    pub id: u32,
    pub title: String,
    pub worth: String,     // ex: "$29.99" or "N/A"
    pub thumbnail: String, // Imagem pequena
    pub image: String,     // Imagem grande (Landscape) - Usaremos essa no card estilo Epic
    pub description: String,
    pub instructions: String,
    #[serde(rename = "open_giveaway_url")]
    pub open_giveaway_url: String,
    #[serde(rename = "published_date")]
    pub published_date: String,
    #[serde(rename = "type")]
    pub type_prop: String, // "Game", "DLC"
    pub platforms: String, // "PC, Steam, Epic Games Store"
    #[serde(rename = "end_date")]
    pub end_date: Option<String>,
    pub status: String, // "Active"
}

/// Busca ofertas ativas para PC
pub async fn fetch_giveaways(app: &AppHandle) -> Result<Vec<Giveaway>, String> {
    let cache_key = GAMERPOWER_LIST_ACTIVE_CACHE_KEY;
    let url = "https://www.gamerpower.com/api/giveaways?platform=pc&type=game&sort-by=popularity";

    // 1. Tenta reaproveitar cache válido antes de consultar a API
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(payload) = cache::get_cached_api_data(&conn, GAMERPOWER_CACHE_SOURCE, cache_key)
        {
            if let Ok(cached_giveaways) = serde_json::from_str::<Vec<Giveaway>>(&payload) {
                return Ok(cached_giveaways);
            }
        }
    }

    // 2. Tenta buscar ONLINE
    if let Ok(res) = HTTP_CLIENT.get(url).send().await {
        if res.status().is_success() {
            let giveaways: Vec<Giveaway> = res.json().await.map_err(|e| e.to_string())?;

            // Filtra apenas ativos
            let active_ones: Vec<Giveaway> = giveaways
                .into_iter()
                .filter(|g| g.status == "Active")
                .collect();

            // Sucesso: Salva a lista filtrada no Cache
            if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
                if let Ok(json) = serde_json::to_string(&active_ones) {
                    let _ = cache::save_cached_api_data(
                        &conn,
                        GAMERPOWER_CACHE_SOURCE,
                        cache_key,
                        &json,
                    );
                }
            }

            return Ok(active_ones);
        }
    }

    Err("Não foi possível carregar jogos grátis (sem conexão e sem cache).".to_string())
}
