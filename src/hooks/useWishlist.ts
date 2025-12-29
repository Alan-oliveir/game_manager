import { useState, useEffect, useCallback } from "react";
import { WishlistGame } from "../types";
import { wishlistService } from "../services/wishlistService";

export function useWishlist() {
  const [games, setGames] = useState<WishlistGame[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const fetchWishlist = useCallback(async () => {
    try {
      const result = await wishlistService.getWishlist();
      setGames(result);
    } catch (error) {
      console.error("Erro ao buscar wishlist:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Carrega ao montar
  useEffect(() => {
    fetchWishlist();
  }, [fetchWishlist]);

  const removeGame = async (id: string) => {
    try {
      await wishlistService.removeFromWishlist(id);
      // Atualização otimista: remove da lista visual imediatamente
      setGames((prev) => prev.filter((g) => g.id !== id));
    } catch (error) {
      console.error(error);
      throw new Error("Erro ao remover jogo.");
    }
  };

  const refreshPrices = async () => {
    setIsRefreshing(true);
    try {
      await wishlistService.refreshPrices();
      // Recarrega a lista para pegar os preços novos
      await fetchWishlist();
    } catch (error) {
      console.error(error);
      throw new Error("Erro ao atualizar preços.");
    } finally {
      setIsRefreshing(false);
    }
  };

  return {
    games,
    isLoading,
    isRefreshing,
    removeGame,
    refreshPrices,
  };
}
