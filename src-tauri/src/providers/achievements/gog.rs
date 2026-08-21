//! Provider de conquistas da GOG (via banco local do GOG Galaxy).
//!
//! Schema real (confirmado via `PRAGMA foreign_key_list` e amostras):
//!   - `UserAchievements`: `isUnlocked` (bool), `unlockTime` (TEXT),
//!     chave composta (`gameReleaseKey`, `apikey`).
//!   - `Achievements` / `LocalizedAchievements`: mesma chave composta
//!     (`gameReleaseKey`, `apikey`); `LocalizedAchievements` tem uma
//!     linha por `languageId`, com `isLocalized` indicando se é uma
//!     tradução de verdade ou só o fallback.
//!   - `ProductsToReleaseKeys`: mapeia `releaseKey` (formato
//!     `gog_<gogId>`) para o `gogId` (productId) numérico — é o que
//!     usamos pra casar com a sua biblioteca já raspada via OAuth.
//!
//! O nome do JOGO não vem do banco do Galaxy: resolvemos pelo `gogId`
//! consultando sua própria base (populada pela raspagem da conta GOG).
//! Ajuste `resolve_game_name` pra função real do seu `database` module.

use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::achievements::core::{AchievementDetail, AchievementProvider, DashboardAchievement, Library};
use async_trait::async_trait;
use tauri::{AppHandle, Manager};

pub struct GogProvider;

#[async_trait]
impl AchievementProvider for GogProvider {
    fn library(&self) -> Library {
        Library::Gog
    }

    async fn is_configured(&self, _app: &AppHandle) -> bool {
        gog_db_path().map(|p| p.exists()).unwrap_or(false)
    }

    async fn fetch_recent_achievements(
        &self,
        app: &AppHandle,
        limit: usize,
    ) -> Result<Vec<DashboardAchievement>, AppError> {
        let Some(db_path) = gog_db_path() else {
            return Ok(vec![]);
        };
        if !db_path.exists() {
            return Ok(vec![]);
        }

        let limit_i64 = limit as i64;
        let rows: Vec<UnlockedRow> =
            tokio::task::spawn_blocking(move || -> Result<Vec<UnlockedRow>, String> {
                let conn = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )
                    .map_err(|e| e.to_string())?;

                // `best` escolhe, por conquista, a linha de tradução
                // preferida (isLocalized=1 primeiro; se não houver,
                // cai pro menor languageId como fallback determinístico).
                let mut stmt = conn
                    .prepare(
                        "SELECT
                            ptr.gogId,
                            best.name,
                            ua.unlockTime
                         FROM UserAchievements ua
                         JOIN ProductsToReleaseKeys ptr
                             ON ptr.releaseKey = ua.gameReleaseKey
                         JOIN (
                             SELECT
                                 gameReleaseKey,
                                 apikey,
                                 name,
                                 ROW_NUMBER() OVER (
                                     PARTITION BY gameReleaseKey, apikey
                                     ORDER BY isLocalized DESC, languageId ASC
                                 ) AS rn
                             FROM LocalizedAchievements
                         ) AS best
                             ON best.gameReleaseKey = ua.gameReleaseKey
                            AND best.apikey = ua.apikey
                            AND best.rn = 1
                         WHERE ua.isUnlocked = 1
                         ORDER BY ua.unlockTime DESC
                         LIMIT ?1",
                    )
                    .map_err(|e| e.to_string())?;

                let rows = stmt
                    .query_map([limit_i64], |row| {
                        Ok(UnlockedRow {
                            gog_id: row.get::<_, i64>(0)?.to_string(),
                            achievement_name: row.get(1)?,
                            unlock_time_raw: row.get(2)?,
                        })
                    })
                    .map_err(|e| e.to_string())?;

                rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
            })
                .await
                .map_err(|e| AppError::ExternalApiError(e.to_string()))?
                .unwrap_or_default(); // esquema mudou / tabela ausente: não derruba o dashboard

        let mut achievements = Vec::with_capacity(rows.len());
        for row in rows {
            // TODO: confirmar o formato real de `unlockTime` (ISO8601?
            // epoch em texto?) antes de finalizar esse parse.
            let Some(unlock_time) = parse_unlock_time(&row.unlock_time_raw) else {
                continue;
            };

            let game_name = resolve_game_name(app, &row.gog_id)
                .await
                .unwrap_or_else(|| row.gog_id.clone());

            achievements.push(DashboardAchievement {
                library: Library::Gog,
                game_name,
                achievement_name: row.achievement_name,
                unlock_time,
                game_id: row.gog_id,
            });
        }

