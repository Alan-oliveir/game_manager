use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CheapSharkDeal {
    #[serde(rename = "storeID")]
    pub store_id: String,
    #[serde(rename = "dealID")]
    pub deal_id: String,
    #[serde(rename = "salePrice")]
    pub price: String,
    #[serde(rename = "normalPrice")]
    pub retail_price: String,
    #[serde(rename = "savings")]
    pub savings: String,
}

#[derive(Debug)]
pub struct DealResult {
    pub price: f64,
    pub url: String,
    pub on_sale: bool,
}

pub async fn find_best_price(game_name: &str) -> Result<Option<DealResult>, String> {
    // Busca por "deals" com o nome do jogo, exact=0 permite busca aproximada.
    let url = format!(
        "https://www.cheapshark.com/api/1.0/deals?title={}&limit=1&exact=0",
        urlencoding::encode(game_name)
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Erro CheapShark: {}", res.status()));
    }

    // Tenta decodificar. Se falhar, retorna o erro original para debug.
    let deals: Vec<CheapSharkDeal> = res.json().await.map_err(|e| format!("Erro ao decodificar JSON: {}", e))?;

    if let Some(deal) = deals.first() {
        let current_price = deal.price.parse::<f64>().unwrap_or(0.0);
        let retail_price = deal.retail_price.parse::<f64>().unwrap_or(0.0);

        // Link direto para o deal na CheapShark
        let deal_url = format!("https://www.cheapshark.com/redirect?dealID={}", deal.deal_id);

        Ok(Some(DealResult {
            price: current_price,
            url: deal_url,
            // Consideramos em oferta se o desconto for maior que 0.1% ou se o preço for menor
            on_sale: current_price < retail_price,
        }))
    } else {
        Ok(None) // Nenhum preço encontrado
    }
}