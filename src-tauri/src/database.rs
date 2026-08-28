//! Serviços para interagir com o banco de dados do aplicativo.
//!
//! **Módulos:**
//! - `backup`: Funcionalidades de backup e restauração do banco de dados.
//! - `configs`: Gerenciamento genérico de configurações da aplicação.
//! - `core`: Gerenciamento da conexão com o banco de dados SQLite.
//! - `migrations`: Gerenciamento de migrações do banco de dados.

pub mod backup;
pub mod configs;
pub mod core;
pub mod migrations;
pub mod achievements;
pub mod secrets;
pub mod cache;
pub mod game_mods;
pub mod cloud_gaming;
pub mod technical;
pub mod libraries;

// Reexporta o módulo core para fácil acesso
pub use core::*;
pub use secrets::{delete_secret, get_secret, list_supported_keys, set_secret};
