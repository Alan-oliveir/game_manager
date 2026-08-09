//! Importação de jogos da plataforma Legacy Games.
//!
//! Lê o arquivo `app-state.json` do launcher da Legacy Games para detectar
//! jogos adquiridos (via compra ou giveaway), cruzando com o catálogo embutido
//! para obter metadados como capa, descrição e verificar instalação local.

use crate::errors::AppError;
use crate::sources::providers::SourceGame;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::warn;

// === ESTRUTURA PÚBLICA DA PLATAFORMA LEGACY GAMES ===

/// Jogo importado da Legacy Games com campos adicionais além do `SourceGame` padrão.
#[derive(Debug, Clone)]
pub struct LegacyGame {
    pub source: SourceGame,
    pub cover_url: Option<String>,
    pub description: Option<String>,
}

// === ESTRUTURAS INTERNAS PARA DESSERIALIZAÇÃO DO JSON DE ESTADO DO LAUNCHER ===

#[derive(Deserialize)]
struct AppStateJson {
    settings: LegacySettings,
    #[serde(rename = "siteData")]
    site_data: SiteData,
    user: UserData,
}

#[derive(Deserialize)]
struct LegacySettings {
    #[serde(rename = "gameLibraryPath")]
    game_library_path: Vec<String>,
}

#[derive(Deserialize)]
struct SiteData {
    catalog: Vec<CatalogProduct>,
}

#[derive(Deserialize)]
struct CatalogProduct {
    // Alguns produtos (ex: itens de reposição, bundles sem jogos) não têm o campo `games`
    #[serde(default)]
    games: Vec<CatalogGame>,
}

#[derive(Deserialize)]
struct CatalogGame {
    game_id: String,
    game_name: String,
    #[serde(rename = "game_description")]
    game_description: Option<String>,
    #[serde(rename = "game_coverart")]
    game_coverart: Option<String>,
}

#[derive(Deserialize)]
struct UserData {
    #[serde(rename = "giveawayDownloads")]
    giveaway_downloads: Vec<AcquiredGame>,
}

#[derive(Deserialize)]
struct AcquiredGame {
    // product_id pode vir como String ou como número inteiro no JSON
    #[serde(deserialize_with = "deserialize_id_as_string")]
    product_id: String,
    // game_id também pode vir como String ou número
    #[serde(deserialize_with = "deserialize_id_as_string")]
    game_id: String,
}

// === NORMALIZAÇÃO DE NOMES ===

/// Normaliza o nome de um jogo removendo sufixos de edição especial comuns da Legacy Games.
///
/// Sufixos removidos (case-insensitive):
/// - `CE` / `C.E.` — Collector's Edition
/// - `Collector's Edition` / `Collectors Edition`
/// - `Special Edition` / `SE`
/// - `Deluxe Edition` / `DE`
/// - `Premium Edition`
/// - `GOTY` / `Game of the Year Edition`
///
/// # Exemplos
/// ```rust
/// # use game_manager_lib::sources::legacy::normalize_game_name;
/// assert_eq!(normalize_game_name("Fable CE"), "Fable");
/// assert_eq!(normalize_game_name("Mystery Game Collector's Edition"), "Mystery Game");
/// assert_eq!(normalize_game_name("Some Game - Deluxe Edition"), "Some Game");
/// ```
fn normalize_game_name(name: &str) -> String {
    // Sufixos a remover, por ordem de prioridade (mais longos primeiro para evitar match parcial)
    const SUFFIXES: &[&str] = &[
        "Collector's Edition",
        "Collectors Edition",
        "Collection Edition",
        "Game of the Year Edition",
        "Special Edition",
        "Deluxe Edition",
        "Premium Edition",
        "GOTY Edition",
        "C.E.",
        "GOTY",
        "CE",
        "SE",
        "DE",
    ];

    let trimmed = name.trim();

    for suffix in SUFFIXES {
        // Compara o final da string ignorando capitalização
        if let Some(rest) = trimmed.to_lowercase().strip_suffix(&suffix.to_lowercase()) {
            let cut = &trimmed[..rest.len()];
            // Remove separadores opcionais antes do sufixo: espaço, hífen, vírgula, dois-pontos
            return cut
                .trim_end_matches(['-', ':', ',', ' '])
                .trim()
                .to_string();
        }
    }

    trimmed.to_string()
}

