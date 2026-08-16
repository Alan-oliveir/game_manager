//! Comandos de depuração/validação de integrações externas.
//!
//! Usaado apenas durante o desenvolvimento.

use crate::providers::metadata::igdb::client::test_connection;
use crate::providers::metadata::igdb::{core::map_igdb_game, fetch::search_and_resolve};
use tauri::AppHandle;

// === IGDB - Auth ===

#[tauri::command]
pub async fn test_igdb_auth(app: AppHandle) -> Result<String, String> {
    test_connection(&app).await
}

// === IGDB - Fetch ===

#[derive(serde::Serialize)]
pub struct IgdbDebugResult {
    pub raw: serde_json::Value,
    pub display_name: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub series: Option<String>,
    pub franchise: Option<Vec<String>>,
    pub game_modes: Option<Vec<String>>,
    pub player_perspectives: Option<Vec<String>>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub critic_score: Option<i32>,
    pub age_ratings: Option<String>,
    pub alternative_names: Option<Vec<String>>,
    pub external_links: Option<String>,
    pub tags: Vec<crate::models::GameTag>,
    pub dlcs: Vec<(String, String)>, // (kind, name)
}

#[tauri::command]
pub async fn debug_igdb_fetch(app: AppHandle, name: String) -> Result<IgdbDebugResult, String> {
    let Some(game) = search_and_resolve(&app, &name).await? else {
        return Err(format!("IGDB: nenhum resultado para '{name}'"));
    };

    let raw = serde_json::to_value(&game).map_err(|e| e.to_string())?;
    let mapped = map_igdb_game(&game, "debug");
    let d = mapped.details;

    Ok(IgdbDebugResult {
        raw,
        display_name: d.display_name,
        release_date: d.release_date,
        genres: d.genres,
        series: d.series,
        franchise: d.franchise,
        game_modes: d.game_modes,
        player_perspectives: d.player_perspectives,
        developer: d.developer,
        publisher: d.publisher,
        critic_score: d.critic_score,
        age_ratings: d.age_ratings,
        alternative_names: d.alternative_names,
        external_links: d.external_links,
        tags: d.tags,
        dlcs: mapped
            .dlcs
            .into_iter()
            .map(|dlc| (dlc.kind.to_string(), dlc.name))
            .collect(),
    })
}
