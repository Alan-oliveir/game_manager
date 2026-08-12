//! Módulo com funções referentes a integraão com PCGamingWiki
//!
//! **Módulos:**
//!
//! - `client` - Conexão com a API do PCGamingWiki para busca de informações sobre jogos.
//! - `db` - Gerenciamento dos dados do PCGamingWiki usando SQLite.
//! - `fetch` - Busca e tratamento de dados do PCGamingWiki, incluindo formatação.
//! - `parsers` - Auxilia com parsing e dados relacionados ao PCGamingWiki, como plataformas, requisitos, etc.
//! - `subscriptions` - Obtem informações de requisitos do sistemas e paths que não são obtidos direto pela API do PCGamingWiki.

pub mod client;
pub mod db;
pub mod fetch;
pub mod parsers;
pub mod scraper;
