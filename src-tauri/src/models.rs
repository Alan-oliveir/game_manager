use serde::{Deserialize, Serialize};

// O derive(Serialize, Deserialize) é o que permite que essa estrutura
// vire um JSON automaticamente para o React ler.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub genre: Option<String>, // Option significa que pode ser null
    pub platform: Option<String>,
    pub cover_url: Option<String>,
    pub playtime: i32,
    pub rating: Option<i32>,
    pub favorite: bool,
}
