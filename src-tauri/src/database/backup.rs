//! Módulo de ‘backup’ e restauração de dados.
//!
//! Fornece funcionalidades para exportar e importar a base de dados completa
//! em formato JSON, incluindo biblioteca de jogos e lista de desejos.
//!
//! **Nota:**
//! Todas as operações usam transações ACID para garantir consistência dos dados.
//!
//! **Módulos:**
//!
//! - models
//! - export_queries
//! - import_queries
//! - auto

pub mod models;
pub mod export_queries;
pub mod import_queries;
pub mod auto;
