//! Source para importar a biblioteca da Amazon Games e Scan local de instalados via Amazon Games App.
//!
//! Diferente de GOG/Epic, a Amazon não usa OAuth2 padrão — usa um esquema de "registro de
//! dispositivo" (o mesmo mecanismo usado por apps Kindle/Alexa), baseado em OpenID 2.0 para
//! o login e um endpoint próprio (`/auth/register`) para trocar o código por tokens.
//!
//! Para detectar jogos instalados lê o SQLite mantido pelo próprio launcher em
//! `%LOCALAPPDATA%\Amazon Games\Data\Games\Sql\GameInstallInfo.sqlite`.

use crate::constants::{
    AMAZON_API, AMAZON_APP_NAME, AMAZON_APP_VERSION, AMAZON_ASSOC_HANDLE, AMAZON_DEVICE_TYPE,
    AMAZON_ENTITLEMENTS_KEY_ID, AMAZON_GAMING_DISTRIBUTION_ENTITLEMENTS, AMAZON_MARKETPLACE_ID,
    AMAZON_REDIRECT_PREFIX, OAUTH_CALLBACK_TIMEOUT_SECS,
};
use crate::errors::AppError;
use crate::sources::providers::{GameSource, SourceGame};
use crate::utils::http_client::HTTP_CLIENT;
use crate::utils::oauth::config::{now_unix, OAuthToken};
use crate::utils::oauth::core::PkceChallenge;
use crate::utils::oauth::token_store::{delete_oauth_token, load_oauth_token, save_oauth_token};
use async_trait::async_trait;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use url::Url;

const AMAZON_PROVIDER_ID: &str = "amazon";

// === STRUCTS: resposta de register_device ===

