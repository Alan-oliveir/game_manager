//! Persistência de conquistas (Steam/Xbox/Epic/GOG) no games.db.
//!
//! Duas tabelas:
//! - `achievements`: conquistas desbloqueadas, já resolvidas (nome, ícone etc).
//! - `achievement_sync_state`: controla, por (plataforma, jogo), quando foi a última sincronização
//! e se o jogo não tem conquistas públicas (has_achievements = 0) e assim pula na API
//! conquistas para esse jogo, já que isso não muda (é o schema do jogo na Steam/Xbox).

use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::achievements::core::{DashboardAchievement, Library};
use rusqlite::params;
use tauri::{AppHandle, Manager};

pub struct AchievementRecord {
    pub library: Library,
    pub game_id: String,
    pub game_name: String,
    pub achievement_key: String,
    pub achievement_name: String,
    pub achievement_description: Option<String>,
    pub unlocked_at: i64,
    pub icon_url: Option<String>,
}

fn library_str(p: Library) -> &'static str {
    match p {
        Library::Steam => "steam",
        Library::Epic => "epic",
        Library::Gog => "gog",
        Library::Xbox => "xbox",
    }
}

fn parse_library(raw: &str) -> Library {
    match raw {
        "epic" => Library::Epic,
        "gog" => Library::Gog,
        "xbox" => Library::Xbox,
        _ => Library::Steam,
    }
}

// === CONQUISTAS ===

/// Insere/atualiza um lote de conquistas. Idempotente — chave é (library, game_id, achievement_key),
/// então rodar o sync de novo não duplica nada, só atualiza se algo mudou.
pub fn upsert_achievements(
    app: &AppHandle,
    records: &[AchievementRecord],
) -> Result<usize, AppError> {
    if records.is_empty() {
        return Ok(0);
    }

    let state: tauri::State<AppState> = app.state();
    let mut conn = state
        .games_db
        .lock()
        .map_err(|_| AppError::DatabaseError("Falha ao bloquear mutex do games_db".into()))?;

    let tx = conn
        .transaction()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let mut affected = 0;

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO achievements
                    (library, game_id, game_name, achievement_key, achievement_name, achievement_description, unlocked_at, icon_url)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    ON CONFLICT(library, game_id, achievement_key) DO UPDATE SET
                        unlocked_at = excluded.unlocked_at,
                        achievement_name = excluded.achievement_name,
                        achievement_description = excluded.achievement_description,
                        icon_url = excluded.icon_url",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for r in records {
            affected += stmt
                .execute(params![
                    library_str(r.library),
                    r.game_id,
                    r.game_name,
                    r.achievement_key,
                    r.achievement_name,
                    r.achievement_description,
                    r.unlocked_at,
                    r.icon_url,
                ])
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
    }

    tx.commit()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(affected)
}

/// Lê as `limit` conquistas mais recentes entre todas as plataformas.
/// É só um SELECT local — o dashboard nunca espera rede pra isso.
pub fn get_dashboard_achievements(
    app: &AppHandle,
    limit: usize,
) -> Result<Vec<DashboardAchievement>, AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = state
        .games_db
        .lock()
        .map_err(|_| AppError::DatabaseError("Falha ao bloquear mutex do games_db".into()))?;

    let mut stmt = conn
        .prepare(
            "SELECT library, game_name, achievement_name, unlocked_at, game_id
             FROM achievements
             ORDER BY unlocked_at DESC
             LIMIT ?1",
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let library_raw: String = row.get(0)?;
            Ok(DashboardAchievement {
                library: parse_library(&library_raw),
                game_name: row.get(1)?,
                achievement_name: row.get(2)?,
                unlock_time: row.get(3)?,
                game_id: row.get(4)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| AppError::DatabaseError(e.to_string()))?);
    }
    Ok(out)
}

// === CONTROLE DE SYNC POR JOGO ===

struct SyncState {
    last_synced_at: i64,
    has_achievements: bool,
}

fn get_sync_state(
    app: &AppHandle,
    library: Library,
    game_id: &str,
) -> Result<Option<SyncState>, AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = state
        .games_db
        .lock()
        .map_err(|_| AppError::DatabaseError("Falha ao bloquear mutex do games_db".into()))?;

    let result = conn.query_row(
        "SELECT last_synced_at, has_achievements FROM achievement_sync_state WHERE library = ?1 AND game_id = ?2",
        params![library_str(library), game_id],
        |row| {
            Ok(SyncState {
                last_synced_at: row.get(0)?,
                has_achievements: row.get::<_, i64>(1)? != 0,
            })
        },
    );

    match result {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

/// Marca (library, game_id) como sincronizado agora. `has_achievements = false` é permanente:
/// significa "confirmamos, via 400 da API, que esse jogo não tem stats de conquista" — não muda.
pub fn mark_synced(
    app: &AppHandle,
    library: Library,
    game_id: &str,
    has_achievements: bool,
) -> Result<(), AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = state
        .games_db
        .lock()
        .map_err(|_| AppError::DatabaseError("Falha ao bloquear mutex do games_db".into()))?;

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO achievement_sync_state (library, game_id, last_synced_at, has_achievements)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(library, game_id) DO UPDATE SET
            last_synced_at = excluded.last_synced_at,
            has_achievements = excluded.has_achievements",
        params![
            library_str(library),
            game_id,
            now,
            has_achievements as i64
        ],
    )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// `true` se este jogo pode ser pulado nesta rodada de sync:
/// - já confirmou antes que ele não tem conquistas (permanente), OU
/// - já foi sincronizado há menos de `ttl_secs`.
///
/// Em caso de erro de leitura no banco, NÃO pula (mais seguro tentar de novo do que perder uma conquista).
pub fn should_skip(app: &AppHandle, library: Library, game_id: &str, ttl_secs: i64) -> bool {
    match get_sync_state(app, library, game_id) {
        Ok(Some(s)) => {
            if !s.has_achievements {
                return true;
            }
            let now = chrono::Utc::now().timestamp();
            (now - s.last_synced_at) < ttl_secs
        }
        _ => false,
    }
}

/// Retorna (library_game_id, name) de todos os jogos já importados na biblioteca para uma
/// plataforma. Reaproveita o que a importação já salvou em `games`, evitando uma nova chamada à API
/// externa (ex.: GetOwnedGames da Steam) só pra montar a lista de jogos a sincronizar.
pub fn get_owned_games_by_library(
    app: &AppHandle,
    library: &str,
) -> Result<Vec<(String, String)>, AppError> {
    let state: tauri::State<AppState> = app.state();
    let conn = state
        .games_db
        .lock()
        .map_err(|_| AppError::DatabaseError("Falha ao bloquear mutex do games_db".into()))?;

    let mut stmt = conn
        .prepare("SELECT library_game_id, name FROM games WHERE library = ?1")
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![library], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| AppError::DatabaseError(e.to_string()))?);
    }
    Ok(out)
}
