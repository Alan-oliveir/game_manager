//! Módulo de comandos relacionados a metadados de jogos
//!
//! Agrupa funcionalidades de enriquecimento, atualização e busca
//! de metadados de APIs externas (RAWG, Steam)
//!
//! Módulos:
//! - `covers`: Comandos para baixar capas de jogos
//! - `enrichment`: Comandos para buscar e atualizar metadados de jogos
//! - `get_metadata`: Comando para preencher campos de metadados faltantes
//! - `pcgamingwiki`: Comandos para obter e atualizar dados de jogos do PC a partir do PC Gaming Wiki
//! - `refresh`: Comandos para atualizar preços e reviews de jogos
//! - `search`: Comandos para buscar metadados externos
//! - `shared`: Funções e estruturas compartilhadas entre os módulos de metadados

pub mod covers;
pub mod enrichment;
pub mod get_metadata;
pub mod pcgamingwiki;
pub mod protondb;
pub mod refresh;
pub mod search;
pub mod shared;
pub mod anticheat;
pub mod winehq;
