//! Comandos Tauri expostos ao frontend.
//!
//! Cada função aqui é invocável via `invoke()` no JavaScript/TypeScript.
//! Todos os comandos lidam com erros e os convertem para 'strings' amigáveis.
//!
//! **Módulos:**
//! - `achievements`: Comandos para buscar conquistas recentes de jogos via Steam API.
//! - `ai_translation`: Comandos para tradução de descrições usando IA.
//! - `caches`: Comandos para gerenciar o cache de metadados.
//! - `games`: Comandos CRUD para a biblioteca de jogos.
//! - `launcher`: Comandos para lançar jogos e interagir com plataformas de jogos.
//! - `metadata`: Comandos para enriquecimento, atualização e busca de metadados via RAWG/Steam API.
//! - `libraries`: Comandos para gerenciar plataformas de jogos.
//! - `recommendation`: Comandos para gerenciar recomendações de jogos.
//! - `settings`: Comandos para gerenciar configurações e segredos do usuário.
//! - `subscriptions`: Comandos para gerenciar assinaturas de serviços de jogos e buscar catálogos.
//! - `system`: Comandos para interagir com o sistema (abrir pastas, arquivos, etc).
//! - `version`: Comandos para gerenciar informações de versão da aplicação.
//! - `wishlist`: Comandos para gerenciar a lista de desejos com 'tracking' de preços.

pub mod achievements;
pub mod translation;
pub mod cache;
pub mod games;
pub mod launcher;
pub mod metadata;
pub mod libraries;
pub mod recommendation;
pub mod settings;
pub mod subscriptions;
pub mod system;
pub mod version;
pub mod wishlist;
pub mod debug;
pub mod cloud_gaming;
pub mod backup;
pub mod database;
