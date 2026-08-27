//! Módulo de gerenciamento da biblioteca de jogos.
//!
//! Implementa operações CRUD para jogos.
//! Inclui validações robustas e manipulação de erros para garantir integridade dos dados.

use crate::constants;
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::models;
use crate::models::Library;
use crate::providers::media::steamgriddb;
use crate::utils::status_logic;
use crate::utils::text::slugify;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use tauri::State;
use url::Url;
use uuid::Uuid;

// === STRUCTS ===

/// Dados de entrada para criar ou atualizar um jogo.
///
/// Reflete os campos da ‘interface’ de adição/edição de jogos.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInput {
    pub id: String,
    pub name: String,
    pub library: Library,
    pub library_game_id: String,
    pub cover_url: Option<String>,
    pub installed: bool,
    pub import_confidence: Option<String>,
    pub playtime: Option<i32>,
    pub user_rating: Option<i32>,
    pub status: Option<String>,
    pub install_path: Option<String>,
    pub executable_path: Option<String>,
    pub launch_args: Option<String>,
}

/// Dados de entrada para atualizar detalhes adicionais do jogo.
///
/// Usado para atualizar a tabela 'game_details'.
#[derive(serde::Deserialize)]
pub struct UpdateGameDetailsInput {
    pub id: String,
    pub description: Option<String>, // Salva na descrição PT-BR
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub released: Option<String>,
}

// === FUNÇÕES AUXILIARES ===

/// Função auxiliar privada para validar dados de entrada.
///
/// Evita duplicação de código entre add e ‘update’.
/// Valida nome, URL da capa, plataforma, tempo jogado e avaliação.
fn validate_input(game: &GameInput) -> Result<(), AppError> {
    if game.name.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Nome do jogo não pode ser vazio".to_string(),
        ));
    }

    if game.name.len() > constants::MAX_NAME_LENGTH {
        return Err(AppError::ValidationError(format!(
            "Nome muito longo (max {})",
            constants::MAX_NAME_LENGTH
        )));
    }

    if let Some(ref url_str) = game.cover_url {
        if url_str.len() > constants::MAX_URL_LENGTH {
            return Err(AppError::ValidationError(format!(
                "URL da capa muito longa (máximo {} caracteres)",
                constants::MAX_URL_LENGTH
            )));
        }
        // Validação básica de URL
        if !url_str.starts_with("http") && !url_str.starts_with("asset://") {
            let url = Url::parse(url_str)
                .map_err(|_| AppError::ValidationError("URL inválida.".to_string()))?;
            if url.scheme() != "http" && url.scheme() != "https" {
                return Err(AppError::ValidationError(
                    "A URL deve ser HTTP, HTTPS ou Asset local.".to_string(),
                ));
            }
        }
    }

    if let Some(time) = game.playtime {
        if time < 0 {
            return Err(AppError::ValidationError(
                "Tempo jogado não pode ser negativo".to_string(),
            ));
        }
        if time > constants::MAX_PLAYTIME {
            return Err(AppError::ValidationError(
                "Tempo jogado excessivo".to_string(),
            ));
        }
    }

    if let Some(r) = game.user_rating {
        if !(constants::MIN_RATING..=constants::MAX_RATING).contains(&r) {
            return Err(AppError::ValidationError(format!(
                "Avaliação deve estar entre {} e {}",
                constants::MIN_RATING,
                constants::MAX_RATING
            )));
        }
    }

    Ok(())
}

// === CRUD ===

