//! Módulo de integração com a API do SteamSpy
//!
//! Usado para obter dados públicos de playtime para estima a duração de um jogo.

use crate::constants::{
    MINUTES_PER_HOUR, STEAMSPY_API_URL, STEAM_REVIEWS_TIMEOUT_SECS, USER_AGENT_BROWSER,
};
use crate::utils::http_client::HTTP_CLIENT;
use serde::Deserialize;
use std::time::Duration;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct SteamSpyResponse {
    median_forever: u32,
}

/// Busca tempo médio de jogo no SteamSpy (em minutos)
pub async fn get_median_playtime(app_id: &str) -> Result<Option<u32>, String> {
    let url = format!("{}?request=appdetails&appid={}", STEAMSPY_API_URL, app_id);

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", USER_AGENT_BROWSER)
        .timeout(Duration::from_secs(STEAM_REVIEWS_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        warn!(
            "SteamSpy respondeu status {} para app_id '{}'",
            response.status(),
            app_id
        );
        return Ok(None);
    }

    // Tenta parsear. Se falhar (ex: jogo não trackeado), retorna None sem erro crítico
    match response.json::<SteamSpyResponse>().await {
        Ok(data) => {
            // SteamSpy retorna em minutos, converter para horas
            let median_hours = data.median_forever / (MINUTES_PER_HOUR as u32);
            // Filtra zeros (jogos sem dados ou nunca jogados)
            if median_hours > 0 {
                Ok(Some(median_hours))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            warn!(
                "Falha ao parsear resposta SteamSpy para app_id '{}': {}",
                app_id, e
            );
            Ok(None)
        }
    }
}
