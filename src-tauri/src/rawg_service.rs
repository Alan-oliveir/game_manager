use serde::{Deserialize, Serialize};

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
    // Ordena por rating, filtrando datas recentes (exemplo simplificado, pega top geral por enquanto)
    // Para "Trending" real, geralmente pegamos lançamentos recentes com rating alto.
    // Vamos usar a endpoint "lists/main" ou ordenar por popularidade.

    let url = format!(
        "https://api.rawg.io/api/games?key={}&dates=2024-01-01,2025-12-31&ordering=-added&page_size=20",
        api_key
    );

    let client = reqwest::Client::new();
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro RAWG: {}", res.status()));
    }

    let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;

    Ok(data.results)
}