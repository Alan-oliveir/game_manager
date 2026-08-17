//! Importação de jogos da itch.io (via app itch / butler.db).
//!
//! Lê diretamente o banco SQLite `butler.db` que o app itch mantém localmente.
//! Os dados vêm de um banco relacional real, então usa `rusqlite` em modo somente-leitura.
//!
//! Tabelas usadas:
//! - `games`: biblioteca completa (posse), independente de estar instalado.
//! - `caves`: instalações — cada linha é um jogo efetivamente instalado no disco (ou que já esteve, caso o app não tenha limpado a tabela).
//! - `install_locations`: pastas raiz configuráveis no app itch onde os jogos são instalados.
//! - `cave_historical_play_times`: histórico de sessões de jogo por `cave_id` (`seconds_run`, `last_touched_at`).
//!
//! Política de "instalado": usa o que está em `caves`, sem verificar se a pasta ainda existe
//! fisicamente no disco. Se o usuário apagar a pasta manualmente por fora do app itch, o jogo
//! continua marcado como instalado até uma reinstalação/ desinstalação real remover a linha de `caves`.

use crate::errors::AppError;
use crate::providers::libraries::providers::SourceGame;
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Jogo importado da itch.io com campos adicionais além do `SourceGame` padrão.
#[derive(Debug, Clone)]
pub struct ItchioGame {
    pub source: SourceGame,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

// === LINHAS BRUTAS DO SQLITE ===

struct GameRow {
    id: i64,
    title: String,
    short_text: Option<String>,
    cover_url: Option<String>,
}

/// Uma instalação (`caves`), já cruzada com `install_locations` pra ter o path raiz configurado,
/// o path real e confiável é o `base_path` de dentro do`verdict` (JSON), quando presente.
/// Se o `verdict` estiver ausente, o `install_location_path` só serve de fallback (ex: verdict corrompido/ausente).
struct CaveRow {
    game_id: i64,
    install_folder_name: Option<String>,
    verdict: Option<String>,
    install_location_path: Option<String>,
}

/// Playtime agregado de todas as sessões históricas de uma `cave`.
#[derive(Default, Clone, Copy)]
struct PlaytimeAgg {
    total_seconds: i64,
    last_touched_unix: Option<i64>,
}

// === verdict (JSON dentro de caves.verdict) ===
//
// Exemplo real:
// {"basePath":"E:\\Itch.io\\games\\project-infinity","totalSize":580173388,
//  "candidates":[{"path":"PI Windows (v0.1 Prologue)/Game.exe","depth":2,
//  "flavor":"windows","arch":"386","size":1604096,"windowsInfo":{"gui":true}}]}

#[derive(Debug, Deserialize)]
struct CaveVerdict {
    #[serde(rename = "basePath")]
    base_path: Option<String>,
    #[serde(default)]
    candidates: Vec<CaveVerdictCandidate>,
}

#[derive(Debug, Deserialize)]
struct CaveVerdictCandidate {
    path: String,
    #[serde(default)]
    flavor: Option<String>,
}

fn current_os_flavor() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        "unknown"
    }
}

/// Extrai `(install_path, executable_path)` a partir do `verdict` JSON de uma cave.
/// Prefere o candidato cujo `flavor` bate com o SO atual; cai pro primeiro candidato
/// disponível quando não achar (ex: jogo só-Windows detectado rodando via Wine no Linux).
fn resolve_from_verdict(verdict_json: &str) -> Option<(String, Option<String>)> {
    let verdict: CaveVerdict = serde_json::from_str(verdict_json).ok()?;
    let base_path = verdict.base_path?;

    let flavor = current_os_flavor();
    let candidate = verdict
        .candidates
        .iter()
        .find(|c| c.flavor.as_deref() == Some(flavor))
        .or_else(|| verdict.candidates.first());

    let executable_path = candidate.map(|c| {
        Path::new(&base_path)
            .join(&c.path)
            .to_string_lossy()
            .to_string()
    });

    Some((base_path, executable_path))
}

