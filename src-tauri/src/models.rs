//! Modelos de dados do banco de dados.
//!
//! Define as structs que representam as tabelas do banco:
//! - Game: tabela `games`
//! - GameDetails: tabela `game_details`
//! - WishlistGame: tabela `wishlist`
//! - Subscriptions: tabela `subscriptions`
//! - PcgwData: tabela `pcgw_data`

// Reexportar GameTag do utils para consistência
pub use crate::utils::tag_utils::GameTag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

// === ENUMS ===

/// Nível de confiança da importação do jogo
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ImportConfidence {
    High,
    Medium,
    Low,
}

impl FromStr for ImportConfidence {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "High" => Ok(ImportConfidence::High),
            "Medium" => Ok(ImportConfidence::Medium),
            "Low" => Ok(ImportConfidence::Low),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ImportConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ImportConfidence::High => "High",
            ImportConfidence::Medium => "Medium",
            ImportConfidence::Low => "Low",
        };
        write!(f, "{}", s)
    }
}

/// Plataformas suportadas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Platform {
    Steam,
    Epic,
    GOG,
    Amazon,
    Ubisoft,
    EA,
    BattleNet,
    Xbox,
    LegacyGames,
    Indiegala,
    Itch,
    Indie,
    Outra,
}

impl FromStr for Platform {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Steam" => Ok(Platform::Steam),
            "Epic" => Ok(Platform::Epic),
            "GOG" => Ok(Platform::GOG),
            "Amazon" => Ok(Platform::Amazon),
            "Ubisoft" => Ok(Platform::Ubisoft),
            "EA" => Ok(Platform::EA),
            "BattleNet" => Ok(Platform::BattleNet),
            "Xbox" => Ok(Platform::Xbox),
            "LegacyGames" => Ok(Platform::LegacyGames),
            "Indiegala" => Ok(Platform::Indiegala),
            "Itch" => Ok(Platform::Itch),
            "Indie" => Ok(Platform::Indie),
            "Outra" => Ok(Platform::Outra),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Platform::Steam => "Steam",
            Platform::Epic => "Epic",
            Platform::GOG => "GOG",
            Platform::Amazon => "Amazon",
            Platform::EA => "EA",
            Platform::Ubisoft => "Ubisoft",
            Platform::BattleNet => "BattleNet",
            Platform::Xbox => "Xbox",
            Platform::LegacyGames => "LegacyGames",
            Platform::Indiegala => "Indiegala",
            Platform::Itch => "Itch",
            Platform::Indie => "Indie",
            Platform::Outra => "Outra",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PlaytimeSource {
    Platform(Platform), // API oficial (Steam, Itch, Indiegala)
    Local,              // tracker local via processo
}

impl PlaytimeSource {
    /// Serializa para o formato salvo na coluna `playtime_source` (TEXT).
    pub fn as_db_str(&self) -> String {
        match self {
            PlaytimeSource::Local => "local".to_string(),
            PlaytimeSource::Platform(p) => p.to_string(),
        }
    }
}

impl FromStr for PlaytimeSource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "local" {
            Ok(PlaytimeSource::Local)
        } else {
            s.parse::<Platform>().map(PlaytimeSource::Platform)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum ManagedTool {
    Legendary,
    // Nile depois
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ToolStatus {
    NotFound,
    Found {
        path: String,
        version: Option<String>,
        source: ToolSource,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ToolSource {
    SystemPath,
    Managed,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameDescription {
    pub summary: Option<String>,           // IGDB
    pub storyline: Option<String>,         // IGDB
    pub short_description: Option<String>, // Steam
    pub description: Option<String>,       // Indiegala / Itch / Legacy Games (fallback genérico)
    pub summary_translated: Option<String>,
    pub storyline_translated: Option<String>,
    pub short_description_translated: Option<String>,
    pub description_translated: Option<String>,
    pub translated_lang: Option<String>,
}

impl GameDescription {
    /// Usado em contextos de preview — texto único.
    pub fn primary(&self) -> Option<&str> {
        self.summary.as_deref()
            .or(self.description.as_deref())
            .or(self.short_description.as_deref())
            .or(self.storyline.as_deref())
    }

    /// Campos a exibir/traduzir na tela de detalhes, seguindo a prioridade:
    /// summary + storyline juntos (se qualquer um existir) > short_description > description.
    pub fn fields_to_display(&self) -> Vec<(&'static str, &str)> {
        let mut fields = Vec::new();
        if let Some(s) = &self.summary {
            fields.push(("summary", s.as_str()));
        }
        if let Some(s) = &self.storyline {
            fields.push(("storyline", s.as_str()));
        }
        if !fields.is_empty() {
            return fields;
        }
        if let Some(s) = &self.short_description {
            return vec![("short_description", s.as_str())];
        }
        if let Some(s) = &self.description {
            return vec![("description", s.as_str())];
        }
        vec![]
    }

    pub fn translated_value(&self, field: &str) -> Option<&str> {
        match field {
            "summary" => self.summary_translated.as_deref(),
            "storyline" => self.storyline_translated.as_deref(),
            "short_description" => self.short_description_translated.as_deref(),
            "description" => self.description_translated.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn set_translated_value(&mut self, field: &str, value: String) {
        match field {
            "summary" => self.summary_translated = Some(value),
            "storyline" => self.storyline_translated = Some(value),
            "short_description" => self.short_description_translated = Some(value),
            "description" => self.description_translated = Some(value),
            _ => {}
        }
    }
}

// === Modelos de Dados ===

/// Jogo na biblioteca do usuário.
///
/// Representa um jogo adicionado à biblioteca pessoal, com metadados
/// importados da plataforma e dados de progresso do usuário.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub alternative_names: Option<Vec<String>>,
    pub source_label: Option<String>,
    pub platform_game_id: String,

    // Metadados básicos
    pub genres: Option<String>,
    pub developer: Option<String>,
    pub platform: Platform,

    // Mídia
    pub cover_url: Option<String>,

    // Execução
    pub installed: bool,
    pub install_path: Option<String>,
    pub executable_path: Option<String>,
    pub launch_args: Option<String>,
    pub import_confidence: Option<ImportConfidence>,

    // Dados do Usuário
    pub user_rating: Option<i32>,
    pub favorite: bool,
    pub status: Option<String>,
    pub playtime: Option<i32>,
    pub playtime_source: Option<PlaytimeSource>,

    // Metadados de Tempo
    pub last_played: Option<String>,
    pub added_at: String,

    // Conteúdo Adulto
    #[serde(default)]
    pub is_adult: bool,
}

/// Detalhes adicionais do jogo (Schema v4).
///
/// Contém metadados enriquecidos obtidos de APIs externas como RAWG, substituindo e expandindo os dados anteriores.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetails {
    pub game_id: String,
    pub steam_app_id: Option<String>,
    pub display_name: Option<String>,

    // Metadados Básicos
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,

    // Categorização
    pub genres: Option<String>,
    pub themes: Option<Vec<String>>,
    pub series: Option<String>,
    pub franchise: Option<Vec<String>>,
    pub game_modes: Option<Vec<String>>,
    pub player_perspectives: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub tags: Option<Vec<GameTag>>, // OLD

    // Descrição
    pub description: GameDescription,

    // Scores e Avaliações
    pub critic_score: Option<i32>,

    // Steam Reviews
    pub steam_review_label: Option<String>,
    pub steam_review_count: Option<i32>,
    pub steam_review_score: Option<f32>,
    pub steam_review_updated_at: Option<String>,

    // Conteúdo Adulto
    pub age_ratings: Option<HashMap<String, String>>,
    pub is_adult: bool,
    pub adult_tags: Option<String>,

    // Links Externos
    pub external_links: Option<HashMap<String, String>>,

    // Tempos de jogo do HowLongToBeat (em horas)
    pub hltb_main_story: Option<f64>,
    pub hltb_main_extra: Option<f64>,
    pub hltb_completionist: Option<f64>,
    pub hltb_coop_time: Option<f64>,

    // Timestamps
    pub updated_at: Option<String>,
}

/// Jogo na lista de desejos (wishlist) com tracking de preços.
///
/// Representa um jogo adicionado à wishlist do usuário,
/// incluindo informações de preço, disponibilidade e vouchers.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WishlistGame {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub store_url: Option<String>,
    pub store_platform: Option<String>,
    pub itad_id: Option<String>,
    pub current_price: Option<f64>,
    pub normal_price: Option<f64>,
    pub lowest_price: Option<f64>,
    pub currency: Option<String>,
    pub on_sale: bool,
    pub voucher: Option<String>,
    pub added_at: Option<String>,
}

/// Assinatura de serviço de jogos rastreada pelo usuário.
///
/// Representa um serviço como Game Pass, Prime Gaming ou EA Play,
/// com controle de ativação e timestamp da última sincronização.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    pub service: String, // EX: "game_pass", "prime_gaming", "ea_play", etc.
    pub enabled: bool,
    // Timestamp ISO 8601 da última sincronização com a API do serviço. `None` se nunca foi sincronizado.
    #[serde(rename = "lastSynced")]
    pub last_synced: Option<String>,
}

/// Dados técnicos do jogo obtidos do PCGamingWiki.
///
/// Tratados como dados estáticos do jogo — não expiram automaticamente.
/// Atualizados apenas por invalidação explícita.
///
/// Tabelas Cargo consultadas: Infobox_game, API, Video, Input, Audio, L10n, Tags.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameExtras {
    pub steam_app_id: String,
    pub pcgw_page_id: Option<String>,
    pub pcgw_page_name: Option<String>,
    pub engine: Option<String>,

    // Plataformas suportadas — string separada por vírgula, ex: "Windows,Linux"
    pub available_on: Option<String>,

    // Graphics APIs (tabela API)
    pub dx_versions: Option<String>, // ex: "11, 12"
    pub vulkan_versions: Option<String>,
    pub opengl_versions: Option<String>,

    // Executáveis por OS — confirma suporte real (tabela API)
    pub win64: Option<String>,
    pub linux64: Option<String>,
    #[serde(rename = "macOsArm")]
    pub macos_arm: Option<String>,
    #[serde(rename = "macOsIntel64")]
    pub macos_intel64: Option<String>,

    // Tecnologias gráficas (tabela Video)
    pub ray_tracing: Option<String>,
    pub upscaling: Option<String>, // ex: "DLSS 3,FSR 3,XeSS"
    pub frame_gen: Option<String>, // ex: "DLSS Frame Generation"

    // Display (tabela Video)
    pub ultrawidescreen: Option<String>,
    pub four_k_support: Option<String>,
    pub hdr: Option<String>,
    pub high_fps: Option<String>, // 120fps+
    pub fov: Option<String>,      // FOV ajustável
    pub borderless_windowed: Option<String>,
    pub color_blind: Option<String>,

    // Controle (tabela Input)
    pub controller_support: Option<String>,
    pub full_controller: Option<String>,
    pub playstation_controllers: Option<String>,
    pub xinput_controllers: Option<String>,

    // Áudio (tabela Audio)
    pub surround_sound: Option<String>,
    pub subtitles: Option<String>,
    pub closed_captions: Option<String>,

    // Presença de dados (tabela Tags)
    pub has_save_data: Option<String>,
    pub has_config_data: Option<String>,

    // Idiomas (tabela L10n) — JSON arrays
    pub languages_interface: Option<Vec<String>>,
    pub languages_audio: Option<Vec<String>>,
    pub languages_subtitles: Option<Vec<String>>,

    pub fetched_at: Option<String>,
}

/// Requisitos de sistema para um único OS/tier.
///
/// Campos `cpu2` e `gpu2` capturam alternativas AMD/Intel quando presentes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemRequirements {
    pub steam_app_id: String,
    pub os_family: String, // OS alvo: "Windows", "Linux", "Mac OS", "DOS", etc.
    pub tier_title: Option<String>, // Rótulo do tier — `None` para padrão (min/rec). Ex: `Some("High")`, `Some("Ultra (1440p)")`.
    pub target: Option<String>,     // Ex: "1080p, DX11".

    // Mínimo
    pub min_os: Option<String>,
    pub min_cpu: Option<String>,
    pub min_cpu2: Option<String>,
    pub min_ram: Option<String>,
    pub min_gpu: Option<String>,
    pub min_gpu2: Option<String>,
    pub min_vram: Option<String>,
    pub min_dx: Option<String>,
    pub min_storage: Option<String>,

    // Recomendado
    pub rec_os: Option<String>,
    pub rec_cpu: Option<String>,
    pub rec_cpu2: Option<String>,
    pub rec_ram: Option<String>,
    pub rec_gpu: Option<String>,
    pub rec_gpu2: Option<String>,
    pub rec_vram: Option<String>,
    pub rec_dx: Option<String>,
    pub rec_storage: Option<String>,
}

/// Caminho de dado do jogo (save ou config) para um OS específico.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDataPath {
    pub steam_app_id: String,
    pub kind: String,                  // Tipo do dado: `"config"` ou `"saves"`.
    pub os: String,                    // OS alvo: `"Windows"`, `"Linux"`, `"OS X"`, etc.
    pub raw_path: String, // Caminho bruto preservando `{{p|variavel}}`. Ex: `"{{p|userprofile\\Documents}}\\Reus\\"`.
    pub expanded_path: Option<String>, // Caminho com variáveis expandidas. `None` até que o frontend expanda `{{p|...}}`.
}

/// Resultado completo do scraping de uma página do PCGamingWiki.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PcgwScrapedData {
    pub system_requirements: Vec<SystemRequirements>,
    pub game_data_paths: Vec<GameDataPath>,
}
