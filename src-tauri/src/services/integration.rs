//! Módulos para integração com serviços externos.
//!
//! Fornece funcionalidades para interagir com APIs e serviços de terceiros.
//! Cada módulo encapsula a lógica necessária para comunicação com um serviço específico,
//! facilitando a manutenção e expansão do código.
//!
//! **Módulos:**
//!
//! - `gamebrain`: Integração com a API GameBrain.
//! - `gamerpower`: Integração com a API GamerPower para busca de jogos grátis.
//! - `gemini`: Integração com a API Gemini para funcionalidade de tradução com IA.
//! - `itad`: Integração com a API IsThereAnyDeal para 'tracking' de preços e ofertas.
//! - `pcgamingwiki`: Integração com a API PCGamingWiki para busca de informações sobre jogos.
//! - `rawg`: Integração com a API RAWG para busca de jogos e tendências.
//! - `steam`: Integração com a API Steam para obter detalhes e conquistas dos jogos.
//! - `protondb`: Integração com a API ProtonDB para verificar compatibilidade de jogos no Linux.

pub mod gamebrain;
pub mod gamerpower;
pub mod gemini;
pub mod itad;
pub mod pcgamingwiki;
pub mod rawg;
pub mod steam_api;
pub mod protondb;
