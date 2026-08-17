//! Source para importar jogos da Epic Games
//!
//! Detecta jogos instalados lendo os arquivos de manifesto `.item` do Epic Games Launcher.
//! Importa a biblioteca completa via OAuth2 (login na conta Epic).
//!
//! **Observações:**
//! - **Windows:** os manifestos estão em `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests`.
//! - **Linux (Wine):** o mesmo caminho é resolvido dentro do Wine prefix em
//!   `<prefix>/drive_c/ProgramData/Epic/EpicGamesLauncher/Data/Manifests`.
//! - Cada arquivo `.item` é um JSON com nome, caminho de instalação e executável do jogo.

use crate::constants::{
    EPIC_CATALOG_BULK_ENDPOINT, EPIC_LIBRARY_ENDPOINT, EPIC_LOGIN_URL, EPIC_OAUTH_CLIENT_ID,
    EPIC_OAUTH_CLIENT_SECRET, EPIC_PSEUDO_REDIRECT_SCHEME, EPIC_REDIRECT_PREFIX,
    EPIC_TOKEN_ENDPOINT, OAUTH_CALLBACK_TIMEOUT_SECS,
};
use crate::errors::AppError;
use crate::providers::libraries::providers::{GameSource, OAuthGameSource, SourceGame};
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::config::{
    exchange_code_for_token, OAuthProviderConfig, TokenAuthMethod, TokenRequestMethod,
};
use crate::utils::oauth::token_store::save_oauth_token;
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

type CatalogTitles = HashMap<(String, String), (String, bool)>;
type CatalogResolution = Result<(String, HashMap<String, (String, bool)>), AppError>;

// === STRUCTS ===

/// Estrutura mínima do JSON dos arquivos `.item`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
    display_name: Option<String>,
    install_location: Option<String>,
    launch_executable: Option<String>,
    app_name: Option<String>,
}

/// Source responsável por importar jogos instalados via Epic Games
pub struct EpicSource {
    app_handle: AppHandle,
    #[allow(dead_code)]
    wine_prefix: Option<PathBuf>, // Wine prefix utilizado no Linux para localizar os manifestos do Epic. Ignorado no Windows.
    config: OAuthProviderConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EpicLibraryItem {
    pub(crate) namespace: String,
    #[serde(rename = "catalogItemId")]
    pub(crate) catalog_item_id: String,
}

#[derive(Debug, Deserialize)]
struct EpicLibraryResponse {
    records: Vec<EpicLibraryItem>,
    #[serde(rename = "responseMetadata", default)]
    response_metadata: Option<EpicLibraryMetadata>,
}

#[derive(Debug, Deserialize)]
struct EpicLibraryMetadata {
    #[serde(rename = "nextCursor", default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpicCatalogItem {
    title: String,
    #[serde(default)]
    categories: Vec<EpicCategory>,
}

#[derive(Debug, Deserialize)]
struct EpicCategory {
    path: String,
}

impl EpicCatalogItem {
    fn is_game(&self) -> bool {
        self.categories.iter().any(|c| c.path == "games")
    }
}

#[derive(Debug, Deserialize)]
struct LegendaryInstalledGame {
    app_name: String,
    install_path: String,
    title: String,
    executable: String,
    is_dlc: bool,
}

// === JOGOS INSTALADOS ===

impl EpicSource {
    pub fn new(app_handle: AppHandle, wine_prefix: Option<PathBuf>) -> Self {
        let config = OAuthProviderConfig {
            provider_id: "epic",
            client_id: EPIC_OAUTH_CLIENT_ID.to_string(),
            client_secret: Some(EPIC_OAUTH_CLIENT_SECRET.to_string()),
            authorize_endpoint: EPIC_LOGIN_URL.to_string(),
            token_endpoint: EPIC_TOKEN_ENDPOINT.to_string(),
            redirect_uri: EPIC_REDIRECT_PREFIX.to_string(),
            scopes: vec![],
            uses_pkce: false,
            extra_params: vec![],
            token_request_method: TokenRequestMethod::Post,
            token_auth_method: TokenAuthMethod::BasicHeader,
        };

        Self {
            app_handle,
            wine_prefix,
            config,
        }
    }

