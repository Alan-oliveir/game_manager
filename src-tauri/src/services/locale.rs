//! Serviço de detecção e persistência de idioma/região da aplicação.
//!
//! Idioma: usado como alvo de tradução (ver `services::translation`).
//! Região: usada para localizar preços via ITAD (ver `services::wishlist`).
//!
//! Ambos seguem o mesmo padrão: se já houver preferência persistida em
//! `app_config`, usa ela; senão, detecta via `sys-locale` (locale do SO),
//! persiste e retorna. Podem ser sobrescritos manualmente pelo usuário.

use crate::constants::{
    CONFIG_KEY_LANGUAGE, CONFIG_KEY_REGION, DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES,
};
use crate::database::configs::{get_config, set_config};
use crate::database::AppState;
use crate::errors::AppError;
use sys_locale::get_locale;
use tauri::{AppHandle, Manager, State};

// === REGIÃO ===

/// Detecta a região a partir do locale do sistema operacional (BCP 47).
/// Fallback "US" se não conseguir detectar ou vier um formato inesperado.
fn detect_region_from_system() -> String {
    get_locale()
        .and_then(|tag| {
            tag.split(['-', '_'])
                .nth(1)
                .map(|region| region.to_uppercase())
        })
        .filter(|r| r.len() == 2)
        .unwrap_or_else(|| "US".to_string())
}

/// Retorna a região configurada em app_config. Se ainda não existir,
/// detecta via sys-locale, persiste e retorna o valor detectado.
pub fn get_or_detect_region(app: &AppHandle) -> Result<String, AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    if let Some(region) = get_config(&conn, CONFIG_KEY_REGION)? {
        return Ok(region);
    }

    let detected = detect_region_from_system();
    set_config(&conn, CONFIG_KEY_REGION, &detected)?;
    Ok(detected)
}

/// Permite sobrescrever a região manualmente.
pub fn set_region(app: &AppHandle, region: &str) -> Result<(), AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    set_config(&conn, CONFIG_KEY_REGION, &region.to_uppercase())
}

// === IDIOMA ===

/// Detecta o idioma a partir do locale do sistema, usado só como fallback
/// antes do frontend ter avisado o backend qual idioma está ativo
/// (ex: em uma tarefa de background que roda antes do primeiro invoke).
fn detect_language_from_system() -> String {
    get_locale()
        .and_then(|tag| tag.split(['-', '_']).next().map(|l| l.to_lowercase()))
        .map(|lang| match lang.as_str() {
            "pt" => "pt-BR".to_string(),
            "en" => "en".to_string(),
            _ => DEFAULT_LANGUAGE.to_string(),
        })
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
}

/// Retorna o idioma configurado em app_config. Se ainda não existir,
/// detecta via sys-locale, persiste e retorna o valor detectado.
pub fn get_or_detect_language(app: &AppHandle) -> Result<String, AppError> {
    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;

    if let Some(lang) = get_config(&conn, CONFIG_KEY_LANGUAGE)? {
        return Ok(lang);
    }

    let detected = detect_language_from_system();
    set_config(&conn, CONFIG_KEY_LANGUAGE, &detected)?;
    Ok(detected)
}

/// Define o idioma explicitamente. Chamado pelo frontend sempre que o
/// i18n mudar de idioma, pra manter o backend sincronizado.
pub fn set_language(app: &AppHandle, language: &str) -> Result<(), AppError> {
    if !SUPPORTED_LANGUAGES.contains(&language) {
        return Err(AppError::ValidationError(format!(
            "Idioma não suportado: {}",
            language
        )));
    }

    let state: State<AppState> = app.state();
    let conn = state.cache_db.lock().map_err(|_| AppError::MutexError)?;
    set_config(&conn, CONFIG_KEY_LANGUAGE, language)
}
