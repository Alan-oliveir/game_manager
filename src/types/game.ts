// === TAGS ===
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

// === BIBLIOTECA (STEAM, EPIG, GOG, etc.) ===

/**
 * Bibliotecas suportadas (deve corresponder ao enum Library do Rust)
 */
export type Library =
  | 'Steam'
  | 'Epic'
  | 'GOG'
  | 'Amazon'
  | 'Ubisoft'
  | 'EA'
  | 'BattleNet'
  | 'Xbox'
  | 'LegacyGames'
  | 'Indiegala'
  | 'Itch'
  | 'Indie'
  | 'Outra';

/**
 * Dicionário para renderização visual na interface
 */
export const LibraryDisplayNames: Record<Library, string> = {
  Steam: 'Steam',
  Epic: 'Epic Games',
  GOG: 'GOG',
  Amazon: 'Amazon Games',
  Ubisoft: 'Ubisoft Connect',
  EA: 'EA App',
  BattleNet: 'Battle.net',
  Xbox: 'Xbox',
  LegacyGames: 'Legacy Games',
  Indiegala: 'IndieGala',
  Itch: 'Itch.io',
  Indie: 'Indie',
  Outra: 'Outra',
};

/**
 * Status exibido nas telas de configuração de bibliotecas (StatusBadge).
 */
export interface ImportStatus {
  type: 'success' | 'error' | null;
  message: string;
}

/**
 * Tipos para iniciar jogos por biblioteca
 */
export type LaunchOutcome =
  | { kind: 'launched' }
  | { kind: 'openedLauncher'; installed: boolean }
  | { kind: 'openedStore' }
  | { kind: 'unavailable' };

export interface GameStoreLink {
  id: string;
  store: string;
}

/**
 * Referência a outra cópia do mesmo jogo em uma biblioteca diferente
 * (ex: o usuário possui o jogo tanto na Steam quanto na GOG).
 */
export interface GameLibraryLink {
  id: string;
  library: Library;
}

export interface GameActions {
  onToggleFavorite: (id: string) => void;
  onGameClick: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  onEditGame: (game: Game) => void;
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

/**
 * Nível de confiança da importação
 *
 * Usado com: Steam
 *
 * - Hight: jogos instalados (appmanifest)
 * - Medium: jogos não instalados (librarycache)
 * - Low: jogos não instalados (Steam API)
 */
export type ImportConfidence = 'High' | 'Medium' | 'Low';

export type PlaytimeSource = 'local' | { store: Library };

export interface GameDescriptionData {
  summary?: string;
  storyline?: string;
  shortDescription?: string;
  description?: string;
  summaryTranslated?: string;
  storylineTranslated?: string;
  shortDescriptionTranslated?: string;
  descriptionTranslated?: string;
  translatedLang?: string;
}

// === MODELOS DE DADOS (SCHEMA 4.0 - Game e GameDetails) ===

/**
 * Informações básicas do jogo - Schema 4.0
 *
 * Dados básicos armazenados no banco de dados local.
 * Esses dados são essenciais para a exibição e gerenciamento dos jogos na biblioteca.
 * Também incluem campos para execução e dados do usuário.
 */
export interface Game {
  id: string;
  name: string;
  slug: string;
  coverUrl?: string;

  // Identificação
  library: Library;
  sourceLabel?: string;
  libraryGameId: string;
  genres?: string;
  developer?: string;
  alternativeNames?: string;

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
  playtimeSource: PlaytimeSource | null;
  lastPlayed?: string;
  addedAt: string;

  // Conteúdo Adulto
  isAdult: boolean;
}

/**
 * Detalhes adicionais do jogo - Schema 4.0
 *
 * Metadados enriquecidos armazenados no banco de dados local,
 * provenientes de APIs externas (IGDB, STEAM).
 */
export interface GameDetails {
  gameId: string;
  steamAppId?: string;

  // Metadados
  description?: GameDescriptionData;
  releaseDate?: string;
  developer?: string;
  publisher?: string;
  genres?: string;
  tags?: GameTag[] | string;
  series?: string;

  // Scores & Reviews
  criticScore?: number; // Metacritic
  steamReviewLabel?: SteamReviewSummary; // "Very Positive"
  steamReviewCount?: number;
  steamReviewScore?: number; // % (0-100)
  steamReviewUpdatedAt?: string;

  // Classificação & Conteúdo
  isAdult?: boolean;
  adultTags?: string;

  // Links & Tempo
  externalLinks?: Record<string, string>; // { "steam": "url", "website": "url" }
  hltbMainStory?: number;
  hltbMainExtra?: number;
  hltbCompletionist?: number;
  hltbCoopTime?: number;
  updatedAt?: string;
}

export type DlcKind = 'expansion' | 'standalone_expansion' | 'dlc' | string;

export interface GameDlc {
  igdbId: number;
  name: string;
  igdbUrl: string | null;
  coverUrl: string | null;
  kind: DlcKind;
  owned: boolean;
}
