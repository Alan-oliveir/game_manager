use crate::services::integration::hltb::models::{AuthInitResponse, GamesFilter, SearchOptions, SearchPayload};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HltbClient {
    client: reqwest::Client,
    base_url: String,
    endpoint: String,
}

impl HltbClient {
    pub fn new() -> Self {
        // HLTB exige um User-Agent de navegador válido para não bloquear a requisição
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://howlongtobeat.com".to_string(),
            endpoint: "/api/bleed".to_string(), // Endpoint fallback utilizado pelo plugin
        }
    }

    /// Passo 1: Obter as chaves criptográficas dinâmicas
    async fn get_auth_tokens(&self) -> Result<AuthInitResponse, Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let init_url = format!("{}{}/init?t={}", self.base_url, self.endpoint, timestamp);

        let response = self.client.get(&init_url)
            .header("Referer", &self.base_url)
            .send()
            .await?;

        let auth_data: AuthInitResponse = response.json().await?;
        Ok(auth_data)
    }

    /// Passo 2: Executar a busca injetando as chaves
    pub async fn search(&self, game_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let auth = self.get_auth_tokens().await?;

        let search_url = format!("{}{}", self.base_url, self.endpoint);

        // Prepara o payload base
        let terms: Vec<String> = game_name.split_whitespace().map(|s| s.to_string()).collect();
        let payload = SearchPayload {
            search_terms: terms,
            search_options: SearchOptions {
                games: GamesFilter { platform: "".to_string() },
            },
        };

        // Converte para um mapa para podermos injetar a chave dinâmica no corpo do JSON
        let mut json_body: HashMap<String, Value> = serde_json::to_value(payload)?
            .as_object()
            .unwrap()
            .clone()
            .into_iter()
            .collect();

        // O HLTB exige que a chave dinâmica seja enviada também no corpo da requisição
        json_body.insert(auth.hp_key.clone(), json!(auth.hp_val));

        // Envia o POST com os headers exigidos
        let response = self.client.post(&search_url)
            .header("Origin", &self.base_url)
            .header("Referer", &self.base_url)
            .header("x-auth-token", &auth.token)
            .header("x-hp-key", &auth.hp_key)
            .header("x-hp-val", &auth.hp_val)
            .json(&json_body)
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data)
    }
}