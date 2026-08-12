//! Caminho: src/services/integration/gamebrain/raw.rs

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RawSearchResponse {
    #[serde(default)]
    pub results: Vec<RawGame>,
}

#[derive(Debug, Deserialize)]
pub struct RawGame {
    pub id: Value,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub year: Option<f64>,
    #[serde(default)]
    pub rating: Option<RawRating>,
    #[serde(default)]
    pub platforms: Vec<RawPlatform>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub cover: Option<RawCover>,
}

#[derive(Debug, Deserialize)]
pub struct RawRating {
    #[serde(default)]
    pub mean: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RawPlatform {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawCover {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct RawSuggestionsResponse {
    #[serde(default)]
    pub results: Vec<RawSuggestion>,
}

#[derive(Debug, Deserialize)]
pub struct RawSuggestion {
    pub id: Value,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawSimilarResponse {
    #[serde(default)]
    pub results: Vec<RawSimilarGame>,
}

#[derive(Debug, Deserialize)]
pub struct RawSimilarGame {
    pub id: Value,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub year: Option<f64>,
    #[serde(default)]
    pub rating: Option<RawRating>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub screenshots: Option<Vec<String>>,
    #[serde(default)]
    pub micro_trailer: Option<String>,
    #[serde(default)]
    pub adult_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct RawGameDetail {
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub screenshots: Option<Vec<String>>,
    #[serde(default)]
    pub videos: Vec<String>,
    #[serde(default)]
    pub micro_trailer: Option<String>,
}
