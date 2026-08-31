//! Query de jogos com detalhes (genres, tags, series, steam_app_id, release_year)
//! usados pelo sistema de recomendação.
//!
//! Camada pura de banco. O algoritmo de ranking fica em `services/recommendation`.

use crate::errors::AppError;
use crate::models::{Game, Library};
use crate::services::recommendation::{parse_release_year, GameWithDetails};
use rusqlite::Connection;

/// Busca todos os jogos da biblioteca com os campos relevantes para recomendação.
pub fn fetch_all_games_with_details(conn: &Connection) -> Result<Vec<GameWithDetails>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.slug, g.playtime, g.favorite, g.user_rating,
            (SELECT url FROM game_images WHERE game_id = g.id AND image_type = 'cover' ORDER BY priority ASC LIMIT 1) AS cover_url,
            g.library_game_id, g.last_played, g.added_at, g.library, g.playtime_source, g.alternative_names,
            gd.genres, gd.steam_app_id, gd.release_date, gd.series, gd.tags, gd.display_name
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
                library_game_id: row.get(7)?,
                last_played: row.get(8)?,
                added_at: row.get(9)?,
                library: row.get::<_, String>(10)?.parse().unwrap_or(Library::Outra),
                critic_score: None,
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
                source_label: None,
                release_date: None,
                display_name: row.get(18)?,
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
