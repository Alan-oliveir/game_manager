//! Persistência dos dados do PCGamingWiki no banco SQLite local.
//!
//! Os dados são armazenados na tabela `game_extras` sem TTL — tratados como
//! características fixas do jogo e atualizados apenas por invalidação explícita
//! via [`invalidate_pcgw_data`].

use crate::models::{GameDataPath, GameExtras, PcgwScrapedData, SystemRequirements};
use rusqlite::{params, Connection};
use tracing::{info, warn};

// === INICIALIZAÇÃO DO BANCO ===

/// Cria a tabela `game_extras`, `system_requirements` e `game_data_paths` em `games.db` se ainda não existirem.
///
/// Chamada uma vez durante a inicialização do banco principal.
pub fn initialize_pcgamingwiki_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS game_extras (
            steam_app_id            TEXT PRIMARY KEY,
            pcgw_page_id            TEXT,
            pcgw_page_name          TEXT,
            engine                  TEXT,
            available_on            TEXT,
            dx_versions             TEXT,
            vulkan_versions         TEXT,
            opengl_versions         TEXT,
            win64                   TEXT,
            linux64                 TEXT,
            macos_arm               TEXT,
            macos_intel64           TEXT,
            ray_tracing             TEXT,
            upscaling               TEXT,
            frame_gen               TEXT,
            ultrawidescreen         TEXT,
            four_k_support          TEXT,
            hdr                     TEXT,
            high_fps                TEXT,
            fov                     TEXT,
            borderless_windowed     TEXT,
            color_blind             TEXT,
            controller_support      TEXT,
            full_controller         TEXT,
            playstation_controllers TEXT,
            xinput_controllers      TEXT,
            surround_sound          TEXT,
            subtitles               TEXT,
            closed_captions         TEXT,
            has_save_data           TEXT,
            has_config_data         TEXT,
            languages_interface     TEXT,
            languages_audio         TEXT,
            languages_subtitles     TEXT,
            fetched_at              TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_game_extras_fetched_at
            ON game_extras(fetched_at);

        CREATE TABLE IF NOT EXISTS system_requirements (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            steam_app_id TEXT NOT NULL,
            os_family    TEXT NOT NULL,
            tier_title   TEXT,
            target       TEXT,
            min_os       TEXT,
            min_cpu      TEXT,
            min_cpu2     TEXT,
            min_ram      TEXT,
            min_gpu      TEXT,
            min_gpu2     TEXT,
            min_vram     TEXT,
            min_dx       TEXT,
            min_storage  TEXT,
            rec_os       TEXT,
            rec_cpu      TEXT,
            rec_cpu2     TEXT,
            rec_ram      TEXT,
            rec_gpu      TEXT,
            rec_gpu2     TEXT,
            rec_vram     TEXT,
            rec_dx       TEXT,
            rec_storage  TEXT,
            fetched_at   TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sysreq_app_id
            ON system_requirements(steam_app_id);

        CREATE TABLE IF NOT EXISTS game_data_paths (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            steam_app_id TEXT NOT NULL,
            kind         TEXT NOT NULL,
            os           TEXT NOT NULL,
            raw_path     TEXT NOT NULL,
            fetched_at   TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_gamedata_app_id
            ON game_data_paths(steam_app_id);",
    )
    .map_err(|e| format!("Erro ao criar tabelas do PCGamingWiki: {}", e))?;

    Ok(())
}

// ===  DADOS DA API (fetch.rs) ===

/// Retorna os dados do PCGamingWiki armazenados para o jogo.
///
/// Retorna `None` se o jogo nunca foi buscado (fetched_at IS NULL ou linha inexistente).
/// Dados expirados **nunca** são descartados automaticamente — a invalidação é explícita.
pub fn get_pcgw_data(conn: &Connection, steam_app_id: &str) -> Option<GameExtras> {
    let result = conn.query_row(
        "SELECT
            steam_app_id,
            pcgw_page_id,
            pcgw_page_name,
            engine,
            available_on,

            dx_versions,
            vulkan_versions,
            opengl_versions,

            win64,
            linux64,
            macos_arm,
            macos_intel64,

            ray_tracing,
            upscaling,
            frame_gen,
            ultrawidescreen,
            four_k_support,
            hdr,
            high_fps,
            fov,
            borderless_windowed,
            color_blind,

            controller_support,
            full_controller,
            playstation_controllers,
            xinput_controllers,

            surround_sound,
            subtitles,
            closed_captions,

            has_save_data,
            has_config_data,

            languages_interface,
            languages_audio,
            languages_subtitles,

            fetched_at
         FROM game_extras
         WHERE steam_app_id = ?1
           AND fetched_at IS NOT NULL",
        params![steam_app_id],
        |row| {
            let parse_json_vec = |s: Option<String>| -> Option<Vec<String>> {
                s.and_then(|v| serde_json::from_str(&v).ok())
            };

            Ok(GameExtras {
                steam_app_id: row.get(0)?,
                pcgw_page_id: row.get(1)?,
                pcgw_page_name: row.get(2)?,
                engine: row.get(3)?,
                available_on: row.get(4)?,

                // API
                dx_versions: row.get(5)?,
                vulkan_versions: row.get(6)?,
                opengl_versions: row.get(7)?,

                win64: row.get(8)?,
                linux64: row.get(9)?,
                macos_arm: row.get(10)?,
                macos_intel64: row.get(11)?,

                // Video
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

                // Input
                controller_support: row.get(22)?,
                full_controller: row.get(23)?,
                playstation_controllers: row.get(24)?,
                xinput_controllers: row.get(25)?,

                // Audio
                surround_sound: row.get(26)?,
                subtitles: row.get(27)?,
                closed_captions: row.get(28)?,

                // Tags
                has_save_data: row.get(29)?,
                has_config_data: row.get(30)?,

                // L10n
                languages_interface: parse_json_vec(row.get(31)?),
                languages_audio: parse_json_vec(row.get(32)?),
                languages_subtitles: parse_json_vec(row.get(33)?),

                fetched_at: row.get(34)?,
            })
        },
    );

    match result {
        Ok(data) => Some(data),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            warn!("Erro ao buscar game_extras para {}: {}", steam_app_id, e);
            None
        }
    }
}