#[derive(Debug, Deserialize)]
struct RegisterDeviceResponse {
    response: RegisterDeviceResponseInner,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceResponseInner {
    success: RegisterDeviceSuccess,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceSuccess {
    tokens: RegisterDeviceTokens,
    extensions: RegisterDeviceExtensions,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceTokens {
    bearer: RegisterDeviceBearerToken,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceBearerToken {
    access_token: String,
    refresh_token: Option<String>,
    #[serde(default, deserialize_with = "deserialize_expires_in")]
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceExtensions {
    customer_info: RegisterDeviceCustomerInfo,
    device_info: RegisterDeviceDeviceInfo,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceCustomerInfo {
    #[serde(default)]
    given_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisterDeviceDeviceInfo {
    #[serde(default)]
    device_serial_number: Option<String>,
}

// === STRUCTS: refresh ===

#[derive(Debug, Deserialize)]
struct AmazonRefreshResponse {
    access_token: String,
    #[serde(default, deserialize_with = "deserialize_expires_in")]
    expires_in: Option<u64>,
}

// === STRUCTS: biblioteca (entitlements) ===

#[derive(Debug, Deserialize)]
struct AmazonEntitlementsResponse {
    entitlements: Vec<AmazonEntitlement>,
    #[serde(rename = "nextToken", default)]
    next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AmazonEntitlement {
    product: AmazonProduct,
}

#[derive(Debug, Deserialize)]
struct AmazonProduct {
    id: String,
    #[serde(default)]
    title: Option<String>,
}

// === STRUCTS: fuel.json ===

#[derive(Deserialize)]
struct FuelMain {
    #[serde(rename = "Command")]
    command: String,
}

#[derive(Deserialize)]
struct FuelManifest {
    #[serde(rename = "Main")]
    main: FuelMain,
}

// === SOURCE ===

pub struct AmazonSource {
    app_handle: AppHandle,
}

impl AmazonSource {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn is_authenticated(&self) -> Result<bool, AppError> {
        Ok(load_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID)?.is_some())
    }

    /// Fluxo de login: gera identidade de dispositivo local (PKCE + serial arbitrário),
    /// abre a tela de login da Amazon, intercepta o redirect final e troca o código
    /// obtido por tokens via `register_device`.
    ///
    /// **Observação:** o esquema OpenID 2.0 usado aqui não tem parâmetro `state` —
    /// diferente de GOG/Epic, não há validação de CSRF nesse retorno porque a própria
    /// Amazon não devolve nada equivalente nesse fluxo.
    pub async fn login(&self) -> Result<(), AppError> {
        let pkce = PkceChallenge::generate();
        let serial = generate_device_serial();
        let client_id_hex = generate_client_id(&serial);

        let auth_url = build_amazon_auth_url(&client_id_hex, &pkce.challenge)?;

        let (tx, rx) = mpsc::channel::<Result<String, String>>();

        let window = WebviewWindowBuilder::new(
            &self.app_handle,
            "amazon_oauth_login",
            WebviewUrl::External(auth_url),
        )
        .title("Login Amazon Games")
        .inner_size(480.0, 720.0)
        .on_navigation(move |url| {
            let url_str = url.as_str();
            if url_str.starts_with(AMAZON_REDIRECT_PREFIX) {
                let code = url
                    .query_pairs()
                    .find(|(k, _)| k == "openid.oa2.authorization_code")
                    .map(|(_, v)| v.to_string());
                let result = code.ok_or_else(|| {
                    "Redirect da Amazon alcançado, mas sem 'openid.oa2.authorization_code' na URL"
                        .to_string()
                });
                let _ = tx.send(result);
                return false;
            }
            true
        })
        .build()
        .map_err(|e| {
            AppError::OAuthConfigError(format!("Falha ao abrir janela de login Amazon: {e}"))
        })?;

        let code = tokio::task::spawn_blocking(move || {
            rx.recv_timeout(Duration::from_secs(OAUTH_CALLBACK_TIMEOUT_SECS))
        })
        .await
        .map_err(|e| AppError::OAuthConfigError(format!("Task de callback falhou: {e}")))?
        .map_err(|_| AppError::OAuthConfigError("Tempo limite de login excedido".to_string()))?
        .map_err(AppError::OAuthConfigError)?;

        let _ = window.close();

        self.register_device(&code, &client_id_hex, &pkce.verifier, &serial)
            .await
    }

    async fn register_device(
        &self,
        code: &str,
        client_id: &str,
        verifier: &str,
        serial: &str,
    ) -> Result<(), AppError> {
        let body = serde_json::json!({
            "auth_data": {
                "authorization_code": code,
                "client_domain": "DeviceLegacy",
                "client_id": client_id,
                "code_algorithm": "SHA-256",
                "code_verifier": verifier,
                "use_global_authentication": false,
            },
            "registration_data": {
                "app_name": AMAZON_APP_NAME,
                "app_version": AMAZON_APP_VERSION,
                "device_model": "Windows",
                "device_name": null,
                "device_serial": serial,
                "device_type": AMAZON_DEVICE_TYPE,
                "domain": "Device",
                "os_version": "10.0.19044.0",
            },
            "requested_extensions": ["customer_info", "device_info"],
            "requested_token_type": ["bearer", "mac_dms"],
            "user_context_map": {},
        });

        let response = HTTP_CLIENT
            .post(format!("{AMAZON_API}/auth/register"))
            .header("User-Agent", "AGSLauncher/1.0.0")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(AppError::OAuthTokenExchangeError(format!(
                "Amazon register_device retornou HTTP {status}: {resp_body}"
            )));
        }

        let parsed: RegisterDeviceResponse = serde_json::from_str(&resp_body).map_err(|e| {
            AppError::OAuthTokenExchangeError(format!(
                "Resposta inesperada do register_device Amazon: {e}"
            ))
        })?;

        let success = parsed.response.success;
        let bearer = success.tokens.bearer;

        let mut extra = HashMap::new();
        let device_serial = success
            .extensions
            .device_info
            .device_serial_number
            .unwrap_or_else(|| serial.to_string()); // fallback pro serial gerado localmente
        extra.insert("device_serial".to_string(), device_serial);

        if let Some(name) = success.extensions.customer_info.given_name {
            extra.insert("given_name".to_string(), name);
        }

        let token = OAuthToken {
            access_token: bearer.access_token,
            refresh_token: bearer.refresh_token,
            expires_at: bearer.expires_in.map(|secs| now_unix() + secs as i64),
            scope: None,
            extra,
        };

        save_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID, &token)?;

        Ok(())
    }

    /// Retorna um access_token válido, renovando via refresh_token se expirado.
    /// Fluxo de refresh custom (não usa o `refresh_access_token` genérico) porque
    /// o corpo esperado pela Amazon não segue o formato OAuth2 padrão.
    pub async fn ensure_valid_token(&self) -> Result<String, AppError> {
        let stored = load_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID)?
            .ok_or_else(|| AppError::OAuthTokenNotFound(AMAZON_PROVIDER_ID.to_string()))?;

        if !stored.is_expired() {
            return Ok(stored.access_token);
        }

        let refresh_token = stored.refresh_token.clone().ok_or_else(|| {
            AppError::OAuthTokenNotFound(
                "amazon: token expirado e sem refresh_token, é necessário novo login".to_string(),
            )
        })?;

        let refreshed = refresh_access_token_amazon(&refresh_token).await?;

        let new_token = OAuthToken {
            access_token: refreshed.access_token,
            refresh_token: stored.refresh_token.clone(), // Amazon não reemite no refresh
            expires_at: refreshed.expires_in.map(|secs| now_unix() + secs as i64),
            scope: stored.scope.clone(),
            extra: stored.extra.clone(), // preserva device_serial
        };

        save_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID, &new_token)?;

        Ok(new_token.access_token)
    }

    /// Desconecta a conta: tenta desregistrar o dispositivo no servidor da Amazon,
    /// mas remove o token local mesmo se a chamada falhar (token já expirado, rede fora do ar).
    pub async fn logout(&self) -> Result<(), AppError> {
        let Some(stored) = load_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID)? else {
            return delete_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID);
        };

        let response = HTTP_CLIENT
            .post(format!("{AMAZON_API}/auth/deregister"))
            .header("Authorization", format!("bearer {}", stored.access_token))
            .header("User-Agent", "AGSLauncher/1.0.0")
            .json(&serde_json::json!({
                "request_metadata": {
                    "app_name": AMAZON_APP_NAME,
                    "app_version": AMAZON_APP_VERSION,
                }
            }))
            .send()
            .await;

        if let Ok(resp) = &response {
            if !resp.status().is_success() {
                log::warn!(
                    "Amazon deregister retornou status não-OK; removendo token local mesmo assim"
                );
            }
        }

        delete_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID)
    }