    /// Resolve o diretório de manifestos do Epic Games Launcher.
    ///
    /// - **Windows:** `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests`
    /// - **Linux:** `<wine_prefix>/drive_c/ProgramData/Epic/EpicGamesLauncher/Data/Manifests`
    fn resolve_manifest_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use crate::constants::EPIC_MANIFEST_PATH_WINDOWS;
            let path = PathBuf::from(EPIC_MANIFEST_PATH_WINDOWS);
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(prefix) = &self.wine_prefix {
                let path = prefix
                    .join("drive_c")
                    .join("ProgramData")
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests");
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn parse_manifest(path: &Path) -> Result<SourceGame, AppError> {
        let content = fs::read_to_string(path)?;

        let manifest: EpicManifest = serde_json::from_str(&content)?;

        let name = manifest
            .display_name
            .unwrap_or_else(|| "Unknown".to_string());

        let install_location = manifest
            .install_location
            .ok_or_else(|| AppError::EpicMissingField("InstallLocation".into()))?;

        let launch_executable = manifest
            .launch_executable
            .ok_or_else(|| AppError::EpicMissingField("LaunchExecutable".into()))?;

        let app_name = manifest
            .app_name
            .ok_or_else(|| AppError::EpicMissingField("AppName".into()))?;

        let full_executable_path = Path::new(&install_location)
            .join(&launch_executable)
            .to_string_lossy()
            .to_string();

        Ok(SourceGame {
            library: "Epic".to_string(),
            library_game_id: app_name,
            name: Some(name),
            installed: true,
            executable_path: Some(full_executable_path),
            install_path: Some(install_location),
            playtime_minutes: None,
            last_played: None,
            source_label: None,
        })
    }

    /// Importa jogos instalados no Linux via Legendary
    #[cfg(target_os = "linux")]
    async fn import_installed_via_legendary(&self) -> Result<Vec<SourceGame>, AppError> {
        use crate::models::ManagedTool;
        use crate::services::tools;

        let status = tools::get_tool_status(&self.app_handle, ManagedTool::Legendary)?;

        let path = match status {
            crate::models::ToolStatus::Found { path, .. } => path,
            crate::models::ToolStatus::NotFound => return Ok(vec![]), // não quebra o import
        };

        let json = tools::run_json(Path::new(&path), &["list-installed", "--json"])?;
        parse_legendary_installed(json)
    }

    /// Importa todos os jogos instalados detectados nos manifestos locais
    pub async fn import_installed(&self) -> Result<Vec<SourceGame>, AppError> {
        let manifest_dir = match self.resolve_manifest_dir() {
            Some(dir) => dir,
            None => {
                #[cfg(target_os = "linux")]
                {
                    return self.import_installed_via_legendary().await;
                }
                #[cfg(not(target_os = "linux"))]
                {
                    return Ok(vec![]); // Epic não-instalada ou sem jogos
                }
            }
        };

        let mut games = Vec::new();

        for entry in fs::read_dir(&manifest_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !is_item_file(&path) {
                continue;
            }

            match Self::parse_manifest(&path) {
                Ok(game) => games.push(game),
                Err(err) => {
                    log::warn!("Erro ao processar manifest {:?}: {}", path, err);
                    continue;
                }
            }
        }

        Ok(games)
    }
}

#[cfg(target_os = "linux")]
fn parse_legendary_installed(json: serde_json::Value) -> Result<Vec<SourceGame>, AppError> {
    let games: Vec<LegendaryInstalledGame> = serde_json::from_value(json)
        .map_err(|e| AppError::ParseError(format!("Falha ao parsear saída do Legendary: {e}")))?;

    Ok(games
        .into_iter()
        .filter(|g| !g.is_dlc)
        .filter(|g| Path::new(&g.install_path).exists())
        .map(|g| {
            let full_executable_path = Path::new(&g.install_path)
                .join(&g.executable)
                .to_string_lossy()
                .to_string();

            SourceGame {
                library: "Epic".to_string(),
                library_game_id: g.app_name,
                name: Some(g.title),
                installed: true,
                executable_path: Some(full_executable_path),
                install_path: Some(g.install_path),
                playtime_minutes: None,
                last_played: None,
            }
        })
        .collect())
}

// === BIBLIOTECA COMPLETA (OAuth) ===

impl EpicSource {
    /// Importa a biblioteca completa de jogos possuídos na conta Epic (requer login prévio).
    pub async fn fetch_library_detailed(&self) -> Result<Vec<SourceGame>, AppError> {
        let access_token = self.ensure_valid_token().await?;
        let items = fetch_all_library_items(&access_token).await?;

        let mut by_namespace: HashMap<String, Vec<String>> = HashMap::new();
        for item in &items {
            if item.namespace == "ue" {
                continue;
            }
            by_namespace
                .entry(item.namespace.clone())
                .or_default()
                .push(item.catalog_item_id.clone());
        }

        let titles = fetch_all_catalog_titles(&access_token, &by_namespace).await?;

        // Escolhe, por namespace, o catalogItemId marcado como "games" no catálogo.
        let mut chosen_per_namespace: HashMap<String, (String, String)> = HashMap::new(); // namespace -> (catalog_item_id, title)

        for item in &items {
            if item.namespace == "ue" {
                continue;
            }

            let Some((title, is_game)) =
                titles.get(&(item.namespace.clone(), item.catalog_item_id.clone()))
            else {
                continue; // catálogo não resolveu esse item; ignora, não usa como fallback de namespace
            };

            if *is_game {
                chosen_per_namespace
                    .entry(item.namespace.clone())
                    .or_insert_with(|| (item.catalog_item_id.clone(), title.clone()));
            }
        }

        let games = chosen_per_namespace
            .into_iter()
            .map(|(namespace, (_, title))| SourceGame {
                library: "Epic".to_string(),
                library_game_id: namespace,
                name: Some(title),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
                source_label: None,
            })
            .collect();

        Ok(games)
    }
}

#[async_trait]
impl GameSource for EpicSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        self.fetch_library_detailed().await
    }
}

