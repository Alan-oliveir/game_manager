//! Serviço de disponibilidade em cloud gaming (GeForce NOW + Xbox Cloud Gaming).
//!
//! Os dados ficam em tabelas dedicadas (`gfn_games`, `xbox_cloud_ids`) mantidas pelos próprios
//! providers — o padrão certo pra lookup pontual por jogo (tela de detalhes), não pra exibição de
//! catálogo completo.

use crate::database::AppState;
use crate::providers::cloud_gaming::geforce_now::{self, GfnAvailability};
use crate::providers::cloud_gaming::xbox_cloud_gaming;
use crate::providers::subscriptions::GamePassGame;
use crate::services::locale;
use crate::services::subscriptions;
use crate::utils::text::{normalize_for_matching, strip_edition_suffix};
use serde::Serialize;
use strsim::jaro_winkler;
use tauri::{AppHandle, Manager, State};
use tracing::warn;

/// Idioma no formato BCP-47 esperado pela API do catálogo Xbox (não é o mesmo formato usado
/// internamente pelo app em `services::locale`, que guarda só "pt-BR"/"en"). Mesma normalização
/// de `providers::subscriptions::normalize_lang_tag`, duplicada aqui de propósito — é uma regra
/// pequena e específica de cada consumidor da API do catálogo Xbox, não compensa acoplar os módulos.
fn xbox_language_tag(app_language: &str) -> &'static str {
    if app_language.to_lowercase().starts_with("pt") {
        "pt-BR"
    } else {
        "en-US"
    }
}

// === INICIALIZAÇÃO ===

/// Cria as tabelas dos dois providers.
pub fn initialize_cloud_gaming_tables(conn: &rusqlite::Connection) -> Result<(), String> {
    geforce_now::initialize_gfn_tables(conn)?;
    xbox_cloud_gaming::initialize_xbox_cloud_tables(conn)?;
    Ok(())
}

// === REFRESH ===

/// Atualiza os dois catálogos se estiverem com cache expirado (best-effort: uma fonte falhando
/// não impede a outra, e nenhuma falha aqui derruba o app).
pub async fn refresh_cloud_gaming_catalogs_if_stale(app: &AppHandle) -> Result<(), String> {
    let state: State<AppState> = app.state();
    let client = reqwest::Client::new();

    if let Err(e) = geforce_now::ensure_fresh(&state.games_db, &client).await {
        warn!("Refresh GeForce NOW falhou: {e}");
    }

    let region = locale::get_or_detect_region(app).unwrap_or_else(|_| "US".to_string());
    let language = locale::get_or_detect_language(app).unwrap_or_else(|_| "en".to_string());
    let xbox_language = xbox_language_tag(&language);

    if let Err(e) =
        xbox_cloud_gaming::refresh_xbox_cloud_ids_if_stale(&state.games_db, &region, xbox_language)
            .await
    {
        warn!("Refresh Xbox Cloud Gaming falhou: {e}");
    }

    Ok(())
}

/// Dispara o refresh em background — usado no startup do app, sem bloquear a inicialização.
pub fn spawn_cloud_gaming_bootstrap(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = refresh_cloud_gaming_catalogs_if_stale(&app).await {
            warn!("Bootstrap Cloud Gaming: {}", e);
        }
    });
}

// === RESOLUÇÃO DO XBOX STORE ID ===

fn normalize_title(name: &str) -> String {
    normalize_for_matching(&strip_edition_suffix(name))
}

fn find_best_game_pass_match<'a>(
    game_name: &str,
    catalog: &'a [GamePassGame],
) -> Option<&'a GamePassGame> {
    let normalized_target = normalize_title(game_name);

    if let Some(exact) = catalog
        .iter()
        .find(|g| normalize_title(&g.title) == normalized_target)
    {
        return Some(exact);
    }

    // Threshold alto o bastante pra evitar falso positivo entre jogos parecidos mas diferentes
    // (ex. "Orcs Must Die" vs "Orcs Must Die 2").
    const SIMILARITY_THRESHOLD: f64 = 0.92;

    catalog
        .iter()
        .map(|g| {
            (
                g,
                jaro_winkler(&normalized_target, &normalize_title(&g.title)),
            )
        })
        .filter(|(_, score)| *score >= SIMILARITY_THRESHOLD)
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(g, _)| g)
}

/// Resolve o store_id da Xbox pra um jogo da biblioteca do Playlite, sem persistência. Se o jogo
/// foi importado da Xbox/Microsoft Store, o store_id já é conhecido e retornado. Se não, faz
/// matching de título contra o catálogo PC Game Pass já cacheado, retornando o store_id do jogo
/// mais parecido, se houver algum acima do limiar de similaridade definido.
pub async fn resolve_xbox_store_id(
    state: &State<'_, AppState>,
    game_name: &str,
    library: &str,
    library_game_id: &str,
) -> Result<Option<String>, String> {
    if matches!(library, "Xbox") {
        return Ok(Some(library_game_id.to_string()));
    }

    let catalog = subscriptions::get_game_pass_games(state, false, "en-US").await?;
    Ok(find_best_game_pass_match(game_name, &catalog).map(|g| g.store_id.clone()))
}

// === LOOKUP ===

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAvailability {
    pub geforce_now: Option<GfnAvailability>,
    pub xbox_cloud: bool,
}

impl CloudAvailability {
    pub fn is_available_anywhere(&self) -> bool {
        self.geforce_now.is_some() || self.xbox_cloud
    }
}

/// Busca disponibilidade em cloud gaming pra um jogo, dados os identificadores que o Playlite já
/// tem armazenados para ele. Os dois parâmetros são independentes e opcionais — um jogo pode ter
/// só steam_app_id, só xbox_store_id, os dois, ou nenhum (nesse caso retorna tudo `None`/`false`
/// sem erro, já que nem todo jogo tem um identificador pesquisável em cada fonte).
pub fn get_cloud_availability(
    state: &State<'_, AppState>,
    steam_app_id: Option<&str>,
    xbox_store_id: Option<&str>,
) -> Result<CloudAvailability, String> {
    let conn = state.games_db.lock().map_err(|e| e.to_string())?;

    let geforce_now = match steam_app_id {
        Some(id) => geforce_now::find_gfn_availability(&conn, id).map_err(|e| e.to_string())?,
        None => None,
    };

    let xbox_cloud = match xbox_store_id {
        Some(id) => xbox_cloud_gaming::is_available_on_xbox_cloud(&conn, id)?,
        None => false,
    };

    Ok(CloudAvailability {
        geforce_now,
        xbox_cloud,
    })
}
