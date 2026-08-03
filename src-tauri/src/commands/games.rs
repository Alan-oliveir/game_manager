//! Módulo de gerenciamento da biblioteca de jogos.
//!
//! Implementa operações CRUD para jogos.
//! Inclui validações robustas e manipulação de erros para garantir integridade dos dados.

use crate::constants;
use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::models;
use crate::models::Platform;
use crate::utils::status_logic;
use crate::utils::text::slugify;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use tauri::State;
use url::Url;
use uuid::Uuid;

/// Dados de entrada para criar ou atualizar um jogo.
///
/// Reflete os campos da ‘interface’ de adição/edição de jogos.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInput {
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub platform_game_id: String,
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
    let platform = game.platform;

    let platform_game_id = if matches!(platform, Platform::Outra) {
        format!("manual-{}", Uuid::new_v4())
    } else {
        game.platform_game_id.clone()
    };

    conn.execute(
        "INSERT INTO games (
        id, name, slug, cover_url, platform, platform_game_id,
        installed, import_confidence, install_path, executable_path, launch_args,
        user_rating, status, playtime, added_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            game.id,
            game.name,
            slugify(&game.name),
            game.cover_url,
            platform.to_string(),
            platform_game_id,
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

    Ok(())
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
        cover_url = ?3,
        platform = ?4,
        platform_game_id = ?5,
        installed = ?6,
        import_confidence = ?7,
        playtime = ?8,
        user_rating = ?9,
        status = ?10,
        install_path = ?11,
        executable_path = ?12,
        launch_args = ?13
    WHERE id = ?14",
        params![
            game.name,
            slugify(&game.name),
            game.cover_url,
            game.platform.to_string(),
            game.platform_game_id,
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
            g.id, g.name, g.slug, g.cover_url, g.platform, g.platform_game_id, g.installed, g.import_confidence, g.install_path, g.executable_path,
            g.launch_args, g.user_rating, g.favorite, g.status, g.playtime, g.playtime_source, g.last_played, g.added_at,
            gd.genres, gd.developer, COALESCE(gd.is_adult, 0) as is_adult
        FROM games g
        LEFT JOIN game_details gd ON g.id = gd.game_id
        ORDER BY g.name ASC"
    )?;

    let games = stmt
        .query_map([], |row| {
            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                cover_url: row.get(3)?,
                platform: row.get::<_, String>(4)?.parse().unwrap_or(Platform::Outra),
                platform_game_id: row.get(5)?,
                installed: row.get(6)?,
                import_confidence: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| s.parse().ok()),
                install_path: row.get(8)?,
                executable_path: row.get(9)?,
                launch_args: row.get(10)?,
                user_rating: row.get(11)?,
                favorite: row.get(12)?,
                status: row.get(13)?,
                playtime: row.get(14)?,
                playtime_source: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| s.parse().ok()),
                last_played: row.get(16)?,
                added_at: row.get(17)?,
                genres: row.get(18)?,
                developer: row.get(19)?,
                is_adult: row.get(20)?,
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
                game_id, steam_app_id, developer, publisher, release_date, genres, tags, series,
                description_raw, description_ptbr, background_image, critic_score,
                steam_review_label, steam_review_count, steam_review_score, steam_review_updated_at,
                esrb_rating, is_adult, adult_tags, external_links, median_playtime,
                estimated_playtime
             FROM game_details
             WHERE game_id = ?1",
    )?;

    let mut rows = stmt.query_map(params![game_id], |row| {
        let links_json: Option<String> = row.get(19)?; // external_links
        let external_links = links_json.and_then(|json| serde_json::from_str(&json).ok());

        let tags_json: Option<String> = row.get(6)?;
        let tags = tags_json.map(|s| database::deserialize_tags(&s));

        Ok(models::GameDetails {
            game_id: row.get(0)?,
            steam_app_id: row.get(1)?,
            developer: row.get(2)?,
            publisher: row.get(3)?,
            release_date: row.get(4)?,
            genres: row.get(5)?,
            tags,
            series: row.get(7)?,
            description_raw: row.get(8)?,
            description_ptbr: row.get(9)?,
            background_image: row.get(10)?,
            critic_score: row.get(11)?,
            steam_review_label: row.get(12)?,
            steam_review_count: row.get(13)?,
            steam_review_score: row.get(14)?,
            steam_review_updated_at: row.get(15)?,
            esrb_rating: row.get(16)?,
            is_adult: row.get(17).unwrap_or(false),
            adult_tags: row.get(18)?,
            external_links,
            median_playtime: row.get(20)?,
            estimated_playtime: row.get(21)?,
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
            g.id, g.name, g.slug, g.cover_url, g.platform, g.platform_game_id, g.installed, g.import_confidence, g.install_path, g.executable_path,
            g.launch_args, g.user_rating, g.favorite, g.status, g.playtime, g.playtime_source, g.last_played, g.added_at,
            gd.genres, gd.developer, COALESCE(gd.is_adult, 0) as is_adult
        FROM games g
        LEFT JOIN game_details gd ON g.id = gd.game_id
        WHERE g.id = ?1",
    )?;

    let game = stmt
        .query_row(params![id], |row| {
            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                cover_url: row.get(3)?,
                platform: row.get::<_, String>(4)?.parse().unwrap_or(Platform::Outra),
                platform_game_id: row.get(5)?,
                installed: row.get(6)?,
                import_confidence: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| s.parse().ok()),
                install_path: row.get(8)?,
                executable_path: row.get(9)?,
                launch_args: row.get(10)?,
                user_rating: row.get(11)?,
                favorite: row.get(12)?,
                status: row.get(13)?,
                playtime: row.get(14)?,
                playtime_source: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| s.parse().ok()),
                last_played: row.get(16)?,
                added_at: row.get(17)?,
                genres: row.get(18)?,
                developer: row.get(19)?,
                is_adult: row.get(20)?,
            })
        })
        .optional()?;

    Ok(game)
}

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

/// Remove permanentemente um jogo da biblioteca.
///
/// **Nota:** Esta ação é irreversível e exclui todos os dados relacionados ao jogo.
#[tauri::command]
pub fn delete_game(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.games_db.lock()?;

    conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;

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
    let current_state: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT description_ptbr, description_raw FROM game_details WHERE game_id = ?1",
            params![payload.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?; // O '?' converte erro de SQL para AppError

    match current_state {
        // CASO 1: Registro existe
        Some((description_ptbr, description_raw)) => {
            // VALIDAÇÃO: Se a descrição PT-BR for nula e description_raw não for nula, impede a edição
            if description_ptbr.is_none() && description_raw.is_some() {
                return Err(AppError::ValidationError(
                    "A descrição precisa ser traduzida (ou gerada) antes de ser editada manualmente.".to_string()
                ));
            }
            conn.execute(
                "UPDATE game_details SET
                    description_ptbr = ?1,
                    developer = ?2,
                    publisher = ?3,
                    release_date = ?4
                 WHERE game_id = ?5",
                params![
                    payload.description,
                    payload.developer,
                    payload.publisher,
                    payload.released,
                    payload.id
                ],
            )?;
        }

        // CASO 2: detalhes do jogo não existe (Novo Jogo Manual)
        None => {
            conn.execute(
                "INSERT INTO game_details (game_id, description_ptbr, developer, publisher, release_date)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    payload.id,
                    payload.description, // Define a primeira versão como "traduzida"
                    payload.developer,
                    payload.publisher,
                    payload.released
                ],
            )?;
        }
    }

    Ok(())
}
