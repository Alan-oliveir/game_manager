//! Sistema de Recomendação v4.0 - Content-Based com Role-Based Tag System
//!
//! Este módulo implementa um sistema de recomendação híbrido que combina
//! filtragem baseada em conteúdo (content-based) com filtragem colaborativa (collaborative).
//!
//! **Organização dos Módulos:**
//! - `core`: Estruturas e tipos fundamentais
//! - `cf_aggregator`: Collaborative-filtering (CF) pré-computado
//! - `profile`: Cálculo de perfil de usuário
//! - `scoring`: Lógica de cálculo de scores
//! - `filtering`: Filtros e regras de diversidade
//! - `ranking`: Algoritmos de ranqueamento
//! - `analysis`: Análise detalhada e debugging
//! - `reports`: Exportação de relatórios
//!
//! **Melhorias v4.0:**
//! - Sistema de roles para tags (Affinity, Context, Filter, Diversity)
//! - Separação de estágios de recomendação
//! - Filtros duros antes do ranqueamento
//! - Penalização de séries/gêneros por seleção estrutural, não score
//! - CF puro sem penalizações matemáticas
//! - Cap individual por tag affinity
//! - Aumento de peso dos gêneros

pub mod analysis;
pub mod cf_aggregator;
pub mod core;
pub mod filtering;
pub mod orchestration;
pub mod profile;
pub mod ranking;
pub mod reports;
pub mod scoring;

// Reexportar os tipos e funções mais usados
pub use analysis::{
    generate_analysis_report, AnalysisStats, DetailedScoreBreakdown, GenreInfluence, ProfileStats,
    RecommendationAnalysisReport, TagInfluence, UserSettingsReport,
};
pub use core::{
    calculate_game_weight, parse_release_year, GameWithDetails, RecommendationConfig,
    RecommendationReason, SeriesLimit, UserPreferenceVector, UserSettings,
};
pub use filtering::{apply_diversity_rules, apply_hard_filters};
pub use orchestration::*;
pub use profile::calculate_user_profile;
pub use ranking::{rank_games_collaborative, rank_games_content_based, rank_games_hybrid};
pub use reports::{export_games_csv, export_report_json, export_report_txt};
pub use scoring::{
    normalize_score, score_game_cb, score_game_cb_detailed, DetailedScoreComponents,
};
