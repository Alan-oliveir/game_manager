//! Módulo para importar jogos da plataforma Steam.
//!
//! Fornece estruturas de dados e funções para: buscar jogos instalados (via arquivos VDF locais) e
//! não instalados (via librarycache). Buscar via API Steam como fallback e faz merge de múltiplas
//! fontes, preservando jogos grátis não instalados.

use crate::constants::{STEAM_STORE_TIMEOUT_SECS, USER_AGENT_BROWSER};
use crate::errors::AppError;
use crate::sources::providers::{GameSource, SourceGame};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Estrutura interna para representar um jogo
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameData {
    pub name: String,
    pub platform: String,
    pub platform_game_id: String,
    pub install_path: Option<String>,
    pub installed: bool,
    pub import_confidence: String,
    pub playtime_forever: i32,
    pub rtime_last_played: i64,
}

// === FUNÇÃO PRINCIPAL ===

/// Obtém a lista completa de jogos Steam do usuário
pub async fn get_complete_library(
    steam_root: &Path,
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<GameData>, String> {
    info!("Iniciando importação da biblioteca Steam");

    // 1. API Steam primeiro (para obter owned AppIDs)
    let api_games = match fetch_steam_api(api_key, steam_id).await {
        Ok(games) => {
            info!("API: {} jogos possuídos", games.len());
            games
        }
        Err(e) => {
            warn!("Aviso ao buscar API: {}", e);
            Vec::new()
        }
    };

    let owned_appids: HashSet<String> = api_games
        .iter()
        .map(|g| g.platform_game_id.clone())
        .collect();

    // 2. Jogos instalados (com filtro leve)
    let installed = match scan_installed_games(steam_root) {
        Ok(mut games) => {
            let mut seen = HashSet::new();
            games.retain(|g| seen.insert(g.platform_game_id.clone()));
            info!("VDF: {} jogos instalados encontrados", games.len());
            games
        }
        Err(e) => {
            warn!("Aviso ao buscar instalados: {}", e);
            Vec::new()
        }
    };

    // 3. Library cache (com filtro leve)
    let cached = match scan_library_cache(steam_root).await {
        Ok(mut games) => {
            let mut seen = HashSet::new();
            games.retain(|g| seen.insert(g.platform_game_id.clone()));
            info!("Cache: {} jogos encontrados", games.len());
            games
        }
        Err(e) => {
            warn!("Aviso ao buscar cache: {}", e);
            Vec::new()
        }
    };

    // 4. Merge de todas as fontes
    let sources = vec![installed, cached, api_games];
    let mut merged = merge_games(sources);
    info!("Merge: {} jogos antes de filtro de demos", merged.len());

    // 5. Filtro inteligente de demos
    merged = filter_duplicate_demos(merged, &owned_appids);
    info!("Final: {} jogos após filtro de demos", merged.len());

    Ok(merged)
}

// === SCAN DE JOGOS INSTALADOS ===

/// Escaneia jogos instalados via VDF e appmanifest
pub fn scan_installed_games(steam_root: &Path) -> Result<Vec<GameData>, String> {
    let mut games = Vec::new();
    let library_folders = read_library_folders(steam_root)?;

    for library in library_folders {
        let steamapps = library.join("steamapps");
        if !steamapps.exists() {
            continue;
        }

        let entries =
            fs::read_dir(&steamapps).map_err(|e| format!("Erro ao ler steamapps: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if is_appmanifest(&path) {
                if let Ok(game) = parse_appmanifest(&path, &library) {
                    games.push(game);
                }
            }
        }
    }

    Ok(games)
}

/// Lê libraryfolders.vdf e retorna lista de bibliotecas Steam
fn read_library_folders(steam_root: &Path) -> Result<Vec<PathBuf>, String> {
    let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    let mut libraries = vec![steam_root.to_path_buf()];

    if !vdf_path.exists() {
        return Ok(libraries);
    }

    let content = fs::read_to_string(&vdf_path)
        .map_err(|e| format!("Erro ao ler libraryfolders.vdf: {}", e))?;

    let mut in_folder = false;
    let mut current_path: Option<PathBuf> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('"') && trimmed.contains('"') {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 2 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                in_folder = true;
            }
        }

        if in_folder && trimmed.starts_with("\"path\"") {
            if let Some(path_str) = extract_vdf_value(trimmed) {
                let cleaned = path_str.replace("\\\\", "\\");
                current_path = Some(PathBuf::from(cleaned));
            }
        }

        if trimmed == "}" && in_folder {
            if let Some(path) = current_path.take() {
                if path != steam_root.to_path_buf() {
                    libraries.push(path);
                }
            }
            in_folder = false;
        }
    }

    Ok(libraries)
}

