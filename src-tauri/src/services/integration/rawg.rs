//! Módulo de integração com a API RAWG.
//!
//! Fornece funcionalidades para buscar informações sobre jogos, incluindo
//! tendências, detalhes específicos e lançamentos futuros.
//!
//! A RAWG é um banco de dados abrangente de jogos que fornece metadados,
//! ratings, gêneros e outras informações relevantes.

use crate::constants::{RAWG_SEARCH_PAGE_SIZE, RAWG_TRENDING_PAGE_SIZE, RAWG_UPCOMING_PAGE_SIZE};
use crate::database::AppState;
use crate::services::cache;
use crate::utils::http_client::HTTP_CLIENT;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

// === ESTRUTURAS DE DADOS PRINCIPAIS ===

#[derive(Debug, Serialize, Deserialize)]
pub struct RawgTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub language: Option<String>,
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
#[derive(Debug, Deserialize, Serialize)]
pub struct RawgGenre {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EsrbRating {
    pub id: Option<i32>,
    pub name: String,
    pub slug: Option<String>,
}

/// Informações sobre a loja onde o jogo está disponível.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreInfo {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

/// Wrapper para informações de loja com URL específica do jogo.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreWrapper {
    pub id: i32,
    pub url: String,
    pub store: StoreInfo,
}

/// Resposta da API RAWG para listagens de jogos.
#[derive(Debug, Deserialize, Serialize)]
struct RawgResponse {
    results: Vec<RawgGame>,
}

/// Representação básica de um jogo na RAWG.
#[derive(Debug, Deserialize, Serialize)]
pub struct RawgGame {
    pub id: u32,
    pub name: String,
    #[serde(rename(deserialize = "background_image", serialize = "backgroundImage"))]
    pub background_image: Option<String>,
    pub rating: f32,
    pub released: Option<String>,
    pub genres: Vec<RawgGenre>,
    #[serde(default)]
    pub tags: Vec<RawgTag>,
    pub slug: String,
}

/// Detalhes completos de um jogo na RAWG.
///
/// Inclui informações expandidas como descrição, metacritic score, desenvolvedoras e tags.
#[derive(Debug, Serialize, Deserialize)]
pub struct RawgGameDetails {
    pub id: i32,
    pub name: String,
    #[serde(rename(deserialize = "description_raw", serialize = "descriptionRaw"))]
    pub description_raw: Option<String>,
    pub metacritic: Option<i32>,
    pub website: Option<String>,
    pub released: Option<String>,
    pub background_image: Option<String>,
    #[serde(default)]
    pub genres: Vec<RawgGenre>,
    #[serde(default)]
    pub tags: Vec<RawgTag>,
    #[serde(default)]
    pub developers: Vec<RawgDeveloper>,
    #[serde(default)]
    pub publishers: Vec<RawgPublisher>,
    #[serde(default)]
    pub reddit_url: Option<String>,
    #[serde(default)]
    pub metacritic_url: Option<String>,
    #[serde(default)]
    pub stores: Vec<StoreWrapper>,
    #[serde(default)]
    pub esrb_rating: Option<EsrbRating>,
}

// === FUNÇÕES DE API ===

/// Busca jogos por texto (Nome, Série, etc).
///
/// Substitui a busca da Steam Store na Wishlist e adição manual.
pub async fn search_games(api_key: &str, query: &str) -> Result<Vec<RawgGame>, String> {
    let url = format!(
        "https://api.rawg.io/api/games?key={}&search={}&page_size={}",
        api_key,
        urlencoding::encode(query),
        RAWG_SEARCH_PAGE_SIZE
    );

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro RAWG Search: {}", res.status()));
    }

    let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data.results)
}

