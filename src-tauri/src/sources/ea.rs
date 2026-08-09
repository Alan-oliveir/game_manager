//! Source para importar jogos instalados via EA App (Electronic Arts).
//!
//! Diferente de Epic/GOG/Battle.net, o EA App não expõe nenhum arquivo estruturado
//! e legível com o status de instalação atual (o antigo `Origin Games` no registro do
//! Windows existe, mas é granular por DLC/expansão, não por jogo — não confiável como
//! fonte de identidade). Por isso a detecção depende do usuário informar a pasta onde o EA
//! instala os jogos (configurável no client, assim como no GOG Galaxy).
//!
//! **Duas fontes cruzadas:**
//! - `InstallData` (`C:\ProgramData\EA Desktop\InstallData`): pasta por jogo já instalado
//!   alguma vez, sobrevive à desinstalação.
//! - Pasta de instalação configurada pelo usuário: escaneada em busca de subpastas —
//!   cada uma é um jogo atualmente instalado. Fonte de verdade sobre o que está instalado.
//!
//! **Observações:**
//! - Windows apenas — o EA App não roda de forma confiável via Wine.
//! - Sem OAuth: tentativas de capturar biblioteca completa via sessão web (WebviewWindow)
//!   se mostraram inviáveis — só jogos instalados.

use crate::errors::AppError;
use crate::sources::providers::SourceGame;
use crate::utils::executable_heuristics::guess_main_executable;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
const EA_INSTALL_DATA_DIR_WINDOWS: &str = r"C:\ProgramData\EA Desktop\InstallData";

/// Source responsável por importar jogos instalados via EA App.
pub struct EaSource {
    /// Pasta configurada pelo usuário onde o EA App instala os jogos.
    install_dir: Option<PathBuf>,
}

impl EaSource {
    pub fn new(install_dir: Option<PathBuf>) -> Self {
        Self { install_dir }
    }

    /// Resolve o diretório `InstallData`.
    fn resolve_install_data_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let path = PathBuf::from(EA_INSTALL_DATA_DIR_WINDOWS);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Carrega os nomes de pasta de `InstallData` (catálogo de jogos já instalados alguma vez).
    fn load_known_names(install_data_dir: &Path) -> Vec<String> {
        let Ok(entries) = fs::read_dir(install_data_dir) else {
            log::warn!("Não foi possível ler InstallData: {install_data_dir:?}");
            return Vec::new();
        };

        entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().trim().to_string();
                (!name.is_empty()).then_some(name)
            })
            .collect()
    }

    /// Importa jogos já instalados
    pub async fn import_installed(&self) -> Result<Vec<SourceGame>, AppError> {
        let known_names = self
            .resolve_install_data_dir()
            .map(|dir| Self::load_known_names(&dir))
            .unwrap_or_default();

        let mut games = Vec::new();
        let mut matched_known: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Scan da pasta de instalação (jogos atualmente instalados)
        if let Some(install_dir) = &self.install_dir {
            if install_dir.exists() && install_dir.is_dir() {
                for entry in fs::read_dir(install_dir)?.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let folder_name = entry.file_name().to_string_lossy().trim().to_string();
                    if folder_name.is_empty() {
                        continue;
                    }

                    let matched = known_names.iter().find(|known| {
                        let known_lower = known.to_lowercase();
                        let folder_lower = folder_name.to_lowercase();
                        folder_lower.starts_with(&known_lower)
                            || known_lower.starts_with(&folder_lower)
                    });

                    let display_name = matched.cloned().unwrap_or_else(|| folder_name.clone());
                    if let Some(m) = matched {
                        matched_known.insert(m.clone());
                    }

                    let executable_path =
                        guess_main_executable(&path).map(|p| p.to_string_lossy().to_string());

                    games.push(SourceGame {
                        platform: "EA".to_string(),
                        platform_game_id: normalize_id(&display_name),
                        name: Some(display_name),
                        installed: true,
                        executable_path,
                        install_path: Some(path.to_string_lossy().to_string()),
                        playtime_minutes: None,
                        last_played: None,
                        source_label: None,
                    });
                }
            } else if install_dir.exists() {
                log::warn!("Caminho de instalação EA configurado não é uma pasta: {install_dir:?}");
            }
        }

        // Jogos conhecidos via InstallData mas não encontrados na pasta de instalação
        // (indício de que o usuário já teve/possui o jogo na conta, mesmo desinstalado).
        for known in &known_names {
            if matched_known.contains(known) {
                continue;
            }

            games.push(SourceGame {
                platform: "EA".to_string(),
                platform_game_id: normalize_id(known),
                name: Some(known.clone()),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
                source_label: None,
            });
        }

        Ok(games)
    }
}

/// Gera um `platform_game_id` estável a partir do nome — minúsculo, sem espaços,
/// já que a EA não expõe nenhum ID numérico estável como Steam/GOG/Battle.net.
fn normalize_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "_")
}
