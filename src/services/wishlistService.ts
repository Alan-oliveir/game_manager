import { invoke } from "@tauri-apps/api/core";
import { WishlistGame } from "../types";

export interface SteamSearchResult {
  id: number;
  name: string;
  tiny_image?: string;
}

export interface AddToWishlistParams {
  id: string;
  name: string;
  coverUrl?: string;
  storeUrl: string;
  currentPrice?: number | null;
  steamAppId: number;
}

export const wishlistService = {
  getWishlist: async (): Promise<WishlistGame[]> => {
    return await invoke<WishlistGame[]>("get_wishlist");
  },

  removeFromWishlist: async (id: string): Promise<void> => {
    await invoke("remove_from_wishlist", { id });
  },

  refreshPrices: async (): Promise<void> => {
    await invoke("refresh_prices");
  },

  searchWishlistGame: async (query: string): Promise<SteamSearchResult[]> => {
    return await invoke<SteamSearchResult[]>("search_wishlist_game", { query });
  },

  addToWishlist: async (game: SteamSearchResult): Promise<void> => {
    await invoke("add_to_wishlist", {
      id: game.id.toString(),
      name: game.name,
      coverUrl: game.tiny_image,
      storeUrl: `https://store.steampowered.com/app/${game.id}`,
      currentPrice: null,
      steamAppId: game.id,
    });
  },
};
