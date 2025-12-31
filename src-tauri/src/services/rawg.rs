use crate::utils::http_client::HTTP_CLIENT;
use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RawgTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub language: String,
    pub games_count: i32,
    pub image_background: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawgDeveloper {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawgPublisher {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameDetails {
    pub id: i32,
    pub name: String,
    pub description_raw: String,
    pub metacritic: Option<i32>,
    pub website: String,
    pub tags: Vec<RawgTag>,
    pub developers: Vec<RawgDeveloper>,
    pub publishers: Vec<RawgPublisher>,
}

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

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro RAWG: {}", res.status()));
    }

    let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;

    Ok(data.results)
}

pub async fn fetch_game_details(api_key: &str, query: String) -> Result<GameDetails, String> {
    // Transforma o nome em slug (Lógica de negócio)
    let slug = query
        .to_lowercase()
        .replace(" ", "-")
        .replace(":", "")
        .replace("'", "")
        .replace("&", "")
        .replace(".", "");

    let url = format!("https://api.rawg.io/api/games/{}?key={}", slug, api_key);

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let details: GameDetails = res.json().await.map_err(|e| e.to_string())?;
        Ok(details)
    } else if res.status().as_u16() == 404 {
        Err("Jogo não encontrado na RAWG".into())
    } else {
        Err(format!("Erro na API RAWG: Status {}", res.status()))
    }
}

pub async fn fetch_upcoming_games(api_key: &str) -> Result<Vec<RawgGame>, String> {
    let current_date = chrono::Utc::now();
    let next_year = current_date.year() + 1;

    // Formata datas: YYYY-MM-DD
    let date_start = current_date.format("%Y-%m-%d").to_string();
    let date_end = format!("{}-12-31", next_year);

    // ordering=-added -> Ordena por popularidade
    // dates=HOJE,ANO_QUE_VEM -> Pega apenas futuros
    let url = format!(
        "https://api.rawg.io/api/games?key={}&dates={},{}&ordering=-added&page_size=10",
        api_key, date_start, date_end
    );

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro RAWG Upcoming: {}", res.status()));
    }

    let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data.results)
}
