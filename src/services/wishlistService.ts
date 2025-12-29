import { invoke } from "@tauri-apps/api/core";
import { WishlistGame } from "../types";

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
};
