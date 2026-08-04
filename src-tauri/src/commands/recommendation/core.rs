//! Comandos Tauri para Sistema de Recomendação v4.0
//!
//! Faz JOIN com game_details para obter genres, tags categorizadas e series.
//! Utiliza abordagem híbrida: perfil do usuário + collaborative filtering.
//! Permite configuração de filtros (playtime), pesos personalizados, feedback (blacklist).
//! Retorna razões detalhadas para cada recomendação.

use crate::database::AppState;
use crate::errors::AppError;
use crate::models::{Game, Platform};
use crate::services::recommendation::cf_aggregator::build_cf_candidates;
use crate::services::recommendation::{
    calculate_user_profile, parse_release_year, rank_games_collaborative, rank_games_content_based,
    rank_games_hybrid, GameWithDetails, RecommendationConfig, RecommendationReason, SeriesLimit,
    UserPreferenceVector, UserSettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{Manager, State};

// === ESTRUTURAS DE DADOS ===

/// Estrutura completa de recomendação para o Frontend
#[derive(Debug, Serialize)]
pub struct GameRecommendation {
    pub game_id: String,
    pub score: f32,
    pub reason: RecommendationReason,
}

/// Struct auxiliar para input de configuração opcional
#[derive(Debug, Deserialize)]
pub struct RecommendationOptions {
    pub min_playtime: Option<i32>,
    pub max_playtime: Option<i32>,
    pub limit: usize,
    pub ignored_game_ids: Option<Vec<String>>, // Blacklist do feedback
    pub config: Option<RecommendationConfig>,  // Pesos personalizados
}

// === COMANDOS PRINCIPAIS ===

/// Comando Híbrido Principal (Usado na Playlist e Home)
///
/// Retorna recomendações de jogos baseadas em perfil do usuário + collaborative filtering.
#[tauri::command]
pub async fn recommend_hybrid_library(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = fetch_all_games_with_details(&state)?;
    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let profile = calculate_user_profile(&games_with_details, &ignored_ids);
    let (cf_scores, _) = build_cf_candidates(&games_with_details);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let config = options.config.unwrap_or_default();
    let user_settings = load_user_settings(&app);

    let ranked = rank_games_hybrid(
        &profile,
        &candidates,
        &cf_scores,
        &ignored_ids,
        config,
        user_settings,
    );

    Ok(format_recommendations(ranked, options.limit))
}

/// Recomendação Content-Based Pura (Biblioteca)
#[tauri::command]
pub async fn recommend_from_library(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = fetch_all_games_with_details(&state)?;
    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let profile = calculate_user_profile(&games_with_details, &ignored_ids);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let config = options.config.unwrap_or_default();
    let user_settings = load_user_settings(&app);
    let ranked = rank_games_content_based(&profile, &candidates, &config, &user_settings);

    Ok(format_recommendations(ranked, options.limit))
}

/// Recomendação Collaborative Pura
#[tauri::command]
pub async fn recommend_collaborative_library(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = fetch_all_games_with_details(&state)?;
    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let (cf_scores, _) = build_cf_candidates(&games_with_details);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let user_settings = load_user_settings(&app);
    let ranked = rank_games_collaborative(&candidates, &cf_scores, &ignored_ids, &user_settings);

    Ok(format_recommendations(ranked, options.limit))
}

/// Retorna informações sobre o perfil do usuário
#[tauri::command]
pub async fn get_user_profile(
    state: State<'_, AppState>,
) -> Result<UserPreferenceVector, AppError> {
    let games_with_details = fetch_all_games_with_details(&state)?;
    let ignored_ids = HashSet::new();
    let profile = calculate_user_profile(&games_with_details, &ignored_ids);
    Ok(profile)
}

// === FUNÇÕES AUXILIARES ===

/// Carrega as configurações de usuário do arquivo JSON
fn load_user_settings(app_handle: &tauri::AppHandle) -> UserSettings {
    // Tentar ler o arquivo de preferências
    let app_data_dir = match app_handle.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return UserSettings::default(),
    };

    let prefs_path = app_data_dir.join("user_preferences.json");

    if !prefs_path.exists() {
        return UserSettings::default();
    }

    let contents = match std::fs::read_to_string(&prefs_path) {
        Ok(c) => c,
        Err(_) => return UserSettings::default(),
    };

    let prefs: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(p) => p,
        Err(_) => return UserSettings::default(),
    };

    let filter_adult = prefs
        .get("filter_adult_content")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let series_limit_str = prefs
        .get("series_limit")
        .and_then(|v| v.as_str())
        .unwrap_or("moderate");

    let series_limit = match series_limit_str {
        "none" => SeriesLimit::None,
        "aggressive" => SeriesLimit::Aggressive,
        _ => SeriesLimit::Moderate,
    };

    UserSettings {
        filter_adult_content: filter_adult,
        series_limit,
    }
}

