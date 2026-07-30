use crate::constants::{
    RAWG_BACKOFF_BASE_MS, RAWG_BACKOFF_MAX_RETRIES, RAWG_MAX_CONCURRENT_REQUESTS,
};
use lazy_static::lazy_static;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::warn;

lazy_static! {
    /// Limita chamadas concorrentes à RAWG entre TODOS os comandos que
    /// enriquecem metadados (enrichment, get_metadata, covers, fill_missing),
    /// não apenas dentro de um único import.
    pub static ref RAWG_SEMAPHORE: Arc<Semaphore> =
        Arc::new(Semaphore::new(RAWG_MAX_CONCURRENT_REQUESTS));
}

/// Executa uma chamada à RAWG respeitando o limite de concorrência global e
/// aplicando exponential backoff em caso de HTTP 429.
///
/// `call` deve retornar `Err` contendo a string "429" quando a RAWG throttle
/// a requisição — hoje isso já acontece naturalmente porque `rawg::search_games`
/// e `fetch_game_details` incluem o status na mensagem de erro.
pub async fn with_rawg_limit<F, Fut, T>(call: F) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let _permit = RAWG_SEMAPHORE.acquire().await.map_err(|e| e.to_string())?;

    let mut attempt = 0;
    loop {
        match call().await {
            Ok(result) => return Ok(result),
            Err(e) if e.contains("429") && attempt < RAWG_BACKOFF_MAX_RETRIES => {
                attempt += 1;
                let backoff = RAWG_BACKOFF_BASE_MS * 2u64.pow(attempt - 1);
                warn!(
                    "RAWG 429 recebido, tentativa {}/{}, aguardando {}ms",
                    attempt, RAWG_BACKOFF_MAX_RETRIES, backoff
                );
                sleep(Duration::from_millis(backoff)).await;
            }
            Err(e) => return Err(e),
        }
    }
    // _permit é liberado automaticamente ao sair do escopo (RAII)
}
