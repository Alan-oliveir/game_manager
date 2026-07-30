//! Limitadores de taxa compartilhados para APIs externas.
//!
//! Cada instância controla concorrência (via semáforo) e aplica exponential
//! backoff em respostas de throttling (HTTP 429/403), compartilhado entre
//! TODOS os comandos que chamam a respectiva API — não apenas dentro de um
//! único fluxo de import/enrichment.

use crate::constants::{
    RAWG_BACKOFF_BASE_MS, RAWG_BACKOFF_MAX_RETRIES, RAWG_MAX_CONCURRENT_REQUESTS,
    STEAM_BACKOFF_BASE_MS, STEAM_BACKOFF_MAX_RETRIES, STEAM_MAX_CONCURRENT_REQUESTS,
};
use lazy_static::lazy_static;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::warn;

pub struct ApiRateLimiter {
    semaphore: Arc<Semaphore>,
    max_retries: u32,
    backoff_base_ms: u64,
    name: &'static str, // só para os logs identificarem qual limiter disparou
}

impl ApiRateLimiter {
    pub fn new(
        max_concurrent: usize,
        max_retries: u32,
        backoff_base_ms: u64,
        name: &'static str,
    ) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_retries,
            backoff_base_ms,
            name,
        }
    }

    /// Executa `call` respeitando o limite de concorrência desta instância, com exponential
    /// backoff quando o erro indica throttling (429/403/502/503/504).
    /// `call` é uma closure que retorna o future a cada tentativa — precisa ser `Fn` (não `FnOnce`)
    /// porque pode ser chamada mais de uma vez em caso de retry.
    pub async fn run<F, Fut, T>(&self, call: F) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output=Result<T, String>>,
    {
        let _permit = self.semaphore.acquire().await.map_err(|e| e.to_string())?;

        let mut attempt = 0;
        loop {
            match call().await {
                Ok(result) => return Ok(result),
                Err(e) if is_throttling_error(&e) && attempt < self.max_retries => {
                    attempt += 1;
                    let backoff = self.backoff_base_ms * 2u64.pow(attempt - 1);
                    warn!(
                        "{}: throttling detectado, tentativa {}/{}, aguardando {}ms — {}",
                        self.name, attempt, self.max_retries, backoff, e
                    );
                    sleep(Duration::from_millis(backoff)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

lazy_static! {
    pub static ref RAWG_LIMITER: ApiRateLimiter = ApiRateLimiter::new(
        RAWG_MAX_CONCURRENT_REQUESTS as usize,
        RAWG_BACKOFF_MAX_RETRIES,
        RAWG_BACKOFF_BASE_MS,
        "RAWG",
    );
    pub static ref STEAM_LIMITER: ApiRateLimiter = ApiRateLimiter::new(
        STEAM_MAX_CONCURRENT_REQUESTS as usize,
        STEAM_BACKOFF_MAX_RETRIES,
        STEAM_BACKOFF_BASE_MS,
        "Steam",
    );
}

// === HELPERS ===

/// Detecta se o erro indica throttling (não erro genérico de rede/parse).
/// Depende das mensagens de erro incluírem o status HTTP — já é o caso hoje em
/// `rawg.rs` e `steam_api.rs` (formato `"... Error: {status}"`).
fn is_throttling_error(err: &str) -> bool {
    err.contains("429")
        || err.contains("403")
        || err.contains("502")
        || err.contains("503")
        || err.contains("504")
}
