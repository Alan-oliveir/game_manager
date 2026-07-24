//! Importação de jogos instalados da IndieGala (via IGClient).
//!
//! Lê `installed.json` do IGClient, que contém só os jogos atualmente instalados,
//! já com metadados completos (descrição, categorias, executável) embutidos e `config.json`
//! que possui a lista de todos os jogos adquiridos, mas sem metadados para todos os jogos.

use crate::errors::AppError;
use crate::models::GameTag;
use crate::services::tags::classify_and_sort_tags;
use crate::sources::providers::SourceGame;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// === JOGO IMPORTADO COM METADADOS EXTRAS (além do SourceGame padrão) ===

/// Jogo importado da IndieGala com campos adicionais além do `SourceGame` padrão.
#[derive(Debug, Clone)]
pub struct IndiegalaGame {
    pub source: SourceGame,
    pub description_raw: Option<String>,
    /// Tags já classificadas via `services::tags` — a IndieGala não separa gênero de tag de forma
    /// consistente entre `installed.json` e `config.json`, então usamos só tags (como as demais
    /// plataformas fazem com dados vindos da RAWG) em vez de um campo de gênero em paralelo.
    pub tags: Option<Vec<GameTag>>,
}

// === ESTRUTURAS INTERNAS PARA DESSERIALIZAÇÃO DE installed.json ===

#[derive(Debug, Deserialize)]
struct InstalledEntry {
    target: Target,
    /// Pasta(s) raiz configurada(s) no client (ex: `["E:\\IGClient\\games"]`).
    /// Cada jogo costuma ficar numa subpasta com o slug dentro dela, o JSON não confirma isso diretamente.
    path: Vec<String>,
    /// Tempo jogado, em **segundos**.
    playtime: f64,
}

#[derive(Debug, Deserialize)]
struct Target {
    item_data: ItemData,
    game_data: GameData,
}

#[derive(Debug, Deserialize)]
struct ItemData {
    name: String,
    slugged_name: String,
    id_key_name: String,
}

#[derive(Debug, Deserialize)]
struct GameData {
    description_short: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    exe_path: Option<String>,
}

// === ESTRUTURAS INTERNAS PARA DESSERIALIZAÇÃO DE config.json ===
//
// config.json tem duas partes: `gala_data` (posse — todos os jogos que a conta possui,
// instalados ou não) e um dicionário achatado no nível raiz, chave = slug do jogo, com
// metadado rico (descrição, tags, exe_path) — mas esse dicionário só cobre jogos que já
// foram instalados alguma vez, não o catálogo inteiro.

#[derive(Debug, Deserialize)]
struct ConfigJson {
    gala_data: GalaData,
    /// Captura todas as outras chaves de nível raiz (uma por slug de jogo já instalado).
    #[serde(flatten)]
    games: HashMap<String, ConfigGameEntry>,
}

#[derive(Debug, Deserialize)]
struct GalaData {
    data: GalaDataInner,
}

#[derive(Debug, Deserialize)]
struct GalaDataInner {
    showcase_content: ShowcaseContent,
}

#[derive(Debug, Deserialize)]
struct ShowcaseContent {
    content: ShowcaseContentInner,
}

#[derive(Debug, Deserialize)]
struct ShowcaseContentInner {
    user_collection: Vec<OwnedGame>,
}

/// Um jogo que a conta possui, segundo `gala_data` — não indica se está instalado.
#[derive(Debug, Deserialize)]
struct OwnedGame {
    prod_name: String,
    prod_slugged_name: String,
    prod_id_key_name: String,
}

/// Metadado rico de um jogo dentro do dicionário achatado do config.json.
/// Só existe para jogos já instalados alguma vez — não é o catálogo completo.
#[derive(Debug, Deserialize)]
struct ConfigGameEntry {
    description_short: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Normaliza uma tag "humana" da IndieGala (ex: "3D Platformer", "old school") para o
/// formato slug esperado por `tag_metadata.json` (ex: "3d-platformer", "old-school").
/// Tags que não existirem no mapa são descartadas silenciosamente por `classify_tags`.
fn slugify_indiegala_tag(tag: &str) -> String {
    tag.trim().to_lowercase().replace(' ', "-")
}

// === INDIEGALA SOURCE ===

/// Provedor de jogos instalados da IndieGala (IGClient).
pub struct IndiegalaSource {
    /// Caminho para `installed.json`. Se `None`, usa o caminho padrão do Windows.
    pub installed_json_path: Option<PathBuf>,
}

impl IndiegalaSource {
    pub fn new(installed_json_path: Option<PathBuf>) -> Self {
        Self {
            installed_json_path,
        }
    }

