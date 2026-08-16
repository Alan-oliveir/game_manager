use super::models::{SgdbGrid, SgdbResponse, SgdbSearchResult};
use crate::services::rate_limiter::STEAMGRIDDB_LIMITER;

const BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

pub struct SteamGridDbClient {
    http: reqwest::Client,
    api_key: String,
}

impl SteamGridDbClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<SgdbResponse<T>, String> {
        let url = format!("{BASE_URL}{path}");

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| format!("SteamGridDB request error: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("SteamGridDB API Error: {}", resp.status().as_u16()));
        }

        resp.json::<SgdbResponse<T>>()
            .await
            .map_err(|e| format!("SteamGridDB parse error: {e}"))
    }

    pub async fn search_autocomplete(&self, term: &str) -> Result<Vec<SgdbSearchResult>, String> {
        let encoded = urlencoding::encode(term);
        let path = format!("/search/autocomplete/{encoded}");
        STEAMGRIDDB_LIMITER
            .run(|| self.get::<SgdbSearchResult>(&path))
            .await
            .map(|res| res.data)
    }

    pub async fn get_grids_by_steam_appid(&self, appid: u32) -> Result<Vec<SgdbGrid>, String> {
        let path = format!("/grids/steam/{appid}?dimensions=600x900&styles=alternate&types=static");
        STEAMGRIDDB_LIMITER
            .run(|| self.get::<SgdbGrid>(&path))
            .await
            .map(|res| res.data)
    }

    pub async fn get_grids_by_game_id(&self, sgdb_id: i64) -> Result<Vec<SgdbGrid>, String> {
        let path =
            format!("/grids/game/{sgdb_id}?dimensions=600x900&styles=alternate&types=static");
        STEAMGRIDDB_LIMITER
            .run(|| self.get::<SgdbGrid>(&path))
            .await
            .map(|res| res.data)
    }
}
