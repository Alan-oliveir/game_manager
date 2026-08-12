// === ARQUIVO CENTRALIZADO COM CONSTANTES DO BACKEND ===

// === URLs e ENDPOINTS ===
pub const STEAM_CDN_URL: &str = "https://cdn.cloudflare.steamstatic.com";
pub const STEAM_CDN_AKAMAI_URL: &str = "https://cdn.akamai.steamstatic.com";
pub const STEAM_STORE_URL: &str = "https://store.steampowered.com";
pub const STEAM_STORE_API_URL: &str = "https://store.steampowered.com/api/appdetails";
pub const STEAM_STORE_SEARCH_URL: &str = "https://store.steampowered.com/api/storesearch/";
pub(crate) const REVIEW_API_URL: &str = "https://store.steampowered.com/appreviews";
pub const ITAD_API_URL: &str = "https://api.isthereanydeal.com";
pub(crate) const GEMINI_API_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
pub const PCGW_API_BASE: &str = "https://www.pcgamingwiki.com/w/api.php"; // Base da MediaWiki Action API do PCGamingWiki.
pub const IGDB_TOKEN_ENDPOINT: &str = "https://id.twitch.tv/oauth2/token";
pub const IGDB_API_BASE: &str = "https://api.igdb.com/v4";

// === METADATA ===
pub const NOT_FOUND_MARKER: &str = "__NOT_FOUND__";

// === CAMINHOS DE IMAGENS DA STEAM ===
pub const STEAM_HEADER_IMAGE_PATH: &str = "steam/apps/{}/header.jpg";
pub const STEAM_LIBRARY_IMAGE_PATH: &str = "steam/apps/{}/library_600x900.jpg";

// === FORMATOS E EXTENSÕES ===
pub const COVER_IMAGE_EXTENSION: &str = ".jpg";

// === DIRETÓRIOS ===
pub const COVERS_DIR_NAME: &str = "covers";
pub const EPIC_MANIFEST_PATH_WINDOWS: &str =
    r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";

// === VALORES PADRÕES E LIMITES ===
pub const MAX_NAME_LENGTH: usize = 200;
pub const MAX_URL_LENGTH: usize = 500;
pub const MAX_PLAYTIME: i32 = 1_000_000;
pub const MIN_RATING: i32 = 1;
pub const MAX_RATING: i32 = 5;

// === LIMITES DO SCANNER DE JOGOS LOCAIS ===
pub const SCANNER_MAX_DEPTH: usize = 4;
pub const SCANNER_MAX_FILES_PER_DIR: usize = 1000;
pub const SCANNER_MAX_TOTAL_FILES: usize = 10000;

// === CONVERSÕES ===
pub const BYTES_PER_MB: u64 = 1024 * 1024;
pub const MINUTES_PER_HOUR: i32 = 60;
pub const MINUTES_PER_HOUR_F32: f32 = 60.0;

// === PESOS DO SISTEMA DE RECOMENDAÇÃO ===
pub const RECOMMENDATION_MAX_TAG_CONTRIBUTION: f32 = 300.0;
pub const RECOMMENDATION_WEIGHT_GENRE: f32 = 3.0;
pub const RECOMMENDATION_WEIGHT_PLAYTIME_HOUR: f32 = 1.2;
pub const RECOMMENDATION_WEIGHT_FAVORITE: f32 = 30.0;
pub const RECOMMENDATION_WEIGHT_USER_RATING: f32 = 8.0;
pub const RECOMMENDATION_WEIGHT_SERIES: f32 = 1.0;
pub const RECOMMENDATION_MAX_PLAYTIME_HOURS: i32 = 100;

// === CONFIGURAÇÕES PADRÃO DE RECOMENDAÇÃO ===
pub const RECOMMENDATION_DEFAULT_CONTENT_WEIGHT: f32 = 0.65;
pub const RECOMMENDATION_DEFAULT_COLLABORATIVE_WEIGHT: f32 = 0.35;
pub const RECOMMENDATION_DEFAULT_AGE_DECAY: f32 = 0.98;

// === MOEDAS ===
pub const DEFAULT_CURRENCY: &str = "BRL";

