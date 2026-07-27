export interface GameTag {
  slug: string;
  name: string;
  category: TagCategory;
  relevance: number;
}

export type TagCategory =
  | 'mode'
  | 'narrative'
  | 'theme'
  | 'gameplay'
  | 'meta'
  | 'technical'
  | 'input';

export const CATEGORY_MULTIPLIERS: Record<TagCategory | 'unknown', number> = {
  gameplay: 2,
  theme: 1.5,
  narrative: 1.3,
  mode: 1.2,
  meta: 0.8,
  technical: 0.7,
  input: 0.5,
  unknown: 0.5,
};

// Plataformas suportadas (deve corresponder ao enum Platform do Rust)
export type Platform =
  | 'Steam'
  | 'Epic'
  | 'GOG'
  | 'Amazon'
  | 'Ubisoft'
  | 'EA'
  | 'BattleNet'
  | 'Xbox'
  | 'Heroic'
  | 'LegacyGames'
  | 'Indiegala'
  | 'Itch'
  | 'Indie'
  | 'Outra';

// Dicionário para renderização visual na interface
export const PlatformDisplayNames: Record<Platform, string> = {
  Steam: 'Steam',
  Epic: 'Epic Games',
  GOG: 'GOG',
  Amazon: 'Amazon Games',
  Ubisoft: 'Ubisoft Connect',
  EA: 'EA App',
  BattleNet: 'Battle.net',
  Xbox: 'Xbox',
  Heroic: 'Heroic Launcher',
  LegacyGames: 'Legacy Games',
  Indiegala: 'IndieGala',
  Itch: 'Itch.io',
  Indie: 'Indie',
  Outra: 'Outra',
};

// Nível de confiança da importação
export type ImportConfidence = 'High' | 'Medium' | 'Low';

/**
 * Tipos para iniciar jogos por plataforma
 */
export type LaunchOutcome =
  | { kind: 'launched' }
  | { kind: 'openedLauncher'; installed: boolean }
  | { kind: 'openedStore' }
  | { kind: 'unavailable' };

/**
 * Informações básicas do jogo - Schema 3.0
 *
 * Dados básicos armazenados no banco de dados local.
 * Esses dados são essenciais para a exibição e gerenciamento dos jogos na biblioteca.
 * Também incluem campos para execução e dados do usuário.
 */
export interface Game {
  id: string;
  name: string;
  coverUrl?: string;

  // Identificação
  platform: Platform;
  platformGameId: string;
  genres?: string;
  developer?: string;

  // Execução
  installed: boolean;
  installPath?: string;
  executablePath?: string;
  launchArgs?: string;
  importConfidence?: ImportConfidence;

  // Dados do usuário
  status?: 'playing' | 'completed' | 'backlog' | 'abandoned' | 'plan_to_play';
  userRating?: number;
  favorite: boolean;

  // Dados de tempo
  playtime?: number;
  lastPlayed?: string;
  addedAt: string;

  // Conteúdo Adulto
  isAdult: boolean;
}

/**
 * Detalhes adicionais do jogo - Schema 3.0
 *
 * Metadados enriquecidos armazenados no banco de dados local,
 * provenientes de APIs externas (RAWG, STEAM).
 */
export interface GameDetails {
  gameId: string;
  steamAppId?: string;

  // Metadados
  descriptionRaw?: string;
  descriptionPtbr?: string;
  releaseDate?: string;
  developer?: string;
  publisher?: string;
  genres?: string;
  tags?: GameTag[] | string;
  series?: string;
  backgroundImage?: string;

  // Scores & Reviews
  criticScore?: number; // Metacritic
  steamReviewLabel?: SteamReviewSummary; // "Very Positive"
  steamReviewCount?: number;
  steamReviewScore?: number; // % (0-100)
  steamReviewUpdatedAt?: string;

  // Classificação & Conteúdo
  esrbRating?: string; // "Mature", "Teen", etc.
  isAdult?: boolean;
  adultTags?: string;

  // Links & Tempo
  externalLinks?: Record<string, string>; // { "steam": "url", "website": "url" }
  medianPlaytime?: number; // Horas (SteamSpy)
  estimatedPlaytime?: number; // Tempo estimado em horas (float)
}

export type SteamReviewSummary =
  | 'Overwhelmingly Positive'
  | 'Very Positive'
  | 'Positive'
  | 'Mostly Positive'
  | 'Mixed'
  | 'Mostly Negative'
  | 'Negative'
  | 'Very Negative'
  | 'Overwhelmingly Negative'
  | 'No user reviews';

export interface GamePlatformLink {
  id: string;
  platform: string;
}

export interface GameActions {
  onToggleFavorite: (id: string) => void;
  onGameClick: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  onEditGame: (game: Game) => void;
}