fn fetch_all_games_with_details(state: &AppState) -> Result<Vec<GameWithDetails>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
            g.id, g.name, g.slug, g.playtime, g.favorite, g.user_rating, g.cover_url,
            g.platform_game_id, g.last_played, g.added_at, g.platform, g.playtime_source, g.alternative_names,
            gd.genres, gd.steam_app_id, gd.release_date, gd.series, gd.tags
         FROM games g
         LEFT JOIN game_details gd ON g.id = gd.game_id
         ORDER BY g.name ASC",
    )?;

    let games: Result<Vec<GameWithDetails>, _> = stmt
        .query_map([], |row| {
            let alt_names_json: Option<String> = row.get(12)?;
            let alternative_names = alt_names_json.and_then(|s| serde_json::from_str(&s).ok());

            let game = Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                playtime: row.get(3)?,
                favorite: row.get(4)?,
                user_rating: row.get(5)?,
                cover_url: row.get(6)?,
                platform_game_id: row.get(7)?,
                last_played: row.get(8)?,
                added_at: row.get(9)?,
                platform: row.get::<_, String>(10)?.parse().unwrap_or(Platform::Outra),
                alternative_names,
                installed: false,
                import_confidence: None,
                install_path: None,
                executable_path: None,
                launch_args: None,
                status: None,
                playtime_source: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| s.parse().ok()),
                genres: None,
                developer: None,
                is_adult: false,
            };

            let genres_json: Option<String> = row.get(13)?;
            let genres: Vec<String> = genres_json
                .as_ref()
                .map(|s| {
                    if let Ok(vec) = serde_json::from_str::<Vec<String>>(s) {
                        vec
                    } else {
                        s.split(',')
                            .map(|g| g.trim().to_string())
                            .filter(|g| !g.is_empty())
                            .collect()
                    }
                })
                .unwrap_or_default();

            let steam_app_id_str: Option<String> = row.get(14)?;
            let steam_app_id: Option<u32> = steam_app_id_str.and_then(|s| s.parse().ok());

            let release_date: Option<String> = row.get(15)?;
            let release_year = release_date.and_then(|d| parse_release_year(&d));

            let series: Option<String> = row.get(16)?;

            let tags_json: Option<String> = row.get(17)?;
            let tags: Vec<crate::models::GameTag> = tags_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            Ok(GameWithDetails {
                game,
                genres,
                tags,
                series,
                release_year,
                steam_app_id,
            })
        })?
        .collect();

    games.map_err(|e| e.into())
}

fn create_ignored_set(ignored_game_ids: Option<Vec<String>>) -> HashSet<String> {
    ignored_game_ids.unwrap_or_default().into_iter().collect()
}

fn filter_candidates_by_playtime(
    games: Vec<GameWithDetails>,
    options: &RecommendationOptions,
) -> Vec<GameWithDetails> {
    let min = options.min_playtime.unwrap_or(0);
    let max = options.max_playtime.unwrap_or(999999);

    games
        .into_iter()
        .filter(|g| {
            let pt = g.game.playtime.unwrap_or(0);
            pt >= min && pt <= max
        })
        .collect()
}

fn format_recommendations(
    ranked: Vec<(GameWithDetails, f32, RecommendationReason)>,
    limit: usize,
) -> Vec<GameRecommendation> {
    ranked
        .into_iter()
        .take(limit)
        .map(|(g, score, reason)| GameRecommendation {
            game_id: g.game.id,
            score,
            reason,
        })
        .collect()
}
