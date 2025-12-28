use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub genre: Option<String>,
    pub platform: Option<String>,
    pub cover_url: Option<String>,
    pub playtime: i32,
    pub rating: Option<i32>,
    pub favorite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WishlistGame {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub store_url: Option<String>,
    pub current_price: Option<f64>, // Usamos Option porque pode ser nulo ou zero
    pub lowest_price: Option<f64>,
    pub on_sale: bool,
    pub added_at: Option<String>, // SQLite retorna datas como String
}

// Enum de erros personalizados para melhor diagnóstico
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    NetworkError(String),
    NotFound(String),
    MutexError,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Erro de banco de dados: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Erro de validação: {}", msg),
            AppError::NetworkError(msg) => write!(f, "Erro de rede: {}", msg),
            AppError::NotFound(msg) => write!(f, "Não encontrado: {}", msg),
            AppError::MutexError => {
                write!(f, "Erro interno: falha ao acessar recurso compartilhado")
            }
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
