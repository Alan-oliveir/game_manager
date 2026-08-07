//! Serviços para interagir com APIs externas e fornecer funcionalidades ao aplicativo.
//!
//! **Módulos:**
//!
//! - `cache`: Cache de metadados para respostas de APIs externas.
//! - `images`: Serviço de download e cache de imagens de capas de jogos.
//! - `integration`: Módulos para integração com serviços externos (ITAD, Steam, RAWG, etc.).
//! - `playtime`: Serviço para rastreamento e gerenciamento do tempo de jogo.
//! - `rate_limiter`: Limitador de taxa para evitar exceder os limites das APIs.
//! - `recommendation`: Sistema de recomendação de jogos v4.0 (modular e refatorado).
//! - `subscriptions`: Gerenciamento de assinaturas de serviços de jogos (Amazon Luna, Xbox Game Pass, etc.).
//! - `tags`: Serviço para classificação e gerenciamento de tags de jogos.
//! - `tools`: Gerenciamento de uso e atualização de ferramentas de externas.

pub mod cache;
pub mod images;
pub mod integration;
pub mod playtime;
pub mod rate_limiter;
pub mod recommendation;
pub mod subscriptions;
pub mod tags;
pub mod tools;
pub mod achievements;