/// Adiciona um novo jogo à biblioteca.
///
/// Insere dados na tabela 'games' após as validações necessárias.
#[tauri::command]
pub fn add_game(state: State<AppState>, game: GameInput) -> Result<(), AppError> {
    validate_input(&game)?;

    let conn = state.games_db.lock()?;

    // Verifica duplicidade
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
        params![game.id],
        |row| row.get(0),
    )?;

    if exists {
        return Err(AppError::AlreadyExists(
            "Já existe um jogo com este ID".to_string(),
        ));
    }

    // Lógica Automática de Status
    let final_status = game
        .status
        .unwrap_or_else(|| status_logic::calculate_status(game.playtime.unwrap_or(0)));

    let added_at = Utc::now().to_rfc3339();
    let library = game.library;

    let library_game_id = if matches!(library, Library::Outra) {
        format!("manual-{}", Uuid::new_v4())
    } else {
        game.library_game_id.clone()
    };

    conn.execute(
        "INSERT INTO games (
        id, name, slug, library, library_game_id,
        installed, import_confidence, install_path, executable_path, launch_args,
        user_rating, status, playtime, added_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            game.id,
            game.name,
            slugify(&game.name),
            library.to_string(),
            library_game_id,
            game.installed,
            game.import_confidence,
            game.install_path,
            game.executable_path,
            game.launch_args,
            game.user_rating,
            final_status,
            game.playtime.unwrap_or(0),
            added_at
        ],
    )?;

    if let Some(url) = &game.cover_url {
        steamgriddb::db::upsert_game_image(&conn, &game.id, "manual", url, None, None, None, -1)?;
    }

    Ok(())
}

/// Recupera todos os jogos da biblioteca.
///
/// Retorna a lista completa de jogos ordenada conforme armazenada no banco.
/// Inclui todos os campos, inclusive o status de favorito.
#[tauri::command]
pub fn get_games(state: State<AppState>) -> Result<Vec<models::Game>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
        g.id, g.name, g.slug, g.library, g.library_game_id, g.installed, g.import_confidence, g.install_path, g.executable_path,
        g.launch_args, g.user_rating, g.favorite, g.status, g.playtime, g.playtime_source, g.last_played, g.added_at, g.alternative_names,
        gd.genres, gd.developer, COALESCE(gd.is_adult, 0) as is_adult, g.source_label,
        (SELECT url FROM game_images WHERE game_id = g.id AND image_type = 'cover' ORDER BY priority ASC LIMIT 1) AS cover_url,
        gd.critic_score, gd.release_date, gd.display_name
    FROM games g
    LEFT JOIN game_details gd ON g.id = gd.game_id
    ORDER BY g.name ASC"
    )?;

    let games = stmt
        .query_map([], |row| {
            let alt_names_json: Option<String> = row.get(17)?;
            let alternative_names = alt_names_json.and_then(|s| serde_json::from_str(&s).ok());
            let genres_json: Option<String> = row.get(18)?;
            let genres = genres_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                library: row.get::<_, String>(3)?.parse().unwrap_or(Library::Outra),
                library_game_id: row.get(4)?,
                installed: row.get(5)?,
                import_confidence: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| s.parse().ok()),
                install_path: row.get(7)?,
                executable_path: row.get(8)?,
                launch_args: row.get(9)?,
                user_rating: row.get(10)?,
                favorite: row.get(11)?,
                status: row.get(12)?,
                playtime: row.get(13)?,
                playtime_source: row
                    .get::<_, Option<String>>(14)?
                    .and_then(|s| s.parse().ok()),
                last_played: row.get(15)?,
                added_at: row.get(16)?,
                alternative_names,
                genres,
                developer: row.get(19)?,
                is_adult: row.get(20)?,
                source_label: row.get(21)?,
                cover_url: row.get(22)?,
                critic_score: row.get(23)?,
                release_date: row.get(24)?,
                display_name: row.get(25)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(games)
}

