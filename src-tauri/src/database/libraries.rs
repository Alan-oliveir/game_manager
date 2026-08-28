//! Persistência de jogos importados de fontes externas (Steam, Epic, GOG, etc.) na tabela `games`.
//!
//! Camada pura de banco. Fetch (HTTP/leitura local) e orquestração (eventos, enrichment) ficam em `services/libraries.rs`.

use crate::constants;
use crate::providers::libraries::indiegala::IndiegalaGame;
use crate::providers::libraries::itch::ItchioGame;
use crate::providers::libraries::legacy::LegacyGame;
use crate::providers::libraries::providers::SourceGame;
use crate::utils::status_logic;
use chrono::{TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

/// Dados mínimos de um jogo recém-inserido, usados para disparar enriquecimento automático logo após a importação.
#[derive(Debug, Clone)]
pub struct NewlyImportedGame {
    pub game_id: String,
    pub name: String,
    pub library: String,
    pub library_game_id: String,
}

/// Persiste uma lista de jogos de uma fonte externa (como Steam) no banco de dados.
///
/// Recebe `&mut Connection` porque abre uma transação única para o lote inteiro —
/// chame com o `MutexGuard` de `games_db` já travado (faz deref mut automaticamente).
///
/// Retorna o número de jogos inseridos e atualizados.
pub fn persist_source_games(
    conn: &mut Connection,
    games: Vec<SourceGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut inserted = 0;
    let mut updated = 0;
    let mut newly_imported = Vec::new();
    let now = Utc::now().to_rfc3339();

    for game in games {
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        let last_played_iso = game.last_played.and_then(|ts| {
            if ts > 0 {
                Some(Utc.timestamp_opt(ts, 0).single().map(|dt| dt.to_rfc3339()))
            } else {
                None
            }
        });

        let is_official_playtime_library = game.library == "Steam";

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            let cover_url = if game.library == "Steam" {
                Some(format!(
                    "{}/{}",
                    constants::STEAM_CDN_URL,
                    constants::STEAM_LIBRARY_IMAGE_PATH.replace("{}", &game.library_game_id)
                ))
            } else {
                None
            };

            let playtime_source = if is_official_playtime_library {
                Some(
                    crate::models::PlaytimeSource::Store(crate::models::Library::Steam)
                        .as_db_str(),
                )
            } else {
                None
            };

            tx.execute(
                "INSERT INTO games (
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path, source_label
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, NULL, ?12, ?13, ?14)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    playtime_source,
                    last_played_iso,
                    now,
                    game.install_path,
                    game.executable_path,
                    game.source_label,
                ],
            )
                .map_err(|e| e.to_string())?;

            if let Some(url) = &cover_url {
                crate::providers::media::steamgriddb::db::upsert_game_image(
                    &tx, &new_id, "steam_cdn", url, None, None, None, 2,
                )
                    .map_err(|e| e.to_string())?;
            }

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                library: game.library.clone(),
                library_game_id: game.library_game_id.clone(),
            });

            inserted += 1;
        } else {
            if is_official_playtime_library {
                tx.execute(
                    "UPDATE games SET
                    installed = ?1,
                    status = ?2,
                    playtime = ?3,
                    playtime_source = ?4,
                    last_played = ?5,
                    install_path = COALESCE(?6, install_path),
                    executable_path = COALESCE(?7, executable_path),
                    source_label = COALESCE(?8, source_label)
                WHERE library = ?9 AND library_game_id = ?10",
                    params![
                        game.installed,
                        status,
                        game.playtime_minutes.unwrap_or(0),
                        crate::models::PlaytimeSource::Store(crate::models::Library::Steam)
                            .as_db_str(),
                        last_played_iso,
                        game.install_path,
                        game.executable_path,
                        game.source_label,
                        game.library,
                        game.library_game_id
                    ],
                )
                    .map_err(|e| e.to_string())?;
            } else {
                tx.execute(
                    "UPDATE games SET
                    installed = ?1,
                    status = ?2,
                    last_played = ?3,
                    install_path = COALESCE(?4, install_path),
                    executable_path = COALESCE(?5, executable_path),
                    source_label = COALESCE(?6, source_label)
                WHERE library = ?7 AND library_game_id = ?8",
                    params![
                        game.installed,
                        status,
                        last_played_iso,
                        game.install_path,
                        game.executable_path,
                        game.source_label,
                        game.library,
                        game.library_game_id
                    ],
                )
                    .map_err(|e| e.to_string())?;
            }

            updated += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok((inserted, updated, newly_imported))
}