/// Parseia um arquivo appmanifest
fn parse_appmanifest(manifest_path: &Path, library_root: &Path) -> Result<GameData, String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| AppError::SteamManifestError(e.to_string()).to_string())?;
    let mut appid: Option<String> = None;
    let mut name: Option<String> = None;
    let mut installdir: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("\"appid\"") {
            appid = extract_vdf_value(trimmed);
        } else if trimmed.starts_with("\"name\"") {
            name = extract_vdf_value(trimmed);
        } else if trimmed.starts_with("\"installdir\"") {
            installdir = extract_vdf_value(trimmed);
        }

        if appid.is_some() && name.is_some() && installdir.is_some() {
            break;
        }
    }

    let appid = appid.ok_or("appid não encontrado")?;
    let name = name.unwrap_or_else(|| "Unknown".to_string());
    let install_path = installdir.map(|dir| {
        library_root
            .join("steamapps")
            .join("common")
            .join(dir)
            .to_string_lossy()
            .to_string()
    });

    Ok(GameData {
        name,
        platform: "Steam".to_string(),
        platform_game_id: appid,
        install_path,
        installed: true,
        import_confidence: "High".to_string(),
        playtime_forever: 0,
        rtime_last_played: 0,
    })
}

fn is_appmanifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|f| f.starts_with("appmanifest_") && f.ends_with(".acf"))
        .unwrap_or(false)
}

fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

// === SCAN DO LIBRARY CACHE ===

/// Escaneia jogos não-instalados via library cache
pub async fn scan_library_cache(steam_root: &Path) -> Result<Vec<GameData>, String> {
    let mut games = Vec::new();
    let userdata = steam_root.join("userdata");

    if !userdata.exists() {
        return Ok(games);
    }

    let user_dirs = fs::read_dir(&userdata).map_err(|e| format!("Erro: {}", e))?;

    for user_entry in user_dirs.flatten() {
        if !user_entry.path().is_dir() {
            continue;
        }

        let cache_dir = user_entry.path().join("config").join("librarycache");

        if cache_dir.exists() {
            scan_librarycache_dir(&cache_dir, &mut games).await?;
        }
    }

    let mut seen = HashSet::new();
    games.retain(|g| seen.insert(g.platform_game_id.clone()));

    Ok(games)
}

/// Escaneia um diretório librarycache
async fn scan_librarycache_dir(dir: &Path, games: &mut Vec<GameData>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Erro: {}", e))?;
    let mut appids = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            if filename.chars().all(|c| c.is_ascii_digit()) {
                appids.push(filename.to_string());
            }
        }
    }

    if appids.is_empty() {
        return Ok(());
    }

    info!("Cache: {} AppIDs para buscar nomes", appids.len());

    // Busca nomes em batch com rate limiting
    let names = fetch_game_names_batch(&appids).await;

    // Cria GameData para cada jogo
    for appid in appids {
        let name = names
            .get(&appid)
            .cloned()
            .unwrap_or_else(|| format!("Game {}", appid));

        games.push(GameData {
            name,
            platform: "Steam".to_string(),
            platform_game_id: appid,
            install_path: None,
            installed: false,
            import_confidence: "Medium".to_string(),
            playtime_forever: 0,
            rtime_last_played: 0,
        });
    }

    Ok(())
}

