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
  added_at: string;
}
