import { invoke } from '@tauri-apps/api/core';

import { Game } from '@/types/game';

export const librariesService = {
  /** Inicializa o banco de dados local (SQLite) se ainda não existir */
  initDb: async (): Promise<void> => {
    await invoke('init_db');
  },

  /** Busca todos os jogos da biblioteca local (SQLite) */
  getGames: async (): Promise<Game[]> => {
    return await invoke<Game[]>('get_games');
  },

  /**
   * Adiciona novo jogo à biblioteca local (SQLite).
   * Se já existir jogo com mesmo ID, sobrescreve completamente os dados.
   */
  addGame: async (game: {
    id: string;
    name: string;
    library: string;
    libraryGameId: string;
    coverUrl: string | null;
    installed: boolean;
    importConfidence: string | null;
    playtime: number;
    userRating: number | null;
    status: 'playing' | 'completed' | 'backlog' | 'abandoned' | 'plan_to_play';
    installPath: string | null;
    executablePath: string | null;
    launchArgs: string | null;
  }) => {
    await invoke('add_game', { game });
  },

  /**
   * Atualiza dados de um jogo existente na biblioteca.
   * Não faz nada se o ID não existir (operação silenciosa).
   */
  updateGame: async (game: {
    id: string;
    name: string;
    library: string;
    libraryGameId: string;
    coverUrl: string | null;
    installed: boolean;
    importConfidence: string | null;
    playtime: number;
    userRating: number | null;
    status: 'playing' | 'completed' | 'backlog' | 'abandoned' | 'plan_to_play';
    installPath: string | null;
    executablePath: string | null;
    launchArgs: string | null;
  }) => {
    await invoke('update_game', { game });
  },

  /** Remove jogo da biblioteca local pelo ID */
  deleteGame: async (id: string): Promise<void> => {
    await invoke('delete_game', { id });
  },

  /** Alterna o status de favorito de um jogo na biblioteca local */
  toggleFavorite: async (id: string): Promise<void> => {
    await invoke('toggle_favorite', { id });
  },
};
