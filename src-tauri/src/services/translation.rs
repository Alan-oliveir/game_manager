//! Serviço de tradução de descrições de jogos.
//!
//! Orquestra: resolve quais campos precisam de tradução (via GameDescription::fields_to_display),
//! chama o provider Gemini sequencialmente por campo, e persiste cada resultado imediatamente
//! (cache por campo, resiliente a falha parcial).

use crate::database;
use crate::database::AppState;
use crate::errors::AppError;
use crate::models::GameDescription;
use crate::providers::translation::gemini;
use crate::services::locale::get_or_detect_language;
use rusqlite::params;
use tauri::{AppHandle, Manager, State};
use tracing::{error, info};

fn persist_translated_field(
    conn: &rusqlite::Connection,
    game_id: &str,
    field: &str,
    value: &str,
    target_lang: &str,
) -> Result<(), AppError> {
    match field {
        "summary" => conn.execute(
            "UPDATE game_descriptions SET summary_translated = ?1, translated_lang = ?2 WHERE game_id = ?3",
            params![value, target_lang, game_id],
        )?,
        "storyline" => conn.execute(
            "UPDATE game_descriptions SET storyline_translated = ?1, translated_lang = ?2 WHERE game_id = ?3",
            params![value, target_lang, game_id],
        )?,
        "short_description" => conn.execute(
            "UPDATE game_descriptions SET short_description_translated = ?1, translated_lang = ?2 WHERE game_id = ?3",
            params![value, target_lang, game_id],
        )?,
        "description" => conn.execute(
            "UPDATE game_descriptions SET description_translated = ?1, translated_lang = ?2 WHERE game_id = ?3",
            params![value, target_lang, game_id],
        )?,
        _ => 0,
    };
    Ok(())
}

fn fetch_description(
    conn: &rusqlite::Connection,
    game_id: &str,
) -> Result<GameDescription, AppError> {
    Ok(conn.query_row(
        "SELECT summary, storyline, short_description, description,
                summary_translated, storyline_translated,
                short_description_translated, description_translated,
                translated_lang
         FROM game_descriptions WHERE game_id = ?1",
        params![game_id],
        |row| {
            Ok(GameDescription {
                summary: row.get(0)?,
                storyline: row.get(1)?,
                short_description: row.get(2)?,
                description: row.get(3)?,
                summary_translated: row.get(4)?,
                storyline_translated: row.get(5)?,
                short_description_translated: row.get(6)?,
                description_translated: row.get(7)?,
                translated_lang: row.get(8)?,
            })
        },
    )?)
}

/// Traduz os campos de exibição da descrição de um jogo (summary + storyline,
/// ou short_description/description como fallback), pro idioma alvo.
///
/// Traduz sequencialmente e persiste cada campo assim que concluído, evitando
/// retrabalho em caso de falha parcial (ex: rate limit no segundo campo).
pub async fn translate_description(
    app: &AppHandle,
    game_id: String,
    target_lang: Option<String>,
) -> Result<GameDescription, AppError> {
    let target_lang = match target_lang {
        Some(lang) => lang,
        None => get_or_detect_language(app)?,
    };

    info!(
        "Tradução solicitada para jogo {} -> idioma {}",
        game_id, target_lang
    );

    let state: State<AppState> = app.state();

    let current: GameDescription = {
        let conn = state.games_db.lock()?;
        fetch_description(&conn, &game_id)?
    };

    let fields = current
        .fields_to_display()
        .into_iter()
        .map(|(field, text)| (field, text.to_owned()))
        .collect::<Vec<_>>();

    if fields.is_empty() {
        return Ok(current);
    }

    let api_key = database::get_secret(app, "gemini_api_key").map_err(|e| {
        error!("Falha ao ler banco de secrets: {}", e);
        AppError::DatabaseError("Erro interno de banco de dados".to_string())
    })?;

    if api_key.is_empty() {
        return Err(AppError::ValidationError(
            "API Key do Gemini não configurada. Vá em Configurações.".to_string(),
        ));
    }

    let mut updated = current.clone();

    for (field, text) in fields {
        let already_done = current.translated_lang.as_deref() == Some(target_lang.as_str())
            && current.translated_value(field).is_some();

        if already_done {
            continue;
        }

        let translated = gemini::translate_single(&api_key, &target_lang, text.as_str())
            .await
            .map_err(|e| {
                error!(
                    "Falha ao traduzir campo '{}' do jogo {}: {}",
                    field, game_id, e
                );
                AppError::NetworkError(e)
            })?;

        updated.set_translated_value(field, translated.clone());
        updated.translated_lang = Some(target_lang.clone());

        let conn = state.games_db.lock()?;
        persist_translated_field(&conn, &game_id, field, &translated, &target_lang)?;
    }

    info!("Tradução concluída para jogo {}", game_id);

    Ok(updated)
}
