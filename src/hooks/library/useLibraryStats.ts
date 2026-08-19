import { useMemo } from 'react';

import { Game } from '@/types';

interface UseLibraryStatsProps {
  games: Game[];
}

/**
 * Hook para calcular estatísticas da biblioteca do usuário.
 *
 * Fornece dados sobre:
 * - Total de jogos, tempo de jogo e favoritos
 * - Jogos "Continue Jogando" (com progresso parcial)
 * - Jogos mais jogados
 * - Gêneros mais populares
 *
 * @param games - Lista completa de jogos na biblioteca
 * @returns Objeto com estatísticas e listas calculadas
 */
export function useLibraryStats({ games }: UseLibraryStatsProps) {
  // Estatísticas gerais
  const stats = useMemo(() => {
    const totalPlaytime = games.reduce((acc, g) => acc + (g.playtime ?? 0), 0);
    const totalFavorites = games.filter(g => g.favorite).length;
    const totalCompleted = games.filter(g => g.status === 'completed').length;
    const totalPlaying = games.filter(g => g.status === 'playing').length;

    return {
      totalGames: games.length,
      totalPlaytime,
      totalFavorites,
      totalCompleted,
      totalPlaying,
    };
  }, [games]);

  // Continue Jogando: Jogos com progresso parcial, ordenados por última jogada
  const continuePlaying = useMemo(() => {
    return games
      .filter(g => (g.playtime ?? 0) > 0 && (g.playtime ?? 0) < 600)
      .sort(
        (a, b) =>
          (b.lastPlayed ? new Date(b.lastPlayed).getTime() : 0) -
          (a.lastPlayed ? new Date(a.lastPlayed).getTime() : 0)
      )
      .slice(0, 5);
  }, [games]);

  // Mais Jogados: Top 3 jogos por tempo de jogo
  const mostPlayed = useMemo(() => {
    return [...games]
      .sort((a, b) => (b.playtime ?? 0) - (a.playtime ?? 0))
      .slice(0, 3);
  }, [games]);

  // Top Gêneros: 6 gêneros mais comuns na biblioteca
  const topGenres = useMemo(() => {
    const genreStats = games.reduce(
      (acc, game) => {
        game.genres?.forEach((g: string) => {
          const clean = g.trim();

          if (clean !== 'Desconhecido' && clean !== '') {
            acc[clean] = (acc[clean] || 0) + 1;
          }
        });

        return acc;
      },
      {} as Record<string, number>
    );

    return Object.entries(genreStats)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 6);
  }, [games]);

  return {
    stats,
    continuePlaying,
    mostPlayed,
    topGenres,
  };
}
