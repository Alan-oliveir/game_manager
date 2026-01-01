import { invoke } from "@tauri-apps/api/core";
import { Game } from "../types";

export const librariesService = {
  initDb: async (): Promise<void> => {
    await invoke("init_db");
  },

  getGames: async (): Promise<Game[]> => {
    return await invoke<Game[]>("get_games");
  },

  addGame: async (game: {
    id: string;
    name: string;
    genre: string;
    platform: string;
    coverUrl: string | null;
    playtime: number;
    rating: number | null;
  }): Promise<void> => {
    await invoke("add_game", game);
  },

  updateGame: async (game: {
    id: string;
    name: string;
    genre: string;
    platform: string;
    coverUrl: string | null;
    playtime: number;
    rating: number | null;
  }): Promise<void> => {
    await invoke("update_game", game);
  },

  deleteGame: async (id: string): Promise<void> => {
    await invoke("delete_game", { id });
  },

  toggleFavorite: async (id: string): Promise<void> => {
    await invoke("toggle_favorite", { id });
  },
};
