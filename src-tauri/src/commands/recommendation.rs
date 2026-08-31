//! Comandos Tauri para Sistema de Recomendação v4.0
//!
//! Faz JOIN com game_details para obter genres, tags categorizadas e series.
//! Utiliza abordagem híbrida: perfil do usuário + collaborative filtering.
//! Permite configuração de filtros (playtime), pesos personalizados, feedback (blacklist).
//! Retorna razões detalhadas para cada recomendação.
//! Gera relatórios do sistema de recomendação.

use crate::constants::MINUTES_PER_HOUR_F32;
use crate::database::recommendation::fetch_all_games_with_details;
use crate::database::AppState;
use crate::errors::AppError;
use crate::services::recommendation::{
    calculate_user_profile, export_games_csv, export_report_json, export_report_txt,
    generate_analysis_report, GameWithDetails, RecommendationConfig, UserSettings,
};
use crate::services::recommendation::{
    get_collaborative_recommendations, get_content_based_recommendations,
    get_hybrid_recommendations, get_user_profile_vector, GameRecommendation, RecommendationOptions,
    UserPreferenceVector,
};
use serde::Serialize;
use std::collections::HashSet;
use tauri::Manager;
use tauri::{AppHandle, State};

// === STRUCTS ===

/// Resposta do comando de análise
#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub success: bool,
    pub json_path: Option<String>,
    pub csv_path: Option<String>,
    pub txt_path: Option<String>,
    pub message: String,
}

// === COMANDOS DE RECOMENDAÇÃO (produção — consumidos pela UI) ===

#[tauri::command]
pub async fn recommend_hybrid_library(
    app: AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    get_hybrid_recommendations(&state, &app, options)
}

#[tauri::command]
pub async fn recommend_from_library(
    app: AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    get_content_based_recommendations(&state, &app, options)
}

#[tauri::command]
pub async fn recommend_collaborative_library(
    app: AppHandle,
    state: State<'_, AppState>,
    options: RecommendationOptions,
) -> Result<Vec<GameRecommendation>, AppError> {
    get_collaborative_recommendations(&state, &app, options)
}

#[tauri::command]
pub async fn get_user_profile(
    state: State<'_, AppState>,
) -> Result<UserPreferenceVector, AppError> {
    get_user_profile_vector(&state)
}

// === COMANDOS DE ANÁLISE E GERAÇÃO DE RELATÓRIO ===

/// Gera análise completa do sistema de recomendação.
///
/// Cria três arquivos:
/// - `recommendation_analysis_TIMESTAMP.json` - Análise completa em JSON
/// - `recommendation_analysis_TIMESTAMP.txt` - Relatório legível em texto
/// - `recommendation_ranking_TIMESTAMP.csv` - Ranking em CSV para Excel
///
/// Os arquivos são salvos em `AppData/Local/Playlite/analysis/`
#[tauri::command]
pub async fn generate_recommendation_analysis(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<AnalysisResponse, String> {
    tracing::info!("Gerando análise de recomendação...");

    let analysis_dir = setup_analysis_directory(&app)?;
    let (json_path, txt_path, csv_path) = create_analysis_file_paths(&analysis_dir)?;

    let state: State<AppState> = app.state();

    let games_with_details = {
        let conn = state
            .games_db
            .lock()
            .map_err(|_| "Falha ao bloquear mutex do games_db".to_string())?;
        fetch_all_games_with_details(&conn).map_err(|e| e.to_string())?
    };

    tracing::info!("Total de jogos na biblioteca: {}", games_with_details.len());

    // `already_played_ids` (>5h jogadas ou favorito) é passado como `ignored_ids` para
    // `generate_analysis_report`, que filtra esses jogos internamente antes de calcular qualquer
    // score ou recomendação.
    let already_played_ids = compute_already_played_ids(&games_with_details);

    let profile = calculate_user_profile(&games_with_details, &HashSet::new());
    let (cf_scores, _) =
        crate::services::recommendation::cf_aggregator::build_cf_candidates(&games_with_details);

    let config = RecommendationConfig::default();
    let user_settings = UserSettings::default();

    let report = generate_analysis_report(
        &profile,
        &games_with_details,
        &cf_scores,
        &already_played_ids,
        config,
        user_settings,
    );

    let limited_report = limit_report(report, limit);

    export_analysis_reports(&limited_report, &json_path, &txt_path, &csv_path)?;

    log_success(&json_path, &txt_path, &csv_path);

    Ok(AnalysisResponse {
        success: true,
        json_path: Some(json_path.to_string_lossy().to_string()),
        txt_path: Some(txt_path.to_string_lossy().to_string()),
        csv_path: Some(csv_path.to_string_lossy().to_string()),
        message: format!(
            "Análise gerada com sucesso! {} jogos analisados.",
            limited_report.games.len()
        ),
    })
}

// === HELPERS (Relatórios) ===

fn setup_analysis_directory(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let analysis_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Erro ao obter diretório de dados: {}", e))?
        .join("analysis");

    std::fs::create_dir_all(&analysis_dir)
        .map_err(|e| format!("Erro ao criar diretório de análise: {}", e))?;

    Ok(analysis_dir)
}

fn create_analysis_file_paths(
    analysis_dir: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf), String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let json_path = analysis_dir.join(format!("recommendation_analysis_{}.json", timestamp));
    let txt_path = analysis_dir.join(format!("recommendation_analysis_{}.txt", timestamp));
    let csv_path = analysis_dir.join(format!("recommendation_ranking_{}.csv", timestamp));

    Ok((json_path, txt_path, csv_path))
}

/// Jogos considerados "já jogados" para fins do relatório: mais de 5h de playtime ou favoritos.
fn compute_already_played_ids(games: &[GameWithDetails]) -> HashSet<String> {
    games
        .iter()
        .filter(|g| {
            let hours = g.game.playtime.unwrap_or(0) as f32 / MINUTES_PER_HOUR_F32;
            hours > 5.0 || g.game.favorite
        })
        .map(|g| g.game.id.clone())
        .collect()
}

fn limit_report(
    mut report: crate::services::recommendation::RecommendationAnalysisReport,
    limit: Option<usize>,
) -> crate::services::recommendation::RecommendationAnalysisReport {
    if let Some(limit) = limit {
        report.games.truncate(limit);
    }
    report
}

fn export_analysis_reports(
    report: &crate::services::recommendation::RecommendationAnalysisReport,
    json_path: &std::path::Path,
    txt_path: &std::path::Path,
    csv_path: &std::path::Path,
) -> Result<(), String> {
    export_report_json(report, json_path.to_str().unwrap())
        .map_err(|e| format!("Erro ao salvar JSON: {}", e))?;

    export_report_txt(report, txt_path.to_str().unwrap())
        .map_err(|e| format!("Erro ao salvar TXT: {}", e))?;

    export_games_csv(&report.games, csv_path.to_str().unwrap())
        .map_err(|e| format!("Erro ao salvar CSV: {}", e))?;

    Ok(())
}

fn log_success(
    json_path: &std::path::Path,
    txt_path: &std::path::Path,
    csv_path: &std::path::Path,
) {
    tracing::info!("Análise gerada com sucesso!");
    tracing::info!("  JSON: {:?}", json_path);
    tracing::info!("  TXT:  {:?}", txt_path);
    tracing::info!("  CSV:  {:?}", csv_path);
}
