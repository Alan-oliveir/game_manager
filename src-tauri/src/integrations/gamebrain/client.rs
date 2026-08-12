//! Caminho: src/services/integration/gamebrain/client.rs

use super::cache;
use super::models::GameMedia;
use super::raw::{RawGameDetail, RawSuggestionsResponse};
use crate::database;
use crate::utils::http_client::HTTP_CLIENT;
use tauri::AppHandle;

pub struct GameBrainClient;

impl GameBrainClient {
    /// Obtém a API Key configurada pelo usuário no banco de dados
    pub fn get_api_key(app: &AppHandle) -> Result<String, String> {
        let api_key = database::get_secret(app, "gamebrain_api_key").map_err(|e| e.to_string())?;

        if api_key.trim().is_empty() {
            return Err("GameBrain API key não configurada".into());
        }

        Ok(api_key)
    }

    /// Resolve o gamebrain_id (u64) a partir do ID interno do Playlite e do Nome do jogo.
    /// Gerencia automaticamente cache (hits) e stale cache (em caso de falha de rede).
    pub async fn resolve_id(
        app: &AppHandle,
        playlite_game_id: &str,
        game_name: &str,
    ) -> Result<u64, String> {
        let id_cache_key = cache::gamebrain_id_cache_key(playlite_game_id);

        // 1. Tenta pegar do cache normal
        if let Some(id) = cache::read_cached_json::<u64>(app, "gamebrain", &id_cache_key, false)? {
            tracing::debug!(
                "GameBrain ID cache hit => game_id='{}' gamebrain_id={}",
                playlite_game_id,
                id
            );
            return Ok(id);
        }

        tracing::debug!(
            "GameBrain ID cache miss => resolvendo '{}' via suggestions",
            game_name
        );

        let api_key = Self::get_api_key(app)?;

        // 2. Chama a API para descobrir o ID
        match Self::fetch_suggestions_id(&api_key, game_name).await {
            Ok(Some(id)) => {
                let _ = cache::save_cached_json(app, "gamebrain", &id_cache_key, &id);
                Ok(id)
            }
            Ok(None) => {
                // Tenta cache antigo (stale) se o jogo não foi encontrado na API
                if let Some(stale_id) =
                    cache::read_stale_gamebrain_id(app, &id_cache_key, playlite_game_id)?
                {
                    Ok(stale_id)
                } else {
                    Err(format!("Jogo '{}' não encontrado na GameBrain", game_name))
                }
            }
            Err(err) => {
                // Tenta cache antigo (stale) se houver erro de rede/API
                if let Some(stale_id) =
                    cache::read_stale_gamebrain_id(app, &id_cache_key, playlite_game_id)?
                {
                    Ok(stale_id)
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Faz a requisição de fato para o endpoint /suggestions para encontrar o ID
    async fn fetch_suggestions_id(api_key: &str, game_name: &str) -> Result<Option<u64>, String> {
        let url = "https://api.gamebrain.co/v1/games/suggestions";

        let response = HTTP_CLIENT
            .get(url)
            .header("x-api-key", api_key)
            .query(&[("query", game_name)])
            .send()
            .await
            .map_err(|e| {
                tracing::error!("GameBrain suggestions request error: {}", e);
                e.to_string()
            })?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                "GameBrain suggestions HTTP Error => status={} body={}",
                status,
                body
            );
            return Err(format!("Erro GameBrain suggestions: {}", status));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;
        let raw: RawSuggestionsResponse = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("GameBrain suggestions JSON parse error: {}", e);
            format!("Erro JSON GameBrain suggestions: {}", e)
        })?;

        let first = match raw.results.into_iter().next() {
            Some(s) => s,
            None => {
                tracing::debug!("GameBrain suggestions => nenhum resultado para '{}'", game_name);
                return Ok(None);
            }
        };

        let query_lower = game_name.to_lowercase();
        let result_lower = first.name.to_lowercase();

        let query_tokens: Vec<&str> = query_lower.split_whitespace().take(2).collect();
        let result_tokens: Vec<&str> = result_lower.split_whitespace().take(2).collect();

        let is_match = result_lower.contains(&query_lower)
            || query_lower.contains(&result_lower)
            || query_tokens == result_tokens;

        if !is_match {
            tracing::debug!(
                "GameBrain suggestions => match rejeitado: query='{}' result='{}'",
                game_name,
                first.name
            );
            return Ok(None);
        }

        let id = Self::parse_gamebrain_id(&first.id);

        tracing::debug!("GameBrain suggestions => '{}' resolvido para id={:?}", game_name, id);

        Ok(id)
    }

    /// Helper: Extrai um ID numérico de um serde_json::Value.
    pub fn parse_gamebrain_id(value: &serde_json::Value) -> Option<u64> {
        match value {
            serde_json::Value::Number(n) => n.as_u64(),
            serde_json::Value::String(s) => s.parse::<u64>().ok(),
            _ => None,
        }
    }

    /// Helper: Constrói GameMedia a partir do raw, classificando os vídeos.
    pub fn build_game_media(raw: RawGameDetail) -> GameMedia {
        let mut trailers = Vec::new();
        let mut youtube_embeds = Vec::new();

        for url in raw.videos {
            if url.ends_with(".webm") {
                trailers.push(url);
            } else if url.contains("youtube-nocookie.com/embed") {
                youtube_embeds.push(url);
            }
        }

        let mut screenshots = raw.screenshots.unwrap_or_default();
        if let Some(ref img) = raw.image {
            if !screenshots.contains(img) {
                screenshots.insert(0, img.clone());
            }
        }

        GameMedia {
            screenshots,
            trailers,
            youtube_embeds,
            micro_trailer: raw.micro_trailer,
        }
    }
}