        Ok(achievements)
    }

    async fn fetch_all_achievements(
        &self,
        app: &AppHandle,
    ) -> Result<Vec<AchievementDetail>, AppError> {
        let Some(db_path) = gog_db_path() else {
            return Ok(vec![]);
        };
        if !db_path.exists() {
            return Ok(vec![]);
        }

        let rows: Vec<UnlockedDetailRow> =
            tokio::task::spawn_blocking(move || -> Result<Vec<UnlockedDetailRow>, String> {
                let conn = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )
                    .map_err(|e| e.to_string())?;

                let mut stmt = conn
                    .prepare(
                        "SELECT
                            ptr.gogId,
                            best.name,
                            best.description,
                            ach.imageUnlockedUrl,
                            ach.rarity,
                            ach.raritySlug,
                            ua.unlockTime
                         FROM UserAchievements ua
                         JOIN ProductsToReleaseKeys ptr
                             ON ptr.releaseKey = ua.gameReleaseKey
                         JOIN Achievements ach
                             ON ach.gameReleaseKey = ua.gameReleaseKey
                            AND ach.apikey = ua.apikey
                         JOIN (
                             SELECT
                                 gameReleaseKey,
                                 apikey,
                                 name,
                                 description,
                                 ROW_NUMBER() OVER (
                                     PARTITION BY gameReleaseKey, apikey
                                     ORDER BY isLocalized DESC, languageId ASC
                                 ) AS rn
                             FROM LocalizedAchievements
                         ) AS best
                             ON best.gameReleaseKey = ua.gameReleaseKey
                            AND best.apikey = ua.apikey
                            AND best.rn = 1
                         WHERE ua.isUnlocked = 1
                         ORDER BY ua.unlockTime DESC",
                    )
                    .map_err(|e| e.to_string())?;

                let rows = stmt
                    .query_map([], |row| {
                        Ok(UnlockedDetailRow {
                            gog_id: row.get::<_, i64>(0)?.to_string(),
                            achievement_name: row.get(1)?,
                            description: row.get(2)?,
                            icon_url: row.get(3)?,
                            rarity_percent: row.get(4)?,
                            rarity_slug: row.get(5)?,
                            unlock_time_raw: row.get(6)?,
                        })
                    })
                    .map_err(|e| e.to_string())?;

                rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
            })
                .await
                .map_err(|e| AppError::ExternalApiError(e.to_string()))?
                .unwrap_or_default();

        let mut achievements = Vec::with_capacity(rows.len());
        for row in rows {
            let Some(unlock_time) = parse_unlock_time(&row.unlock_time_raw) else {
                continue;
            };

            let game_name = resolve_game_name(app, &row.gog_id)
                .await
                .unwrap_or_else(|| row.gog_id.clone());

            achievements.push(AchievementDetail {
                library: Library::Gog,
                game_id: row.gog_id,
                game_name,
                achievement_name: row.achievement_name,
                description: row.description,
                icon_url: row.icon_url,
                rarity_percent: row.rarity_percent,
                rarity_slug: row.rarity_slug,
                category: None, // GOG não expõe categoria de conquista.
                unlock_time,
            });
        }

        Ok(achievements)
    }
}

struct UnlockedRow {
    gog_id: String,
    achievement_name: String,
    unlock_time_raw: String,
}

struct UnlockedDetailRow {
    gog_id: String,
    achievement_name: String,
    description: Option<String>,
    icon_url: Option<String>,
    rarity_percent: Option<f64>,
    rarity_slug: Option<String>,
    unlock_time_raw: String,
}

/// Formato confirmado: 'YYYY-MM-DD HH:MM:SS', sem timezone no texto —
/// o Galaxy grava esse horário em UTC.
fn parse_unlock_time(raw: &str) -> Option<i64> {
    let naive = chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S").ok()?;
    Some(naive.and_utc().timestamp())
}

/// Resolve o nome do jogo pela sua própria tabela `games`, casando
/// `library = 'GOG'` com `library_game_id` (mesmo valor do `gogId`
/// extraído do banco do Galaxy).
///
/// PLACEHOLDER: `database::get_connection` é um chute — troque pelo
/// helper real que seu `database` module expõe pra abrir uma conexão
/// (ou me manda a assinatura real que eu ajusto).
async fn resolve_game_name(app: &AppHandle, gog_id: &str) -> Option<String> {
    let app = app.clone();
    let gog_id_num: i64 = gog_id.parse().ok()?;

    tokio::task::spawn_blocking(move || -> Option<String> {
        let state: tauri::State<AppState> = app.state();
        let conn = state.games_db.lock().ok()?;
        conn.query_row(
            "SELECT name FROM games WHERE library = 'GOG' AND library_game_id = ?1",
            [gog_id_num],
            |row| row.get(0),
        )
            .ok()
    })
        .await
        .ok()
        .flatten()
}

fn gog_db_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("PROGRAMDATA")
            .map(|pd| std::path::PathBuf::from(pd).join("GOG.com/Galaxy/storage/galaxy-2.0.db"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}