/// Busca nomes de jogos em lote, respeitando o limitador compartilhado da Steam
/// (mesmo STEAM_LIMITER usado no enrichment).
async fn fetch_game_names_batch(appids: &[String]) -> HashMap<String, String> {
    let mut names = HashMap::new();

    info!("Iniciando busca em batch de {} jogos", appids.len());

    for appid in appids {
        if let Some(name) = fetch_steam_game_name(appid).await {
            names.insert(appid.clone(), name);
        }
        sleep(Duration::from_millis(crate::constants::STEAM_RATE_LIMIT_MS)).await;
    }

    info!(
        "Busca concluída: {}/{} nomes obtidos",
        names.len(),
        appids.len()
    );
    names
}

/// Busca nome do jogo via Steam Store API, com rate limiting e backoff
/// compartilhados via STEAM_LIMITER (mesmo usado por get_app_details/search_app_by_name).
async fn fetch_steam_game_name(app_id: &str) -> Option<String> {
    use crate::utils::http_client::HTTP_CLIENT;

    crate::services::rate_limiter::STEAM_LIMITER
        .run(|| async {
            let url = format!(
                "https://store.steampowered.com/api/appdetails?appids={}",
                app_id
            );

            let resp = HTTP_CLIENT
                .get(&url)
                .header("User-Agent", USER_AGENT_BROWSER)
                .timeout(Duration::from_secs(STEAM_STORE_TIMEOUT_SECS))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                return Err(format!("Steam appdetails Error: {}", resp.status()));
            }

            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(json)
        })
        .await
        .ok()
        .and_then(|json| {
            let app_data = json.get(app_id)?;
            if !app_data.get("success")?.as_bool()? {
                return None;
            }
            app_data
                .get("data")?
                .get("name")?
                .as_str()
                .map(|s| s.to_string())
        })
}

// === API STEAM ===

/// Busca jogos via API Steam
async fn fetch_steam_api(api_key: &str, steam_id: &str) -> Result<Vec<GameData>, String> {
    let steam_games =
        crate::services::integration::steam_api::list_steam_games(api_key, steam_id).await?;

    Ok(steam_games
        .into_iter()
        .map(|game| GameData {
            name: game.name,
            platform: "Steam".to_string(),
            platform_game_id: game.appid.to_string(),
            install_path: None,
            installed: false,
            import_confidence: "Low".to_string(),
            playtime_forever: game.playtime_forever,
            rtime_last_played: game.rtime_last_played,
        })
        .collect())
}

// === FILTRO DE DEMOS DUPLICADAS ===

/// Detecta se um jogo é uma demo baseado no nome
fn is_demo_game(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    let demo_keywords = [
        " demo",
        "(demo)",
        " - demo",
        " playtest",
        "(playtest)",
        " - playtest",
        " free weekend",
        " trial",
        " beta test",
        " limited test",
        " technical test",
    ];

    demo_keywords
        .iter()
        .any(|keyword| name_lower.contains(keyword))
}

/// Normaliza nome do jogo removendo sufixos de demo
fn normalize_game_name(name: &str) -> String {
    let mut normalized = name.to_lowercase();

    let suffixes_to_remove = [
        " demo",
        " (demo)",
        " - demo",
        " playtest",
        " (playtest)",
        " - playtest",
        " free weekend",
        " trial",
        " beta test",
        " limited test",
        " technical test",
    ];

    for suffix in &suffixes_to_remove {
        if let Some(pos) = normalized.find(suffix) {
            normalized = normalized[..pos].to_string();
            break;
        }
    }

    normalized.trim().to_string()
}

