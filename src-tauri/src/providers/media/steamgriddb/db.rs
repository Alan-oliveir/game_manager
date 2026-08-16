use rusqlite::{params, Connection, OptionalExtension};

/// Insere/atualiza a capa de um jogo para uma fonte específica.
/// `priority` define a ordem de resolução: menor valor = maior preferência.
pub fn upsert_game_image<C: std::ops::Deref<Target=Connection>>(
    conn: &C,
    game_id: &str,
    source: &str, // "steamgriddb" | "igdb" | "steam"
    url: &str,
    thumb_url: Option<&str>,
    width: Option<u32>,
    height: Option<u32>,
    priority: i32,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO game_images (game_id, image_type, source, url, thumb_url, width, height, priority, fetched_at)
         VALUES (?1, 'cover', ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         ON CONFLICT(game_id, image_type, source) DO UPDATE SET
            url = excluded.url,
            thumb_url = excluded.thumb_url,
            width = excluded.width,
            height = excluded.height,
            fetched_at = excluded.fetched_at",
        params![game_id, source, url, thumb_url, width, height, priority],
    )?;
    Ok(())
}

pub fn get_cache_meta(
    conn: &Connection,
    game_id: &str,
) -> rusqlite::Result<Option<(String, bool)>> {
    conn.query_row(
        "SELECT checked_at, found FROM steamgriddb_cache_meta WHERE game_id = ?1",
        params![game_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0)),
    )
        .optional()
}

pub fn set_cache_meta(conn: &Connection, game_id: &str, found: bool) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO steamgriddb_cache_meta (game_id, checked_at, found)
         VALUES (?1, datetime('now'), ?2)
         ON CONFLICT(game_id) DO UPDATE SET checked_at = excluded.checked_at, found = excluded.found",
        params![game_id, found as i64],
    )?;
    Ok(())
}

pub fn has_any_cover<C: std::ops::Deref<Target=Connection>>(conn: &C, game_id: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM game_images WHERE game_id = ?1 AND image_type = 'cover' LIMIT 1",
        params![game_id],
        |_| Ok(()),
    )
        .optional()
        .ok()
        .flatten()
        .is_some()
}

/// Remove uma imagem de uma fonte específica. Usado quando o usuário limpa
/// a capa manual (envia cover_url = None) e queremos voltar ao fallback automático.
pub fn delete_game_image<C: std::ops::Deref<Target=Connection>>(
    conn: &C,
    game_id: &str,
    image_type: &str,
    source: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM game_images WHERE game_id = ?1 AND image_type = ?2 AND source = ?3",
        params![game_id, image_type, source],
    )?;
    Ok(())
}