// === SANITIZAÇÃO DE CAMINHOS ===

/// Remove caracteres inválidos para nomes de arquivo/pasta no Windows.
///
/// A Legacy Games faz o mesmo ao criar a pasta de instalação a partir do
/// nome do catálogo — por exemplo, `"Around The World 1: Travel to Brazil CE"`
/// vira a pasta `Around The World 1 Travel to Brazil CE` (sem o `:`, mas com
/// os espaços preservados e o sufixo de edição mantido). Diferente de
/// `normalize_game_name`, esta função NÃO remove sufixos de edição — apenas
/// os caracteres que o sistema de arquivos do Windows não aceita.
fn sanitize_path_component(name: &str) -> String {
    name.replace([':', '/', '\\', '*', '?', '"', '<', '>', '|'], "")
        // Colapsa espaços duplos que podem sobrar após remover o caractere
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// === HELPER DE DESSERIALIZAÇÃO ===

/// Helper: deserializa product_id tanto como String quanto como número
fn deserialize_id_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Err(<D::Error as serde::de::Error>::custom(format!(
            "expected string or number, got {:?}",
            other
        ))),
    }
}

// === LEGACY GAMES SOURCE ===

/// Provedor de jogos da Legacy Games.
///
/// # Exemplo
/// ```rust,no_run
/// # use game_manager_lib::sources::legacy::LegacySource;
/// let source = LegacySource::new(None); // busca o caminho padrão
/// # tokio_test::block_on(async {
/// let games = source.fetch_games_detailed().await.unwrap();
/// # });
/// ```
pub struct LegacySource {
    /// Caminho para o arquivo `app-state-bck.json`. Se `None`, usa o caminho padrão do sistema operacional.
    pub app_state_path: Option<PathBuf>,
    /// Wine prefix utilizado no Linux para localizar o launcher da Legacy Games. Ignorado no Windows.
    pub wine_prefix: Option<PathBuf>,
}

impl LegacySource {
    pub fn new(app_state_path: Option<PathBuf>) -> Self {
        Self {
            app_state_path,
            wine_prefix: None,
        }
    }

    pub fn new_with_wine(app_state_path: Option<PathBuf>, wine_prefix: Option<PathBuf>) -> Self {
        Self {
            app_state_path,
            wine_prefix,
        }
    }

    /// Retorna o caminho padrão do `app-state.json` conforme o sistema operacional.
    fn default_app_state_path(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            // dirs::data_dir() retorna %APPDATA%\Roaming — caminho correto do Legacy Games Launcher
            dirs::data_dir().map(|d| d.join("legacy-games-launcher").join("app-state.json"))
        }

