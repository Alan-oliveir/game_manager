use crate::utils::http_client::HTTP_CLIENT;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct SteamGame {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: i32,
    pub img_icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SteamResponseData {
    game_count: u32,
    games: Vec<SteamGame>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SteamApiResponse {
    response: SteamResponseData,
}

#[derive(Debug, Deserialize)]
pub struct StoreGameDetails {
    pub short_description: Option<String>,
    pub genres: Option<Vec<StoreGenre>>,
    pub release_date: Option<StoreReleaseDate>,
}

#[derive(Debug, Deserialize)]
pub struct StoreGenre {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct StoreReleaseDate {
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct StoreAppResponse {
    pub success: bool,
    pub data: Option<StoreGameDetails>,
}

#[allow(dead_code)]
pub struct ProcessedGameData {
    pub genre: String,
    pub description: String,
    pub release_date: String,
}

pub async fn fetch_game_metadata(app_id: u32) -> Result<ProcessedGameData, String> {
    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}&l=brazilian",
        app_id
    );

    let res: HashMap<String, StoreAppResponse> = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(entry) = res.get(&app_id.to_string()) {
        if entry.success {
            if let Some(data) = &entry.data {
                // Pega o primeiro gênero da lista ou "Desconhecido"
                let genre = data
                    .genres
                    .as_ref()
                    .and_then(|g| g.first())
                    .map(|g| g.description.clone())
                    .unwrap_or("Desconhecido".to_string());

                let description = data.short_description.clone().unwrap_or_default();
                let release = data
                    .release_date
                    .as_ref()
                    .map(|r| r.date.clone())
                    .unwrap_or_default();

                return Ok(ProcessedGameData {
                    genre,
                    description,
                    release_date: release,
                });
            }
        }
    }

    Err("Dados não encontrados".to_string())
}

pub async fn list_steam_games(api_key: &str, steam_id: &str) -> Result<Vec<SteamGame>, String> {
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json&include_appinfo=true&include_played_free_games=true",
        api_key, steam_id
    );

    println!("Buscando jogos na Steam..."); // Log para debug

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Erro na requisição: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Erro na API Steam: Código {}", res.status()));
    }

    let api_data: SteamApiResponse = res
        .json()
        .await
        .map_err(|e| format!("Erro ao ler JSON da Steam: {}", e))?;

    println!(
        "Sucesso! Encontrados {} jogos.",
        api_data.response.game_count
    );

    Ok(api_data.response.games)
}
