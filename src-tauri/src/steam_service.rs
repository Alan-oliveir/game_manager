use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SteamGame {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: i32, // Tempo total em minutos
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

pub async fn list_steam_games(api_key: &str, steam_id: &str) -> Result<Vec<SteamGame>, String> {
    let url = format!(
        "http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json&include_appinfo=true&include_played_free_games=true",
        api_key, steam_id
    );

    println!("Buscando jogos na Steam..."); // Log para debug

    let client = reqwest::Client::new();

    let res = client
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
