//! Comandos para análise e debug do sistema de recomendação
//!
//! Este módulo fornece comandos Tauri para gerar relatórios de análise
//! do sistema de recomendação, úteis para debug e ajuste fino dos algoritmos.

use crate::constants::MINUTES_PER_HOUR_F32;
use crate::database::AppState;
use crate::errors::AppError;
use crate::models::Platform;
use crate::services::recommendation::{
    calculate_user_profile, export_games_csv, export_report_json, export_report_txt,
    generate_analysis_report, parse_release_year, GameWithDetails, RecommendationConfig,
    UserSettings,
};
use serde::Serialize;
use std::collections::HashSet;
use tauri::{AppHandle, Manager, State};

/// Type alias para dados de recomendação preparados
type PreparedRecommendationData = (Vec<GameWithDetails>, Vec<GameWithDetails>, HashSet<String>);

/// Resposta do comando de análise
#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    pub success: bool,
    pub json_path: Option<String>,
    pub csv_path: Option<String>,
    pub txt_path: Option<String>,
    pub message: String,
}

/// Gera análise completa do sistema de recomendação
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
    let (candidates_with_details, all_games_with_details, already_played_ids) =
        fetch_and_prepare_data(&state)?;

    let profile = calculate_user_profile(&all_games_with_details, &HashSet::new());
    let (cf_scores, _) =
        crate::services::recommendation::cf_aggregator::build_cf_candidates(&all_games_with_details);

    let config = RecommendationConfig::default();
    let user_settings = UserSettings::default();

    let report = generate_analysis_report(
        &profile,
        &candidates_with_details,
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

// === FUNÇÕES AUXILIARES ===

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

fn fetch_and_prepare_data(state: &State<AppState>) -> Result<PreparedRecommendationData, String> {
    let library_games = crate::commands::games::get_games(state.clone())
        .map_err(|e| format!("Erro ao buscar jogos da biblioteca: {}", e))?;

    tracing::info!("Total de jogos na biblioteca: {}", library_games.len());

    let already_played_ids: HashSet<String> = library_games
        .iter()
        .filter(|g| {
            let hours = g.playtime.unwrap_or(0) as f32 / MINUTES_PER_HOUR_F32;
            hours > 5.0 || g.favorite
        })
        .map(|g| g.id.clone())
        .collect();

    let candidate_games: Vec<_> = library_games
        .iter()
        .filter(|g| !already_played_ids.contains(&g.id))
        .cloned()
        .collect();

    tracing::info!("Candidatos para recomendação: {}", candidate_games.len());

    let candidates_with_details = fetch_games_with_details(&candidate_games, state)
        .map_err(|e| format!("Erro ao processar candidatos: {}", e))?;

    let all_games_with_details = fetch_games_with_details(&library_games, state)
        .map_err(|e| format!("Erro ao processar biblioteca completa: {}", e))?;

    Ok((
        candidates_with_details,
        all_games_with_details,
        already_played_ids,
    ))
}

fn fetch_games_with_details(
    _games: &[crate::models::Game],
    state: &State<AppState>,
) -> Result<Vec<GameWithDetails>, AppError> {
    let conn = state.games_db.lock()?;

    let mut stmt = conn.prepare(
        "SELECT
            g.id, g.name, g.slug, g.playtime, g.favorite, g.user_rating, g.cover_url,
            g.platform_game_id, g.last_played, g.added_at, g.platform, g.playtime_source,
            gd.genres, gd.steam_app_id, gd.release_date, gd.series, gd.tags
         FROM games g
         LEFT JOIN game_details gd ON g.id = gd.game_id
         ORDER BY g.name ASC",
    )?;

    let games_with_details: Result<Vec<GameWithDetails>, _> = stmt
        .query_map([], |row| {
            let game = crate::models::Game {
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
                // Campos não utilizados
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

            let genres_json: Option<String> = row.get(12)?;
            let genres: Vec<String> = genres_json
                .as_ref()
                .map(|s| {
                    // Tentar parsear como JSON primeiro
                    if let Ok(vec) = serde_json::from_str::<Vec<String>>(s) {
                        vec
                    } else {
                        // Fallback: parsear como comma-separated string
                        s.split(',')
                            .map(|g| g.trim().to_string())
                            .filter(|g| !g.is_empty())
                            .collect()
                    }
                })
                .unwrap_or_default();

            let steam_app_id_str: Option<String> = row.get(13)?;
            let steam_app_id: Option<u32> = steam_app_id_str.and_then(|s| s.parse().ok());

            let release_date: Option<String> = row.get(14)?;
            let release_year = release_date.and_then(|d| parse_release_year(&d));
            let series: Option<String> = row.get(15)?;

            // Buscar tags do JSON na coluna tags
            let tags_json: Option<String> = row.get(16)?;
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

    games_with_details.map_err(|e| e.into())
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
