/**
 * Caminhos de arquivo/config verificados automaticamente por cada
 * plataforma, exibidos nas caixas de "detecção automática".
 */
export const DETECTED_PATHS = {
  epic: {
    windows: 'C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests',
    linuxWine: '<wine_prefix>/drive_c/ProgramData/Epic/.../Manifests',
  },
  heroic: {
    linuxNative: '~/.config/heroic',
    linuxFlatpak: '~/.var/app/com.heroicgameslauncher.hgl/config/heroic',
    windows: '%APPDATA%\\heroic',
  },
  ubisoft: {
    windows: '%LOCALAPPDATA%\\Ubisoft Game Launcher',
    linux:
      '<wine_prefix>/drive_c/users/<USER>/AppData/Local/Ubisoft Game Launcher',
  },
  legacy: {
    windows: '%APPDATA%\\Roaming\\legacy-games-launcher\\app-state.json',
    linuxWine:
      '<wine_prefix>/drive_c/users/<USER>/AppData/Roaming/legacy-games-launcher/',
  },
  amazon: {
    windows:
      'C:\\Users\\<user>\\AppData\\Local\\Amazon Games\\Data\\Games\\Sql\\GameInstallInfo.sqlite',
  },
  battleNet: {
    windows: 'C:\\ProgramData\\Battle.net\\Agent',
  },
  indiegala: {
    windowsInstalled: '%APPDATA%\\IGClient\\storage\\installed.json',
    windowsConfig: '%APPDATA%\\IGClient\\config.json',
  },
  itch: {
    windows: '%APPDATA%\\itch\\db\\butler.db',
    linux: '~/.config/itch/db/butler.db',
  },
} as const;
