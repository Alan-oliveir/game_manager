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