/// Persiste jogos da IndieGala nas tabelas `games` e `game_details`.
///
/// Difere de `persist_source_games` por também gravar `description_raw` e `tags`
/// em `game_details`. Diferente da Legacy Games, aqui não há `cover_url`.
///
/// `playtime_minutes` é passado como `Option` (não `unwrap_or(0)`) pro `UPDATE`
/// porque no modo `full`, jogos possuídos mas que não foram instalados não têm playtime
/// conhecido — usar `COALESCE` preserva o valor real já salvo de uma importação anterior
/// (ex: jogo que foi desinstalado depois de já ter sido jogado) em vez de zerar por engano.
pub fn persist_indiegala_games(
    conn: &mut Connection,
    games: Vec<IndiegalaGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), String> {
    let mut newly_imported = Vec::new();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for indiegala_game in games {
        let game = &indiegala_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            tx.execute(
                "INSERT INTO games (
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, 0, NULL, ?11, ?12)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    crate::models::PlaytimeSource::Store(crate::models::Library::Indiegala)
                        .as_db_str(),
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
                .map_err(|e| e.to_string())?;

            let tags_json = indiegala_game
                .tags
                .as_ref()
                .and_then(|tags| crate::database::serialize_tags(tags).ok());

            if let Some(tags) = &tags_json {
                tx.execute(
                    "INSERT OR IGNORE INTO game_details (game_id, tags) VALUES (?1, ?2)",
                    params![new_id, tags],
                )
                    .map_err(|e| e.to_string())?;
            }

            if let Some(desc) = &indiegala_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description) VALUES (?1, ?2)
                        ON CONFLICT(game_id) DO UPDATE SET description = COALESCE(game_descriptions.description, excluded.description)",
                    params![new_id, desc],
                )
                    .map_err(|e| e.to_string())?;
            }

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                library: game.library.clone(),
                library_game_id: game.library_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                        installed       = ?1,
                        status          = ?2,
                        playtime        = COALESCE(?3, playtime),
                        playtime_source = CASE WHEN ?3 IS NOT NULL THEN ?4 ELSE playtime_source END,
                        install_path    = COALESCE(?5, install_path),
                        executable_path = COALESCE(?6, executable_path)
                    WHERE library = ?7 AND library_game_id = ?8",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes,
                    crate::models::PlaytimeSource::Store(crate::models::Library::Indiegala)
                        .as_db_str(),
                    game.install_path,
                    game.executable_path,
                    game.library,
                    game.library_game_id,
                ],
            )
                .map_err(|e| e.to_string())?;

            updated += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok((inserted, updated, newly_imported))
}

/// Persiste jogos da Itch.io nas tabelas `games` e `game_details`.
///
/// Grava `cover_url` diretamente na tabela `games` e envia `description_raw` para a tabela `game_details`.
pub fn persist_itch_games(
    conn: &mut Connection,
    games: Vec<ItchioGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), String> {
    let mut newly_imported = Vec::new();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for itchio_game in games {
        let game = &itchio_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        let last_played_iso = game.last_played.and_then(|ts| {
            if ts > 0 {
                Utc.timestamp_opt(ts, 0).single().map(|dt| dt.to_rfc3339())
            } else {
                None
            }
        });

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            tx.execute(
                "INSERT INTO games (
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, NULL, ?12, ?13)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    crate::models::PlaytimeSource::Store(crate::models::Library::Itch)
                        .as_db_str(),
                    last_played_iso,
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
                .map_err(|e| e.to_string())?;

            if let Some(url) = &itchio_game.cover_url {
                crate::providers::media::steamgriddb::db::upsert_game_image(
                    &tx, &new_id, "itch", url, None, None, None, 2,
                )
                    .map_err(|e| e.to_string())?;
            }

            if let Some(desc) = &itchio_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description) VALUES (?1, ?2)
                        ON CONFLICT(game_id) DO UPDATE SET description = COALESCE(game_descriptions.description, excluded.description)",
                    params![new_id, desc],
                )
                    .map_err(|e| e.to_string())?;
            }

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                library: game.library.clone(),
                library_game_id: game.library_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed       = ?1,
                    status          = ?2,
                    playtime        = COALESCE(?3, playtime),
                    playtime_source = CASE WHEN ?3 IS NOT NULL THEN ?4 ELSE playtime_source END,
                    last_played     = COALESCE(?5, last_played),
                    install_path    = COALESCE(?6, install_path),
                    executable_path = COALESCE(?7, executable_path)
                WHERE library = ?8 AND library_game_id = ?9",
                params![
                    game.installed,
                    status,
                    game.playtime_minutes,
                    crate::models::PlaytimeSource::Store(crate::models::Library::Itch)
                        .as_db_str(),
                    last_played_iso,
                    game.install_path,
                    game.executable_path,
                    game.library,
                    game.library_game_id,
                ],
            )
                .map_err(|e| e.to_string())?;

            if let Some(url) = &itchio_game.cover_url {
                let existing_id: Option<String> = tx
                    .query_row(
                        "SELECT id FROM games WHERE library = ?1 AND library_game_id = ?2",
                        params![game.library, game.library_game_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?;

                if let Some(id) = existing_id {
                    crate::providers::media::steamgriddb::db::upsert_game_image(
                        &tx, &id, "itch", url, None, None, None, 2,
                    )
                        .map_err(|e| e.to_string())?;
                }
            }

            if let Some(desc) = &itchio_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description)
                        VALUES ((SELECT id FROM games WHERE library = ?1 AND library_game_id = ?2), ?3)
                        ON CONFLICT(game_id) DO UPDATE SET description = excluded.description",
                    params![game.library, game.library_game_id, desc],
                ).map_err(|e| e.to_string())?;
            }

            updated += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok((inserted, updated, newly_imported))
}

