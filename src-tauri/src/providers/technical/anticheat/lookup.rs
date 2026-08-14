use crate::providers::technical::anticheat::models::AnticheatInfo;
use rusqlite::{Connection, OptionalExtension, Row};
use strsim::jaro_winkler;

fn row_to_info(row: &Row) -> rusqlite::Result<AnticheatInfo> {
    let anticheats_json: String = row.get("anticheats")?;
    let anticheats: Vec<String> = serde_json::from_str(&anticheats_json).unwrap_or_default();

    Ok(AnticheatInfo {
        name: row.get("name")?,
        slug: row.get("slug")?,
        status: row.get("status")?,
        anticheats,
        native: row.get::<_, i32>("native")? != 0,
        reference: row.get("reference")?,
        date_changed: row.get("date_changed")?,
    })
}

fn query_by_steam_id(conn: &Connection, steam_id: &str) -> anyhow::Result<Option<AnticheatInfo>> {
    Ok(conn
        .query_row(
            "SELECT * FROM anticheat_games WHERE steam_id = ?1 LIMIT 1",
            rusqlite::params![steam_id],
            row_to_info,
        )
        .optional()?)
}

fn query_by_slug(conn: &Connection, slug: &str) -> anyhow::Result<Option<AnticheatInfo>> {
    Ok(conn
        .query_row(
            "SELECT * FROM anticheat_games WHERE slug = ?1 LIMIT 1",
            rusqlite::params![slug],
            row_to_info,
        )
        .optional()?)
}

fn query_all_names(conn: &Connection) -> anyhow::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT slug, name FROM anticheat_games")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn find_anticheat_info(
    conn: &Connection,
    steam_id: Option<&str>,
    game_name: &str,
    game_slug: &str,
) -> anyhow::Result<Option<AnticheatInfo>> {
    // 1. Match exato por steam_id (mais confiável)
    if let Some(sid) = steam_id {
        if let Some(info) = query_by_steam_id(conn, sid)? {
            return Ok(Some(info));
        }
    }

    // 2. Match exato por slug
    if let Some(info) = query_by_slug(conn, game_slug)? {
        return Ok(Some(info));
    }

    // 3. Fallback fuzzy por nome (mesmo threshold usado no Nexus)
    let candidates = query_all_names(conn)?;
    candidates
        .into_iter()
        .map(|(slug, name)| {
            (
                slug,
                jaro_winkler(&name.to_lowercase(), &game_name.to_lowercase()),
            )
        })
        .filter(|(_, score)| *score > 0.92)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .and_then(|(slug, _)| query_by_slug(conn, &slug).ok().flatten())
        .map(Ok)
        .transpose()
}