/// Recupera detalhes adicionais de um jogo na biblioteca.
///
/// Busca na tabela 'game_details' usando o game_id fornecido.
/// Usado para obter informações adicionais sobre o jogo que serão exibidas na ‘interface’.
/// Retorna None se não houver detalhes para o jogo.
#[tauri::command]
pub fn get_library_game_details(
    state: State<AppState>,
    game_id: String,
) -> Result<Option<models::GameDetails>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
            gd.game_id, gd.steam_app_id, gd.developer, gd.publisher, gd.release_date,
            gd.genres, gd.tags, gd.series, gd.franchise, gd.game_modes, gd.player_perspectives,
            gd.themes, gd.keywords, gd.critic_score,
            gd.steam_review_label, gd.steam_review_count, gd.steam_review_score, gd.steam_review_updated_at,
            gd.age_ratings, gd.is_adult, gd.adult_tags, gd.external_links,
            gd.hltb_main_story, gd.hltb_main_extra, gd.hltb_completionist, gd.hltb_coop_time,
            gd.display_name, gd.updated_at,
            gdesc.summary, gdesc.storyline, gdesc.short_description, gdesc.description,
            gdesc.summary_translated, gdesc.storyline_translated,
            gdesc.short_description_translated, gdesc.description_translated, gdesc.translated_lang
         FROM game_details gd
         LEFT JOIN game_descriptions gdesc ON gd.game_id = gdesc.game_id
         WHERE gd.game_id = ?1",
    )?;

    let mut rows = stmt.query_map(params![game_id], |row| {
        let genres_json: Option<String> = row.get(5)?;
        let tags_json: Option<String> = row.get(6)?;
        let franchise_json: Option<String> = row.get(8)?;
        let game_modes_json: Option<String> = row.get(9)?;
        let perspectives_json: Option<String> = row.get(10)?;
        let themes_json: Option<String> = row.get(11)?;
        let keywords_json: Option<String> = row.get(12)?;
        let age_ratings_json: Option<String> = row.get(18)?;
        let links_json: Option<String> = row.get(21)?;

        Ok(models::GameDetails {
            game_id: row.get(0)?,
            steam_app_id: row.get(1)?,
            developer: row.get(2)?,
            publisher: row.get(3)?,
            release_date: row.get(4)?,
            genres: genres_json.and_then(|s| serde_json::from_str(&s).ok()),
            tags: tags_json.map(|s| database::deserialize_tags(&s)),
            series: row.get(7)?,
            franchise: franchise_json.and_then(|s| serde_json::from_str(&s).ok()),
            game_modes: game_modes_json.and_then(|s| serde_json::from_str(&s).ok()),
            player_perspectives: perspectives_json.and_then(|s| serde_json::from_str(&s).ok()),
            themes: themes_json.and_then(|s| serde_json::from_str(&s).ok()),
            keywords: keywords_json.and_then(|s| serde_json::from_str(&s).ok()),
            critic_score: row.get(13)?,
            steam_review_label: row.get(14)?,
            steam_review_count: row.get(15)?,
            steam_review_score: row.get(16)?,
            steam_review_updated_at: row.get(17)?,
            age_ratings: age_ratings_json.and_then(|s| serde_json::from_str(&s).ok()),
            is_adult: row.get(19).unwrap_or(false),
            adult_tags: row.get(20)?,
            external_links: links_json.and_then(|s| serde_json::from_str(&s).ok()),
            hltb_main_story: row.get(22)?,
            hltb_main_extra: row.get(23)?,
            hltb_completionist: row.get(24)?,
            hltb_coop_time: row.get(25)?,
            display_name: row.get(26)?,
            updated_at: row.get(27)?,
            description: models::GameDescription {
                summary: row.get(28)?,
                storyline: row.get(29)?,
                short_description: row.get(30)?,
                description: row.get(31)?,
                summary_translated: row.get(32)?,
                storyline_translated: row.get(33)?,
                short_description_translated: row.get(34)?,
                description_translated: row.get(35)?,
                translated_lang: row.get(36)?,
            },
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

/// Recupera um único jogo da biblioteca pelo ID.
///
/// Retorna `None` se o ID não existir — não é considerado erro.
#[tauri::command]
pub fn get_game_by_id(
    state: State<AppState>,
    id: String,
) -> Result<Option<models::Game>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
        g.id, g.name, g.slug, g.library, g.library_game_id, g.installed, g.import_confidence, g.install_path, g.executable_path,
        g.launch_args, g.user_rating, g.favorite, g.status, g.playtime, g.playtime_source, g.last_played, g.added_at, g.alternative_names,
        gd.genres, gd.developer, COALESCE(gd.is_adult, 0) as is_adult, g.source_label,
        (SELECT url FROM game_images WHERE game_id = g.id AND image_type = 'cover' ORDER BY priority ASC LIMIT 1) AS cover_url,
        gd.critic_score, gd.release_date, gd.display_name
    FROM games g
    LEFT JOIN game_details gd ON g.id = gd.game_id
    WHERE g.id = ?"
    )?;

    let game = stmt
        .query_map([&id], |row| {
            let alt_names_json: Option<String> = row.get(17)?;
            let alternative_names = alt_names_json.and_then(|s| serde_json::from_str(&s).ok());
            let genres_json: Option<String> = row.get(18)?;
            let genres = genres_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                library: row.get::<_, String>(3)?.parse().unwrap_or(Library::Outra),
                library_game_id: row.get(4)?,
                installed: row.get(5)?,
                import_confidence: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| s.parse().ok()),
                install_path: row.get(7)?,
                executable_path: row.get(8)?,
                launch_args: row.get(9)?,
                user_rating: row.get(10)?,
                favorite: row.get(11)?,
                status: row.get(12)?,
                playtime: row.get(13)?,
                playtime_source: row
                    .get::<_, Option<String>>(14)?
                    .and_then(|s| s.parse().ok()),
                last_played: row.get(15)?,
                added_at: row.get(16)?,
                alternative_names,
                genres,
                developer: row.get(19)?,
                is_adult: row.get(20)?,
                source_label: row.get(21)?,
                cover_url: row.get(22)?,
                critic_score: row.get(23)?,
                release_date: row.get(24)?,
                display_name: row.get(25)?,
            })
        })?
        .next()
        .transpose()?;

    Ok(game)
}