    /// Importa a biblioteca completa de entitlements possuídos na conta Amazon.
    pub async fn fetch_library_detailed(&self) -> Result<Vec<SourceGame>, AppError> {
        let access_token = self.ensure_valid_token().await?;

        let stored = load_oauth_token(&self.app_handle, AMAZON_PROVIDER_ID)?
            .ok_or_else(|| AppError::OAuthTokenNotFound(AMAZON_PROVIDER_ID.to_string()))?;
        let serial = stored.extra.get("device_serial").ok_or_else(|| {
            AppError::OAuthConfigError(
                "Amazon: device_serial ausente no token salvo — faça login novamente".to_string(),
            )
        })?;

        let hardware_hash = {
            let mut hasher = Sha256::new();
            hasher.update(serial.as_bytes());
            hex_upper(&hasher.finalize())
        };

        let mut products: HashMap<String, String> = HashMap::new(); // id -> title
        let mut next_token: Option<String> = None;

        loop {
            let body = serde_json::json!({
                "Operation": "GetEntitlements",
                "clientId": "Sonic",
                "syncPoint": null,
                "nextToken": next_token,
                "maxResults": 50,
                "productIdFilter": null,
                "keyId": AMAZON_ENTITLEMENTS_KEY_ID,
                "hardwareHash": hardware_hash,
            });

            let response = HTTP_CLIENT
                .post(AMAZON_GAMING_DISTRIBUTION_ENTITLEMENTS)
                .header(
                    "X-Amz-Target",
                    "com.amazon.animusdistributionservice.entitlement.AnimusEntitlementsService.GetEntitlements",
                )
                .header("x-amzn-token", &access_token)
                // Header não-padrão "UserAgent" (sem hífen) — replicado fielmente do cliente de referência, distinto do header HTTP padrão "User-Agent".
                .header("UserAgent", "com.amazon.agslauncher.win/3.0.9202.1")
                .header("Content-Type", "application/json")
                .header("Content-Encoding", "amz-1.0")
                .json(&body)
                .send()
                .await?;

            let status = response.status();
            let resp_body = response.text().await.unwrap_or_default();

            if !status.is_success() {
                return Err(AppError::NetworkError(format!(
                    "Amazon entitlements retornou HTTP {status}: {resp_body}"
                )));
            }

            let parsed: AmazonEntitlementsResponse =
                serde_json::from_str(&resp_body).map_err(|e| {
                    AppError::ParseError(format!(
                        "Falha ao parsear biblioteca Amazon: {e} — corpo: {resp_body}"
                    ))
                })?;

            for entitlement in parsed.entitlements {
                let id = entitlement.product.id;
                let title = entitlement.product.title.unwrap_or_else(|| id.clone());
                products.entry(id).or_insert(title);
            }

            match parsed.next_token {
                Some(t) if !t.is_empty() => next_token = Some(t),
                _ => break,
            }
        }

        let games = products
            .into_iter()
            .map(|(id, title)| SourceGame {
                platform: "Amazon".to_string(),
                platform_game_id: id,
                name: Some(title),
                installed: false,
                executable_path: None,
                install_path: None,
                playtime_minutes: None,
                last_played: None,
            })
            .collect();

        Ok(games)
    }
}

