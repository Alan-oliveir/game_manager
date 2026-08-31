use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbNamed {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbInvolvedCompany {
    pub company: IgdbNamed,
    #[serde(default)]
    pub developer: bool,
    #[serde(default)]
    pub publisher: bool,
    #[serde(default)]
    pub porting: bool,
    #[serde(default)]
    pub supporting: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbRatingCategory {
    pub rating: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbAgeRating {
    pub organization: Option<IgdbNamed>,
    pub rating_category: Option<IgdbRatingCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbAlternativeName {
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbCover {
    pub image_id: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbWebsiteType {
    #[serde(rename = "type")]
    pub name: String, // Ex.: "official", "steam", "wikia", etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbWebsite {
    pub url: String,
    #[serde(rename = "type")]
    pub website_type: Option<IgdbWebsiteType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbExpansionRef {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub cover: Option<IgdbCover>, // reaproveita a struct IgdbCover que já existe
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbTimeToBeat {
    pub game_id: i64,
    pub hastily: Option<i64>,
    pub normally: Option<i64>,
    pub completely: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub url: Option<String>,
    pub first_release_date: Option<i64>,
    pub game_type: Option<i32>,
    pub status: Option<i32>, // enum IGDB: 0=released, 6=cancelled, 7=rumored, 8=delisted, etc.

    #[serde(default)]
    pub genres: Vec<IgdbNamed>,
    #[serde(default)]
    pub themes: Vec<IgdbNamed>,
    #[serde(default)]
    pub keywords: Vec<IgdbNamed>,
    #[serde(default)]
    pub game_modes: Vec<IgdbNamed>,
    #[serde(default)]
    pub player_perspectives: Vec<IgdbNamed>,
    #[serde(default)]
    pub collections: Vec<IgdbNamed>,
    #[serde(default)]
    pub franchises: Vec<IgdbNamed>,
    #[serde(default)]
    pub involved_companies: Vec<IgdbInvolvedCompany>,
    #[serde(default)]
    pub age_ratings: Vec<IgdbAgeRating>,
    #[serde(default)]
    pub alternative_names: Vec<IgdbAlternativeName>,
    #[serde(default)]
    pub game_engines: Vec<IgdbNamed>,
    #[serde(default)]
    pub expansions: Vec<IgdbExpansionRef>,
    #[serde(default)]
    pub standalone_expansions: Vec<IgdbExpansionRef>,
    #[serde(default)]
    pub websites: Vec<IgdbWebsite>,

    pub aggregated_rating: Option<f64>,
    pub aggregated_rating_count: Option<i32>,

    pub cover: Option<IgdbCover>,
    pub parent_game: Option<IgdbNamed>,
    pub version_parent: Option<IgdbNamed>,
    pub version_title: Option<String>,
}
