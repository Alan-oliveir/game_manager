//! Módulo de integração com a API IGDB (Twitch).
//!
//! **Módulos internos:**
//! - `client`: funções de requisição e autenticação.
//! - `models`: structs de deserialização do JSON retornado pelo IGDB.
//! - `fetch`: funções de busca de dados do IGDB.
//! - `core`: funções principais de processamento e manipulação de dados do IGDB.

pub mod client;
pub mod models;
pub mod fetch;
pub mod core;
