//! Módulo para verificar compatibilidade de jogos com ProtonDB - Exclusivo para Linux
//!
//! ProtonDB é uma base de dados colaborativa que coleta relatos de usuários sobre a compatibilidade
//! de jogos do Windows rodando no Linux via Proton/Wine. Este módulo fornece funções para buscar o
//! resumo de compatibilidade de um jogo específico usando seu `app_id` da Steam.

use crate::constants::{HTTP_CONNECT_TIMEOUT_SECS, USER_AGENT_BROWSER};
use crate::utils::http_client::HTTP_CLIENT;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtonDbSummary {
    pub tier: String, // "platinum" | "gold" | "silver" | "bronze" | "borked" | "pending"
    pub trending_tier: String,
    pub best_reported_tier: String,
    pub confidence: String, // "strong" | ...  - nível de confiança
    pub score: f32,
    pub total: u32, // volume de reports
}

/// Busca o resumo de compatibilidade Linux/Proton para um app_id da Steam.
///
/// Endpoint público, sem autenticação. Retorna `Ok(None)` se o jogo não tiver
/// reports suficientes na ProtonDB (404), não trata isso como erro.
pub async fn get_compatibility_summary(app_id: &str) -> Result<Option<ProtonDbSummary>, String> {
    let url = format!(
        "https://www.protondb.com/api/v1/reports/summaries/{}.json",
        app_id
    );

    let res = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", USER_AGENT_BROWSER)
        .timeout(Duration::from_secs(HTTP_CONNECT_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("Erro requisição ProtonDB: {}", e))?;

    if res.status().as_u16() == 404 {
        return Ok(None); // jogo sem reports suficientes — não é erro
    }

    if !res.status().is_success() {
        return Err(format!("ProtonDB API Error: {}", res.status()));
    }

    let data: ProtonDbSummary = res
        .json()
        .await
        .map_err(|e| format!("Erro ao parsear resposta ProtonDB: {}", e))?;

    Ok(Some(data))
}

pub fn is_running_on_linux() -> bool {
    cfg!(target_os = "linux")
}
