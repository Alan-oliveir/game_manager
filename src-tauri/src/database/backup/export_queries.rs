//! Modulo para exportação de dados
//!
//! Este módulo concentra as funções responsáveis por ler os dados do SQLite e transformá-los nas estruturas para o backup.

use crate::database::{current_schema_version, AppState};
use crate::errors::AppError;
use crate::models::{
    Game, GameDataPath, GameDetails, GameExtras, Platform, SystemRequirements, WishlistGame,
};
use rusqlite::Connection;
use tauri::State;

/// Type alias para dados de backup
pub type BackupDataTuple = (
    Vec<Game>,
    Vec<GameDetails>,
    Vec<WishlistGame>,
    Vec<GameExtras>,
    Vec<SystemRequirements>,
    Vec<GameDataPath>,
    u32,
);

/// Função auxiliar interna para buscar dados do backup com transação ACID
pub fn fetch_backup_data(state: &State<AppState>) -> Result<BackupDataTuple, AppError> {
    let conn = state.games_db.lock()?;

    // Inicia transação READ para consistência
    conn.execute("BEGIN TRANSACTION", [])?;

    let games = fetch_games(&conn)?;
    let game_details = fetch_game_details(&conn)?;
    let wishlist_game = fetch_wishlist(&conn)?;
    let game_extras = fetch_game_extras(&conn)?;
    let system_requirements = fetch_system_requirements(&conn)?;
    let game_data_paths = fetch_game_data_paths(&conn)?;
    let schema_version = current_schema_version(&conn)?;

    conn.execute("COMMIT", [])?;

    Ok((
        games,
        game_details,
        wishlist_game,
        game_extras,
        system_requirements,
        game_data_paths,
        schema_version,
    ))
}

/// Busca todos os jogos na biblioteca
fn fetch_games(conn: &Connection) -> Result<Vec<Game>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, slug, cover_url, platform, platform_game_id, installed, import_confidence, install_path, executable_path, launch_args, user_rating, favorite, status, playtime, playtime_source, last_played, added_at, alternative_names, source_label FROM games"
    )?;

    let game_iter = stmt.query_map([], |row| {
        let alt_names_json: Option<String> = row.get(18)?;
        let alternative_names = alt_names_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Game {
            id: row.get(0)?,
            name: row.get(1)?,
            slug: row.get(2)?,
            cover_url: row.get(3)?,
            genres: None,
            developer: None,
            platform: row.get::<_, String>(4)?.parse().unwrap_or(Platform::Outra),
            platform_game_id: row.get(5)?,
            alternative_names,
            installed: row.get(6)?,
            import_confidence: row
                .get::<_, Option<String>>(7)?
                .and_then(|s| s.parse().ok()),
            install_path: row.get(8)?,
            executable_path: row.get(9)?,
            launch_args: row.get(10)?,
            user_rating: row.get(11)?,
            favorite: row.get(12)?,
            status: row.get(13)?,
            playtime: row.get(14)?,
            playtime_source: row
                .get::<_, Option<String>>(15)?
                .and_then(|s| s.parse().ok()),
            last_played: row.get(16)?,
            added_at: row.get(17)?,
            is_adult: false,
            source_label: row.get(19)?,
        })
    })?;

    Ok(game_iter.collect::<Result<Vec<_>, _>>()?)
}

/// Busca todos os detalhes dos jogos na biblioteca
fn fetch_game_details(conn: &Connection) -> Result<Vec<GameDetails>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT
        game_id, steam_app_id, display_name, developer, publisher, release_date, genres, themes,
        series, franchise, game_modes, player_perspectives, keywords, tags,
        summary, storyline, short_description, description_raw, description_ptbr, background_image,
        critic_score, steam_review_label, steam_review_count, steam_review_score, steam_review_updated_at,
        esrb_rating, age_ratings, is_adult, adult_tags, external_links, hltb_main_story,
        hltb_main_extra, hltb_completionist, hltb_coop_time, updated_at
     FROM game_details",
    )?;

    // Auxiliares para ler JSON do banco e converter para Vec ou HashMap
    let parse_json_vec = |s: Option<String>| -> Option<Vec<String>> {
        s.and_then(|v| serde_json::from_str(&v).ok())
    };

    let parse_json_map = |s: Option<String>| -> Option<std::collections::HashMap<String, String>> {
        s.and_then(|v| serde_json::from_str(&v).ok())
    };

    let details_iter = stmt.query_map([], |row| {
        let tags_json: Option<String> = row.get(13)?;
        let tags = tags_json.map(|s| crate::database::deserialize_tags(&s));

        Ok(GameDetails {
            game_id: row.get(0)?,
            steam_app_id: row.get(1)?,
            display_name: row.get(2)?,
            developer: row.get(3)?,
            publisher: row.get(4)?,
            release_date: row.get(5)?,
            genres: row.get(6)?,
            themes: parse_json_vec(row.get(7)?),
            series: row.get(8)?,
            franchise: parse_json_vec(row.get(9)?),
            game_modes: parse_json_vec(row.get(10)?),
            player_perspectives: parse_json_vec(row.get(11)?),
            keywords: parse_json_vec(row.get(12)?),
            tags,
            description: crate::models::GameDescription {
                summary: row.get(14)?,
                storyline: row.get(15)?,
                short_description: row.get(16)?,
                description: row.get(17)?,
                description_ptbr: row.get(18)?,
            },
            background_image: row.get(19)?,
            critic_score: row.get(20)?,
            steam_review_label: row.get(21)?,
            steam_review_count: row.get(22)?,
            steam_review_score: row.get(23)?,
            steam_review_updated_at: row.get(24)?,
            esrb_rating: row.get(25)?,
            age_ratings: parse_json_map(row.get(26)?),
            is_adult: row.get(27).unwrap_or(false),
            adult_tags: row.get(28)?,
            external_links: parse_json_map(row.get(29)?),
            hltb_main_story: row.get(30)?,
            hltb_main_extra: row.get(31)?,
            hltb_completionist: row.get(32)?,
            hltb_coop_time: row.get(33)?,
            updated_at: row.get(34)?,
        })
    })?;

    Ok(details_iter.collect::<Result<Vec<_>, _>>()?)
}

