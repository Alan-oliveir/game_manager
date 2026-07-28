//! Serviços para interagir com APIs externas e fornecer funcionalidades ao aplicativo.
//!
//! **Módulos:**
//!
//! - `cache`: Cache de metadados para respostas de APIs externas.
//! - `cf_aggregator`: Agregador de ofertas de jogos de várias fontes.
//! - `images`: Serviço de download e cache de imagens de capas de jogos.
//! - `integration`: Módulos para integração com serviços externos (ITAD, Steam, RAWG, etc.).
//! - `recommendation`: Sistema de recomendação de jogos v4.0 (modular e refatorado).
//! - `subscriptions`: Gerenciamento de assinaturas de serviços de jogos (Amazon Luna, Xbox Game Pass, etc.).
//! - `tags`: Serviço para classificação e gerenciamento de tags de jogos.

pub mod cache;
pub mod images;
pub mod integration;
pub mod recommendation;
pub mod subscriptions;
pub mod tags;