#[async_trait]
impl GameSource for AmazonSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        self.fetch_library_detailed().await
    }
}

// === FUNÇÕES AUXILIARES ===

/// Aceita `expires_in` tanto como número quanto como string (a Amazon retorna como string em `register_device`).
fn deserialize_expires_in<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u64),
    }

    let opt = Option::<StringOrNumber>::deserialize(deserializer)?;
    Ok(match opt {
        Some(StringOrNumber::String(s)) => s.parse::<u64>().ok(),
        Some(StringOrNumber::Number(n)) => Some(n),
        None => None,
    })
}

async fn refresh_access_token_amazon(
    refresh_token: &str,
) -> Result<AmazonRefreshResponse, AppError> {
    let body = serde_json::json!({
        "source_token": refresh_token,
        "source_token_type": "refresh_token",
        "requested_token_type": "access_token",
        "app_name": AMAZON_APP_NAME,
        "app_version": AMAZON_APP_VERSION,
    });

    let response = HTTP_CLIENT
        .post(format!("{AMAZON_API}/auth/token"))
        .header("User-Agent", "AGSLauncher/1.0.0")
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let resp_body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(AppError::OAuthTokenExchangeError(format!(
            "Amazon refresh retornou HTTP {status}: {resp_body}"
        )));
    }

    serde_json::from_str(&resp_body).map_err(|e| {
        AppError::OAuthTokenExchangeError(format!("Resposta inesperada do refresh Amazon: {e}"))
    })
}

fn build_amazon_auth_url(client_id_hex: &str, challenge: &str) -> Result<Url, AppError> {
    let mut url = Url::parse("https://www.amazon.com/ap/signin")
        .map_err(|e| AppError::OAuthConfigError(format!("URL de login Amazon inválida: {e}")))?;

    url.query_pairs_mut()
        .append_pair("openid.ns", "http://specs.openid.net/auth/2.0")
        .append_pair(
            "openid.claimed_id",
            "http://specs.openid.net/auth/2.0/identifier_select",
        )
        .append_pair(
            "openid.identity",
            "http://specs.openid.net/auth/2.0/identifier_select",
        )
        .append_pair("openid.mode", "checkid_setup")
        .append_pair("openid.oa2.scope", "device_auth_access")
        .append_pair("openid.ns.oa2", "http://www.amazon.com/ap/ext/oauth/2")
        .append_pair("openid.oa2.response_type", "code")
        .append_pair("openid.oa2.code_challenge_method", "S256")
        .append_pair("openid.oa2.client_id", &format!("device:{client_id_hex}"))
        .append_pair("language", "en_US")
        .append_pair("marketPlaceId", AMAZON_MARKETPLACE_ID)
        .append_pair("openid.return_to", "https://www.amazon.com")
        .append_pair("openid.pape.max_auth_age", "0")
        .append_pair("openid.assoc_handle", AMAZON_ASSOC_HANDLE)
        .append_pair("pageId", AMAZON_ASSOC_HANDLE)
        .append_pair("openid.oa2.code_challenge", challenge);

    Ok(url)
}

