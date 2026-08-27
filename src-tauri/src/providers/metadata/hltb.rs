use crate::database::cache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HltbEntry {
    pub game_id: i64,
    pub game_name: String,
    pub game_alias: Option<String>,
    pub game_type: Option<String>,
    pub game_image_url: Option<String>,
    pub game_web_link: String,
    pub review_score: Option<i64>,
    pub profile_dev: Option<String>,
    pub release_world: Option<i64>,

    // Tempos normalizados em horas
    pub main_story: Option<f64>,
    pub main_extra: Option<f64>,
    pub completionist: Option<f64>,
    pub all_styles: Option<f64>,
    pub coop_time: Option<f64>,
    pub mp_time: Option<f64>,

    // Grau de Similaridade com o termo buscado (0.0 a 1.0)
    pub similarity: f64,
}

#[derive(Debug)]
pub(crate) struct AuthStruct {
    pub token: String,
    pub hp_key: String,
    pub hp_val: String,
}

pub struct HltbClient {
    client: reqwest::Client,
    base_url: String,
}

impl HltbClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("*/*"),
                );
                headers
            })
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://howlongtobeat.com".to_string(),
        }
    }

    /// Gera os candidatos de endpoints mesclando descobertas dinâmicas e fallbacks históricos
    async fn build_endpoint_candidates(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut candidates = Vec::new();

        if let Ok(html) = self.client.get(&self.base_url).send().await?.text().await {
            let script_regex = Regex::new(r#"src=["']([^"']+\.js)["']"#)?;
            let script_urls: Vec<String> = script_regex
                .captures_iter(&html)
                .map(|cap| cap[1].to_string())
                .collect();

            let fetch_regex = Regex::new(
                r#"(?i)fetch\s*\(\s*["']/api/([a-zA-Z0-9_/-]+)[^"']*["']\s*,\s*\{[^}]*method:\s*["']POST["']"#,
            )?;

            for url in script_urls {
                let full_url = if url.starts_with("http") {
                    url.clone()
                } else if url.starts_with('/') {
                    format!("{}{}", self.base_url, url)
                } else {
                    format!("{}/{}", self.base_url, url)
                };

                if let Ok(script_content) = self.client.get(&full_url).send().await?.text().await {
                    if let Some(cap) = fetch_regex.captures(&script_content) {
                        let path_suffix = cap[1].to_string();
                        let base_path = path_suffix.split('/').next().unwrap_or("").to_string();
                        candidates.push(format!("/api/{}", base_path));
                        tracing::debug!("HLTB: endpoint descoberto dinamicamente: /api/{}", base_path);
                        break;
                    }
                }
            }
        }

        // Fallbacks históricos
        let fallbacks = vec!["/api/bleed", "/api/finder", "/api/search", "/api/s"];
        for f in fallbacks {
            candidates.push(f.to_string());
        }

        // Remove duplicatas mantendo a ordem (preferência para descoberta dinâmica)
        let mut unique_candidates = Vec::new();
        let mut seen = HashSet::new();
        for c in candidates {
            if seen.insert(c.clone()) {
                unique_candidates.push(c);
            }
        }

        Ok(unique_candidates)
    }

    /// Extrai dinamicamente as chaves (auth_token, key, val) do JSON
    fn extract_auth_from_json(&self, json_data: &Value) -> Option<AuthStruct> {
        let map = json_data.as_object()?;

        let token = map
            .get("token")
            .and_then(|v| v.as_str())
            .or_else(|| {
                map.get("data")
                    .and_then(|d| d.get("token"))
                    .and_then(|v| v.as_str())
            })
            .or_else(|| map.get("auth_token").and_then(|v| v.as_str()))
            .or_else(|| map.get("authToken").and_then(|v| v.as_str()))?;

        let mut auth_key = String::new();
        let mut auth_val = String::new();

        for (k, v) in map {
            let lower = k.to_lowercase();
            if lower.contains("key") {
                if let Some(s) = v.as_str() {
                    auth_key = s.to_string();
                }
            } else if lower.contains("val") {
                if let Some(s) = v.as_str() {
                    auth_val = s.to_string();
                }
            }
        }

        if !token.is_empty() && !auth_key.is_empty() && !auth_val.is_empty() {
            Some(AuthStruct {
                token: token.to_string(),
                hp_key: auth_key,
                hp_val: auth_val,
            })
        } else {
            None
        }
    }

    /// Tenta extrair a autenticação de uma lista de endpoints até obter sucesso
    async fn fetch_search_token(
        &self,
        endpoints: Vec<String>,
    ) -> Result<(String, AuthStruct), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        for endpoint in endpoints {
            let init_url = format!("{}{}/init?t={}", self.base_url, endpoint, timestamp);

            match self
                .client
                .get(&init_url)
                .header("referer", &self.base_url)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        if let Ok(text) = response.text().await {
                            if let Ok(json_data) = serde_json::from_str::<Value>(&text) {
                                if let Some(auth_struct) = self.extract_auth_from_json(&json_data) {
                                    tracing::debug!("HLTB: endpoint '{}' OK, token obtido", endpoint);
                                    return Ok((endpoint, auth_struct));
                                }
                                tracing::debug!(
                                    "HLTB: endpoint '{}' respondeu 200 mas JSON não tinha os campos esperados: {}",
                                    endpoint, text
                                );
                            } else {
                                tracing::debug!(
                                    "HLTB: endpoint '{}' respondeu 200 mas corpo não é JSON válido",
                                    endpoint
                                );
                            }
                        }
                    } else {
                        tracing::debug!("HLTB: endpoint '{}' falhou com status {}", endpoint, status);
                    }
                }
                Err(e) => {
                    tracing::debug!("HLTB: endpoint '{}' erro de rede: {}", endpoint, e);
                }
            }
        }
        Err("Nenhum endpoint válido retornou as chaves de autenticação".into())
    }

    /// Utilitário interno para extrair e normalizar os tempos de jogo
    fn normalize_time(&self, obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<f64> {
        for key in keys {
            if let Some(val) = obj.get(*key) {
                if let Some(num) = val.as_f64() {
                    if num > 0.0 {
                        // Trata payloads antigos que expunham em segundos
                        let hours = if num > 500.0 { num / 3600.0 } else { num };
                        return Some((hours * 100.0).round() / 100.0);
                    }
                }
            }
        }
        None
    }

    /// Implementação da Distância de Levenshtein para similaridade
    fn similar(&self, query: &str, target: &str, query_numbers: &[&str]) -> f64 {
        if query.is_empty() || target.is_empty() {
            return 0.0;
        }

        let s1: Vec<char> = query.to_lowercase().chars().collect();
        let s2: Vec<char> = target.to_lowercase().chars().collect();
        let m = s1.len();
        let n = s2.len();

        let mut matrix = vec![vec![0; n + 1]; m + 1];
        for i in 0..=m {
            matrix[i][0] = i;
        }
        for j in 0..=n {
            matrix[0][j] = j;
        }

        for j in 1..=n {
            for i in 1..=m {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        let distance = matrix[m][n];
        let max_len = m.max(n) as f64;
        let mut similarity = 1.0 - (distance as f64 / max_len);

        // Penalidade se a busca continha números e o resultado não
        if !query_numbers.is_empty() {
            let re = Regex::new(r"[^\w\s]").unwrap();
            let cleaned_target = re.replace_all(target, "");
            let target_words: Vec<&str> = cleaned_target.split_whitespace().collect();

            let mut number_found = false;
            for word in target_words {
                if word.chars().all(char::is_numeric) && query_numbers.contains(&word) {
                    number_found = true;
                    break;
                }
            }
            if !number_found {
                similarity -= 0.1;
            }
        }

        similarity
    }

    /// Realiza a busca oficial no HowLongToBeat
    pub async fn search(
        &self,
        game_name: &str,
    ) -> Result<Vec<HltbEntry>, Box<dyn std::error::Error>> {
        let candidates = self.build_endpoint_candidates().await?;
        let (endpoint, auth) = self.fetch_search_token(candidates).await?;

        let search_url = format!("{}{}", self.base_url, endpoint);
        let terms: Vec<String> = game_name
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut json_body = json!({
            "searchType": "games",
            "searchTerms": terms,
            "searchPage": 1,
            "size": 20,
            "searchOptions": {
                "games": {
                    "userId": 0,
                    "platform": "",
                    "sortCategory": "popular",
                    "rangeCategory": "main",
                    "rangeTime": { "min": 0, "max": 0 },
                    "gameplay": { "perspective": "", "flow": "", "genre": "", "difficulty": "" },
                    "rangeYear": { "min": "", "max": "" },
                    "modifier": ""
                },
                "users": { "sortCategory": "postcount" },
                "lists": { "sortCategory": "follows" },
                "filter": "",
                "sort": 0,
                "randomizer": 0
            },
            "useCache": true
        });

        json_body
            .as_object_mut()
            .unwrap()
            .insert(auth.hp_key.clone(), json!(auth.hp_val));

        let response = self
            .client
            .post(&search_url)
            .header("Origin", &self.base_url)
            .header("Referer", &self.base_url)
            .header("x-auth-token", &auth.token)
            .header("x-hp-key", &auth.hp_key)
            .header("x-hp-val", &auth.hp_val)
            .header("content-type", "application/json")
            .json(&json_body)
            .send()
            .await?;

        let text = response.text().await?;
        let json_response: Value = serde_json::from_str(&text)?;

        // Extrai a array correta de resultados dependendo do payload
        let items_array = json_response
            .get("data")
            .or_else(|| json_response.get("results"))
            .or_else(|| json_response.get("items"))
            .and_then(|v| v.as_array());

        let total_from_api = items_array.map(|v| v.len()).unwrap_or(0);
        tracing::debug!("HLTB: API devolveu {} item(ns) para '{}'", total_from_api, game_name);

        let mut results = Vec::new();
        let query_numbers: Vec<&str> = game_name
            .split_whitespace()
            .filter(|w| w.chars().all(char::is_numeric))
            .collect();

        if let Some(items) = items_array {
            for item in items {
                if let Some(obj) = item.as_object() {
                    let id = obj
                        .get("game_id")
                        .or_else(|| obj.get("gameId"))
                        .or_else(|| obj.get("id"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(-1);
                    let name = obj
                        .get("game_name")
                        .or_else(|| obj.get("gameName"))
                        .or_else(|| obj.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let alias = obj
                        .get("game_alias")
                        .or_else(|| obj.get("gameAlias"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let sim_name = self.similar(game_name, &name, &query_numbers);
                    let sim_alias = if let Some(ref a) = alias {
                        self.similar(game_name, a, &query_numbers)
                    } else {
                        0.0
                    };
                    let similarity = sim_name.max(sim_alias);

                    // Só retorna resultados com no mínimo 40% de similaridade (como no Ruby/Python)
                    if similarity >= 0.4 {
                        let entry = HltbEntry {
                            game_id: id,
                            game_name: name,
                            game_alias: alias,
                            game_type: obj
                                .get("game_type")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            game_image_url: obj
                                .get("game_image")
                                .and_then(|v| v.as_str())
                                .map(|s| format!("https://howlongtobeat.com/games/{}", s)),
                            game_web_link: format!("https://howlongtobeat.com/game/{}", id),
                            review_score: obj.get("review_score").and_then(|v| v.as_i64()),
                            profile_dev: obj
                                .get("profile_dev")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            release_world: obj.get("release_world").and_then(|v| v.as_i64()),

                            main_story: self
                                .normalize_time(obj, &["comp_main", "compMain", "main_story"]),
                            main_extra: self
                                .normalize_time(obj, &["comp_plus", "compPlus", "main_extra"]),
                            completionist: self
                                .normalize_time(obj, &["comp_100", "comp100", "completionist"]),
                            all_styles: self
                                .normalize_time(obj, &["comp_all", "compAll", "all_styles"]),
                            coop_time: self
                                .normalize_time(obj, &["invested_co", "investedCo", "coop_time"]),
                            mp_time: self
                                .normalize_time(obj, &["invested_mp", "investedMp", "mp_time"]),

                            similarity,
                        };
                        results.push(entry);
                    }
                }
            }
        }

        tracing::debug!(
            "HLTB: {}/{} item(ns) passaram no filtro de similaridade (>= 0.4) para '{}'",
            results.len(), total_from_api, game_name
        );

        // Ordena do mais parecido para o menos parecido
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    pub async fn search_hltb_with_cache(
        game_name: &str,
        cache_conn: &rusqlite::Connection,
    ) -> Result<Vec<HltbEntry>, String> {
        let cache_key = format!("search_hltb_{}", game_name.to_lowercase());

        // 1. Tenta ler do cache primeiro
        if let Some(cached) = cache::get_cached_api_data(cache_conn, "hltb", &cache_key) {
            if let Ok(entries) = serde_json::from_str::<Vec<HltbEntry>>(&cached) {
                tracing::info!("HLTB (Cache hit) para '{}'", game_name);
                return Ok(entries);
            }
        }

        // 2. Se não estiver no cache, faz o scraping real na web
        tracing::info!("HLTB (Cache miss - Buscando online) para '{}'", game_name);
        let client = HltbClient::new();
        let results = client.search(game_name).await.map_err(|e| e.to_string())?;

        // 3. Salva o resultado (mesmo que seja um array vazio) no cache
        if let Ok(json) = serde_json::to_string(&results) {
            let _ = cache::save_cached_api_data(cache_conn, "hltb", &cache_key, &json);
        }

        Ok(results)
    }
}
