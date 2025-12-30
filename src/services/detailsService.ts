import { invoke } from "@tauri-apps/api/core";
import { GameDetails } from "../types";

export const detailsService = {
  // Busca detalhes extras na API (RAWG) baseado no nome
  getGameDetails: async (gameName: string): Promise<GameDetails | null> => {
    try {
      return await invoke<GameDetails>("fetch_game_details", {
        query: gameName,
      });
    } catch (error) {
      console.error("Erro ao buscar detalhes:", error);
      return null;
    }
  },
};