#[async_trait]
impl OAuthGameSource for EpicSource {
    fn oauth_config(&self) -> &OAuthProviderConfig {
        &self.config
    }

    fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }

    /// A Epic não devolve `code` na query do redirect (como o GOG) — o `authorizationCode`
    /// vem como JSON no corpo da própria página de redirect. Um script injetado via
    /// `on_page_load` lê esse corpo e navega para uma pseudo-URL própria
    /// (`playlite://epic-auth-code?data=...`), que o `on_navigation` consegue interceptar
    /// (ele só recebe a URL da navegação, nunca o conteúdo da página).
    async fn login(&self) -> Result<(), AppError> {
        let config = self.oauth_config();
        let login_url = url::Url::parse(&config.authorize_endpoint)
            .map_err(|e| AppError::OAuthConfigError(format!("URL de login Epic inválida: {e}")))?;

        let (tx, rx) = mpsc::channel::<Result<String, String>>();
        let tx_nav = tx.clone();

        let window = WebviewWindowBuilder::new(
            &self.app_handle,
            "epic_oauth_login",
            WebviewUrl::External(login_url),
        )
            .title("Login Epic Games")
            .inner_size(480.0, 720.0)
            .on_navigation(move |url| {
                let url_str = url.as_str();
                if url_str.starts_with(EPIC_PSEUDO_REDIRECT_SCHEME) {
                    let data = url
                        .query_pairs()
                        .find(|(k, _)| k == "data")
                        .map(|(_, v)| v.to_string());
                    let result =
                        data.ok_or_else(|| "Payload 'data' ausente no callback Epic".to_string());
                    let _ = tx_nav.send(result);
                    return false;
                }
                true
            })
            .on_page_load(move |window, payload| {
                if payload.url().as_str().starts_with(EPIC_REDIRECT_PREFIX) {
                    let script = format!(
                        r#"(function() {{
                        try {{
                            var data = document.body.innerText || document.body.textContent || "";
                            window.location.href = "{scheme}?data=" + encodeURIComponent(data);
                        }} catch (e) {{
                            window.location.href = "{scheme}?data=" + encodeURIComponent(JSON.stringify({{error: String(e)}}));
                        }}
                    }})();"#,
                        scheme = EPIC_PSEUDO_REDIRECT_SCHEME
                    );

                    let _ = window.eval(&script);
                }
            })
            .build()
            .map_err(|e| {
                AppError::OAuthConfigError(format!("Falha ao abrir janela de login Epic: {e}"))
            })?;

        let raw_json = tokio::task::spawn_blocking(move || {
            rx.recv_timeout(Duration::from_secs(OAUTH_CALLBACK_TIMEOUT_SECS))
        })
            .await
            .map_err(|e| AppError::OAuthConfigError(format!("Task de callback falhou: {e}")))?
            .map_err(|_| AppError::OAuthConfigError("Tempo limite de login excedido".to_string()))?
            .map_err(AppError::OAuthConfigError)?;

        let _ = window.close();

        #[derive(Deserialize)]
        struct EpicAuthRedirect {
            #[serde(rename = "authorizationCode")]
            authorization_code: Option<String>,
        }

        let parsed: EpicAuthRedirect = serde_json::from_str(&raw_json).map_err(|e| {
            AppError::OAuthConfigError(format!("Resposta de login Epic inesperada: {e}"))
        })?;

        let code = parsed.authorization_code.ok_or_else(|| {
            AppError::OAuthConfigError(
                "Epic não retornou 'authorizationCode' — login cancelado ou falhou".to_string(),
            )
        })?;

        let token_response = exchange_code_for_token(config, &code, None).await?;
        save_oauth_token(&self.app_handle, config.provider_id, &token_response.into())?;

        Ok(())
    }
}

// === FUNÇÕES AUXILIARES: biblioteca + catálogo ===

