use crate::models::{Game, Platform};
use std::env;
use std::path::PathBuf;
use tracing::warn;

#[derive(Debug, Clone)]
pub enum LaunchResolution {
    Protocol(String),   // steam://, battlenet:// — protocolos confiáveis e documentados
    Executable(String), // Executa o jogo diretamente (Epic, GOG, etc)
    Launcher(String),   //  Fallback: abre o launcher da plataforma
    Store(String),      // Launcher não encontrado no disco — abre a página da loja
    Unavailable,
}

pub struct PlatformFallback {
    pub launcher_candidates: &'static [&'static str],
    pub store_url: &'static str,
}

pub fn platform_fallback(platform: &Platform) -> PlatformFallback {
    match platform {
        Platform::Amazon => PlatformFallback {
            launcher_candidates: &[r"%LOCALAPPDATA%\Amazon Games\App\Amazon Games.exe"],
            store_url: "",
        },
        Platform::BattleNet => PlatformFallback {
            launcher_candidates: &[
                r"C:\Program Files (x86)\Battle.net\Battle.net.exe",
                r"C:\Program Files (x86)\Battle.net\Battle.net Launcher.exe",
            ],
            store_url: "https://battle.net/",
        },
        Platform::EA => PlatformFallback {
            launcher_candidates: &[
                r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe",
                r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EALauncher.exe",
            ],
            store_url: "https://www.ea.com/",
        },
        Platform::Epic => PlatformFallback {
            launcher_candidates: &[
                r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win32\EpicGamesLauncher.exe",
                r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
            ],
            store_url: "https://store.epicgames.com/",
        },
        Platform::GOG => PlatformFallback {
            launcher_candidates: &[r"C:\Program Files (x86)\GOG Galaxy\GalaxyClient.exe"],
            store_url: "https://www.gog.com/",
        },
        Platform::Indiegala => PlatformFallback {
            launcher_candidates: &[r"C:\Program Files (x86)\IGClient\IGClient.exe"],
            store_url: "https://www.indiegala.com/store",
        },
        Platform::Itch => PlatformFallback {
            launcher_candidates: &[
                r"C:\Users\%USERNAME%\AppData\Local\itch\itch.exe",
                r"C:\Program Files (x86)\itch\itch.exe",
            ],
            store_url: "https://itch.io/",
        },
        Platform::LegacyGames => PlatformFallback {
            launcher_candidates: &[
                r"C:\Program Files (x86)\Legacy Games\Legacy Games Launcher\Legacy Games Launcher.exe",
            ],
            store_url: "https://legacygames.com/",
        },
        Platform::Steam => PlatformFallback {
            launcher_candidates: &[r"C:\Program Files (x86)\Steam\Steam.exe"],
            store_url: "https://store.steampowered.com/",
        },
        Platform::Ubisoft => PlatformFallback {
            launcher_candidates: &[
                r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe",
                r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\UbisoftGameLauncher.exe",
            ],
            store_url: "https://store.ubisoft.com/",
        },
        // Demais plataformas (Indie, Outra) sem launcher próprio, ou scan de pastas com jogos.
        _ => PlatformFallback {
            launcher_candidates: &[],
            store_url: "",
        },
    }
}

/// `override_path`: valor cru vindo do localStorage via frontend (mesmo padrão de
/// `gog_games_dir`/`ea_install_dir`). Ignorado se o caminho não existir mais no disco.
pub fn resolve_launcher_path(platform: &Platform, override_path: Option<&str>) -> Option<PathBuf> {
    // 1. Path customizado pelo usuário, se configurado e ainda existir
    if let Some(custom) = override_path.filter(|s| !s.trim().is_empty()) {
        let path = PathBuf::from(custom);
        if path.exists() {
            return Some(path);
        }
        warn!("Launcher path configurado para {platform:?} não existe mais: {path:?}");
    }

    // 2. Fallback: tenta os caminhos padrão conhecidos
    platform_fallback(platform)
        .launcher_candidates
        .iter()
        .filter_map(|template| expand_path_template(template))
        .find(|p| p.exists())
}

/// Plataformas cujo executável, mesmo quando localizado, não é confiável o suficiente pra rodar direto,
/// dependem do runtime/DRM/overlay do próprio launcher pra funcionar (anti-cheat, licenciamento, etc).
/// Sempre abre o launcher da plataforma em vez de tentar o `.exe` diretamente.
fn is_launcher_only(platform: &Platform) -> bool {
    matches!(platform, Platform::EA | Platform::Ubisoft)
}

/// Plataformas cujo protocolo funciona mesmo com o jogo não instalado — o próprio client resolve a
/// instalação sozinho quando necessário. Documentado oficialmente pela Valve: rungameid "instala se
/// necessário" antes de rodar. Só Steam tem essa garantia confirmada;
fn protocol_handles_install(platform: &Platform) -> bool {
    matches!(platform, Platform::Steam)
}

/// Protocolos confiáveis por plataforma. Epic/GOG/EA/Amazon não têm protocolo documentado — usam executable_path.
fn protocol_url_for(game: &Game) -> Option<String> {
    match game.platform {
        Platform::Steam => Some(format!("steam://rungameid/{}", game.platform_game_id)),
        Platform::BattleNet => Some(format!("battlenet://{}", game.platform_game_id)),
        _ => None,
    }
}

/// Decide como iniciar um jogo, na seguinte ordem de prioridade:
/// 1. Instalado + protocolo confiável (Steam, Battle.net)
/// 2. Instalado + executável resolvido (Amazon, Epic, GOG, Itch.io, Xbox)
/// 3. Launcher da plataforma encontrado no disco (EA/Ubisoft; demais quando não instalado, ou sem executável resolvido)
/// 4. Site da loja (launcher não encontrado)
pub fn resolve_launch(game: &Game, launcher_path_override: Option<&str>) -> LaunchResolution {
    let protocol_bypasses_install_check =
        protocol_handles_install(&game.platform) && protocol_url_for(game).is_some();

    if (game.installed || protocol_bypasses_install_check) && !is_launcher_only(&game.platform) {
        if let Some(url) = protocol_url_for(game) {
            return LaunchResolution::Protocol(url);
        }
        if let Some(exec) = &game.executable_path {
            return LaunchResolution::Executable(exec.clone());
        }
    }

    if let Some(launcher_path) = resolve_launcher_path(&game.platform, launcher_path_override) {
        return LaunchResolution::Launcher(launcher_path.to_string_lossy().to_string());
    }

    let store_url = platform_fallback(&game.platform).store_url;
    if store_url.is_empty() {
        return LaunchResolution::Unavailable;
    }

    LaunchResolution::Store(store_url.to_string())
}

/// Expande `%LOCALAPPDATA%` e `%USERNAME%` em templates de caminho
fn expand_path_template(template: &str) -> Option<PathBuf> {
    let mut resolved = template.to_string();

    if resolved.contains("%LOCALAPPDATA%") {
        let local_app_data = env::var("LOCALAPPDATA").ok()?;
        resolved = resolved.replace("%LOCALAPPDATA%", &local_app_data);
    }

    if resolved.contains("%USERNAME%") {
        let username = env::var("USERNAME").ok()?;
        resolved = resolved.replace("%USERNAME%", &username);
    }

    Some(PathBuf::from(resolved))
}
