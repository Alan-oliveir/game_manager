import { useEffect, useState } from 'react';

import { useNetworkStatus } from '@/hooks/common';
import { trendingService } from '@/services/trendingService';
import { UpcomingGame } from '@/types';

const UPCOMING_TTL_MS = 30 * 60 * 1000; // 30 minutos

interface UseUpcomingOptions {
  cachedGames: UpcomingGame[];
  setCachedGames: (games: UpcomingGame[]) => void;
  cachedFetchedAt: number | null;
  setCachedFetchedAt: (value: number | null) => void;
}

interface UseUpcomingResult {
  upcomingGames: UpcomingGame[];
  loading: boolean;
  error: string | null;
}

/**
 * Hook para gerenciar jogos de lançamentos futuros (upcoming games).
 * Busca os próximos lançamentos através da API da RAWG.
 *
 * @returns {Object} Estado dos lançamentos futuros
 *   - upcomingGames: Lista de jogos que serão lançados em breve
 *   - loading: Estado de carregamento
 *   - error: Mensagem de erro, se houver
 */
export function useUpcoming(options?: UseUpcomingOptions): UseUpcomingResult {
  const [upcomingGames, setUpcomingGames] = useState<UpcomingGame[]>(
    options?.cachedGames ?? []
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const isOnline = useNetworkStatus();

  useEffect(() => {
    if (options?.cachedGames?.length) {
      setUpcomingGames(options.cachedGames);
    }
  }, [options?.cachedGames]);

  useEffect(() => {
    const fetchUpcoming = async () => {
      const now = Date.now();
      const cacheFresh =
        options?.cachedFetchedAt &&
        now - options.cachedFetchedAt < UPCOMING_TTL_MS;

      if (!isOnline && (options?.cachedGames?.length ?? 0) > 0) {
        setLoading(false);

        return;
      }

      if (cacheFresh) {
        setLoading(false);

        return;
      }

      try {
        const upcoming = await trendingService.getUpcoming();
        setUpcomingGames(upcoming);
        options?.setCachedGames(upcoming);
        options?.setCachedFetchedAt(Date.now());
      } catch (e) {
        console.error('Erro ao buscar lançamentos:', e);
        setError(e instanceof Error ? e.message : 'Erro desconhecido');
      } finally {
        setLoading(false);
      }
    };

    fetchUpcoming();
  }, [
    isOnline,
    options?.cachedFetchedAt,
    options?.cachedGames?.length,
    options?.setCachedGames,
    options?.setCachedFetchedAt,
  ]);

  return {
    upcomingGames,
    loading,
    error,
  };
}