// === LIMITES DE REQUISIÇÕES A SERVIÇOS EXTERNOS ===
pub const RAWG_RATE_LIMIT_MS: u64 = 150;
pub const RAWG_MAX_CONCURRENT_REQUESTS: u32 = 4;
pub const REQUISITIONS_PER_BATCH: u32 = 25;
pub const RAWG_BACKOFF_BASE_MS: u64 = 2000; // espera inicial após 429
pub const RAWG_BACKOFF_MAX_RETRIES: u32 = 3; // 2s, 4s, 8s antes de desistir
pub const RAWG_SEARCH_PAGE_SIZE: u32 = 10;
pub const RAWG_TRENDING_PAGE_SIZE: u32 = 15;
pub const RAWG_UPCOMING_PAGE_SIZE: u32 = 10;
pub const STEAM_RATE_LIMIT_MS: u64 = 200;
pub const STEAM_MAX_CONCURRENT_REQUESTS: u32 = 2;
pub const STEAM_BACKOFF_BASE_MS: u64 = 2000;
pub const STEAM_BACKOFF_MAX_RETRIES: u32 = 3;
pub const IGDB_MAX_CONCURRENT_REQUESTS: u32 = 4;
pub const IGDB_BACKOFF_MAX_RETRIES: u32 = 3;
pub const IGDB_BACKOFF_BASE_MS: u64 = 2000;
pub const GAME_PASS_BATCH_SIZE: usize = 20;
pub const GAMEBRAIN_SIMILAR_REQUEST_LIMIT: u32 = 12;
pub const REQUEST_PCGW_DELAY_MS: u64 = 250; // Delay (ms) devido ao rate limit de 30 req/min de PCGamingWiki.

// === TIMEOUTS DE REQUISIÇÕES HTTP (em segundos) ===
pub const HTTP_REQUEST_TIMEOUT_SECS: u64 = 30;
pub const HTTP_CONNECT_TIMEOUT_SECS: u64 = 10;
pub const STEAM_STORE_TIMEOUT_SECS: u64 = 10;

// === USER AGENTS ===
pub const USER_AGENT_DEFAULT: &str = "GameManager/0.1.0";
pub const USER_AGENT_STEAM: &str = "Valve/Steam HTTP Client 1.0";
pub const USER_AGENT_BROWSER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

// === TIMEOUTS DE AUTENTICAÇÃO OAuth (em segundos) ===
pub const OAUTH_CALLBACK_TIMEOUT_SECS: u64 = 120;

// === DELAYS DE INICIALIZAÇÃO E BACKGROUND TASKS (em segundos) ===
pub const STARTUP_DELAY_SECS: u64 = 5;
pub const BACKGROUND_TASK_INTERVAL_SECS: u64 = 2;

// === TTL DE CACHE (Time To Live em dias) ===
pub const CACHE_STEAM_RESOLVE_TTL_DAYS: i64 = 90;
pub const CACHE_PROTON_DB_TTL_DAYS: i64 = 30;
pub const CACHE_RAWG_GAME_TTL_DAYS: i64 = 30;
pub const CACHE_RAWG_LIST_TTL_DAYS: i64 = 1; // Trending/Upcoming/Giveaways
pub const CACHE_GAMEBRAIN_ID_TTL_DAYS: i64 = 30;
pub const CACHE_GAMEBRAIN_SIMILAR_TTL_DAYS: i64 = 30;
pub const CACHE_GAMEBRAIN_MEDIA_TTL_DAYS: i64 = 30;
pub const CACHE_STEAM_STORE_TTL_DAYS: i64 = 30;
pub const CACHE_STEAM_REVIEWS_TTL_DAYS: i64 = 7;
pub const CACHE_STEAM_PLAYTIME_TTL_DAYS: i64 = 15;
pub const CACHE_AMAZON_LUNA_TTL_DAYS: i64 = 1;
pub const CACHE_GAMERPOWER_TTL_DAYS: i64 = 1;
pub const CACHE_GAME_PASS_FULL_TTL_DAYS: i64 = 15;
pub const CACHE_UBISOFT_PLUS_TTL_DAYS: i64 = 30;
pub const CACHE_EA_PLAY_TTL_DAYS: i64 = 30;
pub const NEXUS_CACHE_TTL_DAYS: i64 = 20;
pub const CACHE_HLTB_TTL_DAYS: i64 = 30;
pub const CACHE_DEFAULT_TTL_DAYS: i64 = 7;
pub const CACHE_NEXUS_TRENDING_MODS_TTL_DAYS: i64 = 28;
pub const CACHE_STEAM_TRENDING_TTL_DAYS: i64 = 7;
pub const CACHE_IGDB_UPCOMING_TTL_DAYS: i64 = 14;

// === CHAVES DE CACHE DE ASSINATURAS ===
pub const AMAZON_LUNA_CACHE_SOURCE: &str = "amazon_luna";
pub const AMAZON_LUNA_CACHE_KEY: &str = "catalog_amazon_luna";
pub const GAMERPOWER_CACHE_SOURCE: &str = "gamerpower";
pub const GAMERPOWER_LIST_ACTIVE_CACHE_KEY: &str = "gamerpower_list_active";
pub const EA_PLAY_CACHE_SOURCE: &str = "ea_play";
pub const EA_PLAY_CACHE_KEY: &str = "catalog_ea_play";
pub const GAME_PASS_CACHE_SOURCE: &str = "game_pass_pc";
pub const GAME_PASS_FULL_CACHE_KEY: &str = "catalog_game_pass_full";
pub const UBISOFT_PLUS_CACHE_SOURCE: &str = "ubisoft_plus";
pub const UBISOFT_PLUS_CACHE_KEY: &str = "catalog_ubisoft_plus";

