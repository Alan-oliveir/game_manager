//! Integração com a SteamGridDB — fonte primária de capas (covers).
//!
//! - `client`: chamadas HTTP autenticadas via Bearer token, com rate limiting.
//! - `models`: structs de deserialização das respostas da API v2.
//! - `db`: persistência em `game_images` e `steamgriddb_cache_meta`.
//! - `core`: orquestra a resolução da capa (AppID direto ou autocomplete + match).

pub mod client;
pub mod core;
pub mod db;
pub mod models;

pub use client::SteamGridDbClient;
pub use core::{resolve_cover, CoverResult};
