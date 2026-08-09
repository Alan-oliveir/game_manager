//! Source para importar jogos instalados via Microsoft Store / Xbox App (Gaming Services).
//!
//! Diferente do mecanismo UWP clássico (`Get-AppxPackage`), jogos "pesados" instalados
//! via Xbox App ou Microsoft Store com suporte a Gaming Services ficam registrados em
//! uma pasta própria por drive, localizada através de um arquivo marcador `.GamingRoot`
//! na raiz do drive.
//!
//! **Formato do `.GamingRoot`** (confirmado empiricamente, não documentado oficialmente):
//! - 4 bytes de assinatura: `RGBX`
//! - 4 bytes: flag/contador (observado sempre `01 00 00 00`)
//! - nome da pasta em UTF-16LE, terminado em `\0\0` (ex: "XboxGames")
//!
//! Cada jogo fica em `<drive>\<pasta>\<nome ou GUID>\content\MicrosoftGame.config`,
//! um manifesto XML com nome de exibição, executável principal e `StoreId`.
//!
//! **Filtro de DLC/addons:** um manifesto sem nenhum `<Executable>` dentro de
//! `<ExecutableList>` não é um jogo standalone — é conteúdo adicional vinculado a um
//! jogo base via `<AllowedProducts>`/`<MainPackageDependency>`. Esses são descartados.
//!
//! **Limitação conhecida:** jogos que usam apenas o mecanismo UWP clássico sem passar
//! por Gaming Services (raro para jogos "de peso"; mais comum em apps casuais como
//! Microsoft Solitaire Collection) não são detectados por este scanner.

use crate::errors::AppError;
use crate::sources::providers::SourceGame;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

// === STRUCTS: MicrosoftGame.config ===

#[derive(Debug, Deserialize)]
#[serde(rename = "Game")]
struct GameManifest {
    #[serde(rename = "Identity")]
    identity: Identity,
    #[serde(rename = "StoreId", default)]
    store_id: Option<String>,
    #[serde(rename = "ShellVisuals", default)]
    shell_visuals: Option<ShellVisuals>,
    #[serde(rename = "ExecutableList", default)]
    executable_list: Option<ExecutableList>,
}

#[derive(Debug, Deserialize)]
struct Identity {
    #[serde(rename = "@Name")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ShellVisuals {
    #[serde(rename = "@DefaultDisplayName", default)]
    default_display_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ExecutableList {
    #[serde(rename = "Executable", default)]
    executables: Vec<Executable>,
}

#[derive(Debug, Deserialize)]
struct Executable {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@OverrideDisplayName", default)]
    override_display_name: Option<String>,
}

// === SCAN ===

/// Importa jogos instalados detectados via Gaming Services em todos os drives do sistema.
pub fn import_installed() -> Result<Vec<SourceGame>, AppError> {
    let mut games = Vec::new();

    for drive in candidate_drives() {
        let Some(games_folder_name) = read_gaming_root(&drive.join(".GamingRoot")) else {
            continue; // sem marcador nesse drive, nada a fazer
        };

        let games_root = drive.join(&games_folder_name);
        if !games_root.is_dir() {
            log::warn!(
                "'.GamingRoot' aponta para '{games_folder_name}' mas a pasta não existe em {drive:?}"
            );
            continue;
        }

        let Ok(entries) = fs::read_dir(&games_root) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            match parse_game_folder(&path) {
                Ok(Some(game)) => games.push(game),
                Ok(None) => {} // DLC/addon, descartado silenciosamente
                Err(err) => {
                    log::warn!("Erro ao processar manifesto Xbox em {path:?}: {err}");
                }
            }
        }
    }

    Ok(games)
}

/// Enumera drives existentes no sistema (A: até Z:).
#[cfg(target_os = "windows")]
fn candidate_drives() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .map(|b| PathBuf::from(format!("{}:\\", b as char)))
        .filter(|p| p.exists())
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn candidate_drives() -> Vec<PathBuf> {
    Vec::new() // Gaming Services é exclusivo do Windows
}

/// Lê e decodifica o arquivo `.GamingRoot`, retornando o nome da pasta de jogos
/// (ex: "XboxGames"). Retorna `None` se o arquivo não existir ou o formato não bater.
fn read_gaming_root(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;

    if bytes.len() < 8 || &bytes[0..4] != b"RGBX" {
        return None;
    }

    let payload = &bytes[8..];
    let mut units = Vec::with_capacity(payload.len() / 2);
    let mut i = 0;
    while i + 1 < payload.len() {
        let unit = u16::from_le_bytes([payload[i], payload[i + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        i += 2;
    }

    String::from_utf16(&units).ok().filter(|s| !s.is_empty())
}

/// Lê e interpreta o manifesto de uma pasta de jogo. Retorna `None` se for DLC/addon
/// (sem executável próprio), ou erro se o manifesto estiver ausente/corrompido.
fn parse_game_folder(game_folder: &Path) -> Result<Option<SourceGame>, AppError> {
    let config_path = game_folder.join("content").join("MicrosoftGame.config");
    if !config_path.exists() {
        return Ok(None); // pasta sem manifesto reconhecível; ignora
    }

    let content = fs::read_to_string(&config_path)?;
    let manifest: GameManifest = quick_xml::de::from_str(&content)
        .map_err(|e| AppError::ParseError(format!("Falha ao parsear {config_path:?}: {e}")))?;

    let Some(executable_list) = &manifest.executable_list else {
        return Ok(None); // sem <ExecutableList> — DLC/addon
    };

    let Some(executable) = executable_list.executables.first() else {
        return Ok(None); // <ExecutableList> vazio — DLC/addon
    };

    let name = executable
        .override_display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            manifest
                .shell_visuals
                .as_ref()
                .and_then(|sv| sv.default_display_name.clone())
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| manifest.identity.name.clone());

    // Executable.Name pode conter subcaminho (ex: "launcher/idTechLauncher.exe");
    // PathBuf::join lida com isso normalmente independente do separador usado.
    let executable_path = game_folder
        .join("content")
        .join(&executable.name)
        .to_string_lossy()
        .to_string();

    let platform_game_id = manifest
        .store_id
        .clone()
        .unwrap_or_else(|| manifest.identity.name.clone());

    Ok(Some(SourceGame {
        platform: "Xbox".to_string(),
        platform_game_id,
        name: Some(name),
        installed: true,
        executable_path: Some(executable_path),
        install_path: Some(game_folder.join("content").to_string_lossy().to_string()),
        playtime_minutes: None,
        last_played: None,
        source_label: None,
    }))
}
