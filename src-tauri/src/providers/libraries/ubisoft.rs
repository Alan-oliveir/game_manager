//! Implementação do GameSource para a Ubisoft
//!
//! Fontes de dados utilizadas:
//!
//! - `%LOCALAPPDATA%\Ubisoft Game Launcher\settings.yaml`
//!   Contém o caminho base onde os jogos estão instalados (`misc.game_installation_path`).
//!
//! - `%LOCALAPPDATA%\Ubisoft Game Launcher\cache\configuration\configurations`
//!   Arquivo binário com entradas YAML embutidas que lista todos os jogos da conta,
//!   incluindo nome, identificador e caminho relativo do executável.
//!
//! **Nota sobre jogos instalados:**
//! Para determinar se um jogo está instalado, verifica se sua pasta existe dentro
//! de `game_installation_path`. O nome da pasta é inferido a partir do `game_identifier`
//! (que normalmente corresponde ao nome da pasta de instalação).
//!
//! **Linux (Wine):**
//! No Linux, o diretório de dados é resolvido a partir do `wine_prefix` configurado.
//! O caminho equivale a `<prefix>/drive_c/users/<USER>/AppData/Local/Ubisoft Game Launcher`.
//! Se nenhum prefix for fornecido, a importação não estará disponível no Linux.

use crate::errors::AppError;
use crate::providers::libraries::providers::{GameSource, SourceGame};
use crate::utils::text::{is_likely_non_base_game, strip_trademark_symbols};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

pub struct UbisoftSource {
    pub include_library_cache: bool,
    /// Wine prefix utilizado no Linux para localizar o diretório de dados do Ubisoft Game Launcher. Ignorado no Windows.
    #[allow(dead_code)]
    pub wine_prefix: Option<PathBuf>,
}

impl UbisoftSource {
    pub fn new(include_library_cache: bool, wine_prefix: Option<PathBuf>) -> Self {
        Self {
            include_library_cache,
            wine_prefix,
        }
    }

