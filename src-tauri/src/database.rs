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
pub(crate) mod migrations;
pub mod achievements;

// Reexporta o módulo core para fácil acesso
pub use core::*;
