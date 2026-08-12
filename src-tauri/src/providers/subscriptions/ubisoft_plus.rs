//! Scraper - Ubisoft+
//!
//! - URL: https://store.ubisoft.com/ofertas/ubisoftplus/games?access=ubisoft&offer=premium
//! - Descrição: Scraper para obter a lista de jogos disponíveis no Ubisoft+ Premium para PC,
//!   incluindo título, edição, gênero, imagem e link da loja.
//! - Método: Requisição POST direta à API pública do Algolia utilizada pelo site da Ubisoft,
//!   com chave de busca read-only exposta no frontend (padrão Algolia, sem autenticação de sessão).

use reqwest::Client;
use serde::{Deserialize, Serialize};

// === Configuração do Algolia ===

const ALGOLIA_BASE_URL: &str = "https://xely3u4lod-dsn.algolia.net/1/indexes/\
     production__br_ubisoft__products__pt_BR__release_date/query";

const ALGOLIA_AGENT: &str = "Algolia for JavaScript (4.13.1); Browser (lite)";
const ALGOLIA_API_KEY: &str = "5638539fd9edb8f2c6b024b49ec375bd";
const ALGOLIA_APP_ID: &str = "XELY3U4LOD";

/// ID da oferta Ubisoft+ Premium (filtra apenas jogos do plano Premium).
const UBISOFT_PLUS_PREMIUM_OFFER_ID: &str = "5f44de7b5cdf9a0c2027ca78";

// === Tipos de dados ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UbisoftGame {
    pub title: String,
    pub short_title: Option<String>,
    pub edition: Option<String>,
    pub genre: Option<String>,
    pub image_url: Option<String>,
    pub store_url: String,
    pub release_date: Option<String>,
    /// Data em que o jogo sai do catálogo Ubisoft+, se informada.
    pub subscription_expiration_date: Option<String>,
    /// `true` quando o jogo é classificado como adulto (+18).
    pub adult: bool,
    /// Plataformas além de PC disponíveis via streaming (ex.: "luna", "xbox").
    pub streaming_platforms: Vec<String>,
}

// === Tipos internos para deserialização da resposta do Algolia ===

#[derive(Debug, Deserialize)]
struct AlgoliaResponse {
    hits: Vec<AlgoliaHit>,
}

#[derive(Debug, Deserialize)]
struct AlgoliaHit {
    title: Option<String>,
    // A API retorna snake_case; sem rename_all, os campos Rust batem direto.
    short_title: Option<String>,
    #[serde(rename = "Edition")]
    edition: Option<String>,
    #[serde(rename = "Genre")]
    genre: Option<String>,
    // BUG CORRIGIDO: rename_all = "camelCase" convertia este campo para "imageLink",
    // que não existe na resposta — por isso a imagem nunca chegava.
    image_link: Option<String>,
    id: Option<String>,
    release_date: Option<String>,
    /// String vazia quando não há expiração definida.
    #[serde(rename = "subscriptionExpirationDate")]
    subscription_expiration_date: Option<String>,
    /// "TRUE" | "FALSE" como string (padrão da API Algolia da Ubisoft).
    adult: Option<String>,
    #[serde(default, rename = "anywherePlatforms")]
    anywhere_platforms: Vec<String>,
}

// === Função principal ===

/// Obtém todos os jogos disponíveis no Ubisoft+ Premium para PC.
///
/// Realiza uma única requisição POST à API Algolia da Ubisoft.
/// Não requer autenticação de sessão; a chave de API é pública (somente leitura).
pub async fn fetch_ubisoft_plus_catalog() -> Result<Vec<UbisoftGame>, String> {
    let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36",
        )
        .build()
        .map_err(|e| e.to_string())?;

    // Payload idêntico ao enviado pelo site da Ubisoft (capturado via DevTools).
    let payload = serde_json::json!({
        "query": "",
        "attributesToRetrieve": [
            "title", "image_link", "short_title", "id", "MasterID",
            "Genre", "release_date", "partOfUbisoftPlus", "anywherePlatforms",
            "subscriptionExpirationDate", "Edition", "adult", "partofSubscriptionOffer"
        ],
        "hitsPerPage": 9999,
        "facetFilters": [
            ["partOfUbisoftPlus:true"],
            [],
            [],
            [format!("partofSubscriptionOfferID:{UBISOFT_PLUS_PREMIUM_OFFER_ID}")]
        ],
        "clickAnalytics": true
    });

    let url = format!(
        "{}?x-algolia-agent={}&x-algolia-api-key={}&x-algolia-application-id={}",
        ALGOLIA_BASE_URL,
        urlencoding::encode(ALGOLIA_AGENT),
        ALGOLIA_API_KEY,
        ALGOLIA_APP_ID,
    );

    let response: AlgoliaResponse = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Origin", "https://store.ubisoft.com")
        .header("Referer", "https://store.ubisoft.com/")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Erro na requisição: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Erro ao parsear resposta: {e}"))?;

    let games = response
        .hits
        .into_iter()
        .filter_map(|hit| {
            // `title` e `id` são obrigatórios para construir a entrada.
            let title = hit.title?;
            let id = hit.id?;

            // URL da loja construída a partir do ID do produto.
            let store_url = format!("https://store.ubisoft.com/br/game/{id}.html");

            Some(UbisoftGame {
                title,
                short_title: hit.short_title,
                edition: hit.edition,
                genre: hit.genre,
                image_url: hit.image_link,
                store_url,
                release_date: hit.release_date,
                subscription_expiration_date: hit
                    .subscription_expiration_date
                    .filter(|s| !s.is_empty()),
                adult: hit.adult.as_deref() == Some("TRUE"),
                streaming_platforms: hit.anywhere_platforms,
            })
        })
        .collect();

    Ok(games)
}
