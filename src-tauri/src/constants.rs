// Constantes da aplicação para facilitar manutenção

// URLs e endpoints
pub const STEAM_CDN_URL: &str = "https://cdn.cloudflare.steamstatic.com";
#[allow(dead_code)]
pub const STEAM_STORE_API_URL: &str = "https://store.steampowered.com/api/appdetails";
#[allow(dead_code)]
pub const STEAM_PLAYER_API_URL: &str = "http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001";
#[allow(dead_code)]
pub const RAWG_API_URL: &str = "https://api.rawg.io/api/games";

// Valores padrão
pub const DEFAULT_GENRE: &str = "Desconhecido";
#[allow(dead_code)]
pub const DEFAULT_PLATFORM_MANUAL: &str = "Manual";
pub const DEFAULT_PLATFORM_STEAM: &str = "Steam";

// Limites de validação
pub const MAX_NAME_LENGTH: usize = 200;
pub const MAX_GENRE_LENGTH: usize = 100;
pub const MAX_PLATFORM_LENGTH: usize = 100;
pub const MAX_URL_LENGTH: usize = 500;
pub const MAX_PLAYTIME: i32 = 1_000_000;
pub const MIN_RATING: i32 = 1;
pub const MAX_RATING: i32 = 5;

// Rate limiting
pub const STEAM_RATE_LIMIT_MS: u64 = 1500;
#[allow(dead_code)]
pub const RAWG_PAGE_SIZE: u32 = 20;

// Configuração de banco de dados
#[allow(dead_code)]
pub const DB_FILENAME: &str = "library.db";
#[allow(dead_code)]
pub const DB_JOURNAL_MODE: &str = "WAL";

