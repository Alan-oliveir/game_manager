//! Módulo de gerenciamento de lista de desejos (wishlist).
//!
//! Adaptado para v2.0 com integração IsThereAnyDeal.
//! Centraliza a importação via arquivos JSON (Steam e ITAD).

use crate::constants::{
    DEFAULT_CURRENCY, STEAM_CDN_AKAMAI_URL, STEAM_HEADER_IMAGE_PATH, STEAM_STORE_URL,
};
use crate::database::AppState;
use crate::errors::AppError;
use crate::integrations::gamebrain::models::{
    GameBrainSearchParams, GameBrainSort, GameBrainSortOrder,
};
use crate::models::WishlistGame;
use crate::providers::discovery::gamebrain as gamebrain_discovery;
use crate::services::wishlist as wishlist_service;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use tauri::{AppHandle, State};

// === STRUCT - Adaptador local para retorno de busca (compatível com frontend) ===

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
}

// === STRUCTS - IMPORTAÇÃO POR ARQUIVOS EXTERNOS ===

#[derive(Deserialize)]
struct SteamExportRoot {
    data: Vec<SteamExportItem>,
}

#[derive(Deserialize)]
struct SteamExportItem {
    title: String,
    gameid: Vec<String>,        // ex: ["steam", "app/7520"]
    price: Option<String>,      // ex: "R$ 73,99"
    added_date: Option<String>, // ex: "26/12/2022"
}

#[derive(Deserialize)]
struct ItadExportRoot {
    data: ItadDataWrapper,
}

#[derive(Deserialize)]
struct ItadDataWrapper {
    data: Vec<ItadGroup>,
}

#[derive(Deserialize)]
struct ItadGroup {
    games: Vec<ItadGame>,
}

#[derive(Deserialize)]
struct ItadGame {
    id: String, // UUID do ITAD
    title: String,
    added: i64, // Timestamp Unix
}

// === LÓGICA DE INSERÇÃO COMPARTILHADA ===

/// Função auxiliar privada que contém o SQL de inserção.
/// Aceita uma conexão (ou transação) já aberta.
fn insert_game_internal(conn: &Connection, game: &WishlistGame) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO wishlist (
            id, name, cover_url, store_url, store,
            current_price, normal_price, lowest_price,
            currency, on_sale, voucher, itad_id, added_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            game.id,
            game.name,
            game.cover_url,
            game.store_url,
            game.store,
            game.current_price,
            game.normal_price,
            game.lowest_price,
            game.currency,
            game.on_sale,
            game.voucher,
            game.itad_id,
            game.added_at
        ],
    )?;
    Ok(())
}

// === IMPORTAÇÃO POR ARQUIVOS EXTERNOS (Steam e ITAD) ===

fn parse_steam_price(price_str: Option<&String>) -> Option<f64> {
    // Remove "R$", espaços e troca vírgula por ponto
    price_str.as_ref().and_then(|s| {
        let clean = s.replace("R$", "").replace(' ', "").replace(',', ".");
        clean.parse::<f64>().ok()
    })
}

fn parse_steam_date(date_str: Option<&String>) -> String {
    if let Some(s) = date_str {
        // Tenta DD/MM/YYYY
        if let Ok(date) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
            if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
                return datetime.and_utc().to_rfc3339();
            }
        }
    }
    chrono::Utc::now().to_rfc3339()
}

/// Tenta processar o conteúdo como exportação da Steam
fn parse_steam_wishlist(content: &str) -> Option<Vec<WishlistGame>> {
    let export: SteamExportRoot = serde_json::from_str(content).ok()?;
    let mut games = Vec::new();

    for item in export.data {
        // Extrai ID da Steam ("app/7520" -> "7520")
        let app_id = item
            .gameid
            .get(1)
            .and_then(|s| s.strip_prefix("app/"))
            .unwrap_or("0")
            .to_string();

        let price = parse_steam_price(item.price.as_ref());

        // Steam Export não tem imagem direta, monta a URL padrão
        let cover_url = format!(
            "{}/{}",
            STEAM_CDN_AKAMAI_URL,
            STEAM_HEADER_IMAGE_PATH.replace("{}", &app_id)
        );

        games.push(WishlistGame {
            id: app_id.clone(),
            name: item.title,
            cover_url: Some(cover_url),
            store_url: Some(format!("{}/app/{}", STEAM_STORE_URL, app_id)),
            store: Some("Steam".to_string()),
            itad_id: None,
            current_price: price,
            normal_price: price,
            lowest_price: price,
            currency: Some(DEFAULT_CURRENCY.to_string()),
            on_sale: false,
            voucher: None,
            added_at: Some(parse_steam_date(item.added_date.as_ref())),
        });
    }
    Some(games)
}

