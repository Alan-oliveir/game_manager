//! Xbox Cloud Gaming — reaproveita o catálogo do Game Pass PC já existente (game_pass.rs).
//!
//! Estratégia: em vez de tentar identificar um campo "disponível na nuvem" dentro do payload de
//! detalhes de cada produto (schema não documentado publicamente — arriscado supor um nome de
//! campo sem confirmar), buscamos só a lista de IDs do catálogo "Xbox Cloud Gaming" (mesmo
//! endpoint sigls/v2 usado no Game Pass, outro SIGL) e cruzamos com os store_ids que já vêm de
//! `fetch_game_pass_pc_catalog`. A interseção dos dois conjuntos é o que interessa: jogos
//! disponíveis tanto na nuvem quanto em PC — que é exatamente o filtro pedido.
//!
//! IDs confirmados via inspeção de rede (DevTools em xbox.com/en-us/play, filtro "Cloud games"),
//! endpoint sigls/v3 (não v2 — v3 exige subscriptionContext e platformContext):
//!
//! - `1bf84c2b-0643-4591-893f-d9edb703f692` — CONFIRMADO: metadata retorna `title: "All games"`,
//!   consistente com "todos os jogos jogáveis via cloud" dentro do contexto
//!   `platformContext=Cloud:XGPUWEB`

use crate::providers::subscriptions::GamePassGame;
use chrono::Utc;
use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

const XBOX_CLOUD_SIGL: &str = "1bf84c2b-0643-4591-893f-d9edb703f692";

// Catálogo "Buy and stream select games" — fora de escopo atual (não é catálogo Game Pass),
// mantido aqui apenas para referência caso o escopo mude no futuro.
#[allow(dead_code)]
const XBOX_BUY_AND_STREAM_SIGL: &str = "e78d9a61-5ef4-43af-b400-edba1250b18e";

// Cloud Gaming muda com menos frequência que preço/reviews — 30 dias, mesmo TTL usado no Nexus.
const XBOX_CLOUD_CACHE_TTL_DAYS: i64 = 30;

// === TABELAS ===

pub fn initialize_xbox_cloud_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xbox_cloud_ids (store_id TEXT PRIMARY KEY)",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela xbox_cloud_ids: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS xbox_cloud_meta (
            id         INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela xbox_cloud_meta: {e}"))?;

    Ok(())
}

// === CLIENTE ===

/// Busca só os IDs do catálogo Xbox Cloud Gaming (etapa 1 do endpoint sigls — a mesma "Etapa 1" de
/// `fetch_game_pass_pc_catalog`, sem a etapa 2 de detalhes: não precisa de título/imagem , só do
/// conjunto de IDs para cruzar com o catálogo PC já existente, que já tem os títulos).
///
/// Nota: usa sigls/v3, não v2 — v3 exige `subscriptionContext` e `platformContext`, ausentes na
/// v2 usada em game_pass.rs. `platformContext=Cloud:XGPUWEB` foi o valor observado na chamada real
/// feita pelo client web do Xbox ao navegar no catálogo de Cloud Gaming.
///
/// O primeiro item da resposta é sempre a metadata do catálogo (`siglId`/`title`/`description`,
/// sem campo `id`) — o `filter_map` abaixo já ignora esse item automaticamente, igual acontece em
/// `fetch_game_pass_pc_catalog`.
pub async fn fetch_xbox_cloud_ids(market: &str, language: &str) -> Result<HashSet<String>, String> {
    let client = Client::new();
    let url = format!(
        "https://catalog.gamepass.com/sigls/v3?id={}&market={}&language={}&subscriptionContext=none&platformContext=Cloud%3AXGPUWEB",
        XBOX_CLOUD_SIGL, market, language
    );

    let sigls: Vec<serde_json::Value> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(sigls
        .iter()
        .filter_map(|v| v["id"].as_str().map(str::to_string))
        .collect())
}

// === DATABASE E CACHE (mesmo padrão de nexus_games / nexus_games_cache_meta) ===

pub fn save_xbox_cloud_ids_cache(conn: &Connection, ids: &HashSet<String>) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM xbox_cloud_ids", [])
        .map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("INSERT OR REPLACE INTO xbox_cloud_ids (store_id) VALUES (?1)")
            .map_err(|e| e.to_string())?;
        for id in ids {
            stmt.execute(params![id]).map_err(|e| e.to_string())?;
        }
    }

    let now = Utc::now().timestamp();
    tx.execute(
        "INSERT OR REPLACE INTO xbox_cloud_meta (id, fetched_at) VALUES (1, ?1)",
        params![now],
    )
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())
}

pub fn xbox_cloud_cache_is_stale(conn: &Connection) -> Result<bool, String> {
    let fetched_at: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM xbox_cloud_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(match fetched_at {
        None => true,
        Some(ts) => Utc::now().timestamp() - ts > XBOX_CLOUD_CACHE_TTL_DAYS * 24 * 60 * 60,
    })
}

// === LOOKUP ===

/// Verifica se um store_id (o mesmo campo `store_id` de `GamePassGame`) também está disponível no Xbox Cloud Gaming.
pub fn is_available_on_xbox_cloud(conn: &Connection, store_id: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT 1 FROM xbox_cloud_ids WHERE store_id = ?1",
        [store_id],
        |_| Ok(()),
    )
        .optional()
        .map(|r| r.is_some())
        .map_err(|e| e.to_string())
}

// === ORQUESTRAÇÃO ===

/// Atualiza o cache de IDs do Xbox Cloud Gaming se estiver expirado.
///
/// Diferente do Nexus (que só é pulado por falta de API key), aqui não há key envolvida — mas o
/// catálogo pode não existir mais ou o SIGL pode mudar sem aviso. Falhas aqui não devem derrubar
/// o app: seguimos o mesmo espírito de "melhor esforço" usado no restante do enriquecimento.
pub async fn refresh_xbox_cloud_ids_if_stale(
    conn: &std::sync::Mutex<Connection>,
    market: &str,
    language: &str,
) -> Result<(), String> {
    let needs_refresh = {
        let c = conn.lock().map_err(|_| "Falha DB Games Lock")?;
        xbox_cloud_cache_is_stale(&c)?
    };

    if !needs_refresh {
        return Ok(());
    }

    let ids = fetch_xbox_cloud_ids(market, language).await?;

    let c = conn.lock().map_err(|_| "Falha DB Games Lock")?;
    save_xbox_cloud_ids_cache(&c, &ids)?;

    tracing::info!("Cache Xbox Cloud Gaming atualizado: {} IDs salvos", ids.len());
    Ok(())
}

// === INTERSEÇÃO COM O CATÁLOGO PC GAME PASS ===

/// Filtra, dentre os jogos do catálogo PC Game Pass já obtidos via `fetch_game_pass_pc_catalog`,
/// apenas os que também estão disponíveis no Xbox Cloud Gaming (interseção pelo `store_id`, que é
/// o mesmo identificador nos dois catálogos).
pub fn cloud_available_pc_games<'a>(
    pc_games: &'a [GamePassGame],
    cloud_ids: &HashSet<String>,
) -> Vec<&'a GamePassGame> {
    pc_games
        .iter()
        .filter(|g| cloud_ids.contains(&g.store_id))
        .collect()
}