        #[cfg(target_os = "linux")]
        {
            // No Linux só há suporte via Wine prefix configurado pelo usuário
            let prefix = self.wine_prefix.as_ref()?;
            let user = std::env::var("USER").ok()?;

            Some(
                prefix
                    .join("drive_c")
                    .join("users")
                    .join(&user)
                    .join("AppData")
                    .join("Roaming")
                    .join("legacy-games-launcher")
                    .join("app-state.json"),
            )
        }
    }

    /// Verifica se um jogo está instalado no diretório da biblioteca.
    ///
    /// A Legacy Games instala cada jogo em `<gameLibraryPath>/<game_name>/`, onde `<game_name>` é o
    /// nome do catálogo com os caracteres inválidos de caminho removidos (o `:` de "Nome: Subtítulo",
    /// por exemplo) — mas com sufixos de edição como "CE" preservados. O executável principal costuma
    /// ter o mesmo nome, sem espaços.
    fn resolve_install_info(
        library_paths: &[String],
        game_name: &str,
    ) -> (bool, Option<String>, Option<String>) {
        let sanitized_name = sanitize_path_component(game_name);

        for base in library_paths {
            let game_dir = Path::new(base).join(&sanitized_name);
            if game_dir.is_dir() {
                let install_path = game_dir.to_string_lossy().to_string();

                // Tenta localizar o executável com o mesmo nome do jogo
                let exe_path = Self::find_executable(&game_dir, &sanitized_name);

                return (true, Some(install_path), exe_path);
            }
        }
        (false, None, None)
    }

    /// Procura o executável `.exe` dentro do diretório de instalação.
    ///
    /// Estratégia: primeiro tenta `<game_name>.exe` (sem espaços nem caracteres inválidos, como a
    /// Legacy Games nomeia o binário), depois qualquer `.exe` que não seja desinstalador ou redistributível.
    fn find_executable(game_dir: &Path, game_name: &str) -> Option<String> {
        // 1. Tenta o executável com o mesmo nome do jogo (sem espaços)
        let sanitized = sanitize_path_component(game_name).replace(' ', "");
        let candidate = game_dir.join(format!("{}.exe", sanitized));
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }

        // 2. Busca qualquer .exe no diretório raiz, ignorando utilitários comuns
        let ignored = [
            "unins",
            "redist",
            "setup",
            "vc_redist",
            "dxsetup",
            "directx",
        ];

        if let Ok(entries) = std::fs::read_dir(game_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("exe") {
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if !ignored.iter().any(|&ig| file_stem.contains(ig)) {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        None
    }

    /// Busca os jogos adquiridos com metadados completos.
    ///
    /// Retorna [`LegacyGame`] ao invés de [`SourceGame`] puro, permitindo persistir `cover_url` e `description_raw`.
    pub async fn fetch_games_detailed(&self) -> Result<Vec<LegacyGame>, AppError> {
        // Resolve o caminho do arquivo de estado
        let state_path = self
            .app_state_path
            .clone()
            .or_else(|| self.default_app_state_path())
            .ok_or_else(|| {
                AppError::NotFound("Caminho do app-state da Legacy Games não encontrado.".into())
            })?;

        if !state_path.exists() {
            return Err(AppError::NotFound(format!(
                "Arquivo de estado da Legacy Games não encontrado em: {}",
                state_path.display()
            )));
        }

        // Lê e desserializa o JSON
        let content =
            std::fs::read_to_string(&state_path).map_err(|e| AppError::IoError(e.to_string()))?;

        let app_state: AppStateJson = serde_json::from_str(&content)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        let library_paths = &app_state.settings.game_library_path;

        // Monta índice game_id -> CatalogGame para busca O(1)
        let catalog_index: std::collections::HashMap<&str, &CatalogGame> = app_state
            .site_data
            .catalog
            .iter()
            .flat_map(|product| product.games.iter())
            .map(|game| (game.game_id.as_str(), game))
            .collect();

        let mut results: Vec<LegacyGame> = Vec::new();

        for acquired in &app_state.user.giveaway_downloads {
            // Cruza game_id com o catálogo local para obter metadados
            let Some(catalog_game) = catalog_index.get(acquired.game_id.as_str()) else {
                warn!(
                    game_id = %acquired.game_id,
                    product_id = %acquired.product_id,
                    "Jogo não encontrado no catálogo — pode ter sido removido da Legacy Games. Ignorado."
                );
                continue;
            };

            // IMPORTANTE: usa o nome cru do catálogo (com sufixo de edição, ex. "CE")para resolver
            // a pasta de instalação, pois é assim que a Legacy Games nomeia o diretório no disco.
            // `normalize_game_name` só deve ser aplicado ao nome exibido ao usuário, não ao caminho
            // no sistema de arquivos.
            let (installed, install_path, executable_path) =
                Self::resolve_install_info(library_paths, &catalog_game.game_name);

            let source = SourceGame {
                platform: "LegacyGames".to_string(),
                platform_game_id: acquired.product_id.clone(),
                name: Some(normalize_game_name(&catalog_game.game_name)),
                installed,
                executable_path,
                install_path,
                playtime_minutes: Some(0),
                last_played: None,
            };

            results.push(LegacyGame {
                source,
                cover_url: catalog_game.game_coverart.clone(),
                description: catalog_game.game_description.clone(),
            });
        }

        Ok(results)
    }
}

// Implementação da trait padrão (retorna apenas SourceGame, sem os extras)
#[async_trait]
impl crate::sources::providers::GameSource for LegacySource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        let detailed = self.fetch_games_detailed().await?;
        Ok(detailed.into_iter().map(|g| g.source).collect())
    }
}