/// Persiste jogos da Legacy Games nas tabelas `games` e `game_details`.
///
/// Difere de `persist_source_games` por também gravar `cover_url` e
/// `description_raw` em `game_details`, que não fazem parte do `SourceGame` padrão.
pub fn persist_legacy_games(
    conn: &mut Connection,
    games: Vec<LegacyGame>,
) -> Result<(u32, u32, Vec<NewlyImportedGame>), String> {
    let mut newly_imported = Vec::new();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut inserted = 0u32;
    let mut updated = 0u32;
    let now = Utc::now().to_rfc3339();

    for legacy_game in games {
        let game = &legacy_game.source;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM games WHERE library = ?1 AND library_game_id = ?2)",
                params![&game.library, &game.library_game_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let status = status_logic::calculate_status(game.playtime_minutes.unwrap_or(0) as i32);

        if !exists {
            let new_id = Uuid::new_v4().to_string();
            let display_name = game.name.clone().unwrap_or_else(|| "Unknown".to_string());
            let slug = crate::utils::text::slugify(&display_name);

            tx.execute(
                "INSERT INTO games (
                    id, name, slug, library, library_game_id,
                    installed, status, playtime, playtime_source, last_played, added_at,
                    favorite, user_rating, install_path, executable_path
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, ?9, 0, NULL, ?10, ?11)",
                params![
                    new_id,
                    display_name,
                    slug,
                    game.library,
                    game.library_game_id,
                    game.installed,
                    status,
                    game.playtime_minutes.unwrap_or(0),
                    now,
                    game.install_path,
                    game.executable_path,
                ],
            )
                .map_err(|e| e.to_string())?;

            if let Some(url) = &legacy_game.cover_url {
                crate::providers::media::steamgriddb::db::upsert_game_image(
                    &tx, &new_id, "legacy", url, None, None, None, 2,
                )
                    .map_err(|e| e.to_string())?;
            }

            if let Some(desc) = &legacy_game.description {
                tx.execute(
                    "INSERT INTO game_descriptions (game_id, description) VALUES (?1, ?2)
                        ON CONFLICT(game_id) DO UPDATE SET description = COALESCE(game_descriptions.description, excluded.description)",
                    params![new_id, desc],
                )
                    .map_err(|e| e.to_string())?;
            }

            newly_imported.push(NewlyImportedGame {
                game_id: new_id,
                name: display_name,
                library: game.library.clone(),
                library_game_id: game.library_game_id.clone(),
            });

            inserted += 1;
        } else {
            tx.execute(
                "UPDATE games SET
                    installed   = ?1,
                    status      = ?2,
                    install_path     = COALESCE(?3, install_path),
                    executable_path  = COALESCE(?4, executable_path)
                 WHERE library = ?5 AND library_game_id = ?6",
                params![
                    game.installed,
                    status,
                    game.install_path,
                    game.executable_path,
                    game.library,
                    game.library_game_id,
                ],
            )
                .map_err(|e| e.to_string())?;

            updated += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok((inserted, updated, newly_imported))
}
