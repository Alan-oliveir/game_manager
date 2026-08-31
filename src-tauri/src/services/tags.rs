//! Serviço para classificação e gerenciamento de tags de jogos
//!
//! Usa regras definidas em `tag_metadata.json` para classificar e filtrar tags obtidas da API RAWG.

use crate::utils::tag_utils::{GameTag, TagMetadata};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Manager};

const TAG_METADATA_JSON: &str = include_str!("../data/tag_metadata.json");

// Estrutura para estatísticas de tags
#[derive(Debug, Serialize)]
pub struct TagStats {
    pub total: usize,
    pub visible: usize,
    pub hidden: usize,
    pub by_category: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct AnalysisReport {
    timestamp: String,
    stats: TagStats,
    unknown_tags_count: usize,
    unknown_tags: Vec<String>,
}

// Cache estático das regras de tags
static TAG_METADATA: Lazy<HashMap<String, TagMetadata>> = Lazy::new(|| {
    load_tag_metadata().unwrap_or_else(|e| {
        eprintln!("Erro ao carregar tag_metadata.json: {}", e);
        HashMap::new()
    })
});

// Regex para normalização (compilada uma vez)
static NORMALIZE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"-(\d+)$").unwrap());

/// Carrega o metadata a partir da string embutida
fn load_tag_metadata() -> Result<HashMap<String, TagMetadata>, Box<dyn std::error::Error>> {
    let tags: Vec<TagMetadata> = serde_json::from_str(TAG_METADATA_JSON)?;

    let map = tags
        .into_iter()
        .map(|tag| (tag.slug.clone(), tag))
        .collect();

    Ok(map)
}

/// Normaliza slugs removendo sufixos numéricos (ex: "-2", "-3") e mantém números sem hífen (2d, 3d)
pub fn normalize_tag_slug(slug: &str) -> String {
    NORMALIZE_REGEX.replace(slug, "").to_string()
}

/// Classifica tags da RAWG usando as regras, normalizando slugs antes de buscar
pub fn classify_tags(raw_tags: Vec<String>) -> Vec<GameTag> {
    raw_tags
        .iter()
        .filter_map(|slug| {
            // Normaliza o slug removendo sufixos numéricos
            let normalized_slug = normalize_tag_slug(slug);

            TAG_METADATA.get(&normalized_slug).and_then(|metadata| {
                // Só retorna tags visíveis
                if metadata.visible {
                    Some(GameTag {
                        slug: metadata.slug.clone(),
                        name: metadata.name.clone(),
                        category: metadata.category.clone(),
                        role: metadata.role.clone(),
                        relevance: metadata.relevance as f32,
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Classifica e ordena tags por relevância, removendo duplicatas
pub fn classify_and_sort_tags(raw_tags: Vec<String>, limit: usize) -> Vec<GameTag> {
    let mut classified = classify_tags(raw_tags);

    // Remove duplicatas (tags com mesmo slug)
    let mut seen = HashSet::new();
    classified.retain(|tag| seen.insert(tag.slug.clone()));

    // Ordena por relevância (maior primeiro)
    classified.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limita quantidade
    classified.truncate(limit);

    classified
}

/// Retorna estatísticas das tags
pub fn get_tag_stats() -> TagStats {
    let total = TAG_METADATA.len();
    let visible = TAG_METADATA.values().filter(|t| t.visible).count();

    let by_category: HashMap<String, usize> =
        TAG_METADATA.values().fold(HashMap::new(), |mut acc, tag| {
            let category = format!("{:?}", tag.category);
            *acc.entry(category).or_insert(0) += 1;
            acc
        });

    TagStats {
        total,
        visible,
        hidden: total - visible,
        by_category,
    }
}

/// Gera relatório de análise com tags normalizadas
/// TODO - mudar o nome desta função para não confundir com a do sistema de recomendação com o mesmo nome.
pub fn generate_analysis_report(
    app: &AppHandle,
    encountered_slugs: HashSet<String>,
) -> Result<String, String> {
    let stats = get_tag_stats();

    // Normaliza e filtra tags desconhecidas
    let mut unknown_tags: Vec<String> = encountered_slugs
        .into_iter()
        .map(|slug| normalize_tag_slug(&slug))
        .filter(|normalized| !TAG_METADATA.contains_key(normalized))
        .collect::<HashSet<_>>() // Remove duplicatas
        .into_iter()
        .collect();

    unknown_tags.sort();

    let report = AnalysisReport {
        timestamp: chrono::Local::now().to_rfc3339(),
        stats,
        unknown_tags_count: unknown_tags.len(),
        unknown_tags,
    };

    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    let file_path = log_dir.join("tag_analysis_report.json");

    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}
