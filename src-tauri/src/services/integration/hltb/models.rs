use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct AuthInitResponse {
    pub(crate) token: String,
    #[serde(rename = "hpKey")]
    pub(crate) hp_key: String,
    #[serde(rename = "hpVal")]
    pub(crate) hp_val: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchOptions {
    pub(crate) games: GamesFilter,
}

#[derive(Debug, Serialize)]
pub(crate) struct GamesFilter {
    pub(crate) platform: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchPayload {
    pub(crate) search_terms: Vec<String>,
    pub(crate) search_options: SearchOptions,
}