/// Salva ou atualiza os dados do PCGamingWiki para um jogo.
pub fn save_pcgw_data(conn: &Connection, data: &GameExtras) -> Result<(), String> {
    let serialize_vec = |v: &Option<Vec<String>>| -> Option<String> {
        v.as_ref().and_then(|list| serde_json::to_string(list).ok())
    };

    conn.execute(
        "INSERT OR REPLACE INTO game_extras (
            steam_app_id,
            pcgw_page_id,
            pcgw_page_name,
            engine,
            available_on,

            dx_versions,
            vulkan_versions,
            opengl_versions,

            win64,
            linux64,
            macos_arm,
            macos_intel64,

            ray_tracing,
            upscaling,
            frame_gen,
            ultrawidescreen,
            four_k_support,
            hdr,
            high_fps,
            fov,
            borderless_windowed,
            color_blind,

            controller_support,
            full_controller,
            playstation_controllers,
            xinput_controllers,

            surround_sound,
            subtitles,
            closed_captions,

            has_save_data,
            has_config_data,

            languages_interface,
            languages_audio,
            languages_subtitles,

            fetched_at
        ) VALUES (
            ?1,  ?2,  ?3,  ?4,  ?5,
            ?6,  ?7,  ?8,
            ?9,  ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            ?18, ?19, ?20, ?21, ?22,
            ?23, ?24, ?25, ?26,
            ?27, ?28, ?29,
            ?30, ?31,
            ?32, ?33, ?34,
            ?35
        )",
        params![
            data.steam_app_id,
            data.pcgw_page_id,
            data.pcgw_page_name,
            data.engine,
            data.available_on,
            data.dx_versions,
            data.vulkan_versions,
            data.opengl_versions,
            data.win64,
            data.linux64,
            data.macos_arm,
            data.macos_intel64,
            data.ray_tracing,
            data.upscaling,
            data.frame_gen,
            data.ultrawidescreen,
            data.four_k_support,
            data.hdr,
            data.high_fps,
            data.fov,
            data.borderless_windowed,
            data.color_blind,
            data.controller_support,
            data.full_controller,
            data.playstation_controllers,
            data.xinput_controllers,
            data.surround_sound,
            data.subtitles,
            data.closed_captions,
            data.has_save_data,
            data.has_config_data,
            serialize_vec(&data.languages_interface),
            serialize_vec(&data.languages_audio),
            serialize_vec(&data.languages_subtitles),
            data.fetched_at,
        ],
    )
    .map_err(|e| format!("Erro ao salvar game_extras: {}", e))?;

    Ok(())
}

/// Invalida os dados de um jogo, forçando nova busca na próxima chamada.
///
/// Define `fetched_at = NULL` — a linha permanece no banco mas
/// `get_pcgw_data` ignora registros sem `fetched_at`.
pub fn invalidate_pcgw_data(conn: &Connection, steam_app_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE game_extras SET fetched_at = NULL WHERE steam_app_id = ?1",
        params![steam_app_id],
    )
    .map_err(|e| format!("Erro ao invalidar game_extras: {}", e))?;

    info!("game_extras invalidado para steam_app_id={}", steam_app_id);
    Ok(())
}