/// Busca a biblioteca completa da Epic Games, paginando via cursor.
///
/// **Importante:** `includeMetadata` precisa ser `"true"` — com `"false"`, a API omite
/// `responseMetadata`/`nextCursor` da resposta e retorna só a primeira página sem sinalizar
/// que há mais páginas disponíveis.
async fn fetch_all_library_items(access_token: &str) -> Result<Vec<EpicLibraryItem>, AppError> {
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut request = HTTP_CLIENT
            .get(EPIC_LIBRARY_ENDPOINT)
            .bearer_auth(access_token)
            .query(&[("includeMetadata", "true")]);

        if let Some(c) = &cursor {
            request = request.query(&[("cursor", c)]);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(AppError::NetworkError(format!(
                "Epic retornou HTTP {status}: {body}"
            )));
        }

        let parsed: EpicLibraryResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::ParseError(format!(
                "Falha ao parsear biblioteca Epic: {e} — corpo: {body}"
            ))
        })?;

        let record_count = parsed.records.len();
        all.extend(parsed.records);

        let next_cursor = parsed.response_metadata.and_then(|m| m.next_cursor);
        log::debug!(
            "Epic: página com {record_count} itens (total acumulado: {})",
            all.len()
        );

        match next_cursor {
            Some(c) if !c.is_empty() => cursor = Some(c),
            _ => break,
        }
    }

    Ok(all)
}

/// Resolve os títulos reais dos itens de um namespace via catálogo.
async fn fetch_catalog_titles(
    access_token: &str,
    namespace: &str,
    catalog_item_ids: &[String],
) -> Result<HashMap<String, (String, bool)>, AppError> {
    if catalog_item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let url = format!("{EPIC_CATALOG_BULK_ENDPOINT}/{namespace}/bulk/items");

    let mut query: Vec<(&str, &str)> = catalog_item_ids
        .iter()
        .map(|id| ("id", id.as_str()))
        .collect();
    query.push(("includeDLCDetails", "false"));
    query.push(("includeMainGameDetails", "false"));

    let response = HTTP_CLIENT
        .get(&url)
        .bearer_auth(access_token)
        .query(&query)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log::warn!("Epic catalog (namespace={namespace}) retornou HTTP {status}: {body}");
        return Ok(HashMap::new());
    }

    let parsed: HashMap<String, EpicCatalogItem> = serde_json::from_str(&body).map_err(|e| {
        AppError::ParseError(format!(
            "Falha ao parsear catálogo Epic: {e} — corpo: {body}"
        ))
    })?;

    Ok(parsed
        .into_iter()
        .map(|(id, item)| {
            let is_game = item.is_game();
            (id, (item.title, is_game))
        })
        .collect())
}

/// Resolve os títulos de todos os namespaces em paralelo (até 8 chamadas simultâneas) em vez de sequencialmente — reduz o tempo de import.
async fn fetch_all_catalog_titles(
    access_token: &str,
    by_namespace: &HashMap<String, Vec<String>>,
) -> Result<CatalogTitles, AppError> {
    // Clonar o token de acesso para que cada tarefa assíncrona possua seu próprio dono do token e não dependa de um borrow com lifetime restrito.
    let access_owned = access_token.to_string();

    let jobs: Vec<(String, Vec<String>)> = by_namespace
        .iter()
        .map(|(namespace, ids)| (namespace.clone(), ids.clone()))
        .collect();

    let results: Vec<CatalogResolution> =
        stream::iter(jobs)
            .map(move |(namespace, ids)| {
                let access = access_owned.clone();
                async move {
                    let resolved = fetch_catalog_titles(&access, &namespace, &ids).await?;
                    Ok::<_, AppError>((namespace, resolved))
                }
            })
            .buffer_unordered(5)
            .collect()
            .await;

    let mut titles = HashMap::new();
    for result in results {
        let (namespace, resolved) = result?;
        for (catalog_item_id, (title, is_game)) in resolved {
            titles.insert((namespace.clone(), catalog_item_id), (title, is_game));
        }
    }

    Ok(titles)
}

/// Cruza a biblioteca completa (OAuth) com os jogos detectados localmente via manifesto,
/// marcando como instalados os que baterem por nome (case-insensitive) — já que o `AppName`
/// do manifesto local e o `catalogItemId` da API não são o mesmo ID. Jogos instalados que,
/// por algum motivo, não aparecerem na biblioteca são mantidos como entradas à parte.
pub fn merge_local_install_status(
    library_games: &mut Vec<SourceGame>,
    local_games: Vec<SourceGame>,
) {
    for local in local_games {
        let local_name = local
            .name
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        let matched = library_games
            .iter_mut()
            .find(|g| g.name.as_deref().unwrap_or_default().trim().to_lowercase() == local_name);

        match matched {
            Some(g) => {
                g.installed = true;
                g.executable_path = local.executable_path;
                g.install_path = local.install_path;
            }
            None => library_games.push(local),
        }
    }
}

fn is_item_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("item"))
        .unwrap_or(false)
}
