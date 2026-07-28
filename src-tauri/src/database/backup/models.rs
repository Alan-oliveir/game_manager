//! Modelos usados no backup

use crate::models::{
    Game, GameDataPath, GameDetails, GameExtras, SystemRequirements, WishlistGame,
};
use serde::{Deserialize, Serialize};

/// Type alias para dados de backup
pub type BackupDataTuple = (
    Vec<Game>,
    Vec<GameDetails>,
    Vec<WishlistGame>,
    Vec<GameExtras>,
    Vec<SystemRequirements>,
    Vec<GameDataPath>,
    u32,
);

/// Estrutura do arquivo de ‘backup’.
///
/// Contém metadados e todos os dados exportados da aplicação.
#[derive(Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32, // schema == backup
    pub app_version: String,
    pub date: String,
    pub games: Vec<Game>,
    pub game_details: Vec<GameDetails>,
    pub wishlist_game: Vec<WishlistGame>,
    /// Dados técnicos obtidos do PCGamingWiki.
    /// Campo ausente em backups anteriores ao schema v4 — tratado como lista vazia.
    #[serde(default)]
    pub game_extras: Vec<GameExtras>,
    /// Requisitos de sistema por jogo/OS/tier.
    #[serde(default)]
    pub system_requirements: Vec<SystemRequirements>,
    /// Caminhos de save e config por jogo/OS.
    #[serde(default)]
    pub game_data_paths: Vec<GameDataPath>,
}
