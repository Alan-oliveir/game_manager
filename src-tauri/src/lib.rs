//! # Playlite - Game Manager Library
//!
//! Fornece funcionalidades para:
//!
//! - Importar jogos do Steam
//! - Gerenciar biblioteca pessoal
//! - Wishlist com tracking de preços
//! - Busca de jogos em tendência (RAWG API)
//! - Busca de jogos grátis (GamerPower API)
//! - Recomendação de jogos

pub mod commands;
mod constants;
mod crypto;
pub mod database;
mod errors;
pub mod initialization;
pub mod models;
pub mod scrapers;
mod secrets;
mod security;
pub mod services;
pub mod sources;
pub mod utils;

use crate::initialization::initialize_app;
use crate::utils::logger;
use services::recommendation;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle();

            // === WAYLAND: Define app_id para ícone correto na barra de tarefas ===
            #[cfg(target_os = "linux")]
            {
                glib::set_prgname(Some("Playlite"));
                glib::set_application_name("Playlite");
            }

            // === LOGGING ===

            let log_dir = app_handle
                .path()
                .app_log_dir()
                .expect("Falha ao pegar pasta de log");

            std::fs::create_dir_all(&log_dir).expect("Falha ao criar pasta de dev_logs");

            let _guard = logger::init_logging(log_dir.clone());
            app.manage(_guard);

            // === SEGURANÇA ===

            security::init_security(app_handle).expect("Falha ao inicializar sistema de segurança");

            // === BANCOS DE DADOS ===

            let db_state = database::initialize_databases(app_handle)
                .expect("Falha ao inicializar bancos de dados");

            app.manage(db_state);

            // === INICIALIZAÇÃO PÓS-UPDATE ===

            // Verifica se houve atualização e faz backup/migração se necessário
            if let Err(e) = initialize_app(app_handle) {
                tracing::error!("Erro na inicialização pós-update: {}", e);
                // Não falha a aplicação, apenas loga o erro
            }

            // === COLLABORATIVE FILTERING ===

            if let Err(e) = recommendation::cf_aggregator::init_cf_index() {
                tracing::warn!(
                    "C pub(crate) pub(crate) pub(crate)F desativado (fallback CB ativo): {}",
                    e
                );
            }

            // Log de inicialização completa
            tracing::info!("Playlite iniciado com sucesso");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Comando de Inicialização do Banco de Dados
            database::init_db,
            // Comandos de Backup e Restauração
            database::backup::core::export_database,
            database::backup::core::import_database,
            // Comandos de Jogos (CRUD)
            commands::games::add_game,
            commands::games::get_games,
            commands::games::get_library_game_details,
            commands::games::toggle_favorite,
            commands::games::delete_game,
            commands::games::update_game,
            commands::games::update_game_details,
            // Comandos da Lista de Desejos
            commands::wishlist::search_wishlist_game,
            commands::wishlist::search_wishlist_game_by_features,
            commands::wishlist::add_to_wishlist,
            commands::wishlist::get_wishlist,
            commands::wishlist::remove_from_wishlist,
            commands::wishlist::check_wishlist_status,
            commands::wishlist::refresh_prices,
            commands::wishlist::import_wishlist,
            commands::wishlist::fetch_wishlist_covers,
            // Comandos de Importação de Plataformas
            commands::platforms::scanner::scan_games_folder,
            commands::platforms::scanner::add_game_from_scan,
            commands::platforms::scanner::add_games_from_scan,
            commands::platforms::steam::import_steam_library,
            commands::platforms::epic::epic_login,
            commands::platforms::epic::epic_logout,
            commands::platforms::epic::epic_is_authenticated,
            commands::platforms::epic::import_epic_games,
            commands::platforms::ubisoft::import_ubisoft_games,
            commands::platforms::legacy::import_legacy_games,
            commands::platforms::gog::gog_login,
            commands::platforms::gog::gog_logout,
            commands::platforms::gog::gog_is_authenticated,
            commands::platforms::gog::import_gog_games,
            commands::platforms::battle_net::import_battle_net_games,
            commands::platforms::ea::import_ea_games,
            commands::platforms::amazon::amazon_login,
            commands::platforms::amazon::amazon_logout,
            commands::platforms::amazon::amazon_is_authenticated,
            commands::platforms::amazon::import_amazon_games,
            commands::platforms::xbox::import_xbox_games,
            commands::platforms::indiegala::import_indiegala_games,
            commands::platforms::itch::import_itch_games,
            // Comando para iniciar um jogo
            commands::launcher::launch_game,
            // Comandos de Metadados (Enriquecimento, Capas, Refresh, Busca)
            commands::metadata::get_metadata::fill_missing_metadata,
            commands::metadata::covers::fetch_missing_covers,
            commands::metadata::refresh::check_and_refresh_background,
            commands::metadata::search::get_trending_games,
            commands::metadata::search::get_upcoming_games,
            commands::metadata::search::fetch_game_details,
            commands::metadata::search::get_active_giveaways,
            commands::metadata::search::get_similar_games,
            commands::metadata::search::get_game_media,
            commands::metadata::search::get_profile_similar_games,
            commands::metadata::search::search_hltb,
            commands::metadata::search::get_trending_mods,
            commands::metadata::pcgamingwiki::get_or_fetch_pcgw_data,
            commands::metadata::pcgamingwiki::refresh_pcgw_data,
            commands::metadata::pcgamingwiki::search_pcgw_games,
            commands::metadata::pcgamingwiki::get_pcgw_scraped_data,
            commands::metadata::protondb::fetch_protondb_data,
            // Comandos de Configuração (Secrets)
            commands::settings::set_secret,
            commands::settings::get_secret,
            commands::settings::delete_secret,
            commands::settings::list_secrets,
            commands::settings::get_secrets,
            commands::settings::set_secrets,
            // Comandos de Recomendação
            commands::recommendation::core::get_user_profile,
            commands::recommendation::core::recommend_hybrid_library,
            commands::recommendation::core::recommend_collaborative_library,
            commands::recommendation::core::recommend_from_library,
            commands::recommendation::analysis::generate_recommendation_analysis,
            // Comandos de Tradução de Descrição
            commands::ai_translation::translate_description,
            // Comandos de Conquistas de Jogos
            commands::achievements::get_recent_achievements,
            // Comandos de Cache de Metadados
            commands::caches::cleanup_cache,
            commands::caches::clear_all_cache,
            commands::caches::get_detailed_cache_stats,
            // Comandos de Sistema
            commands::system::open_folder,
            commands::system::open_file,
            // Comandos de Versão e SO
            commands::version::get_app_version,
            commands::version::get_app_version_info,
            commands::version::is_updater_enabled,
            // Comandos de Imagem
            services::images::cache_cover_image,
            services::images::check_local_cover,
            services::images::clear_cover_cache,
            // Comandos de Subscricoes
            commands::subscriptions::get_amazon_luna_catalog,
            commands::subscriptions::get_game_pass_catalog,
            commands::subscriptions::get_ea_play_catalog,
            commands::subscriptions::get_ubisoft_plus_catalog,
            commands::subscriptions::get_subscription_settings,
            commands::subscriptions::save_subscription_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
