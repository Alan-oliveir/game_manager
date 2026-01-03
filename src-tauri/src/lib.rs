mod commands;
mod constants;
mod database;
mod models;
mod security;
mod services;
mod storage;
mod utils;

use crate::utils::logger;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_shell;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_machine_uid::init())
        .setup(|app| {
            let app_handle = app.handle();
            let log_dir = app_handle
                .path()
                .app_log_dir()
                .expect("Falha ao pegar pasta de log");

            std::fs::create_dir_all(&log_dir).expect("Falha ao criar pasta de logs");

            let _guard = logger::init_logging(log_dir.clone());
            // Armazena o guard no state do Tauri para manter vivo durante toda execução
            app.manage(_guard);

            tracing::info!("Aplicação iniciada! Logs em: {:?}", log_dir);

            // Inicializa o sistema de segurança (derivação de chave)
            security::init_security(&app_handle)
                .expect("Falha ao inicializar sistema de segurança");

            tracing::info!("Sistema de segurança inicializado");

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Falha ao obter diretório de dados da aplicação");

            std::fs::create_dir_all(&app_data_dir).expect("Falha ao criar diretório de dados");

            let db_path = app_data_dir.join("library.db");

            let conn =
                Connection::open(&db_path).expect(&format!("Erro ao abrir banco em {:?}", db_path));

            let _ = conn.execute("PRAGMA journal_mode=WAL", []);

            app.manage(database::AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
        // Registra todos os comandos chamando a partir dos módulos
        .invoke_handler(tauri::generate_handler![
            // Comando de Inicialização do Banco de Dados
            database::init_db,
            // Comandos de Jogos (CRUD)
            commands::games::add_game,
            commands::games::get_games,
            commands::games::toggle_favorite,
            commands::games::delete_game,
            commands::games::update_game,
            // Comandos da Lista de Desejos
            commands::wishlist::search_wishlist_game,
            commands::wishlist::add_to_wishlist,
            commands::wishlist::get_wishlist,
            commands::wishlist::remove_from_wishlist,
            commands::wishlist::check_wishlist_status,
            commands::wishlist::refresh_prices,
            // Comandos de Integração (Steam/RAWG)
            commands::integrations::import_steam_library,
            commands::integrations::enrich_library,
            commands::integrations::get_trending_games,
            commands::integrations::get_upcoming_games,
            commands::integrations::fetch_game_details,
            // Comandos de Configuração (Secrets)
            commands::settings::set_secret,
            commands::settings::get_secret,
            commands::settings::delete_secret,
            commands::settings::list_secrets,
            commands::settings::get_secrets,
            commands::settings::set_secrets,
            // Comandos de Backup e Restauração
            commands::backup::export_database,
            commands::backup::import_database,
            // Comando de Recomendação
            commands::recommendations::get_user_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
