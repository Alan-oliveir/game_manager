use crate::constants::{CACHE_GFN_TTL_DAYS, GFN_GAMES_URL};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Estrutura bruta de uma entrada no JSON estático da GeForce NOW.
/// URL: https://static.nvidiagrid.net/supported-public-game-list/locales/gfnpc-en-US.json
///
/// ATENÇÃO: schema não-oficial, inferido a partir de projetos de terceiros que consomem esse arquivo
/// (ex. SkYNewZ/GeForce-Now-Games, Shushuda/GeForce-NOW-generator).
///
/// - Os nomes podem ser alterados pela Nvidia sem aviso prévio.
#[derive(Debug, Deserialize)]
pub struct GfnGameRaw {
    pub title: String,
    pub store: String, // "Steam", "Epic Games Store", "Origin", "Ubisoft Connect", ...
    #[serde(default)]
    pub steam_url: Option<String>,
    #[serde(default)]
    pub status: Option<String>, // "AVAILABLE", "MAINTENANCE", ...
}

impl GfnGameRaw {
    /// Extrai o Steam App ID a partir da steam_url (ex: ".../app/1091500" -> "1091500").
    /// Único identificador usado pra casar com os jogos do Playlite — títulos de outras lojas (Epic,
    /// Origin...) ficam de fora por ora, até termos um critério de matching confiável
    /// pra elas (não faz sentido arriscar matching fuzzy por nome aqui).
    pub fn steam_app_id(&self) -> Option<String> {
        self.steam_url
            .as_deref()?
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .map(str::to_string)
    }
}

/// Registro normalizado, já pronto para cache local em `gfn_games`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GfnAvailability {
    pub steam_app_id: String,
    pub title: String,
    pub store: String,
    pub status: Option<String>,
}

// === TABELAS ===

pub fn initialize_gfn_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS gfn_games (
            steam_app_id TEXT PRIMARY KEY,
            title        TEXT NOT NULL,
            store        TEXT NOT NULL,
            status       TEXT
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela gfn_games: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS gfn_games_meta (
            id         INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at INTEGER NOT NULL
        )",
        [],
    )
        .map_err(|e| format!("Erro ao criar tabela gfn_games_meta: {e}"))?;

    Ok(())
}

// === REFRESH ===

fn get_last_fetched(conn: &Connection) -> anyhow::Result<Option<DateTime<Utc>>> {
    let ts: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM gfn_games_meta WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    Ok(ts.and_then(|t| DateTime::from_timestamp(t, 0)))
}

/// Garante que o cache local da GFN está dentro do TTL. Mesmo padrão do AWACY (anticheat):
/// o `MutexGuard` nunca atravessa um `.await` — o lock é preso e solto em blocos síncronos.
pub async fn ensure_fresh(
    conn: &Mutex<Connection>,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let needs_refresh = {
        let c = conn
            .lock()
            .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
        match get_last_fetched(&c)? {
            Some(last) => {
                Utc::now().signed_duration_since(last) >= Duration::days(CACHE_GFN_TTL_DAYS)
            }
            None => true,
        }
        // `c` dropado aqui, antes de qualquer await
    };

    if !needs_refresh {
        return Ok(());
    }

    refresh_dataset(conn, client).await
}

async fn refresh_dataset(conn: &Mutex<Connection>, client: &reqwest::Client) -> anyhow::Result<()> {
    let resp = client.get(GFN_GAMES_URL).send().await?; // nenhum lock seguro durante o await

    if !resp.status().is_success() {
        anyhow::bail!("Lista da GFN retornou HTTP {}", resp.status());
    }

    let raw_games: Vec<GfnGameRaw> = resp.json().await?; // ainda sem lock

    let normalized: Vec<GfnAvailability> = raw_games
        .iter()
        .filter_map(|g| {
            g.steam_app_id().map(|steam_app_id| GfnAvailability {
                steam_app_id,
                title: g.title.clone(),
                store: g.store.clone(),
                status: g.status.clone(),
            })
        })
        .collect();

    // Só agora travamos, para a parte 100% síncrona de escrita.
    let mut c = conn
        .lock()
        .map_err(|_| anyhow::anyhow!("mutex do games_db envenenado"))?;
    let tx = c.transaction()?;
    tx.execute("DELETE FROM gfn_games", [])?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO gfn_games (steam_app_id, title, store, status)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for g in &normalized {
            stmt.execute(params![g.steam_app_id, g.title, g.store, g.status])?;
        }
    }
    tx.execute(
        "INSERT INTO gfn_games_meta (id, fetched_at) VALUES (1, ?1)
         ON CONFLICT(id) DO UPDATE SET fetched_at = ?1",
        params![Utc::now().timestamp()],
    )?;
    tx.commit()?;

    tracing::info!(
        "Catálogo GeForce NOW atualizado: {} jogos com Steam App ID salvos em cache",
        normalized.len()
    );

    Ok(())
}

/// Busca disponibilidade na GFN por Steam App ID.
///
/// Diferente do anticheat (que usa fallback fuzzy por nome quando steam_id falha), aqui usamos
/// só o match exato: como a lista da GFN praticamente só resolve títulos vinculados à Steam, um
/// matching fuzzy por nome traria falsos positivos sem ganhar cobertura relevante. Se no futuro
/// quisermos cobrir títulos de outras lojas na lista da GFN, faz mais sentido resolver isso com
/// os campos `store` + IDs específicos de cada loja do que com fuzzy matching por nome.
pub fn find_gfn_availability(
    conn: &Connection,
    steam_app_id: &str,
) -> anyhow::Result<Option<GfnAvailability>> {
    Ok(conn
        .query_row(
            "SELECT steam_app_id, title, store, status FROM gfn_games WHERE steam_app_id = ?1 LIMIT 1",
            [steam_app_id],
            |row| {
                Ok(GfnAvailability {
                    steam_app_id: row.get(0)?,
                    title: row.get(1)?,
                    store: row.get(2)?,
                    status: row.get(3)?,
                })
            },
        )
        .optional()?)
}
