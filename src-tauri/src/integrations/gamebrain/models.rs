//! Caminho: src/services/integration/gamebrain/models.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBrainSearchResult {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub rating: Option<f64>,
    pub platforms: Vec<String>,
    pub link: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct GameBrainSearchParams {
    pub filters: Vec<GameBrainFilter>,
    pub sort: Option<GameBrainSort>,
    pub sort_order: Option<GameBrainSortOrder>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameBrainFilter {
    pub key: String,
    pub values: Vec<GameBrainFilterValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameBrainFilterValue {
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum GameBrainSort {
    Rating,
    ReleaseDate,
    Price,
}

impl GameBrainSort {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            GameBrainSort::Rating => "computed_rating",
            GameBrainSort::ReleaseDate => "release_date",
            GameBrainSort::Price => "price",
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameBrainSortOrder {
    Asc,
    Desc,
}

impl GameBrainSortOrder {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            GameBrainSortOrder::Asc => "asc",
            GameBrainSortOrder::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarGame {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub rating: Option<f64>,
    pub link: Option<String>,
    pub screenshots: Vec<String>,
    pub micro_trailer: Option<String>,
    pub adult_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMedia {
    pub screenshots: Vec<String>,
    pub trailers: Vec<String>,
    pub youtube_embeds: Vec<String>,
    pub micro_trailer: Option<String>,
}
