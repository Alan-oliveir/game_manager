use crate::providers::metadata::igdb::client::igdb_request;
use crate::providers::metadata::igdb::models::{IgdbGame, IgdbNamed};
use serde::Deserialize;
use tauri::AppHandle;

const CANONICAL_FIELDS: &str = "\
name, slug, summary, storyline, url, first_release_date, game_type, status, \
genres.name, themes.name, keywords.name, game_modes.name, player_perspectives.name, \
collections.name, franchises.name, \
involved_companies.company.name, involved_companies.developer, involved_companies.publisher, \
involved_companies.porting, involved_companies.supporting, \
age_ratings.organization.name, age_ratings.rating_category.rating, \
aggregated_rating, aggregated_rating_count, \
alternative_names.name, alternative_names.comment, \
cover.image_id, cover.url, \
game_engines.name, \
language_supports.language.name, language_supports.language_support_type.name, \
expansions.name, expansions.slug, expansions.cover.image_id, \
standalone_expansions.name, standalone_expansions.slug, standalone_expansions.cover.image_id, \
parent_game.name, version_parent.name, version_title, \
websites.url, websites.type.type";

// === STRUCTS ===

#[derive(Debug, Deserialize)]
struct IgdbGenreLookup {
    #[serde(default)]
    genres: Vec<IgdbNamed>,
    #[serde(default)]
    collections: Vec<IgdbNamed>,
}

// === FUNCTIONS ===

/// Rank de qualidade do candidato — menor é melhor. Usado como desempate
/// quando múltiplos resultados têm o mesmo nome exato (ex: jogo base vs.
/// port/versão cancelada com nome idêntico, como "Dead Space" PC vs Wii).
fn candidate_rank(game: &IgdbGame) -> (u8, u8) {
    let status_rank = match game.status {
        Some(6) | Some(7) | Some(8) => 1, // cancelled, rumored, delisted
        _ => 0,
    };
    let type_rank = if game.game_type == Some(0) { 0 } else { 1 }; // main_game primeiro
    (status_rank, type_rank)
}

async fn search_best_match(app: &AppHandle, name: &str) -> Result<Option<IgdbGame>, String> {
    let escaped = name.replace('"', "\\\"");
    let query = format!(
        "search \"{escaped}\"; fields {CANONICAL_FIELDS}; where game_type != (1,2,5,6,7,13); limit 10;"
    );
    let body = igdb_request(app, "games", &query).await?;
    let mut candidates: Vec<IgdbGame> = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    if candidates.is_empty() {
        return Ok(None);
    }

    let target = crate::utils::text::normalize_for_matching(name);
    let exact_indices: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, g)| crate::utils::text::normalize_for_matching(&g.name) == target)
        .map(|(i, _)| i)
        .collect();

    let pool = if !exact_indices.is_empty() {
        exact_indices
    } else {
        (0..candidates.len()).collect()
    };

    let best_idx = pool
        .into_iter()
        .min_by_key(|&i| candidate_rank(&candidates[i]))
        .unwrap();

    Ok(Some(candidates.remove(best_idx)))
}

/// Busca por nome exato ou por edição/versão base quando o resultado for uma variante
/// (ex: "BioShock Infinite: The Complete Edition" -> "BioShock Infinite").
pub async fn search_and_resolve(app: &AppHandle, name: &str) -> Result<Option<IgdbGame>, String> {
    if let Some(game) = search_best_match(app, name).await? {
        return Ok(Some(game));
    }

    let stripped = crate::utils::text::strip_edition_suffix(name);
    if stripped != name {
        return search_best_match(app, &stripped).await;
    }

    Ok(None)
}
