mod commands;
mod constants;
mod database;
mod models;
mod services;
mod storage;

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
        .setup(|app| {
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
            database::init_db,
            // Comandos de Jogos (CRUD)
            commands::games::add_game,
            commands::games::get_games,
            commands::games::toggle_favorite,
            commands::games::delete_game,
            commands::games::update_game,
            // Comandos da Lista de Desejos
            commands::wishlist::add_to_wishlist,
            commands::wishlist::remove_from_wishlist,
            commands::wishlist::get_wishlist,
            commands::wishlist::check_wishlist_status,
            commands::wishlist::refresh_prices,
            // Comandos de Integração (Steam/RAWG)
            commands::integrations::import_steam_library,
            commands::integrations::enrich_library,
            commands::integrations::get_trending_games,
            // Comandos de Configuração (Secrets)
            commands::settings::set_secret,
            commands::settings::get_secret,
            commands::settings::delete_secret,
            commands::settings::list_secrets,
            commands::settings::get_secrets,
            commands::settings::set_secrets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
