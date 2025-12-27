use serde::{Deserialize, Serialize};
use chrono::Datelike;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawgGame {
    pub id: u32,
    pub name: String,
    pub background_image: Option<String>,
    pub rating: f32,
    pub released: Option<String>,
    pub genres: Vec<RawgGenre>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawgGenre {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawgResponse {
    results: Vec<RawgGame>,
}

// Busca os jogos mais populares do momento
pub async fn fetch_trending_games(api_key: &str) -> Result<Vec<RawgGame>, String> {
    // Ordena por rating, filtrando datas recentes
    let current_year = chrono::Utc::now().year();
    let last_year = current_year - 1;

    let url = format!(
        "https://api.rawg.io/api/games?key={}&dates={}-01-01,{}-12-31&ordering=-added&page_size=20",
        api_key, last_year, current_year
    );

    let client = reqwest::Client::new();
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro RAWG: {}", res.status()));
    }

    let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;

    Ok(data.results)
}
