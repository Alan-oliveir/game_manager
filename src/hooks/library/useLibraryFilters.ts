import { useMemo } from 'react';

import { Game } from '@/types';

export interface ViewFilters {
  hideAdult: boolean;
  hideDuplicates: boolean;
  hideNotInstalled: boolean;
}

interface UseLibraryFilterOptions {
  games: Game[];
  searchTerm: string;
  hideAdult?: boolean;
  hideDuplicates?: boolean;
  hideNotInstalled?: boolean;
}

/**
 * Dado um grupo de jogos com o mesmo nome, retorna o mais relevante:
 * prefere o que tiver favorito, rating, status ou playtime > 0.
 * Caso empate, retorna o primeiro da lista.
 */
function pickBestGame(group: Game[]): Game {
  return (
    group.find(
      g => g.favorite || g.userRating || g.status || (g.playtime ?? 0) > 0
    ) ?? group[0]
  );
}

/**
 * Hook para filtrar jogos da biblioteca por busca, conteúdo adulto e duplicatas.
 * Busca em: nome, gêneros e plataforma.
 *
 * @param games - Lista completa de jogos
 * @param searchTerm - Termo de busca
 * @param hideAdult - Se true, oculta jogos marcados como adultos
 * @param hideDuplicates - Se true, mantém apenas uma entrada por nome de jogo
 * @param hideNotInstalled
 * @returns Jogos filtrados
 */
export function useLibraryFilter({
  games,
  searchTerm,
  hideAdult,
  hideDuplicates,
  hideNotInstalled,
}: UseLibraryFilterOptions) {
  return useMemo(() => {
    // 1. Filtro de conteúdo adulto
    let result = hideAdult ? games.filter(game => !game.isAdult) : games;

    // 2. Filtro de não-instalados
    if (hideNotInstalled) {
      result = result.filter(game => game.installed);
    }

    // 3. Oculta duplicatas (mesmo nome, plataformas diferentes)
    if (hideDuplicates) {
      const groups = new Map<string, Game[]>();

      for (const game of result) {
        const key = game.slug;
        const group = groups.get(key) ?? [];
        group.push(game);
        groups.set(key, group);
      }

      result = Array.from(groups.values()).map(pickBestGame);
    }

    // 4. Busca
    if (!searchTerm) return result;

    const term = searchTerm.toLowerCase();

    return result.filter(
      game =>
        game.name.toLowerCase().includes(term) ||
        game.genres?.toLowerCase().includes(term) ||
        game.platform?.toLowerCase().includes(term)
    );
  }, [games, hideAdult, hideDuplicates, hideNotInstalled, searchTerm]);
}