/// Busca todos os jogos da wishlist
fn fetch_wishlist(conn: &Connection) -> Result<Vec<WishlistGame>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, cover_url, store_url, store_platform, itad_id, current_price, normal_price, lowest_price, currency, on_sale, voucher, added_at FROM wishlist"
    )?;

    let wishlist_iter = stmt.query_map([], |row| {
        Ok(WishlistGame {
            id: row.get(0)?,
            name: row.get(1)?,
            cover_url: row.get(2)?,
            store_url: row.get(3)?,
            store_platform: row.get(4)?,
            itad_id: row.get(5)?,
            current_price: row.get(6)?,
            normal_price: row.get(7)?,
            lowest_price: row.get(8)?,
            currency: row.get(9)?,
            on_sale: row.get(10)?,
            voucher: row.get(11)?,
            added_at: row.get(12)?,
        })
    })?;

    Ok(wishlist_iter.collect::<Result<Vec<_>, _>>()?)
}

/// Busca todos os dados técnicos do PCGamingWiki para backup
fn fetch_game_extras(conn: &Connection) -> Result<Vec<GameExtras>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT
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
         FROM game_extras
         WHERE fetched_at IS NOT NULL",
    )?;

    let parse_json_vec = |s: Option<String>| -> Option<Vec<String>> {
        s.and_then(|v| serde_json::from_str(&v).ok())
    };

    let iter = stmt.query_map([], |row| {
        Ok(GameExtras {
            steam_app_id: row.get(0)?,
            pcgw_page_id: row.get(1)?,
            pcgw_page_name: row.get(2)?,
            engine: row.get(3)?,
            available_on: row.get(4)?,
            dx_versions: row.get(5)?,
            vulkan_versions: row.get(6)?,
            opengl_versions: row.get(7)?,
            win64: row.get(8)?,
            linux64: row.get(9)?,
            macos_arm: row.get(10)?,
            macos_intel64: row.get(11)?,
            ray_tracing: row.get(12)?,
            upscaling: row.get(13)?,
            frame_gen: row.get(14)?,
            ultrawidescreen: row.get(15)?,
            four_k_support: row.get(16)?,
            hdr: row.get(17)?,
            high_fps: row.get(18)?,
            fov: row.get(19)?,
            borderless_windowed: row.get(20)?,
            color_blind: row.get(21)?,
            controller_support: row.get(22)?,
            full_controller: row.get(23)?,
            playstation_controllers: row.get(24)?,
            xinput_controllers: row.get(25)?,
            surround_sound: row.get(26)?,
            subtitles: row.get(27)?,
            closed_captions: row.get(28)?,
            has_save_data: row.get(29)?,
            has_config_data: row.get(30)?,
            languages_interface: parse_json_vec(row.get(31)?),
            languages_audio: parse_json_vec(row.get(32)?),
            languages_subtitles: parse_json_vec(row.get(33)?),
            fetched_at: row.get(34)?,
        })
    })?;

    Ok(iter.collect::<Result<Vec<_>, _>>()?)
}

/// Busca todos os requisitos de sistema para backup
fn fetch_system_requirements(conn: &Connection) -> Result<Vec<SystemRequirements>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT
            steam_app_id, os_family, tier_title, target,
            min_os, min_cpu, min_cpu2, min_ram, min_gpu, min_gpu2, min_vram, min_dx, min_storage,
            rec_os, rec_cpu, rec_cpu2, rec_ram, rec_gpu, rec_gpu2, rec_vram, rec_dx, rec_storage
         FROM system_requirements
         ORDER BY steam_app_id, id ASC",
    )?;

    let iter = stmt.query_map([], |row| {
        Ok(SystemRequirements {
            steam_app_id: row.get(0)?,
            os_family: row.get(1)?,
            tier_title: row.get(2)?,
            target: row.get(3)?,
            min_os: row.get(4)?,
            min_cpu: row.get(5)?,
            min_cpu2: row.get(6)?,
            min_ram: row.get(7)?,
            min_gpu: row.get(8)?,
            min_gpu2: row.get(9)?,
            min_vram: row.get(10)?,
            min_dx: row.get(11)?,
            min_storage: row.get(12)?,
            rec_os: row.get(13)?,
            rec_cpu: row.get(14)?,
            rec_cpu2: row.get(15)?,
            rec_ram: row.get(16)?,
            rec_gpu: row.get(17)?,
            rec_gpu2: row.get(18)?,
            rec_vram: row.get(19)?,
            rec_dx: row.get(20)?,
            rec_storage: row.get(21)?,
        })
    })?;

    Ok(iter.collect::<Result<Vec<_>, _>>()?)
}

/// Busca todos os caminhos de game data para backup
fn fetch_game_data_paths(conn: &Connection) -> Result<Vec<GameDataPath>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT steam_app_id, kind, os, raw_path
         FROM game_data_paths
         ORDER BY steam_app_id, id ASC",
    )?;

    let iter = stmt.query_map([], |row| {
        Ok(GameDataPath {
            steam_app_id: row.get(0)?,
            kind: row.get(1)?,
            os: row.get(2)?,
            raw_path: row.get(3)?,
            expanded_path: None,
        })
    })?;

    Ok(iter.collect::<Result<Vec<_>, _>>()?)
}
