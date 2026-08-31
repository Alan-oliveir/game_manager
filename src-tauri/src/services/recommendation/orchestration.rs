//! Orquestração dos comandos de recomendação: busca dados (database), monta perfil,
//! aplica ranking (algoritmo já existente neste módulo) e formata a resposta.
//!
//! `GameRecommendation`/`RecommendationOptions` moram aqui (não em commands/) porque
//! representam o contrato da operação de recomendação, não só o formato de fio do Tauri.

use super::cf_aggregator::build_cf_candidates;
use super::{
    calculate_user_profile, rank_games_collaborative, rank_games_content_based, rank_games_hybrid,
    GameWithDetails, RecommendationConfig, RecommendationReason, SeriesLimit, UserPreferenceVector,
    UserSettings,
};
use crate::database::recommendation::fetch_all_games_with_details;
use crate::database::AppState;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
pub struct GameRecommendation {
    pub game_id: String,
    pub score: f32,
    pub reason: RecommendationReason,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RecommendationOptions {
    pub min_playtime: Option<i32>,
    pub max_playtime: Option<i32>,
    pub limit: usize,
    pub ignored_game_ids: Option<Vec<String>>,
    pub config: Option<RecommendationConfig>,
}

// === HELPERS INTERNOS ===

fn load_user_settings(app_handle: &AppHandle) -> UserSettings {
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

// === OPERAÇÕES ===

pub fn get_hybrid_recommendations(
    state: &AppState,
    app: &AppHandle,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = {
        let conn = state.games_db.lock()?;
        fetch_all_games_with_details(&conn)?
    };

    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let profile = calculate_user_profile(&games_with_details, &ignored_ids);
    let (cf_scores, _) = build_cf_candidates(&games_with_details);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let config = options.config.clone().unwrap_or_default();
    let user_settings = load_user_settings(app);

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

pub fn get_content_based_recommendations(
    state: &AppState,
    app: &AppHandle,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = {
        let conn = state.games_db.lock()?;
        fetch_all_games_with_details(&conn)?
    };

    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let profile = calculate_user_profile(&games_with_details, &ignored_ids);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let config = options.config.clone().unwrap_or_default();
    let user_settings = load_user_settings(app);
    let ranked = rank_games_content_based(&profile, &candidates, &config, &user_settings);

    Ok(format_recommendations(ranked, options.limit))
}

pub fn get_collaborative_recommendations(
    state: &AppState,
    app: &AppHandle,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    let games_with_details = {
        let conn = state.games_db.lock()?;
        fetch_all_games_with_details(&conn)?
    };

    let ignored_ids = create_ignored_set(options.ignored_game_ids.clone());
    let (cf_scores, _) = build_cf_candidates(&games_with_details);
    let candidates = filter_candidates_by_playtime(games_with_details, &options);
    let user_settings = load_user_settings(app);
    let ranked = rank_games_collaborative(&candidates, &cf_scores, &ignored_ids, &user_settings);

    Ok(format_recommendations(ranked, options.limit))
}

pub fn get_user_profile_vector(state: &AppState) -> Result<UserPreferenceVector, AppError> {
    let games_with_details = {
        let conn = state.games_db.lock()?;
        fetch_all_games_with_details(&conn)?
    };
    Ok(calculate_user_profile(&games_with_details, &HashSet::new()))
}