/// Tenta processar o conteúdo como exportação da ITAD (IsThereAnyDeal)
fn parse_itad_wishlist(content: &str) -> Option<Vec<WishlistGame>> {
    let export: ItadExportRoot = serde_json::from_str(content).ok()?;
    let mut games = Vec::new();

    for group in export.data.data {
        for item in group.games {
            // Conversão de data Unix
            let added_at = chrono::DateTime::from_timestamp(item.added, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            games.push(WishlistGame {
                id: item.id, // Usa o UUID da ITAD como ID
                name: item.title,
                cover_url: None, // ITAD export não tem capa, frontend deve mostrar placeholder
                store_url: None,
                store: Some("ITAD".to_string()),
                itad_id: None,
                current_price: None,
                normal_price: None,
                lowest_price: None,
                currency: Some("BRL".to_string()),
                on_sale: false,
                voucher: None,
                added_at: Some(added_at),
            });
        }
    }
    Some(games)
}

/// Importa wishlist a partir de um arquivo JSON local (Steam ou ITAD)
#[tauri::command]
pub async fn import_wishlist(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<usize, AppError> {
    // 1. Lê o arquivo
    let content = fs::read_to_string(&file_path)?;

    // 2. Tenta detectar o formato usando os parsers do steam_store e wishlist logic
    let games = if let Some(steam_games) = parse_steam_wishlist(&content) {
        steam_games
    } else if let Some(itad_games) = parse_itad_wishlist(&content) {
        itad_games
    } else {
        return Err(AppError::ValidationError(
            "Formato de arquivo não reconhecido.".to_string(),
        ));
    };

    let total = games.len();
    if total == 0 {
        return Ok(0);
    }

    // 3. Salva no banco
    {
        let mut conn = state.games_db.lock()?;
        let tx = conn.transaction()?;

        for game in games {
            insert_game_internal(&tx, &game)?;
        }
        tx.commit()?;
    }

    Ok(total)
}

// === GERENCIAMENTO DA WISHLIST (CRUD , Covers e Preços) ===

/// Dispara a busca de capas faltantes na IGDB para jogos na Wishlist. Roda em background e retorna
/// imediatamente, o frontend é notificado via evento `wishlist_updated` quando concluir.
#[tauri::command]
pub async fn fetch_wishlist_covers(app: AppHandle) -> Result<(), AppError> {
    wishlist_service::spawn_fill_missing_covers(app);
    Ok(())
}

/// Busca jogos na IGDB para adicionar à Wishlist.
#[tauri::command]
pub async fn search_wishlist_game(
    app: AppHandle,
    query: String,
) -> Result<Vec<SearchResult>, AppError> {
    let results = wishlist_service::search_games(&app, query)
        .await
        .map_err(AppError::NetworkError)?;

    Ok(results
        .into_iter()
        .map(|g| SearchResult {
            id: g.id.to_string(),
            name: g.name,
            cover_url: g.cover_url,
        })
        .collect())
}

/// Busca jogos por características/descrição via GameBrain para adicionar à Wishlist.
///
/// Diferente de `search_wishlist_game` (que busca pelo nome exato na RAWG),
/// este comando aceita descrições livres como "medieval strategy games" ou
/// "RPG cooperativo parecido com Skyrim" e retorna sugestões semânticas.
///
/// Retorna o mesmo `SearchResult` do comando RAWG para manter
/// compatibilidade com o frontend existente.
#[tauri::command]
pub async fn search_wishlist_game_by_features(
    app: AppHandle,
    query: String,
) -> Result<Vec<SearchResult>, AppError> {
    let results = gamebrain_discovery::search_pc_games_by_features(
        &app,
        &query,
        GameBrainSearchParams {
            sort: Some(GameBrainSort::Rating),
            sort_order: Some(GameBrainSortOrder::Desc),
            limit: Some(20),
            ..Default::default()
        },
    )
        .await
        .map_err(AppError::NetworkError)?;

    Ok(results
        .into_iter()
        .map(|g| SearchResult {
            id: g.id,
            name: g.name,
            cover_url: g.cover_url,
        })
        .collect())
}

/// Adiciona um jogo à lista de desejos.
#[tauri::command]
pub fn add_to_wishlist(
    state: State<AppState>,
    id: String,
    name: String,
    cover_url: Option<String>,
    store_url: Option<String>,
    current_price: Option<f64>,
    itad_id: Option<String>,
) -> Result<String, AppError> {
    let game = WishlistGame {
        id,
        name,
        cover_url,
        store_url,
        store: None,
        itad_id,
        current_price,
        normal_price: current_price,
        lowest_price: current_price,
        currency: Some(DEFAULT_CURRENCY.to_string()),
        on_sale: false,
        voucher: None,
        added_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let conn = state.games_db.lock()?;

    insert_game_internal(&conn, &game)?;

    Ok("Adicionado à Wishlist!".to_string())
}

/// Remove um jogo da lista de desejos.
#[tauri::command]
pub fn remove_from_wishlist(state: State<AppState>, id: String) -> Result<String, AppError> {
    let conn = state.games_db.lock()?;

    conn.execute("DELETE FROM wishlist WHERE id = ?1", params![id])?;

    Ok("Jogo removido da lista de desejos.".to_string())
}

/// Recupera todos os jogos da lista de desejos.
#[tauri::command]
pub fn get_wishlist(state: State<AppState>) -> Result<Vec<WishlistGame>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn
        .prepare("SELECT id, name, cover_url, store_url, store, current_price, normal_price, lowest_price, currency, on_sale, voucher, added_at, itad_id FROM wishlist ORDER BY added_at DESC")?;

    let games = stmt
        .query_map([], |row| {
            Ok(WishlistGame {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_url: row.get(2)?,
                store_url: row.get(3)?,
                store: row.get(4)?,
                current_price: row.get(5)?,
                normal_price: row.get(6)?,
                lowest_price: row.get(7)?,
                currency: row.get(8)?,
                on_sale: row.get(9)?,
                voucher: row.get(10)?,
                added_at: row.get(11)?,
                itad_id: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(games)
}

/// Verifica se um jogo está na lista de desejos.
#[tauri::command]
pub fn check_wishlist_status(state: State<AppState>, id: String) -> Result<bool, AppError> {
    let conn = state.games_db.lock()?;

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(1) FROM wishlist WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(count > 0)
}

/// Atualiza os preços de todos os jogos na Wishlist usando a API da ITAD.
#[tauri::command]
pub async fn refresh_prices(app: AppHandle) -> Result<String, AppError> {
    wishlist_service::refresh_prices(&app).await
}
