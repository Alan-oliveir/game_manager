import { invoke } from "@tauri-apps/api/core";
import { RawgGame } from "../types";

export const trendingService = {
  getApiKey: async (): Promise<string> => {
    return await invoke<string>("get_secret", { keyName: "rawg_api_key" });
  },

  getTrending: async (apiKey: string): Promise<RawgGame[]> => {
    return await invoke<RawgGame[]>("get_trending_games", { apiKey });
  },

  getUpcoming: async (apiKey: string): Promise<RawgGame[]> => {
    return await invoke<RawgGame[]>("get_upcoming_games", { apiKey });
  },

  addToWishlist: async (game: RawgGame): Promise<void> => {
    await invoke("add_to_wishlist", {
      id: game.id.toString(),
      name: game.name,
      coverUrl: game.background_image,
      storeUrl: null,
      currentPrice: null,
    });
  },
};
