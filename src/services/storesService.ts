import { invoke } from '@tauri-apps/api/core';

import { LaunchOutcome } from '@/types';

export const storesService = {
  /**
   * Importa a biblioteca completa de jogos Steam do usuário.
   * Obtém jogos instalados via VDF local, não instalados via cache e API como fallback.
   *
   * @throws Se as credenciais forem inválidas ou a API estiver indisponível
   */
  importSteamLibrary: async (
    steamId: string,
    apiKey: string,
    steamRoot: string
  ): Promise<void> => {
    return await invoke<void>('import_steam_library', {
      steamId,
      apiKey,
      steamRoot,
    });
  },

  epicLogin: async (): Promise<string> => {
    return await invoke<string>('epic_login');
  },

  epicLogout: async (): Promise<void> => {
    return await invoke<void>('epic_logout');
  },

  epicIsAuthenticated: async (): Promise<boolean> => {
    return await invoke<boolean>('epic_is_authenticated');
  },

  /**
   * Importa jogos instalados da Epic Games Store.
   * Detecta automaticamente via manifestos do Epic Games Launcher.
   *
   * Windows: C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests
   * Linux (Wine): <wine_prefix>/drive_c/ProgramData/Epic/EpicGamesLauncher/Data/Manifests
   *
   * @throws Se o Epic Games Launcher não estiver instalado ou não houver jogos
   */
  importEpicGames: async (): Promise<void> => {
    const winePrefix = localStorage.getItem('wine_prefix') || undefined;

    return await invoke<void>('import_epic_games', {
      winePrefix: winePrefix ?? null,
    });
  },

  /**
   * Importa jogos da Ubisoft lendo o cache de configuração do Ubisoft Connect.
   * Detecta automaticamente via %LOCALAPPDATA%\Ubisoft Game Launcher (Windows)
   * ou via Wine prefix configurado (Linux).
   *
   * @throws Se o Ubisoft Connect não estiver instalado ou não houver jogos
   */
  importUbisoftGames: async (): Promise<void> => {
    const winePrefix = localStorage.getItem('wine_prefix') || undefined;

    return await invoke<void>('import_ubisoft_games', {
      winePrefix: winePrefix ?? null,
    });
  },

  /**
   * Importa a biblioteca de jogos da Legacy Games.
   * Lê o arquivo app-state-bck.json do launcher.
   *
   * Windows: %APPDATA%\legacy-games-launcher\app-state-bck.json
   * Linux (Wine): <wine_prefix>/drive_c/users/<USER>/AppData/Roaming/legacy-games-launcher/app-state-bck.json
   *
   * @throws Se o Legacy Games Launcher não estiver instalado ou não houver jogos
   */
  importLegacyGames: async (appStatePath?: string): Promise<void> => {
    const winePrefix = localStorage.getItem('wine_prefix') || undefined;

    return await invoke<void>('import_legacy_games', {
      appStatePath: appStatePath ?? null,
      winePrefix: winePrefix ?? null,
    });
  },

  /**
   * Inicia o fluxo de login OAuth2 da conta GOG.
   * Abre uma janela de login; a Promise resolve quando o token é obtido e salvo.
   *
   * @throws Se o login falhar, for cancelado, ou a troca de token falhar
   */
  gogLogin: async (): Promise<string> => {
    return await invoke<string>('gog_login');
  },

  /**
   * Remove o token OAuth salvo da conta GOG (logout).
   */
  gogLogout: async (): Promise<void> => {
    return await invoke<void>('gog_logout');
  },

  /**
   * Verifica se existe uma conta GOG conectada (token salvo).
   * Não garante que o token ainda é válido — apenas que existe um login prévio.
   */
  gogIsAuthenticated: async (): Promise<boolean> => {
    return await invoke<boolean>('gog_is_authenticated');
  },

  /**
   * Importa a biblioteca completa de jogos possuídos na conta GOG.
   * Requer login prévio via `gogLogin`.
   *
   * @throws Se não houver conta conectada ou a API estiver indisponível
   */
  importGogGames: async (): Promise<void> => {
    const gogGamesDir = localStorage.getItem('gog_games_dir') || undefined;

    return await invoke<void>('import_gog_games', {
      gogGamesDir: gogGamesDir ?? null,
    });
  },

  /**
   * Importa jogos instalados via Battle.net (Blizzard/Activision).
   * Detecta automaticamente lendo `product.db` do Battle.net Agent.
   *
   * Windows apenas: C:\ProgramData\Battle.net\Agent\product.db
   * (Battle.net não roda de forma confiável via Wine no Linux.)
   *
   * @throws Se o Battle.net não estiver instalado ou não houver jogos
   */
  importBattleNetGames: async (): Promise<void> => {
    return await invoke<void>('import_battle_net_games');
  },

  /**
   * Importa jogos instalados via EA App (Electronic Arts).
   * A EA não expõe um arquivo estruturado e confiável com o status de instalação,
   * nem oferece um jeito viável de autenticar para listar a biblioteca completa —
   * a detecção depende inteiramente da pasta de instalação configurada pelo usuário.
   *
   * @throws Se a pasta não estiver configurada ou não houver jogos
   */
  importEaGames: async (): Promise<void> => {
    const eaInstallDir = localStorage.getItem('ea_install_dir') || undefined;

    return await invoke<void>('import_ea_games', {
      eaInstallDir: eaInstallDir ?? null,
    });
  },

  amazonLogin: async (): Promise<string> => {
    return await invoke<string>('amazon_login');
  },

  amazonLogout: async (): Promise<void> => {
    return await invoke<void>('amazon_logout');
  },

  amazonIsAuthenticated: async (): Promise<boolean> => {
    return await invoke<boolean>('amazon_is_authenticated');
  },

  /**
   * Importa a biblioteca completa de jogos possuídos na conta Amazon Games.
   * Cruza com jogos instalados detectados via Amazon Games App (Windows apenas).
   * Sem conta conectada, importa somente os jogos instalados localmente.
   *
   * @throws Se a API estiver indisponível
   */
  importAmazonGames: async (): Promise<void> => {
    return await invoke<void>('import_amazon_games');
  },

  /**
   * Importa jogos instalados via Xbox App / Microsoft Store (Gaming Services).
   * Detecta automaticamente em qualquer drive com um marcador `.GamingRoot`
   * na raiz. Windows apenas.
   *
   * Jogos instalados via Game Pass permanecem na biblioteca mesmo depois de saírem
   * do catálogo ou a assinatura ser cancelada — só a remoção manual do jogo
   * instalado tira ele da biblioteca.
   *
   * @throws Se nenhum jogo for detectado
   */
  importXboxGames: async (): Promise<void> => {
    return await invoke<void>('import_xbox_games');
  },

  /**
   * Importa jogos da IndieGala via IGClient. Detecção totalmente automática
   * — sem login e sem pasta a configurar.
   *
   * `full=false`: só jogos instalados no momento, via installed.json
   * (%APPDATA%\IGClient\storage\installed.json).
   * `full=true`: biblioteca completa de posse via config.json
   * (%APPDATA%\IGClient\config.json), cruzada com installed.json pra marcar
   * o que está instalado e reaproveitar metadados completos desses casos.
   *
   * @throws Se nenhum jogo for encontrado
   */
  importIndiegalaGames: async (full: boolean): Promise<void> => {
    return await invoke<void>('import_indiegala_games', {
      full,
      installedJsonPath: null,
      configJsonPath: null,
    });
  },

  /**
   * Importa jogos da Itch.io lendo o butler.db. Detecção automática.
   *
   * `full=false`: apenas jogos efetivamente instalados localmente.
   * `full=true`: biblioteca completa de posse do usuário na plataforma.
   *
   * @throws Se nenhum jogo for encontrado ou o banco de dados não existir.
   */
  importItchGames: async (full: boolean): Promise<void> => {
    return await invoke<void>('import_itch_games', {
      full,
      butlerDbPath: null,
    });
  },

  /**
   * Inicia o jogo especificado pelo ID, resolvendo automaticamente a melhor estratégia disponível:
   * protocolo do launcher (Steam, Battle.net), executável direto (Epic, GOG), ou abre o launcher/loja da plataforma
   * caso o jogo não esteja instalado.
   *
   * @param gameId
   * @param launcherPathOverride
   */
  launchGame: async (
    gameId: string,
    launcherPathOverride?: string
  ): Promise<LaunchOutcome> => {
    return await invoke<LaunchOutcome>('launch_game', {
      gameId,
      launcherPathOverride: launcherPathOverride ?? null,
    });
  },
};
