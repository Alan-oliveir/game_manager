import { invoke } from '@tauri-apps/api/core';

import { TrendingGame, UpcomingGame } from '@/types';

export const trendingService = {
  getTrending: async (): Promise<TrendingGame[]> => {
    return await invoke<TrendingGame[]>('get_trending_games');
  },

  getUpcoming: async (): Promise<UpcomingGame[]> => {
    return await invoke<UpcomingGame[]>('get_upcoming_games');
  },

  addToWishlist: async (game: TrendingGame): Promise<void> => {
    await invoke('add_to_wishlist', {
      id: game.id.toString(),
      name: game.name,
      coverUrl: game.coverUrl,
      storeUrl: `https://www.igdb.com/games/${game.slug}`,
      currentPrice: null,
    });
  },
};
