//! Modelos usados no backup

use crate::models::{
    GameDataPath, GameDescription, GameDetailsRecord, GameExtras, GameRecord, SystemRequirements,
    WishlistGame,
};
use serde::{Deserialize, Serialize};

/// Type alias para dados de backup
pub type BackupDataTuple = (
    Vec<GameRecord>,
    Vec<GameDetailsRecord>,
    Vec<(String, GameDescription)>,
    Vec<WishlistGame>,
    Vec<GameExtras>,
    Vec<SystemRequirements>,
    Vec<GameDataPath>,
    u32,
);

/// Estrutura do arquivo de ‘backup’.
///
/// Contém metadados e todos os dados exportados da aplicação.
/// Os campos `games`/`game_details`/`game_descriptions` espelham as tabelas
/// `games`/`game_details`/`game_descriptions` 1:1 (ver `GameRecord`,
/// `GameDetailsRecord`, `GameDescription`) — não são os modelos de API
/// (`Game`/`GameDetails`) usados pelo frontend.
#[derive(Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32, // schema == backup
    pub app_version: String,
    pub date: String,
    pub games: Vec<GameRecord>,
    pub game_details: Vec<GameDetailsRecord>,
    /// Descrições por jogo, como pares `(game_id, GameDescription)`.
    /// Campo ausente em backups anteriores a esta refatoração — tratado como lista vazia
    /// (jogos ficam sem descrição ao restaurar backups antigos; re-enrichment resolve).
    #[serde(default)]
    pub game_descriptions: Vec<(String, GameDescription)>,
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
