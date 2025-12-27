mod models;
mod rawg_service;
mod steam_service;
mod constants;

use rusqlite::{params, Connection};
use std::sync::Mutex;
use tauri::{Manager, State};

use std::time::Duration;
use tokio::time::sleep;

// Estado da aplicação
pub struct AppState {
    db: Mutex<Connection>,
}

// Comando para inicializar o banco
#[tauri::command]
fn init_db(state: State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            genre TEXT,
            platform TEXT,
            cover_url TEXT,
            playtime INTEGER DEFAULT 0,
            rating INTEGER,
            favorite BOOLEAN DEFAULT FALSE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Criar índices para melhorar performance de queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_favorite ON games(favorite)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_name ON games(name COLLATE NOCASE)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_platform ON games(platform)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok("Banco inicializado com sucesso!".to_string())
}

// Comando para adicionar um jogo
#[tauri::command]
fn add_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    // Validações de entrada
    if name.trim().is_empty() {
        return Err("Nome do jogo não pode ser vazio".to_string());
    }

    if name.len() > constants::MAX_NAME_LENGTH {
        return Err(format!("Nome do jogo muito longo (máximo {} caracteres)", constants::MAX_NAME_LENGTH));
    }

    // Validação de URL
    if let Some(ref url) = cover_url {
        if url.len() > constants::MAX_URL_LENGTH {
            return Err(format!("URL da capa muito longa (máximo {} caracteres)", constants::MAX_URL_LENGTH));
        }
        // Verifica se a URL é válida (começa com http:// ou https://)
        if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL da capa deve começar com http:// ou https://".to_string());
        }
    }

    // Validação de genre e platform
    if let Some(ref g) = genre {
        if g.len() > constants::MAX_GENRE_LENGTH {
            return Err(format!("Gênero muito longo (máximo {} caracteres)", constants::MAX_GENRE_LENGTH));
        }
    }

    if let Some(ref p) = platform {
        if p.len() > constants::MAX_PLATFORM_LENGTH {
            return Err(format!("Plataforma muito longa (máximo {} caracteres)", constants::MAX_PLATFORM_LENGTH));
        }
    }

    if let Some(time) = playtime {
        if time < 0 {
            return Err("Tempo jogado não pode ser negativo".to_string());
        }
        if time > constants::MAX_PLAYTIME {
            return Err(format!("Tempo jogado inválido (máximo {} horas)", constants::MAX_PLAYTIME));
        }
    }

    if let Some(r) = rating {
        if r < constants::MIN_RATING || r > constants::MAX_RATING {
            return Err(format!("Avaliação deve estar entre {} e {}", constants::MIN_RATING, constants::MAX_RATING));
        }
    }

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    // Verifica se já existe um jogo com este ID
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Erro ao verificar duplicata: {}", e))?;

    if exists {
        return Err("Já existe um jogo com este ID".to_string());
    }

    conn.execute(
        "INSERT INTO games (id, name, genre, platform, cover_url, playtime, rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, genre, platform, cover_url, playtime, rating],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para listar todos os jogos
#[tauri::command]
fn get_games(state: State<AppState>) -> Result<Vec<models::Game>, String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, genre, platform, cover_url, playtime, rating, favorite FROM games",
        )
        .map_err(|e| e.to_string())?;

    let games = stmt
        .query_map([], |row| {
            Ok(models::Game {
                id: row.get(0)?,
                name: row.get(1)?,
                genre: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                playtime: row.get(5)?,
                rating: row.get(6)?,
                favorite: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(games)
}

// Comando para marcar/desmarcar favorito
#[tauri::command]
fn toggle_favorite(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET favorite = NOT favorite WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para deletar um jogo
#[tauri::command]
fn delete_game(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute("DELETE FROM games WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para atualizar informações de um jogo
#[tauri::command]
fn update_game(
    state: State<AppState>,
    id: String,
    name: String,
    genre: Option<String>,
    platform: Option<String>,
    cover_url: Option<String>,
    playtime: Option<i32>,
    rating: Option<i32>,
) -> Result<(), String> {
    // Validações de entrada
    if name.trim().is_empty() {
        return Err("Nome do jogo não pode ser vazio".to_string());
    }

    if name.len() > constants::MAX_NAME_LENGTH {
        return Err(format!("Nome do jogo muito longo (máximo {} caracteres)", constants::MAX_NAME_LENGTH));
    }

    if let Some(time) = playtime {
        if time < 0 {
            return Err("Tempo jogado não pode ser negativo".to_string());
        }
        if time > constants::MAX_PLAYTIME {
            return Err(format!("Tempo jogado inválido (máximo {} horas)", constants::MAX_PLAYTIME));
        }
    }

    if let Some(r) = rating {
        if r < constants::MIN_RATING || r > constants::MAX_RATING {
            return Err(format!("Avaliação deve estar entre {} e {}", constants::MIN_RATING, constants::MAX_RATING));
        }
    }

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    conn.execute(
        "UPDATE games SET name = ?1, genre = ?2, platform = ?3, cover_url = ?4, playtime = ?5, rating = ?6 WHERE id = ?7",
        params![name, genre, platform, cover_url, playtime, rating, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Comando para importar jogos da biblioteca Steam
#[tauri::command]
async fn import_steam_library(
    state: State<'_, AppState>,
    api_key: String,
    steam_id: String,
) -> Result<String, String> {
    // Busca os jogos na API da Steam
    let steam_games = steam_service::list_steam_games(&api_key, &steam_id).await?;

    let conn = state.db.lock().map_err(|_| "Falha ao bloquear mutex")?;

    let mut count = 0;

    // Salva cada jogo no banco (se não existir)
    for game in steam_games {
        // Monta a URL da capa (formato atualizado da Steam CDN)
        let cover_url = format!(
            "{}/steam/apps/{}/library_600x900.jpg",
            constants::STEAM_CDN_URL,
            game.appid
        );

        // Tenta inserir. O "OR IGNORE" pula se o ID já existir.
        let result = conn.execute(
            "INSERT OR IGNORE INTO games (id, name, genre, platform, cover_url, playtime, rating)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                game.appid.to_string(), // Usamos o ID da Steam como ID do jogo
                game.name,
                constants::DEFAULT_GENRE, // API básica não dá gênero do jogo
                constants::DEFAULT_PLATFORM_STEAM,
                cover_url,
                (game.playtime_forever as f32 / 60.0).round() as i32, // Converte minutos para horas corretamente
                None::<i32>                                           // Sem avaliação inicial
            ],
        );

        if let Ok(rows) = result {
            count += rows;
        }
    }

    Ok(format!(
        "Importação concluída! {} novos jogos adicionados.",
        count
    ))
}

#[tauri::command]
async fn enrich_library(state: State<'_, AppState>) -> Result<String, String> {
    // 1. Busca IDs de jogos que ainda estão como "Desconhecido" e são da Steam
    let games_to_update =
        {
            let conn = state.db.lock().map_err(|_| "Mutex error")?;
            let mut stmt = conn.prepare(
            "SELECT id, name FROM games WHERE genre = ?1 AND platform = ?2"
        ).map_err(|e| e.to_string())?;

            let mut rows = stmt.query([constants::DEFAULT_GENRE, constants::DEFAULT_PLATFORM_STEAM])
                .map_err(|e| e.to_string())?;
            let mut games = Vec::new();

            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let id: String = row.get(0).map_err(|e| e.to_string())?;
                let name: String = row.get(1).map_err(|e| e.to_string())?;
                games.push((id, name));
            }

            games
        }; // conn, stmt, and rows are all dropped here

    let total = games_to_update.len();
    if total == 0 {
        return Ok("Todos os jogos Steam já estão atualizados!".to_string());
    }

    println!("Iniciando enriquecimento de {} jogos...", total);
    let mut success_count = 0;

    // 2. Loop do Crawler
    for (id_str, name) in games_to_update {
        // Tenta converter ID para u32 (Steam ID)
        if let Ok(app_id) = id_str.parse::<u32>() {
            // Chama a API da Loja
            match steam_service::fetch_game_metadata(app_id).await {
                Ok(metadata) => {
                    // Salva no Banco
                    {
                        let conn = state.db.lock().map_err(|_| "Mutex error")?;
                        let _ = conn.execute(
                            "UPDATE games SET genre = ?1 WHERE id = ?2",
                            params![metadata.genre, id_str],
                        );
                        // conn is dropped here
                    }
                    // Futuro: Poderíamos salvar description e release_date se tivéssemos colunas pra isso

                    println!("Atualizado: {} -> {}", name, metadata.genre);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("[ERRO] Falha ao buscar metadata para {}: {}", name, e);
                }
            }

            // RATE LIMIT THROTTLE
            // Espera entre requisições para respeitar limites da API Steam
            // A Steam permite ~200 reqs/5min. 1.5s = 200 reqs em 5 min. Seguro.
            sleep(Duration::from_millis(constants::STEAM_RATE_LIMIT_MS)).await;
        }
    }

    Ok(format!(
        "Processo finalizado. {} de {} jogos atualizados.",
        success_count, total
    ))
}

#[tauri::command]
async fn get_trending_games(api_key: String) -> Result<Vec<rawg_service::RawgGame>, String> {
    rawg_service::fetch_trending_games(&api_key).await
}

// Função principal que configura o Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Obtém o diretório de dados da aplicação
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Falha ao obter diretório de dados da aplicação");

            // Cria o diretório se não existir
            std::fs::create_dir_all(&app_data_dir).expect("Falha ao criar diretório de dados");

            // Caminho completo do banco de dados
            let db_path = app_data_dir.join("library.db");

            // Inicializa conexão com arquivo no diretório de dados
            let conn =
                Connection::open(&db_path).expect(&format!("Erro ao abrir banco em {:?}", db_path));

            // Configura o journal mode para WAL (Write-Ahead Logging) para melhor performance
            let _ = conn.execute("PRAGMA journal_mode=WAL", []);

            // Registra o estado da aplicação
            app.manage(AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_db,
            add_game,
            get_games,
            toggle_favorite,
            delete_game,
            update_game,
            import_steam_library,
            enrich_library,
            get_trending_games
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