/// Atualiza informações de um jogo existente.
///
/// Atualiza os campos, preservando added_at e favorite, com os novos valores fornecidos.
/// Realiza as mesmas validações de 'add_game'.
///
/// **Nota:** Não retorna erro se 'ID' não existe ('update' silencioso).
#[tauri::command]
pub fn update_game(state: State<AppState>, game: GameInput) -> Result<(), AppError> {
    validate_input(&game)?;

    let conn = state.games_db.lock()?;

    conn.execute(
        "UPDATE games SET
        name = ?1,
        slug = ?2,
        library = ?3,
        library_game_id = ?4,
        installed = ?5,
        import_confidence = ?6,
        playtime = ?7,
        user_rating = ?8,
        status = ?9,
        install_path = ?10,
        executable_path = ?11,
        launch_args = ?12
    WHERE id = ?13",
        params![
            game.name,
            slugify(&game.name),
            game.library.to_string(),
            game.library_game_id,
            game.installed,
            game.import_confidence,
            game.playtime,
            game.user_rating,
            game.status,
            game.install_path,
            game.executable_path,
            game.launch_args,
            game.id
        ],
    )?;

    match &game.cover_url {
        Some(url) => steamgriddb::db::upsert_game_image(
            &conn, &game.id, "manual", url, None, None, None, -1,
        )?,
        None => steamgriddb::db::delete_game_image(&conn, &game.id, "cover", "manual")?,
    }

    Ok(())
}

