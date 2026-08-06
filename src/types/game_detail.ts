/**
 * Item unificado da fila de mídia
 * Usada em GameMedia.tsx e MediaViewer.tsx
 */
export type MediaItem =
  | { kind: 'screenshot'; url: string }
  | { kind: 'trailer'; url: string }
  | { kind: 'youtube'; url: string };

/**
 * Espelha SimilarGame do backend (gamebrain.rs)
 * OBS.: campo becauseOf adicionado por ProfileSimilarGame
 */
export interface SimilarGame {
  id: string;
  name: string;
  coverUrl: string | null;
  genre: string | null;
  year: number | null;
  rating: number | null;
  link: string | null;
  screenshots: string[];
  microTrailer: string | null;
  adultOnly: boolean;
  becauseOf: string; // nome do jogo âncora
}

/**
 * Requisitos de sistema de um bloco `{{System requirements}}`.
 * Um bloco representa um OS (`os_family`) e opcionalmente um tier extra
 * (`tier_title`), como "High" ou "Ultra" em jogos AAA modernos.
 *
 * Campos `min_cpu2` / `min_gpu2` chegam já combinados pelo backend como
 * "Intel Core i5 / AMD Ryzen 5" — mantidos aqui apenas para compatibilidade
 * caso o backend passe a enviar separado no futuro.
 */
export interface Sysreq {
  osFamily: string;
  tierTitle: string | null;
  target: string | null;
  // Minimum
  minOs: string | null;
  minCpu: string | null;
  minCpu2: string | null;
  minRam: string | null;
  minGpu: string | null;
  minGpu2: string | null;
  minVram: string | null;
  minDx: string | null;
  minStorage: string | null;
  // Recommended
  recOs: string | null;
  recCpu: string | null;
  recCpu2: string | null;
  recRam: string | null;
  recGpu: string | null;
  recGpu2: string | null;
  recVram: string | null;
  recDx: string | null;
  recStorage: string | null;
}

/**
 * Caminho de save ou configuração extraído de `{{Game data/config|OS|path}}`
 * e `{{Game data/saves|OS|path}}`.
 *
 * `raw_path` preserva a sintaxe `{{p|variavel}}` do wikitext.
 * `expanded_path` é preenchido pelo backend quando possível; caso contrário
 * o frontend expande via `expandPathVars()` em `utils/pcgw.ts`.
 */
export interface GameDataPath {
  kind: 'config' | 'saves';
  os: string;
  rawPath: string;
  expandedPath: string | null;
}

/**
 * Conjunto completo de dados extraídos por scraping de wikitext.
 * Retornado pelo comando Tauri `get_pcgw_scraped_data`.
 */
export interface PcgwScrapedData {
  systemRequirements: Sysreq[];
  configPaths: GameDataPath[];
  savePaths: GameDataPath[];
}

/**
 * Dados estruturados retornados pela Cargo API do PCGamingWiki.
 * Retornado pelo comando Tauri `get_or_fetch_pcgw_data`.
 *
 * Campos booleanos do PCGamingWiki (`true`, `false`, `hackable`, `unknown`,
 * `n/a`) chegam como string para que o frontend controle a semântica de
 * exibição via `BoolBadge`.
 *
 * Renomeado de `GameExtras` (nome original no componente) para `PcgwCargoData`
 * a fim de evitar conflito com o nome do componente `GameExtras`.
 */
export interface PcgwCargoData {
  steamAppId: string;
  pcgwPageId: string | null;
  pcgwPageName: string | null;
  // Geral
  engine: string | null;
  availableOn: string | null;
  // API gráfica
  dxVersions: string | null;
  vulkanVersions: string | null;
  openglVersions: string | null;
  // Suporte a SO (executáveis)
  win64: string | null;
  linux64: string | null;
  macOsArm: string | null;
  macOsIntel64: string | null;
  // Vídeo
  rayTracing: string | null;
  upscaling: string | null;
  frameGen: string | null;
  ultrawidescreen: string | null;
  fourKSupport: string | null;
  hdr: string | null;
  highFps: string | null;
  fov: string | null;
  borderlessWindowed: string | null;
  colorBlind: string | null;
  // Controles
  controllerSupport: string | null;
  fullController: string | null;
  playstationControllers: string | null;
  xinputControllers: string | null;
  // Áudio e acessibilidade
  surroundSound: string | null;
  subtitles: string | null;
  closedCaptions: string | null;
  // Localização
  languagesInterface: string[] | null;
  languagesAudio: string[] | null;
  languagesSubtitles: string[] | null;
  // Metadados
  hasSaveData: string | null;
  hasConfigData: string | null;
  fetchedAt: string | null;
}

/**
 * Linha normalizada da tabela de idiomas, derivada de `PcgwCargoData.languages*`.
 * Produzida por `buildLanguageRows()` em `utils/pcgw.ts`.
 */
export interface LangRow {
  lang: string;
  interface: boolean;
  audio: boolean;
  subtitles: boolean;
}

/**
 * Dados estruturados retornados pela API do ProtonDB.
 */
export interface ProtonDbSummary {
  tier: string;
  trendingTier: string;
  bestReportedTier: string;
  confidence: string;
  score: number;
  total: number;
}

export interface TrendingMod {
  name: string;
  author: string;
  summary: string;
  pictureUrl: string;
  modPageUrl: string;
}

export type ModsAvailability = 'NoNexusMatch' | 'Available';

export interface TrendingModsResult {
  availability: ModsAvailability;
  mods: TrendingMod[];
}