/// Filtra demos duplicadas mantendo jogos gratuitos
fn filter_duplicate_demos(games: Vec<GameData>, owned_appids: &HashSet<String>) -> Vec<GameData> {
    let mut by_normalized_name: HashMap<String, Vec<GameData>> = HashMap::new();

    for game in games {
        let normalized = normalize_game_name(&game.name);
        by_normalized_name.entry(normalized).or_default().push(game);
    }

    let mut result = Vec::new();

    for (normalized_name, mut games_group) in by_normalized_name {
        if games_group.len() == 1 {
            result.push(games_group.pop().unwrap());
            continue;
        }

        let mut demos: Vec<GameData> = Vec::new();
        let mut base_games: Vec<GameData> = Vec::new();

        for game in games_group {
            if is_demo_game(&game.name) {
                demos.push(game);
            } else {
                base_games.push(game);
            }
        }

        // Caso 1: Tem jogo base E demo
        if !base_games.is_empty() && !demos.is_empty() {
            let has_owned_base = base_games
                .iter()
                .any(|g| owned_appids.contains(&g.platform_game_id));

            if has_owned_base {
                debug!(
                    "Removendo demos de '{}' (jogo base possuído)",
                    normalized_name
                );
                result.extend(base_games);
            } else {
                debug!(
                    "Mantendo demos de '{}' (jogo base não possuído)",
                    normalized_name
                );
                result.extend(demos);
            }
        }
        // Caso 2: Só tem demos (sem jogo base)
        else if !demos.is_empty() {
            result.extend(demos);
        }
        // Caso 3: Só tem jogos-base (sem demos)
        else {
            result.extend(base_games);
        }
    }

    result
}

// === MERGE DE JOGOS ===

/// Faz merge de múltiplas fontes de jogos
pub fn merge_games(sources: Vec<Vec<GameData>>) -> Vec<GameData> {
    let mut map: HashMap<String, Vec<GameData>> = HashMap::new();

    for list in sources {
        for game in list {
            let key = format!("{}::{}", game.platform, game.platform_game_id);
            map.entry(key).or_default().push(game);
        }
    }

    let mut result: Vec<GameData> = map.into_values().map(merge_multiple).collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}

fn merge_multiple(mut games: Vec<GameData>) -> GameData {
    if games.is_empty() {
        panic!("Lista vazia");
    }

    if games.len() == 1 {
        return games.into_iter().next().unwrap();
    }

    games.sort_by(|a, b| {
        let a_score = confidence_score(&a.import_confidence);
        let b_score = confidence_score(&b.import_confidence);
        b_score.cmp(&a_score)
    });

    games
        .into_iter()
        .reduce(|acc, next| enrich(&acc, &next))
        .unwrap()
}

fn confidence_score(conf: &str) -> i32 {
    match conf {
        "High" => 3,
        "Medium" => 2,
        "Low" => 1,
        _ => 0,
    }
}

fn enrich(primary: &GameData, secondary: &GameData) -> GameData {
    GameData {
        name: if primary.name.len() >= secondary.name.len() {
            primary.name.clone()
        } else {
            secondary.name.clone()
        },
        platform: primary.platform.clone(),
        platform_game_id: primary.platform_game_id.clone(),
        install_path: primary
            .install_path
            .clone()
            .or_else(|| secondary.install_path.clone()),
        installed: primary.installed || secondary.installed,
        import_confidence: primary.import_confidence.clone(),
        playtime_forever: primary.playtime_forever.max(secondary.playtime_forever),
        rtime_last_played: primary.rtime_last_played.max(secondary.rtime_last_played),
    }
}

/// Estrutura que implementa a interface comum de importação
pub struct SteamSource {
    pub steam_root: String,
    pub api_key: String,
    pub steam_id: String,
}

#[async_trait]
impl GameSource for SteamSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        let steam_root_path = Path::new(&self.steam_root);

        // Reutiliza a lógica robusta de merge (VDF + Cache + API)
        let library = get_complete_library(steam_root_path, &self.api_key, &self.steam_id)
            .await
            .map_err(AppError::NetworkError)?; //

        // Converte o GameData interno para o SourceGame genérico do provedor
        let games = library
            .into_iter()
            .map(|game| {
                SourceGame {
                    platform: "Steam".to_string(),
                    platform_game_id: game.platform_game_id,
                    name: Some(game.name),
                    installed: game.installed,
                    executable_path: None, // Steam usa AppIDs, não paths diretos no import inicial
                    install_path: game.install_path,
                    playtime_minutes: Some(game.playtime_forever as u32),
                    last_played: Some(game.rtime_last_played),
                    source_label: None,
                }
            })
            .collect();

        Ok(games)
    }
}
