//! Rastreamento local de tempo jogado, usado como fallback para plataformas
//! que não expõem playtime via API própria (todas exceto Steam/Itch/Indiegala).
//!
//! Funciona observando, por polling, se o executável resolvido do jogo (já
//! persistido em `games.executable_path` na importação) está em execução.
//! Não depende do processo ser filho direto do Playlite — cobre também os
//! casos em que o Playlite só abriu o launcher da plataforma (protocolo ou
//! executável do launcher) e o jogo de fato sobe depois, por fora.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use tauri::{AppHandle, Manager};

use crate::database::AppState;
use crate::models::Platform;
use crate::utils::status_logic;

/// Mesma granularidade das plataformas com API oficial (Steam/Itch/Indiegala
/// reportam playtime em minutos) — evita precisão inconsistente entre fontes.
const POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Tempo máximo esperando o processo do jogo aparecer depois que o launcher
/// foi aberto (ex: usuário demora navegando antes de clicar "Play", ou desiste).
/// Evita task órfã rodando indefinidamente.
const MAX_WAIT_FOR_START: Duration = Duration::from_secs(10 * 60);

/// Evita observar o mesmo jogo duas vezes (ex: usuário clica "Jogar" repetidamente).
#[derive(Default)]
pub struct PlaytimeRegistry {
    watching: Mutex<HashSet<String>>,
}

impl PlaytimeRegistry {
    /// Retorna `true` se conseguiu reservar o watch (ninguém mais observando este jogo).
    fn try_start(&self, game_id: &str) -> bool {
        self.watching.lock().unwrap().insert(game_id.to_string())
    }

    fn finish(&self, game_id: &str) {
        self.watching.lock().unwrap().remove(game_id);
    }
}

/// Plataformas que já expõem playtime oficial — o tracker local nunca roda pra elas,
/// pra não gerar dado duplicado/conflitante com o valor sincronizado na importação.
pub fn has_official_playtime_source(platform: &Platform) -> bool {
    matches!(
        platform,
        Platform::Steam | Platform::Itch | Platform::Indiegala
    )
}

/// Dispara o watcher em background para o jogo informado. Idempotente: se já
/// houver um watch em andamento para o mesmo `game_id`, não faz nada.
pub fn watch_game(app: AppHandle, game_id: String, exe_path: PathBuf) {
    let state = app.state::<AppState>();
    if !state.playtime_registry.try_start(&game_id) {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );

        let mut found = false;
        let mut waited = Duration::ZERO;

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            sys.refresh_processes(ProcessesToUpdate::All, true);

            let running = sys
                .processes()
                .values()
                .any(|p| p.exe().map(|p| p == exe_path.as_path()).unwrap_or(false));

            if running {
                found = true;
                if let Err(e) = increment_playtime(&app, &game_id) {
                    tracing::warn!("[playtime] falha ao gravar incremento para {game_id}: {e}");
                }
            } else if found {
                break; // processo existiu e sumiu -> sessão encerrada
            } else {
                waited += POLL_INTERVAL;
                if waited >= MAX_WAIT_FOR_START {
                    break; // desistiu de esperar o jogo abrir
                }
            }
        }

        app.state::<AppState>().playtime_registry.finish(&game_id);
    });
}

/// Incrementa 1 minuto de playtime e recalcula `status`, no mesmo padrão usado
/// pelos importadores (ex: `persist_indiegala_games`), pra manter a UI de
/// status ("jogando"/"backlog") coerente sem esperar uma reimportação.
fn increment_playtime(app: &AppHandle, game_id: &str) -> Result<(), rusqlite::Error> {
    let state = app.state::<AppState>();
    let conn = state.games_db.lock().unwrap();

    let current: Option<i32> = conn.query_row(
        "SELECT playtime FROM games WHERE id = ?1",
        [game_id],
        |row| row.get(0),
    )?;
    let new_playtime = current.unwrap_or(0) + 1;
    let status = status_logic::calculate_status(new_playtime);

    conn.execute(
        "UPDATE games SET playtime = ?1, status = ?2, playtime_source = 'local' WHERE id = ?3",
        rusqlite::params![new_playtime, status, game_id],
    )?;
    Ok(())
}