// === NOMES DE ARQUIVOS E CONFIGURAÇÕES DE BANCO DE DADOS ===
pub const DB_FILENAME_GAMES: &str = "games.db";
pub const DB_FILENAME_CACHE: &str = "cache.db";
pub const DB_FILENAME_SECRETS: &str = "secrets.db";
pub const DB_JOURNAL_MODE: &str = "WAL";

// === OAuth GOG ===
pub const GOG_CLIENT_ID: &str = "46899977096215655";
pub const GOG_CLIENT_SECRET: &str =
    "9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9";
pub const GOG_AUTH_ENDPOINT: &str = "https://login.gog.com/auth";
pub const GOG_TOKEN_ENDPOINT: &str = "https://auth.gog.com/token";
pub const GOG_REDIRECT_URI: &str = "https://embed.gog.com/on_login_success?origin=client";
pub const GOG_FILTERED_PRODUCTS_ENDPOINT: &str =
    "https://embed.gog.com/account/getFilteredProducts";

// === OAuth Epic Games — endpoints e client_id/secret públicos ===
pub const EPIC_OAUTH_CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
pub const EPIC_OAUTH_CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";
pub const EPIC_LOGIN_URL: &str = "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode";
pub const EPIC_REDIRECT_PREFIX: &str = "https://www.epicgames.com/id/api/redirect";
pub const EPIC_PSEUDO_REDIRECT_SCHEME: &str = "playlite://epic-auth-code";
pub const EPIC_TOKEN_ENDPOINT: &str =
    "https://account-public-service-prod.ol.epicgames.com/account/api/oauth/token";
pub const EPIC_LIBRARY_ENDPOINT: &str =
    "https://library-service.live.use1a.on.epicgames.com/library/api/public/items";
pub const EPIC_CATALOG_BULK_ENDPOINT: &str =
    "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace";

// === Amazon Games (device registration flow — baseado no cliente open-source Nile) ===
pub const AMAZON_API: &str = "https://api.amazon.com";
pub const AMAZON_GAMING_DISTRIBUTION_ENTITLEMENTS: &str =
    "https://gaming.amazon.com/api/distribution/entitlements";
pub const AMAZON_MARKETPLACE_ID: &str = "ATVPDKIKX0DER";
pub const AMAZON_ASSOC_HANDLE: &str = "amzn_sonic_games_launcher";
pub const AMAZON_DEVICE_TYPE: &str = "A2UMVHOX7UP4V7";
pub const AMAZON_APP_NAME: &str = "AGSLauncher for Windows";
pub const AMAZON_APP_VERSION: &str = "1.0.0";
pub const AMAZON_REDIRECT_PREFIX: &str =
    "https://www.amazon.com/?openid.assoc_handle=amzn_sonic_games_launcher";
pub const AMAZON_ENTITLEMENTS_KEY_ID: &str = "d5dc8b8b-86c8-4fc4-ae93-18c0def5314d";

// === XBOX LIVE  — Autenticação (MSA → XAU → XSTS) ===
pub const XBOX_MSA_AUTHORIZE_ENDPOINT: &str = "https://login.live.com/oauth20_authorize.srf";
pub const XBOX_MSA_TOKEN_ENDPOINT: &str = "https://login.live.com/oauth20_token.srf";
pub const XBOX_MSA_REDIRECT_URI: &str = "http://localhost:8902/auth/callback";
pub const XBOX_MSA_SCOPES: &str = "Xboxlive.signin Xboxlive.offline_access";
pub const XBOX_USER_AUTH_ENDPOINT: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub const XBOX_XSTS_AUTH_ENDPOINT: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
pub const XBOX_ACHIEVEMENTS_BASE_URL: &str = "https://achievements.xboxlive.com";

// === Legendary (Epic CLI) — Linux only ===
pub const LEGENDARY_GITHUB_REPO: &str = "derrod/legendary";
pub const LEGENDARY_BIN_NAME: &str = "legendary";

// === Diretório de ferramentas gerenciadas ===
pub const TOOLS_DIR_NAME: &str = "tools";
pub const TOOL_CONFIG_KEY_PREFIX: &str = "tool_path_"; // ex: "tool_path_legendary"
