//! Módulo responsável por restaurar os dados do backup no banco de dados SQLite.
//!
//! Este módulo isola toda a lógica pesada de inserção/restauração dentro de uma transação ACID atômica,
//! garantindo que o banco de dados permaneça consistente mesmo em caso de falhas durante a operação.

use crate::database::backup::models::BackupData;
use crate::errors::AppError;
use chrono::Utc;
use rusqlite::{params, Connection};

/// Restaura todos os dados do backup no banco de dados usando uma transação única
pub fn restore_backup_data(conn: &Connection, backup: &BackupData) -> Result<String, AppError> {
    // Transação única para todas as operações
    conn.execute("BEGIN IMMEDIATE TRANSACTION", [])?;

    let serialize_vec = |v: &Option<Vec<String>>| -> Option<String> {
        v.as_ref().and_then(|list| serde_json::to_string(list).ok())
    };

    let serialize_map = |v: &Option<std::collections::HashMap<String, String>>| -> Option<String> {
        v.as_ref().and_then(|map| serde_json::to_string(map).ok())
    };

    // Prepared statements para melhor desempenho
    let mut game_stmt = conn.prepare(
        "INSERT OR REPLACE INTO games (id, name, cover_url, platform, platform_game_id, installed, import_confidence, install_path, executable_path, launch_args, user_rating, favorite, status, playtime, last_played, added_at, alternative_names, source_label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)"
    )?;

    let mut details_stmt = conn.prepare(
        "INSERT OR REPLACE INTO game_details (
            game_id, steam_app_id, display_name, developer, publisher, release_date, genres, themes,
            series, franchise, game_modes, player_perspectives, keywords, tags,
            critic_score, steam_review_label, steam_review_count, steam_review_score, steam_review_updated_at,
            age_ratings, is_adult, adult_tags, external_links, hltb_main_story,
            hltb_main_extra, hltb_completionist, hltb_coop_time, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
            ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28
        )"
    )?;

    let mut descriptions_stmt = conn.prepare(
        "INSERT OR REPLACE INTO game_descriptions (
            game_id, summary, storyline, short_description, description,
            summary_translated, storyline_translated, short_description_translated, description_translated, translated_lang
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
        )"
    )?;

    let mut wishlist_stmt = conn.prepare(
        "INSERT OR REPLACE INTO wishlist (id, name, cover_url, store_url, store_platform, current_price, normal_price, lowest_price, currency, on_sale, voucher, added_at, itad_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )?;

    for game in &backup.games {
        let alt_names_json = game
            .alternative_names
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        game_stmt.execute(params![
            game.id,
            game.name,
            game.cover_url,
            game.platform.to_string(),
            game.platform_game_id,
            game.installed,
            game.import_confidence.as_ref().map(|ic| ic.to_string()),
            game.install_path,
            game.executable_path,
            game.launch_args,
            game.user_rating,
            game.favorite,
            game.status,
            game.playtime,
            game.last_played,
            game.added_at,
            alt_names_json,
            game.source_label,
        ])?;
    }

    for detail in &backup.game_details {
        let tags_json = detail
            .tags
            .as_ref()
            .and_then(|tags| crate::database::serialize_tags(tags).ok());

        details_stmt.execute(params![
            detail.game_id,
            detail.steam_app_id,
            detail.display_name,
            detail.developer,
            detail.publisher,
            detail.release_date,
            detail.genres,
            serialize_vec(&detail.themes),
            detail.series,
            serialize_vec(&detail.franchise),
            serialize_vec(&detail.game_modes),
            serialize_vec(&detail.player_perspectives),
            serialize_vec(&detail.keywords),
            tags_json,
            detail.critic_score,
            detail.steam_review_label,
            detail.steam_review_count,
            detail.steam_review_score,
            detail.steam_review_updated_at,
            serialize_map(&detail.age_ratings),
            detail.is_adult,
            detail.adult_tags,
            serialize_map(&detail.external_links),
            detail.hltb_main_story,
            detail.hltb_main_extra,
            detail.hltb_completionist,
            detail.hltb_coop_time,
            detail.updated_at
        ])?;

        descriptions_stmt.execute(params![
            detail.game_id,
            detail.description.summary,
            detail.description.storyline,
            detail.description.short_description,
            detail.description.description,
            detail.description.summary_translated,
            detail.description.storyline_translated,
            detail.description.short_description_translated,
            detail.description.description_translated,
            detail.description.translated_lang,
        ])?;
    }

    for item in &backup.wishlist_game {
        wishlist_stmt.execute(params![
            item.id,
            item.name,
            item.cover_url,
            item.store_url,
            item.store_platform,
            item.current_price,
            item.normal_price,
            item.lowest_price,
            item.currency,
            item.on_sale,
            item.voucher,
            item.added_at,
            item.itad_id
        ])?;
    }

    let mut extras_stmt = conn.prepare(
        "INSERT OR REPLACE INTO game_extras (
        steam_app_id, pcgw_page_id, pcgw_page_name, engine,
        available_on,
        dx_versions, vulkan_versions, opengl_versions,
        win64, linux64, macos_arm, macos_intel64,
        ray_tracing, upscaling, frame_gen,
        ultrawidescreen, four_k_support, hdr, high_fps, fov, borderless_windowed, color_blind,
        controller_support, full_controller, playstation_controllers, xinput_controllers,
        surround_sound, subtitles, closed_captions,
        has_save_data, has_config_data,
        languages_interface, languages_audio, languages_subtitles,
        fetched_at
    ) VALUES (
        ?1,  ?2,  ?3,  ?4,
        ?5,
        ?6,  ?7,  ?8,
        ?9,  ?10, ?11, ?12,
        ?13, ?14, ?15,
        ?16, ?17, ?18, ?19, ?20, ?21, ?22,
        ?23, ?24, ?25, ?26,
        ?27, ?28, ?29,
        ?30, ?31,
        ?32, ?33, ?34,
        ?35
    )",
    )?;

    let mut sysreq_stmt = conn.prepare(
        "INSERT INTO system_requirements (
            steam_app_id, os_family, tier_title, target,
            min_os, min_cpu, min_cpu2, min_ram, min_gpu, min_gpu2, min_vram, min_dx, min_storage,
            rec_os, rec_cpu, rec_cpu2, rec_ram, rec_gpu, rec_gpu2, rec_vram, rec_dx, rec_storage,
            fetched_at
        ) VALUES (
            ?1,  ?2,  ?3,  ?4,
            ?5,  ?6,  ?7,  ?8,  ?9,  ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
            ?23
        )",
    )?;

    let mut paths_stmt = conn.prepare(
        "INSERT INTO game_data_paths (steam_app_id, kind, os, raw_path, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;

    let now = Utc::now().to_rfc3339();

    // Limpa dados anteriores das tabelas com múltiplas linhas por jogo antes de reinserir
    conn.execute("DELETE FROM system_requirements", [])?;
    conn.execute("DELETE FROM game_data_paths", [])?;

    for extras in &backup.game_extras {
        extras_stmt.execute(params![
            extras.steam_app_id,
            extras.pcgw_page_id,
            extras.pcgw_page_name,
            extras.engine,
            extras.available_on,
            extras.dx_versions,
            extras.vulkan_versions,
            extras.opengl_versions,
            extras.win64,
            extras.linux64,
            extras.macos_arm,
            extras.macos_intel64,
            extras.ray_tracing,
            extras.upscaling,
            extras.frame_gen,
            extras.ultrawidescreen,
            extras.four_k_support,
            extras.hdr,
            extras.high_fps,
            extras.fov,
            extras.borderless_windowed,
            extras.color_blind,
            extras.controller_support,
            extras.full_controller,
            extras.playstation_controllers,
            extras.xinput_controllers,
            extras.surround_sound,
            extras.subtitles,
            extras.closed_captions,
            extras.has_save_data,
            extras.has_config_data,
            serialize_vec(&extras.languages_interface),
            serialize_vec(&extras.languages_audio),
            serialize_vec(&extras.languages_subtitles),
            extras.fetched_at,
        ])?;
    }

    for req in &backup.system_requirements {
        sysreq_stmt.execute(params![
            req.steam_app_id,
            req.os_family,
            req.tier_title,
            req.target,
            req.min_os,
            req.min_cpu,
            req.min_cpu2,
            req.min_ram,
            req.min_gpu,
            req.min_gpu2,
            req.min_vram,
            req.min_dx,
            req.min_storage,
            req.rec_os,
            req.rec_cpu,
            req.rec_cpu2,
            req.rec_ram,
            req.rec_gpu,
            req.rec_gpu2,
            req.rec_vram,
            req.rec_dx,
            req.rec_storage,
            now,
        ])?;
    }

    for path in &backup.game_data_paths {
        paths_stmt.execute(params![
            path.steam_app_id,
            path.kind,
            path.os,
            path.raw_path,
            now,
        ])?;
    }

    conn.execute("COMMIT", [])?;

    Ok(format!(
        "Backup restaurado! {} jogos, {} detalhes, {} itens da wishlist, {} dados técnicos, {} requisitos de sistema e {} caminhos.",
        backup.games.len(),
        backup.game_details.len(),
        backup.wishlist_game.len(),
        backup.game_extras.len(),
        backup.system_requirements.len(),
        backup.game_data_paths.len(),
    ))
}