// ===  DADOS OBTIDOS VIA SCRAPING DA PÁGINA (scraper.rs) ===

/// Salva os dados raspados no banco, substituindo entradas anteriores do jogo.
///
/// Usa DELETE + INSERT em vez de REPLACE porque os dados têm múltiplas linhas
/// por jogo (não há PK natural além de `steam_app_id + os_family + tier_title`).
pub fn save_scraped_data(
    conn: &Connection,
    steam_app_id: &str,
    data: &PcgwScrapedData,
) -> Result<(), String> {
    use chrono::Utc;
    use rusqlite::params;

    let now = Utc::now().to_rfc3339();

    // Remove dados anteriores deste jogo antes de reinserir
    conn.execute(
        "DELETE FROM system_requirements WHERE steam_app_id = ?1",
        params![steam_app_id],
    )
    .map_err(|e| format!("Erro ao limpar system_requirements: {}", e))?;

    conn.execute(
        "DELETE FROM game_data_paths WHERE steam_app_id = ?1",
        params![steam_app_id],
    )
    .map_err(|e| format!("Erro ao limpar game_data_paths: {}", e))?;

    // Insere requisitos de sistema
    let mut sysreq_stmt = conn
        .prepare(
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
        )
        .map_err(|e| format!("Erro ao preparar insert de system_requirements: {}", e))?;

    for req in &data.system_requirements {
        sysreq_stmt
            .execute(params![
                steam_app_id,
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
            ])
            .map_err(|e| {
                let msg = format!("Erro ao inserir system_requirement: {}", e);
                tracing::error!("{}", msg);
                msg
            })?;
    }

    // Insere caminhos de game data
    let mut path_stmt = conn
        .prepare(
            "INSERT INTO game_data_paths (steam_app_id, kind, os, raw_path, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| format!("Erro ao preparar insert de game_data_paths: {}", e))?;

    for path in &data.game_data_paths {
        path_stmt
            .execute(params![
                steam_app_id,
                path.kind,
                path.os,
                path.raw_path,
                now,
            ])
            .map_err(|e| format!("Erro ao inserir game_data_path: {}", e))?;
    }

    Ok(())
}

/// Retorna os requisitos de sistema salvos para um jogo.
/// Retorna vetor vazio se nunca foram buscados.
pub fn get_system_requirements(conn: &Connection, steam_app_id: &str) -> Vec<SystemRequirements> {
    let mut stmt = match conn.prepare(
        "SELECT os_family, tier_title, target,
                min_os, min_cpu, min_cpu2, min_ram, min_gpu, min_gpu2, min_vram, min_dx, min_storage,
                rec_os, rec_cpu, rec_cpu2, rec_ram, rec_gpu, rec_gpu2, rec_vram, rec_dx, rec_storage
         FROM system_requirements
         WHERE steam_app_id = ?1
         ORDER BY id ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let iter = stmt.query_map(rusqlite::params![steam_app_id], |row| {
        Ok(SystemRequirements {
            steam_app_id: steam_app_id.to_string(),
            os_family: row.get(0)?,
            tier_title: row.get(1)?,
            target: row.get(2)?,
            min_os: row.get(3)?,
            min_cpu: row.get(4)?,
            min_cpu2: row.get(5)?,
            min_ram: row.get(6)?,
            min_gpu: row.get(7)?,
            min_gpu2: row.get(8)?,
            min_vram: row.get(9)?,
            min_dx: row.get(10)?,
            min_storage: row.get(11)?,
            rec_os: row.get(12)?,
            rec_cpu: row.get(13)?,
            rec_cpu2: row.get(14)?,
            rec_ram: row.get(15)?,
            rec_gpu: row.get(16)?,
            rec_gpu2: row.get(17)?,
            rec_vram: row.get(18)?,
            rec_dx: row.get(19)?,
            rec_storage: row.get(20)?,
        })
    });

    match iter {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Retorna os caminhos de game data salvos para um jogo.
/// Retorna vetor vazio se nunca foram buscados.
pub fn get_game_data_paths(conn: &rusqlite::Connection, steam_app_id: &str) -> Vec<GameDataPath> {
    let mut stmt = match conn.prepare(
        "SELECT kind, os, raw_path
         FROM game_data_paths
         WHERE steam_app_id = ?1
         ORDER BY id ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let iter = stmt.query_map(rusqlite::params![steam_app_id], |row| {
        Ok(GameDataPath {
            steam_app_id: steam_app_id.to_string(),
            kind: row.get(0)?,
            os: row.get(1)?,
            raw_path: row.get(2)?,
            expanded_path: None,
        })
    });

    match iter {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}
