//! Serviço para interagir com a API Gemini da Google para tradução de texto.
//!
//! Utiliza o modelo Gemini 2.5 para traduções de descrições de jogos.
//!
//! **Função Principal:**
//! - `translate_text`: Traduz texto para português brasileiro mantendo termos técnicos de jogos em inglês.

use crate::constants::GEMINI_API_URL;
use crate::utils::http_client::HTTP_CLIENT;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>, // Captura erros da API
}

#[derive(Deserialize, Debug)]
struct GeminiError {
    message: String,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<Content>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>, // Útil para saber se foi bloqueado
}

#[derive(Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SingleTranslationOutput {
    detected_lang: String,
    translated: String,
}

/// Converte o código de idioma (mesmo formato do frontend/i18n) pro nome
/// por extenso, deixando o prompt sem ambiguidade pro modelo.
fn language_display_name(code: &str) -> &str {
    match code {
        "pt-BR" => "Brazilian Portuguese",
        "en" => "English",
        other => other, // fallback: manda o próprio código, o modelo geralmente entende
    }
}

/// Traduz um único texto pro idioma alvo, detectando a língua de origem.
pub async fn translate_single(
    api_key: &str,
    target_lang: &str,
    text: &str,
) -> Result<String, String> {
    debug!("Traduzindo campo único no Gemini -> {}", target_lang);

    let target_name = language_display_name(target_lang);

    let prompt = format!(
        "Detect the source language of the following game description text, then translate it to {target_name}. \
        If it's already in {target_name}, return it unchanged. \
        Keep technical gaming terms in their commonly-used form for {target_name} speakers when there's an established convention \
        (e.g., 'Roguelike', 'Metroidvania', 'Permadeath', 'Loot', 'Crafting'). \
        Preserve tone (exciting, narrative). Return ONLY valid JSON, no preambles, no markdown.\n\nText:\n{text}"
    );

    let body = json!({
        "contents": [{ "parts": [{ "text": prompt }] }],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "OBJECT",
                "properties": {
                    "detectedLang": { "type": "STRING" },
                    "translated": { "type": "STRING" }
                },
                "required": ["detectedLang", "translated"]
            },
            "thinkingConfig": { "thinkingBudget": 0 }
        },
        "safetySettings": [
            { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
            { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
            { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
            { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
        ]
    });

    let url = GEMINI_API_URL.to_string();
    
    let res = HTTP_CLIENT
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            error!("Erro de rede Gemini: {}", e);
            "Erro de rede ao contatar o Gemini".to_string()
        })?;

    let status = res.status();

    if status.as_u16() == 429 {
        let body = res.text().await.unwrap_or_default();
        error!("Rate limit do Gemini atingido: {}", body);
        return Err("Limite de requisições da IA atingido por hoje. Tente novamente mais tarde.".to_string());
    }

    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        error!("Erro API Gemini ({}): {}", status, body);
        return Err(format!("A API retornou erro {}: Verifique sua chave ou cota.", status));
    }

    let data: GeminiResponse = res.json().await.map_err(|e| {
        error!("Erro ao ler JSON Gemini: {}", e);
        format!("Erro ao ler JSON Gemini: {}", e)
    })?;

    if let Some(api_error) = data.error.as_ref() {
        error!("Gemini API Error: {:?}", api_error);
        return Err(format!("Gemini: {}", api_error.message));
    }

    let raw_text = data
        .candidates
        .as_ref()
        .and_then(|c| c.first())
        .and_then(|c| {
            if let Some(reason) = &c.finish_reason {
                if reason != "STOP" {
                    error!("Gemini bloqueou conteúdo. Motivo: {}", reason);
                    return None;
                }
            }
            c.content.as_ref()
        })
        .and_then(|c| c.parts.first())
        .map(|p| p.text.clone())
        .ok_or_else(|| {
            error!("Resposta Gemini inesperada ou bloqueada: {:?}", data);
            "A IA não retornou nenhuma tradução válida.".to_string()
        })?;

    let cleaned = raw_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let output: SingleTranslationOutput = serde_json::from_str(cleaned).map_err(|e| {
        error!("JSON inválido do Gemini: {} | raw: {}", e, cleaned);
        format!("JSON inválido do Gemini: {}", e)
    })?;

    info!("Tradução concluída (detectado: {})", output.detected_lang);
    Ok(output.translated)
}

/// Traduz uma query de busca para inglês, se necessário.
///
/// Usado para normalizar buscas antes de enviar para APIs que só
/// funcionam bem com termos em inglês (ex: GameBrain).
///
/// Se o texto já estiver em inglês, o modelo retorna sem alterações.
pub async fn translate_query_to_english(api_key: &str, text: &str) -> Result<String, String> {
    debug!("Traduzindo query para inglês via Gemini...");

    let url = format!("{}?key={}", GEMINI_API_URL, api_key);

    let prompt = format!(
        "If the following search query is not in English, translate it to English. \
        If it is already in English, return it exactly as-is. \
        Output ONLY the translated query, no explanations or punctuation:\n\n{}",
        text
    );

    let body = json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }]
    });

    let res = HTTP_CLIENT
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Erro de rede Gemini: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let error_body = res.text().await.unwrap_or_default();
        error!(
            "Erro API Gemini ao traduzir query ({}): {}",
            status, error_body
        );
        // Falha silenciosa: usa a query original para não bloquear a busca
        return Ok(text.to_string());
    }

    let data: GeminiResponse = res
        .json()
        .await
        .map_err(|e| format!("Erro ao ler JSON Gemini: {}", e))?;

    let translated = data
        .candidates
        .as_ref()
        .and_then(|c| c.first())
        .and_then(|c| c.content.as_ref())
        .and_then(|c| c.parts.first())
        .map(|p| p.text.trim().to_string());

    match translated {
        Some(t) if !t.is_empty() => {
            info!("Query traduzida: '{}' -> '{}'", text, t);
            Ok(t)
        }
        // Falha silenciosa: se Gemini não retornar nada, usa a query original
        _ => {
            error!("Gemini não retornou tradução para a query, usando original.");
            Ok(text.to_string())
        }
    }
}