/// Atualiza detalhes adicionais de um jogo na biblioteca.
///
/// Insere ou atualiza os campos na tabela 'game_details' conforme o ID do jogo.
/// Se os detalhes já existirem, realiza um UPDATE; caso contrário, faz um INSERT.
/// Aceita os campos: descrição (traduzido), desenvolvedor, publicadora e data de lançamento.
#[tauri::command]
pub fn update_game_details(
    state: State<AppState>,
    payload: UpdateGameDetailsInput,
) -> Result<(), AppError> {
    let conn = state.games_db.lock().map_err(|_| AppError::MutexError)?;

    // Verifica o estado atual do jogo no banco
    let current_state: Option<(Option<String>, Option<String>, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT summary, storyline, short_description, description FROM game_descriptions WHERE game_id = ?1",
            params![payload.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;

    match current_state {
        Some((summary, storyline, short_description, description)) => {
            let has_original_description =
                summary.is_some() || storyline.is_some() || short_description.is_some();

            if description.is_none() && has_original_description {
                return Err(AppError::ValidationError(
                    "A descrição precisa ser traduzida (ou gerada) antes de ser editada manualmente.".to_string()
                ));
            }

            conn.execute(
                "INSERT OR IGNORE INTO game_descriptions (game_id) VALUES (?1)",
                params![payload.id],
            )?;

            conn.execute(
                "UPDATE game_descriptions SET description = ?1 WHERE game_id = ?2",
                params![payload.description, payload.id],
            )?;

            conn.execute(
                "INSERT OR IGNORE INTO game_details (game_id) VALUES (?1)",
                params![payload.id],
            )?;

            conn.execute(
                "UPDATE game_details SET
                    developer = ?1,
                    publisher = ?2,
                    release_date = ?3
                 WHERE game_id = ?4",
                params![
                    payload.developer,
                    payload.publisher,
                    payload.released,
                    payload.id
                ],
            )?;
        }

        None => {
            conn.execute(
                "INSERT OR IGNORE INTO game_descriptions (game_id, description) VALUES (?1, ?2)",
                params![payload.id, payload.description],
            )?;

            conn.execute(
                "INSERT OR IGNORE INTO game_details (game_id) VALUES (?1)",
                params![payload.id],
            )?;

            conn.execute(
                "UPDATE game_details SET
                    developer = ?1,
                    publisher = ?2,
                    release_date = ?3
                 WHERE game_id = ?4",
                params![
                    payload.developer,
                    payload.publisher,
                    payload.released,
                    payload.id
                ],
            )?;
        }
    }

    Ok(())
}

/// Remove permanentemente um jogo da biblioteca.
///
/// **Nota:** Esta ação é irreversível e exclui todos os dados relacionados ao jogo.
#[tauri::command]
pub fn delete_game(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.games_db.lock()?;

    conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
    conn.execute("DELETE FROM game_details WHERE game_id = ?1", params![id])?;

    Ok(())
}

// === OPERAÇÕES DOS USUÁRIOS ===

/// Alterna o status de favorito de um jogo.
///
/// Inverte o valor booleano do campo 'favorite' usando NOT lógico.
/// Se era favorito, deixa de ser; se não era, passa a ser.
///
/// **Nota:** Esta operação é idempotente e não retorna erro se o ‘ID’ não existir.
#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.games_db.lock()?;

    conn.execute(
        "UPDATE games SET favorite = NOT favorite WHERE id = ?1",
        params![id],
    )?;

    Ok(())
}

/// Define o status de um jogo na biblioteca.
///
/// Altera o campo 'status' para a condição fornecida para o jogo.
/// Não há validação do valor; espera-se que o frontend envie valores válidos.
/// A lista de status possíveis inclui "completed", "playing", "backlog" e "abandoned".
#[tauri::command]
pub fn set_game_status(state: State<AppState>, id: String, status: String) -> Result<(), AppError> {
    let conn = state.games_db.lock()?;
    conn.execute(
        "UPDATE games SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

/// Define a avaliação pessoal de um jogo.
///
/// Atualiza o campo 'user_rating' com o valor fornecido.
/// Aceita valores de 0 a 5, onde 0 remove a avaliação (define como NULL).
#[tauri::command]
pub fn set_game_rating(state: State<AppState>, id: String, rating: i32) -> Result<(), AppError> {
    // Validação rápida
    if !(0..=5).contains(&rating) {
        return Err(AppError::ValidationError("Rating inválido".to_string()));
    }

    let conn = state.games_db.lock()?;

    // Se rating for 0, remove a avaliação (NULL)
    let val = if rating == 0 { None } else { Some(rating) };

    conn.execute(
        "UPDATE games SET user_rating = ?1 WHERE id = ?2",
        params![val, id],
    )?;
    Ok(())
}
