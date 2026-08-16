export interface TrendingGame {
  id: number;
  name: string;
  slug: string;
  coverUrl: string | null;
  genres: string[];
  series: string | null;
}

export interface UpcomingGame {
  name: string;
  slug: string;
  releaseDate: string | null;
  coverUrl: string | null;
  genres: string[];
  series: string | null;
}

export interface KeysBatch {
  steamId: string;
  steamApiKey: string;
  steamgriddbApiKey?: string;
  igdbClientId: string;
  igdbClientSecret?: string;
  rawgApiKey: string;
  geminiApiKey?: string;
  gamebrainApiKey?: string;
  nexusApiKey?: string;
  xboxLiveClientId?: string;
  xboxLiveClientSecret?: string;
}

export interface ImportSummary {
  successCount: number;
  errorCount: number;
  totalProcessed: number;
  message: string;
  errors: string[];
}

export interface Giveaway {
  id: number;
  title: string;
  image: string;
  worth: string;
  platforms: string;
  open_giveaway_url: string;
  end_date: string | null;
  description: string;
}