    /// Caminho padrão: `%APPDATA%\IGClient\storage\installed.json`.
    /// IGClient é Windows-only por enquanto (Linux/macOS "coming soon" segundo a própria IndieGala).
    #[cfg(target_os = "windows")]
    fn default_installed_json_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("IGClient").join("storage").join("installed.json"))
    }

    #[cfg(not(target_os = "windows"))]
    fn default_installed_json_path() -> Option<PathBuf> {
        None
    }

    /// Busca os jogos instalados com metadados completos.
    ///
    /// Retorna [`IndiegalaGame`] ao invés de [`SourceGame`] puro, permitindo
    /// persistir `description_raw` e `tags` (não fazem parte do `SourceGame` padrão).
    pub async fn fetch_installed_detailed(&self) -> Result<Vec<IndiegalaGame>, AppError> {
        let path = self
            .installed_json_path
            .clone()
            .or_else(Self::default_installed_json_path)
            .ok_or_else(|| {
                AppError::NotFound("Caminho do installed.json da IndieGala não encontrado.".into())
            })?;

        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Arquivo installed.json da IndieGala não encontrado em: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(&path).map_err(|e| AppError::IoError(e.to_string()))?;

        let entries: Vec<InstalledEntry> = serde_json::from_str(&content)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        let mut results = Vec::with_capacity(entries.len());

        for entry in entries {
            let item = &entry.target.item_data;
            let game_data = &entry.target.game_data;

            let install_path = entry.path.first().map(|root| {
                let candidate = Path::new(root).join(&item.slugged_name);
                if candidate.is_dir() {
                    candidate.to_string_lossy().to_string()
                } else {
                    log::warn!(
                        "IndieGala: pasta esperada '{}' não existe, usando raiz configurada '{root}' como install_path",
                        candidate.display()
                    );
                    root.clone()
                }
            });

            // exe_path costuma ser relativo (ex: "deponia.exe", "ScummVM/scummvm.exe").
            let executable_path = match (&game_data.exe_path, &install_path) {
                (Some(exe), Some(base)) => {
                    Some(Path::new(base).join(exe).to_string_lossy().to_string())
                }
                _ => None,
            };

            // playtime vem em SEGUNDOS.
            let playtime_minutes = Some((entry.playtime / 60.0).round() as u32);

            let raw_tag_slugs: Vec<String> = game_data
                .tags
                .iter()
                .map(|t| slugify_indiegala_tag(t))
                .collect();
            let tags =
                (!raw_tag_slugs.is_empty()).then(|| classify_and_sort_tags(raw_tag_slugs, 10));

            let source = SourceGame {
                platform: "Indiegala".to_string(),
                platform_game_id: item.id_key_name.clone(),
                name: Some(item.name.clone()),
                installed: true,
                executable_path,
                install_path,
                playtime_minutes,
                last_played: None,
            };

            results.push(IndiegalaGame {
                source,
                description_raw: game_data.description_short.clone(),
                tags,
            });
        }

        Ok(results)
    }

    /// Caminho padrão: `%APPDATA%\IGClient\config.json`.
    #[cfg(target_os = "windows")]
    fn default_config_json_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("IGClient").join("config.json"))
    }

    #[cfg(not(target_os = "windows"))]
    fn default_config_json_path() -> Option<PathBuf> {
        None
    }

    /// Busca a biblioteca completa (posse), cruzando com `installed.json` pra marcar o que já está instalado.
    ///
    /// Jogos possuídos mas nunca instalados entram com `installed: false` e, quando existir entrada
    /// correspondente no dicionário achatado do config.json (só cobre jogos já instalados alguma
    /// vez — não é garantido para todos), ganham descrição/tags também. Senão, ficam só com o nome.
    pub async fn fetch_full_library_detailed(
        &self,
        config_json_path: Option<PathBuf>,
    ) -> Result<Vec<IndiegalaGame>, AppError> {
        let path = config_json_path
            .or_else(Self::default_config_json_path)
            .ok_or_else(|| {
                AppError::NotFound("Caminho do config.json da IndieGala não encontrado.".into())
            })?;

        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Arquivo config.json da IndieGala não encontrado em: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(&path).map_err(|e| AppError::IoError(e.to_string()))?;
        let config: ConfigJson = serde_json::from_str(&content)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        // Reaproveita o que já está instalado (metadados completos, incluindo executable_path/install_path/playtime).
        // Se installed.json não existir ainda (usuário nunca instalou nada via IGClient), não falha a importação inteira.
        let installed = self.fetch_installed_detailed().await.unwrap_or_else(|e| {
            log::warn!(
                "IndieGala: não foi possível ler installed.json ({e}), seguindo só com posse"
            );
            Vec::new()
        });
        let installed_by_id: HashMap<String, IndiegalaGame> = installed
            .into_iter()
            .map(|g| (g.source.platform_game_id.clone(), g))
            .collect();

        let owned_games = config
            .gala_data
            .data
            .showcase_content
            .content
            .user_collection;

        let mut results = Vec::with_capacity(owned_games.len());

        for owned in owned_games {
            // Já instalado — usa a entrada completa que já tem.
            if let Some(installed_game) = installed_by_id.get(&owned.prod_id_key_name) {
                results.push(installed_game.clone());
                continue;
            }

            // Não instalado: tenta enriquecer com o dicionário achatado do config.json.
            let extra = config.games.get(&owned.prod_slugged_name);

            let raw_tag_slugs: Vec<String> = extra
                .map(|e| e.tags.iter().map(|t| slugify_indiegala_tag(t)).collect())
                .unwrap_or_default();
            let tags =
                (!raw_tag_slugs.is_empty()).then(|| classify_and_sort_tags(raw_tag_slugs, 10));

            let source = SourceGame {
                platform: "Indiegala".to_string(),
                platform_game_id: owned.prod_id_key_name.clone(),
                name: Some(owned.prod_name.clone()),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
            };

            results.push(IndiegalaGame {
                source,
                description_raw: extra.and_then(|e| e.description_short.clone()),
                tags,
            });
        }

        Ok(results)
    }
}

// Implementação da trait padrão (retorna apenas SourceGame, sem os extras)
#[async_trait]
impl crate::sources::providers::GameSource for IndiegalaSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        let detailed = self.fetch_installed_detailed().await?;
        Ok(detailed.into_iter().map(|g| g.source).collect())
    }
}
