export interface RawgGame {
  id: number;
  name: string;
  backgroundImage: string | null;
  rating: number;
  released: string | null;
  genres: { name: string }[];
  tags?: { name: string }[];
  series?: string | null;
}

export interface KeysBatch {
  steamId: string;
  steamApiKey: string;
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