/// Busca detalhes completos de um jogo específico.
///
/// Converte o nome do jogo em slug (formato URL-friendly) e busca informações detalhadas na API RAWG.
pub async fn fetch_game_details(api_key: &str, query: String) -> Result<RawgGameDetails, String> {
    let identifier = if query.chars().all(char::is_numeric) {
        query
    } else {
        query
            .to_lowercase()
            .replace(" ", "-")
            .replace(":", "")
            .replace("'", "")
            .replace("&", "")
            .replace(".", "")
    };

    let url = format!(
        "https://api.rawg.io/api/games/{}?key={}",
        identifier, api_key
    );

    let res = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let details: RawgGameDetails = res.json().await.map_err(|e| e.to_string())?;
        Ok(details)
    } else if res.status().as_u16() == 404 {
        Err("Jogo não encontrado na RAWG".into())
    } else {
        Err(format!("Erro RAWG Details: {}", res.status()))
    }
}

/// Busca os jogos mais populares do momento ('trending games').
///
/// Retorna até 20 jogos lançados entre o ano passado e o ano atual,
/// ordenados por popularidade (adições recentes).
pub async fn fetch_trending_games(app: &AppHandle, api_key: &str) -> Result<Vec<RawgGame>, String> {
    let current_year = chrono::Utc::now().year();
    let last_year = current_year - 1;
    let cache_key = "rawg_list_trending";

    let url = format!(
        "https://api.rawg.io/api/games?key={}&dates={}-01-01,{}-12-31&ordering=-added&page_size={}",
        api_key, last_year, current_year, RAWG_TRENDING_PAGE_SIZE
    );

    // 1. Tenta buscar ONLINE
    match HTTP_CLIENT.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;

                // Sucesso: Salva no Cache (Fire & Forget)
                if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
                    if let Ok(json) = serde_json::to_string(&data.results) {
                        let _ = cache::save_cached_api_data(&conn, "rawg", cache_key, &json);
                    }
                }

                return Ok(data.results);
            }
        }
        Err(_) => {
            // Falha de rede: Silenciosamente cai para o fallback
        }
    }

    // 2. FALLBACK: Tenta buscar Cache Offline ("Stale")
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(payload) = cache::get_stale_api_data(&conn, "rawg", cache_key) {
            if let Ok(cached_games) = serde_json::from_str::<Vec<RawgGame>>(&payload) {
                return Ok(cached_games);
            }
        }
    }

    Err("Não foi possível carregar os jogos em alta (sem conexão e sem cache).".to_string())
}

/// Busca jogos com lançamento futuro.
///
/// Retorna até 10 jogos que ainda serão lançados, desde a data atual
/// até o final do próximo ano, ordenados por popularidade.
pub async fn fetch_upcoming_games(app: &AppHandle, api_key: &str) -> Result<Vec<RawgGame>, String> {
    let current_date = chrono::Utc::now();
    let next_year = current_date.year() + 1;
    let date_start = current_date.format("%Y-%m-%d").to_string();
    let date_end = format!("{}-12-31", next_year);
    let cache_key = "rawg_list_upcoming";

    let url = format!(
        "https://api.rawg.io/api/games?key={}&dates={},{}&ordering=-added&page_size={}",
        api_key, date_start, date_end, RAWG_UPCOMING_PAGE_SIZE
    );

    // 1. Tenta buscar ONLINE
    if let Ok(res) = HTTP_CLIENT.get(&url).send().await {
        if res.status().is_success() {
            let data: RawgResponse = res.json().await.map_err(|e| e.to_string())?;

            // Sucesso: Salva no Cache
            if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
                if let Ok(json) = serde_json::to_string(&data.results) {
                    let _ = cache::save_cached_api_data(&conn, "rawg", cache_key, &json);
                }
            }

            return Ok(data.results);
        }
    }

    // 2. FALLBACK: Cache Offline
    if let Ok(conn) = app.state::<AppState>().cache_db.lock() {
        if let Some(payload) = cache::get_stale_api_data(&conn, "rawg", cache_key) {
            if let Ok(cached_games) = serde_json::from_str::<Vec<RawgGame>>(&payload) {
                return Ok(cached_games);
            }
        }
    }

    Err("Não foi possível carregar lançamentos (sem conexão e sem cache).".to_string())
}
