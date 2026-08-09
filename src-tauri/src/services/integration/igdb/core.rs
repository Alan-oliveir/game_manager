use crate::commands::metadata::shared::ProcessedGameDetails;
use crate::services::integration::igdb::models::IgdbGame;
use std::collections::HashMap;

pub struct IgdbDlc {
    pub igdb_id: i64,
    pub name: String,
    pub kind: &'static str, // "expansion" | "standalone_expansion"
}

pub struct IgdbMappedResult {
    pub details: ProcessedGameDetails,
    pub dlcs: Vec<IgdbDlc>,
}

/// Developer/publisher via as flags booleanas do IGDB — não por ordem/índice.
fn extract_developer_publisher(game: &IgdbGame) -> (Option<String>, Option<String>) {
    let developer = game
        .involved_companies
        .iter()
        .find(|c| c.developer)
        .map(|c| c.company.name.clone());
    let publisher = game
        .involved_companies
        .iter()
        .find(|c| c.publisher)
        .map(|c| c.company.name.clone());
    (developer, publisher)
}

fn build_age_ratings(game: &IgdbGame) -> HashMap<String, String> {
    game.age_ratings
        .iter()
        .filter_map(|ar| {
            let org = ar.organization.as_ref()?.name.clone();
            let rating = ar.rating_category.as_ref()?.rating.clone();
            Some((org, rating))
        })
        .collect()
}

/// Descarta títulos localizados/traduzidos e nomes de executável,
/// mantendo só variações que o próprio IGDB rotula como "Alternative".
fn valid_alternative_names(game: &IgdbGame) -> Vec<String> {
    game.alternative_names
        .iter()
        .filter(|alt| {
            if alt.name.to_lowercase().ends_with(".exe") {
                return false;
            }
            match &alt.comment {
                None => true,
                Some(c) => c.to_lowercase().contains("alternative"),
            }
        })
        .map(|alt| alt.name.clone())
        .collect()
}

/// Mescla links do IGDB no mapa existente sem sobrescrever o que já foi preenchido por outras fontes
/// (Ex.: Steam via steam_api) — segue o padrão `.entry().or_insert()` já usado.
fn normalize_link_key(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "official website" => "website".to_string(),
        "subreddit" => "reddit".to_string(),
        "community wiki" | "wiki" => "wiki".to_string(),
        other => other.replace(' ', "_"),
    }
}

fn merge_igdb_links(links_map: &mut HashMap<String, String>, game: &IgdbGame) {
    for site in &game.websites {
        // Só guarda Steam e os institucionais (site oficial, wiki, reddit) —
        // descarta consoles/mobile/redes sociais que não interessam pro Playlite.
        let Some(website_type) = site.website_type.as_ref() else {
            continue;
        };
        let key = normalize_link_key(&website_type.name);

        const ALLOWED: &[&str] = &[
            "steam",
            "website",
            "reddit",
            "wiki",
            "wikipedia",
            "twitter",
            "twitch",
            "youtube",
            "discord",
        ];
        if !ALLOWED.contains(&key.as_str()) {
            continue;
        }

        links_map.entry(key).or_insert_with(|| site.url.clone());
    }

    links_map.entry("igdb".to_string()).or_insert_with(|| {
        game.url
            .clone()
            .unwrap_or_else(|| format!("https://www.igdb.com/games/{}", game.slug))
    });
}

pub fn map_igdb_game(game: &IgdbGame, game_id: &str) -> IgdbMappedResult {
    let (developer, publisher) = extract_developer_publisher(game);
    let age_ratings = build_age_ratings(game);
    let alt_names = valid_alternative_names(game);

    let mut links_map: HashMap<String, String> = HashMap::new();
    merge_igdb_links(&mut links_map, game);

    let genres: Vec<String> = game.genres.iter().map(|g| g.name.clone()).collect();
    let themes: Option<Vec<String>> = (!game.themes.is_empty())
        .then(|| game.themes.iter().map(|t| t.name.clone()).collect());

    let series = game.collections.first().map(|c| c.name.clone());
    let franchise: Option<Vec<String>> = (!game.franchises.is_empty())
        .then(|| game.franchises.iter().map(|f| f.name.clone()).collect());

    let game_modes: Option<Vec<String>> = (!game.game_modes.is_empty())
        .then(|| game.game_modes.iter().map(|m| m.name.clone()).collect());
    let player_perspectives: Option<Vec<String>> =
        (!game.player_perspectives.is_empty()).then(|| {
            game.player_perspectives
                .iter()
                .map(|p| p.name.clone())
                .collect()
        });

    let raw_keyword_names: Vec<String> = game.keywords.iter().map(|k| k.name.clone()).collect();
    let keywords = (!raw_keyword_names.is_empty()).then(|| raw_keyword_names.clone());
    let tags = crate::services::tags::classify_and_sort_tags(raw_keyword_names, 10);

    let release_date = game.first_release_date.and_then(|ts| {
        chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
    });

    let cover_url = game.cover.as_ref().map(|c| {
        format!(
            "https://images.igdb.com/igdb/image/upload/t_1080p/{}.jpg",
            c.image_id
        )
    });

    let details = ProcessedGameDetails {
        game_id: game_id.to_string(),
        description: crate::models::GameDescription {
            summary: game.summary.clone(),
            storyline: game.storyline.clone(),
            short_description: None,
            description: None,
            description_ptbr: None,
        },
        release_date,
        genres,
        tags,
        developer,
        publisher,
        critic_score: game.aggregated_rating.map(|r| r.round() as i32),
        background_image: cover_url,
        series,
        steam_review_label: None,
        steam_review_count: None,
        steam_review_score: None,
        steam_review_updated_at: None,
        esrb_rating: age_ratings.get("ESRB").cloned(),
        is_adult: false,
        adult_tags: None,
        external_links: serde_json::to_string(&links_map).ok(),
        steam_app_id: None, // resolvido separadamente pela cadeia Steam em paralelo
        hltb_main_story: None,
        hltb_main_extra: None,
        hltb_completionist: None,
        hltb_coop_time: None,
        alternative_names: (!alt_names.is_empty()).then_some(alt_names),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        franchise,
        game_modes,
        player_perspectives,
        themes,
        keywords,
        age_ratings: serde_json::to_string(&age_ratings).ok(),
        display_name: Some(game.name.clone()),
    };

    let mut dlcs: Vec<IgdbDlc> = game
        .expansions
        .iter()
        .map(|e| IgdbDlc {
            igdb_id: e.id,
            name: e.name.clone(),
            kind: "expansion",
        })
        .collect();
    dlcs.extend(game.standalone_expansions.iter().map(|e| IgdbDlc {
        igdb_id: e.id,
        name: e.name.clone(),
        kind: "standalone_expansion",
    }));

    IgdbMappedResult { details, dlcs }
}

// === FUNÇÕES DE PERSISTÊNCIA ===

pub fn save_game_dlcs<C>(conn: &C, game_id: &str, dlcs: &[IgdbDlc]) -> Result<(), rusqlite::Error>
where
    C: std::ops::Deref<Target=rusqlite::Connection>,
{
    for dlc in dlcs {
        conn.execute(
            "INSERT INTO game_dlcs (game_id, igdb_id, name, kind, owned)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(game_id, igdb_id) DO UPDATE SET name = excluded.name, kind = excluded.kind",
            rusqlite::params![game_id, dlc.igdb_id, dlc.name, dlc.kind],
        )?;
    }
    Ok(())
}
