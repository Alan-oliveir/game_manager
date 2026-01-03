export interface Game {
  id: string;
  name: string;
  genre?: string;
  platform?: string;
  cover_url?: string;
  playtime: number;
  rating?: number;
  favorite: boolean;
}

export interface GameActions {
  onToggleFavorite: (id: string) => void;
  onGameClick: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  onEditGame: (game: Game) => void;
}

export interface RawgGame {
  id: number;
  name: string;
  background_image: string | null;
  rating: number;
  released: string | null;
  genres: { name: string }[];
}

export interface WishlistGame {
  id: string;
  name: string;
  cover_url: string | null;
  store_url: string | null;
  current_price: number | null;
  lowest_price: number | null;
  on_sale: boolean;
  localized_price: number | null;
  localized_currency: string | null;
  steam_app_id: number | null;
  added_at: string;
}

export interface KeysBatch {
  steam_id: string;
  steam_api_key: string;
  rawg_api_key: string;
}

export interface ImportSummary {
  success_count: number;
  error_count: number;
  total_processed: number;
  message: string;
  errors: string[];
}

export interface GameDetails {
  description_raw: string; // Descrição em texto puro
  metacritic: number | null;
  website: string;
  tags: { id: number; name: string }[];
  developers: { name: string }[];
  publishers: { name: string }[];
}

export interface GamePlatformLink {
  id: string;
  platform: string;
}

export interface GenreScore {
  name: string;
  score: number;
  game_count: number;
}

export interface UserProfile {
  top_genres: GenreScore[];
  total_playtime: number;
  total_games: number;
}