    /// Resolve o diretório de dados do Ubisoft Game Launcher.
    ///
    /// - **Windows:** `%LOCALAPPDATA%\Ubisoft Game Launcher`
    /// - **Linux:** `<wine_prefix>/drive_c/users/<USER>/AppData/Local/Ubisoft Game Launcher`
    fn resolve_launcher_data_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = Path::new(&local_app_data).join("Ubisoft Game Launcher");
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(prefix) = &self.wine_prefix {
                let user = std::env::var("USER").ok()?;
                let path = prefix
                    .join("drive_c")
                    .join("users")
                    .join(&user)
                    .join("AppData")
                    .join("Local")
                    .join("Ubisoft Game Launcher");
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    /// Lê o `settings.yaml` e retorna o caminho base de instalação dos jogos.
    ///
    /// Extrai `misc.game_installation_path`, que indica onde os jogos ficam instalados.
    fn read_game_installation_path(data_dir: &Path) -> Option<PathBuf> {
        let settings_path = data_dir.join("settings.yaml");
        let content = fs::read_to_string(&settings_path).ok()?;

        // O campo está sempre dentro do bloco `misc:`, como:
        //   misc:
        //     game_installation_path: E:/Ubisoft Game Launcher/games/
        let mut in_misc = false;
        for line in content.lines() {
            if line.trim_end() == "misc:" {
                in_misc = true;
                continue;
            }

            if in_misc {
                // Se encontra outro bloco de nível superior, sai do bloco misc
                if !line.starts_with(' ') && !line.is_empty() {
                    break;
                }

                if let Some(rest) = line.trim_start().strip_prefix("game_installation_path: ") {
                    let raw = rest.trim().replace('/', std::path::MAIN_SEPARATOR_STR);
                    let path = PathBuf::from(raw);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    /// Lê o cache de biblioteca da Ubisoft.
    ///
    /// Retorna uma tupla `(Vec<SourceGame>, HashMap<game_identifier, exe_relativo>)`.
    fn read_configuration_library(
        data_dir: &Path,
        games_base_path: Option<&Path>,
    ) -> Vec<SourceGame> {
        let config_path = data_dir
            .join("cache")
            .join("configuration")
            .join("configurations");

        if !config_path.exists() {
            return Vec::new();
        }

        let content = match fs::read(&config_path) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => return Vec::new(),
        };

        let mut games = Vec::new();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut current_name: Option<String> = None;
        let mut current_id: Option<String> = None;
        let mut current_exe: Option<String> = None;

        let flush = |name: Option<String>,
                     id: Option<String>,
                     exe: Option<String>,
                     games: &mut Vec<SourceGame>,
                     seen_ids: &mut std::collections::HashSet<String>,
                     seen_names: &mut std::collections::HashSet<String>| {
            // Regra 1: game_identifier obrigatório — descarta stubs binários sem ID
            let Some(game_id) = id else { return };
            if game_id.is_empty() {
                return;
            }

            // Resolve nome de exibição: usa o nome do bloco quando válido, senão o game_identifier
            let raw_name = name.unwrap_or_default();
            let is_placeholder =
                raw_name.is_empty() || raw_name == "GAMENAME" || raw_name.to_lowercase() == "l1";
            let n = if is_placeholder {
                game_id.clone()
            } else {
                raw_name
            };

            if n.len() <= 2 || is_likely_non_base_game(&n) {
                return;
            }

            // Nome com " - " mas game_identifier sem " - " indica DLC, conteúdo extra ou são dados incorretos
            if n.contains(" - ") && !game_id.contains(" - ") {
                return;
            }

            let name_key = strip_trademark_symbols(&n).to_lowercase();

            if seen_ids.contains(&game_id) || seen_names.contains(&name_key) {
                return;
            }

            seen_ids.insert(game_id.clone());
            seen_names.insert(name_key);

            // Verifica se o jogo está instalado procurando sua pasta em game_installation_path.
            let (installed, install_path) = if let Some(base) = games_base_path {
                let candidate = base.join(&game_id);
                if candidate.exists() {
                    (true, Some(candidate.to_string_lossy().to_string()))
                } else {
                    (false, None)
                }
            } else {
                (false, None)
            };

            // Constrói o caminho absoluto do executável combinando install_path + relative
            let executable_path = install_path.as_ref().and_then(|dir| {
                exe.as_ref()
                    .map(|rel| PathBuf::from(dir).join(rel).to_string_lossy().to_string())
            });

            games.push(SourceGame {
                library: "Ubisoft".to_string(),
                library_game_id: game_id,
                name: Some(strip_trademark_symbols(&n)),
                installed,
                executable_path,
                install_path,
                playtime_minutes: None,
                last_played: None,
                source_label: None,
            });
        };

        for raw_line in content.lines() {
            let line = raw_line.trim_end_matches('\r');

            if line == "version: 2.0" {
                let name = current_name.take();
                let id = current_id.take();
                let exe = current_exe.take();
                flush(name, id, exe, &mut games, &mut seen_ids, &mut seen_names);
                continue;
            }

            if let Some(rest) = line.strip_prefix("  name: ") {
                let name = parse_yaml_string_value(rest);
                if name.is_empty() || name == "GAMENAME" {
                    continue;
                }
                let prev_name = current_name.take();
                let prev_id = current_id.take();
                let prev_exe = current_exe.take();
                flush(
                    prev_name,
                    prev_id,
                    prev_exe,
                    &mut games,
                    &mut seen_ids,
                    &mut seen_names,
                );
                current_name = Some(name);
                current_id = None;
                current_exe = None;
                continue;
            }

            if let Some(rest) = line.strip_prefix("    game_identifier: ") {
                let id = parse_yaml_string_value(rest);
                if !id.is_empty() && current_id.is_none() {
                    current_id = Some(id);
                }
                continue;
            }

            // Caminho relativo do executável, filtra pelo nome do arquivo para excluir utilitários.
            if current_exe.is_none() {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed.strip_prefix("relative: ") {
                    let rel = rest.trim().trim_end_matches('\r');
                    if rel.to_lowercase().ends_with(".exe") {
                        let filename = Path::new(rel)
                            .file_name()
                            .map(|f| f.to_string_lossy().to_lowercase())
                            .unwrap_or_default();

                        let is_auxiliary = ["updater", "editor", "launcher", "setup", "patcher"]
                            .iter()
                            .any(|kw| filename.contains(kw));

                        if !is_auxiliary {
                            current_exe = Some(rel.to_string());
                        }
                    }
                }
            }
        }

        let name = current_name.take();
        let id = current_id.take();
        let exe = current_exe.take();
        flush(name, id, exe, &mut games, &mut seen_ids, &mut seen_names);

        games
    }
}

/// Remove aspas simples, duplas e espaços ao redor do valor de um campo YAML.
fn parse_yaml_string_value(s: &str) -> String {
    let trimmed = s.trim().trim_end_matches('\r').trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(trimmed);
    unquoted.trim().trim_end_matches('\r').trim().to_string()
}

#[async_trait]
impl GameSource for UbisoftSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        let data_dir = self.resolve_launcher_data_dir().ok_or_else(|| {
            AppError::NotFound(
                "Pasta de dados do Ubisoft Game Launcher não encontrada. \
                Verifique se o Ubisoft Connect está instalado."
                    .to_string(),
            )
        })?;

        // Lê o caminho base de instalação dos jogos a partir do settings.yaml.
        // Se não encontrado, importa os jogos sem install_path e executable_path.
        let games_base_path = Self::read_game_installation_path(&data_dir);

        if !self.include_library_cache {
            return Ok(Vec::new());
        }

        let games = Self::read_configuration_library(&data_dir, games_base_path.as_deref());

        Ok(games)
    }
}
