//! Orquestração de importação de bibliotecas externas (Steam, Epic, GOG, etc.).
//!
//! Fluxo: chama o fetch fornecido pelo provider da plataforma, persiste via
//! `database::libraries`, emite eventos para o frontend e dispara enrichment
//! em background para jogos recém-importados.

use crate::database::libraries::{persist_source_games, NewlyImportedGame};
use crate::database::AppState;
use crate::errors::AppError;
use crate::providers::libraries::providers::SourceGame;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

/// Resultado de uma importação, usado por `spawn_import_custom` para fontes com persistência
/// própria (IndieGala, Itch.io, Legacy Games), que gravam campos extras (`description_raw`,
/// `tags`, `cover_url`) fora do `SourceGame` padrão e por isso não passam por `persist_source_games`.
pub enum ImportOutcome {
    Empty,
    Persisted {
        inserted: u32,
        updated: u32,
        newly_imported: Vec<NewlyImportedGame>,
    },
}

/// Dispara o enriquecimento de metadados em background para jogos recém-importados, se houver algum.
pub fn trigger_enrichment_if_needed(app: &AppHandle, newly_imported: Vec<NewlyImportedGame>) {
    if !newly_imported.is_empty() {
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            crate::commands::metadata::enrichment::enrich_newly_imported(app_clone, newly_imported)
                .await;
        });
    }
}

/// Executa uma importação de plataforma em background, sem bloquear o comando Tauri.
///
/// `fetch` recebe uma cópia do `AppHandle` (necessário para fontes com OAuth, como Amazon/Epic/GOG)
/// e deve conter toda a parte lenta (rede, fetch, merge de fontes), retornando a lista final de
/// `SourceGame` pronta para persistência. O comando retorna imediatamente.
///
/// O resultado chega ao **frontend** via eventos:
///
/// - `import_started`   → payload: library (string)
/// - `import_complete`  → payload: (library, message)
/// - `import_error`     → payload: (library, error)
/// - `library_updated`  → (mantido, sem payload, para compatibilidade com listeners existentes)
pub fn spawn_import<F, Fut>(app: AppHandle, library: &'static str, fetch: F)
where
    F: FnOnce(AppHandle) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<SourceGame>, AppError>> + Send + 'static,
{
    let _ = app.emit("import_started", library);
    let app_task = app.clone();
    let app_for_fetch = app.clone();

    tauri::async_runtime::spawn(async move {
        let games = match fetch(app_for_fetch).await {
            Ok(g) => g,
            Err(e) => {
                warn!("Import {} falhou: {}", library, e);
                let _ = app_task.emit("import_error", (library, e.to_string()));
                return;
            }
        };

        if games.is_empty() {
            let msg = format_import_empty(library);
            let _ = app_task.emit("import_complete", (library, msg));
            return;
        }

        let state: tauri::State<AppState> = app_task.state();
        let (inserted, updated, newly_imported) = {
            let mut conn = match state.games_db.lock() {
                Ok(c) => c,
                Err(_) => {
                    let _ = app_task.emit(
                        "import_error",
                        (library, "Falha ao bloquear mutex do games_db".to_string()),
                    );
                    return;
                }
            };
            match persist_source_games(&mut conn, games) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Persist {} falhou: {}", library, e);
                    let _ = app_task.emit("import_error", (library, e));
                    return;
                }
            }
        };

        let message = format_import_summary(library, inserted, updated);
        info!("{}", message);

        let _ = app_task.emit("library_updated", ());
        let _ = app_task.emit("import_complete", (library, message));

        trigger_enrichment_if_needed(&app_task, newly_imported);
    });
}

/// Como `spawn_import`, mas para fontes cuja persistência já está embutida em `run`
/// (fetch + persist_*_games próprio), retornando o resultado final e não uma lista crua de `SourceGame`.
pub fn spawn_import_custom<F, Fut>(app: AppHandle, library: &'static str, run: F)
where
    F: FnOnce(AppHandle) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<ImportOutcome, AppError>> + Send + 'static,
{
    let _ = app.emit("import_started", library);
    let app_task = app.clone();
    let app_for_run = app.clone();

    tauri::async_runtime::spawn(async move {
        match run(app_for_run).await {
            Ok(ImportOutcome::Empty) => {
                let msg = format_import_empty(library);
                let _ = app_task.emit("import_complete", (library, msg));
            }
            Ok(ImportOutcome::Persisted {
                inserted,
                updated,
                newly_imported,
            }) => {
                let message = format_import_summary(library, inserted, updated);
                info!("{}", message);
                let _ = app_task.emit("library_updated", ());
                let _ = app_task.emit("import_complete", (library, message));
                trigger_enrichment_if_needed(&app_task, newly_imported);
            }
            Err(e) => {
                warn!("Import {} falhou: {}", library, e);
                let _ = app_task.emit("import_error", (library, e.to_string()));
            }
        }
    });
}

// === Mensagens padronizadas ===

/// Mensagem padrão de sucesso: "<Plataforma>: X adicionados, Y atualizados".
pub fn format_import_summary(library: &str, inserted: u32, updated: u32) -> String {
    format!("{library}: {inserted} adicionados, {updated} atualizados")
}

/// Mensagem padrão de biblioteca vazia: "Nenhum jogo <plataforma> encontrado."
pub fn format_import_empty(library: &str) -> String {
    format!("Nenhum jogo {library} encontrado.")
}

/// Mensagem padrão de conexão bem sucedida: "Conta <plataforma> conectada com sucesso!"
pub fn format_login_success(library: &str) -> String {
    format!("Conta {library} conectada com sucesso!")
}
