use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SgdbResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    #[serde(default)]
    pub total: Option<u32>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SgdbSearchResult {
    pub id: i64,
    pub name: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SgdbGrid {
    pub id: i64,
    pub style: String, // "alternate" | "white_logo" | "no_logo" | "material" | "blurred"
    pub width: u32,
    pub height: u32,
    pub mime: String,
    pub url: String,
    pub thumb: String,
}
