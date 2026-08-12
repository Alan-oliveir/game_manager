//! Provedor de Mídia via GameBrain (Screenshots, Trailers e Embeds).
//!
//! Este módulo fornece funcionalidades para buscar e exibir mídia de jogos através da API do GameBrain.

use crate::integrations::gamebrain::cache::{
    gamebrain_media_cache_key, read_cached_json, save_cached_json,
};
use crate::integrations::gamebrain::client::GameBrainClient;
use crate::integrations::gamebrain::models::GameMedia;
use crate::integrations::gamebrain::raw::RawGameDetail;
use crate::utils::http_client::HTTP_CLIENT;
use tauri::AppHandle;

/// Busca a mídia (screenshots, trailers, embeds YouTube) de um jogo via GameBrain.
pub async fn fetch_game_media(
    app: &AppHandle,
    playlite_game_id: &str,
    game_name: &str,
) -> Result<GameMedia, String> {
    let api_key = GameBrainClient::get_api_key(app)?;

    // Etapa 1: Resolver o gamebrain_id
    // Se a Descoberta já buscou por esse jogo, isso vai dar cache hit instantâneo!
    let gamebrain_id = GameBrainClient::resolve_id(app, playlite_game_id, game_name).await?;

    // Etapa 2: Buscar mídia do game detail
    let media_cache_key = gamebrain_media_cache_key(gamebrain_id);

    if let Some(cached) = read_cached_json::<GameMedia>(app, "gamebrain", &media_cache_key, false)? {
        tracing::debug!("GameBrain media cache hit => gamebrain_id={}", gamebrain_id);
        return Ok(cached);
    }

    // Cache miss: chama GET /v1/games/{id}
    tracing::debug!(
        "GameBrain media cache miss => chamando /v1/games/{} para mídia",
        gamebrain_id
    );

    let url = format!("https://api.gamebrain.co/v1/games/{}", gamebrain_id);
    let response = HTTP_CLIENT
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await;

    let response = match response {
        Ok(r) => r,
        Err(err) => {
            // Stale fallback em caso de falha de rede
            tracing::error!("GameBrain game detail request error: {}", err);
            return read_cached_json::<GameMedia>(app, "gamebrain", &media_cache_key, true)?
                .ok_or(err.to_string());
        }
    };

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::error!("GameBrain game detail HTTP Error => status={} body={}", status, body);
        return read_cached_json::<GameMedia>(app, "gamebrain", &media_cache_key, true)?
            .ok_or_else(|| format!("Erro GameBrain game detail: {}", status));
    }

    let text = response.text().await.map_err(|e| e.to_string())?;

    let raw: RawGameDetail = serde_json::from_str(&text).map_err(|e| {
        tracing::error!("GameBrain game detail JSON parse error: {}", e);
        format!("Erro JSON GameBrain game detail: {}", e)
    })?;

    // Classifica trailers, embeds e deduplica imagens usando o helper centralizado
    let media = GameBrainClient::build_game_media(raw);

    tracing::debug!(
        "GameBrain media => {} screenshots, {} trailers, {} embeds para gamebrain_id={}",
        media.screenshots.len(),
        media.trailers.len(),
        media.youtube_embeds.len(),
        gamebrain_id
    );

    let _ = save_cached_json(app, "gamebrain", &media_cache_key, &media);

    Ok(media)
}