/// `last_touched_at` pode vir como string RFC3339 ou como número (segundos/ms) —
/// depende de como o driver Go do butler serializou. Trata os dois formatos pra
/// não quebrar a importação inteira por causa de um valor inesperado.
fn parse_unix_timestamp(raw: &str) -> Option<i64> {
    if let Ok(n) = raw.parse::<i64>() {
        return Some(if n > 10_000_000_000 { n / 1000 } else { n });
    }
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.timestamp())
}

// === ITCHIO SOURCE ===

/// Provedor de jogos da itch.io, lendo diretamente `butler.db` (SQLite) do app itch.
pub struct ItchioSource {
    pub butler_db_path: Option<PathBuf>,
}

impl ItchioSource {
    pub fn new(butler_db_path: Option<PathBuf>) -> Self {
        Self { butler_db_path }
    }

    /// Caminhos padrão do app itch (o app itch roda nativo em Windows e Linux):
    /// - Windows: `%APPDATA%\itch\db\butler.db`
    /// - Linux: `~/.config/itch/db/butler.db`
    fn default_butler_db_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            dirs::data_dir().map(|d| d.join("itch").join("db").join("butler.db"))
        }
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir().map(|d| d.join("itch").join("db").join("butler.db"))
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }

    fn resolve_path(&self) -> Result<PathBuf, AppError> {
        let path = self
            .butler_db_path
            .clone()
            .or_else(Self::default_butler_db_path)
            .ok_or_else(|| {
                AppError::NotFound("Caminho do butler.db da itch.io não encontrado.".into())
            })?;

        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Arquivo butler.db da itch.io não encontrado em: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    /// Abre o `butler.db` em modo somente-leitura. O app itch pode estar rodando e mantendo o arquivo
    /// aberto para escrita — abrir como somente-leitura evita corrupção e reduz o risco de "database is locked".
    fn open_readonly(path: &Path) -> Result<Connection, AppError> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| AppError::DatabaseError(format!("Falha ao abrir butler.db: {e}")))?;

        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(conn)
    }

    /// Playtime agregado por `game_id`, somando diretamente da tabela `caves`.
    fn fetch_playtime_by_game(conn: &Connection) -> Result<HashMap<i64, PlaytimeAgg>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT game_id, SUM(seconds_run) AS total_seconds, MAX(last_touched_at) AS last_touched
                 FROM caves
                 GROUP BY game_id",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let game_id: i64 = row.get(0)?; // Pega o game_id como chave
                let total_seconds: i64 = row.get::<_, Option<i64>>(1)?.unwrap_or(0);
                let last_touched_raw: Option<String> = row.get(2)?;
                Ok((game_id, total_seconds, last_touched_raw))
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut map = HashMap::new();
        for row in rows {
            let (game_id, total_seconds, last_touched_raw) =
                row.map_err(|e| AppError::DatabaseError(e.to_string()))?;
            let last_touched_unix = last_touched_raw.as_deref().and_then(parse_unix_timestamp);
            map.insert(
                game_id,
                PlaytimeAgg {
                    total_seconds,
                    last_touched_unix,
                },
            );
        }

        Ok(map)
    }

    /// Todas as instalações (`caves`), cruzadas com `install_locations` pro fallback de path.
    /// Indexado por `game_id` — assume no máximo uma instalação ativa relevante por jogo.
    fn fetch_caves_by_game(conn: &Connection) -> Result<HashMap<i64, CaveRow>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT c.game_id, c.install_folder_name, c.verdict, il.path
                 FROM caves c
                 LEFT JOIN install_locations il ON il.id = c.install_location_id",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CaveRow {
                    game_id: row.get(0)?,
                    install_folder_name: row.get(1)?,
                    verdict: row.get(2)?,
                    install_location_path: row.get(3)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut map = HashMap::new();
        for row in rows {
            let cave = row.map_err(|e| AppError::DatabaseError(e.to_string()))?;
            map.insert(cave.game_id, cave);
        }

        Ok(map)
    }

    /// Monta um `ItchioGame` a partir de uma `CaveRow`, resolvendo `install_path` e `executable_path`
    /// (via `verdict`, com fallback para `install_locations.path` + `install_folder_name`) e o playtime agregado.
    fn build_installed_game(
        game: &GameRow,
        cave: &CaveRow,
        playtime_by_game: &HashMap<i64, PlaytimeAgg>, // <-- Altere o tipo e o nome aqui
    ) -> ItchioGame {
        let (install_path, executable_path) = cave
            .verdict
            .as_deref()
            .and_then(resolve_from_verdict)
            .unwrap_or_else(|| {
                let fallback = match (&cave.install_location_path, &cave.install_folder_name) {
                    (Some(root), Some(folder)) => {
                        Some(Path::new(root).join(folder).to_string_lossy().to_string())
                    }
                    (Some(root), None) => Some(root.clone()),
                    _ => None,
                };
                (fallback.unwrap_or_default(), None)
            });

        // <-- Altere a busca para usar &game.id
        let playtime = playtime_by_game.get(&game.id).copied().unwrap_or_default();
        let playtime_minutes = Some((playtime.total_seconds / 60) as u32);

        let source = SourceGame {
            library: "Itch".to_string(),
            library_game_id: game.id.to_string(),
            name: Some(game.title.clone()),
            installed: true,
            executable_path,
            install_path: (!install_path.is_empty()).then_some(install_path),
            playtime_minutes,
            last_played: playtime.last_touched_unix,
            source_label: None,
        };

        ItchioGame {
            source,
            description: game.short_text.clone(),
            cover_url: game.cover_url.clone(),
        }
    }

    /// Busca só os jogos atualmente com instalação registrada (`caves`).
    pub async fn fetch_installed_detailed(&self) -> Result<Vec<ItchioGame>, AppError> {
        let path = self.resolve_path()?;
        let conn = Self::open_readonly(&path)?;

        let caves_by_game = Self::fetch_caves_by_game(&conn)?;
        let playtime_by_cave = Self::fetch_playtime_by_game(&conn)?;

        if caves_by_game.is_empty() {
            return Ok(Vec::new());
        }

        let game_ids: Vec<i64> = caves_by_game.keys().copied().collect();
        let placeholders = vec!["?"; game_ids.len()].join(",");
        let sql = format!(
            "SELECT id, title, short_text, cover_url FROM games WHERE id IN ({placeholders})"
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let params = rusqlite::params_from_iter(game_ids.iter());
        let rows = stmt
            .query_map(params, |row| {
                Ok(GameRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    short_text: row.get(2)?,
                    cover_url: row.get(3)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut results = Vec::with_capacity(game_ids.len());
        for row in rows {
            let game = row.map_err(|e| AppError::DatabaseError(e.to_string()))?;
            if let Some(cave) = caves_by_game.get(&game.id) {
                results.push(Self::build_installed_game(&game, cave, &playtime_by_cave));
            }
        }

        Ok(results)
    }

    /// Busca a biblioteca completa de posse (tabela `games`, filtrando `classification = 'game'` para
    /// não trazer assets/tools/soundtracks/etc que a itch.io também guarda como itens da conta do usuário),
    /// cruzando com `caves` pra marcar o que está instalado e reaproveitar path/playtime desses casos.
    pub async fn fetch_full_library_detailed(&self) -> Result<Vec<ItchioGame>, AppError> {
        let path = self.resolve_path()?;
        let conn = Self::open_readonly(&path)?;

        let caves_by_game = Self::fetch_caves_by_game(&conn)?;
        let playtime_by_game = Self::fetch_playtime_by_game(&conn)?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, short_text, cover_url FROM games WHERE classification = 'game'",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(GameRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    short_text: row.get(2)?,
                    cover_url: row.get(3)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let game = row.map_err(|e| AppError::DatabaseError(e.to_string()))?;

            if let Some(cave) = caves_by_game.get(&game.id) {
                results.push(Self::build_installed_game(&game, cave, &playtime_by_game));
                continue;
            }

            let source = SourceGame {
                library: "Itch".to_string(),
                library_game_id: game.id.to_string(),
                name: Some(game.title.clone()),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
                source_label: None,
            };

            results.push(ItchioGame {
                source,
                description: game.short_text.clone(),
                cover_url: game.cover_url.clone(),
            });
        }

        Ok(results)
    }
}

// Implementação da trait padrão (retorna apenas SourceGame, sem os extras)
#[async_trait]
impl crate::providers::libraries::providers::GameSource for ItchioSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        let detailed = self.fetch_installed_detailed().await?;
        Ok(detailed.into_iter().map(|g| g.source).collect())
    }
}