fn generate_device_serial() -> String {
    uuid::Uuid::new_v4().simple().to_string().to_uppercase()
}

fn generate_client_id(serial: &str) -> String {
    let serial_ex = format!("{serial}#{AMAZON_DEVICE_TYPE}");
    serial_ex
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

// === JOGOS INSTALADOS ===

#[cfg(target_os = "windows")]
fn resolve_db_path() -> Option<PathBuf> {
    let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
    let path = PathBuf::from(local_appdata)
        .join("Amazon Games")
        .join("Data")
        .join("Games")
        .join("Sql")
        .join("GameInstallInfo.sqlite");
    path.exists().then_some(path)
}

#[cfg(not(target_os = "windows"))]
fn resolve_db_path() -> Option<PathBuf> {
    None // Amazon Games App é Windows-only
}

/// Copia o DB pra um arquivo temporário antes de abrir — evita conflito de lock com o Amazon Games App, caso esteja rodando ao mesmo tempo.
fn copy_to_temp(source: &PathBuf) -> Result<PathBuf, AppError> {
    let temp_path = std::env::temp_dir().join("playlite_amazon_gameinstallinfo.sqlite");
    fs::copy(source, &temp_path)?;
    Ok(temp_path)
}

/// Resolve o executável de um jogo Amazon Games a partir do `fuel.json` salvo na pasta de instalação.
///
/// **Observação:** formato não documentado oficialmente pela Amazon — inferido a partir do próprio
/// arquivo e de integrações da comunidade (Nile, Playnite). Se o arquivo não existir, vier malformado,
/// ou `Command` estiver vazio, retorna `None` sem erro — quem chama cai pro fallback (launcher).
fn resolve_amazon_executable(install_dir: &Path) -> Option<PathBuf> {
    let fuel_path = install_dir.join("fuel.json");
    let content = fs::read_to_string(&fuel_path).ok()?;
    let manifest: FuelManifest = serde_json::from_str(&content).ok()?;

    let command = manifest.main.command.trim();
    if command.is_empty() {
        return None;
    }

    Some(install_dir.join(command))
}

/// Importa jogos instalados detectados no banco local do Amazon Games App.
pub fn import_installed() -> Result<Vec<SourceGame>, AppError> {
    let Some(db_path) = resolve_db_path() else {
        return Ok(vec![]); // Amazon Games App não instalado, ou não é Windows
    };

    let temp_path = copy_to_temp(&db_path)?;
    let conn = rusqlite::Connection::open_with_flags(
        &temp_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;

    let mut stmt = conn.prepare(
        "SELECT Id, InstallDirectory, Installed, ProductTitle FROM DbSet WHERE Installed = 1",
    )?;

    let games = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let install_dir: Option<String> = row.get(1)?;
            let title: Option<String> = row.get(3)?;

            let executable_path = install_dir
                .as_ref()
                .and_then(|dir| resolve_amazon_executable(Path::new(dir)))
                .map(|p| p.to_string_lossy().to_string());

            Ok(SourceGame {
                platform: "Amazon".to_string(),
                platform_game_id: id,
                name: title,
                installed: true,
                executable_path, // resolvido via fuel.json quando disponível
                install_path: install_dir,
                playtime_minutes: None,
                last_played: None,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let _ = fs::remove_file(&temp_path);

    Ok(games)
}

/// Cruza a biblioteca completa (OAuth) com os jogos detectados localmente.
/// Diferente da Epic, o merge é por `platform_game_id` exato (mesmo formato de ID em ambas as fontes),
/// não por nome — mais confiável, sem falso-positivo/negativo por variação de grafia entre local e API.
pub fn merge_local_install_status(
    library_games: &mut Vec<SourceGame>,
    local_games: Vec<SourceGame>,
) {
    for local in local_games {
        let matched = library_games
            .iter_mut()
            .find(|g| g.platform_game_id == local.platform_game_id);

        match matched {
            Some(g) => {
                g.installed = true;
                g.install_path = local.install_path;
                g.executable_path = local.executable_path;
            }
            None => library_games.push(local), // instalado mas ausente da lib (raro; mantém mesmo assim)
        }
    }
}
