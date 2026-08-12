import { useEffect, useState } from 'react';

import { useNetworkStatus } from '@/hooks/common';
import { useLibraryStats } from '@/hooks/library';
import { useRecommendation } from '@/hooks/recommendation';
import { Game, TrendingGame, UserPreferenceVector } from '@/types';

import { trendingService } from '../services/trendingService';

const HOME_TRENDING_TTL_MS = 10 * 60 * 1000;

interface UseHomeProps {
  games: Game[];
  trendingCache: TrendingGame[];
  setTrendingCache: (games: TrendingGame[]) => void;
  profileCache: UserPreferenceVector | null;
  setProfileCache: (profile: UserPreferenceVector) => void;
  trendingFetchedAt: number | null;
  setTrendingFetchedAt: (value: number | null) => void;
}

/**
 * Hook orquestrador para a página Home
 *
 * Responsabilidade: Agregar dados de múltiplas fontes:
 * - Estatísticas da biblioteca (useLibraryStats)
 * - Recomendações (useRecommendation)
 * - Jogos em tendência (API RAWG)
 *
 * Mantém apenas lógica de orquestração e cache de tendências.
 *
 * @param games - Lista completa de jogos na biblioteca do usuário
 * @param trendingCache - Cache local dos jogos em tendência
 * @param setTrendingCache - Função para atualizar o cache de jogos em tendência
 * @param profileCache - Cache local do perfil de preferências do usuário
 * @param setProfileCache - Função para atualizar o cache do perfil de preferências
 * @returns Objeto contendo dados e estados para a página inicial
 */
export function useHome({
  games: library,
  trendingCache,
  setTrendingCache,
  profileCache,
  setProfileCache,
  trendingFetchedAt,
  setTrendingFetchedAt,
}: UseHomeProps) {
  // === ESTATÍSTICAS DA BIBLIOTECA ===
  const { stats, continuePlaying, mostPlayed, topGenres } = useLibraryStats({
    games: library,
  });

  // === RECOMENDAÇÕES ===
  const {
    profile,
    recommendations: backlogRecommendations, // Content-Based
    collaborativeRecs, // Collaborative
    loadingRecommendations,
    loading: profileLoading,
  } = useRecommendation({
    profileCache,
    setProfileCache,
    allGames: library,
    enableContentBased: true,
    enableCollaborative: true,
    enableHybrid: false, // Home não usa a lista unificada (apenas Playlist)
    contentBasedParams: {
      minPlaytime: 0,
      maxPlaytime: 300,
      limit: 5,
    },
    collaborativeParams: {
      minPlaytime: 0,
      maxPlaytime: 120,
      limit: 5,
    },
  });

  const isOnline = useNetworkStatus();

  // === TRENDING (IGDB API) ===
  const [trending, setTrending] = useState<TrendingGame[]>(trendingCache);
  const [loadingTrending, setLoadingTrending] = useState(false);

  useEffect(() => {
    async function fetchTrendingIfNeeded() {
      if (trendingCache.length > 0) {
        setTrending(trendingCache);
      }

      const now = Date.now();
      const cacheFresh =
        trendingFetchedAt && now - trendingFetchedAt < HOME_TRENDING_TTL_MS;

      if (!isOnline && trendingCache.length > 0) {
        setLoadingTrending(false);

        return;
      }

      if (cacheFresh) {
        setLoadingTrending(false);

        return;
      }

      setLoadingTrending(true);

      try {
        const result = await trendingService.getTrending();
        setTrending(result);
        setTrendingCache(result);
        setTrendingFetchedAt(Date.now());
      } catch (error) {
        console.error('Erro ao carregar trending:', error);
      } finally {
        setLoadingTrending(false);
      }
    }
    fetchTrendingIfNeeded();
  }, [
    trendingCache,
    setTrendingCache,
    trendingFetchedAt,
    setTrendingFetchedAt,
    isOnline,
  ]);

  // === RETORNO AGREGADO ===
  return {
    // Estatísticas da biblioteca
    stats,
    continuePlaying,
    mostPlayed,
    topGenres,

    // Recomendações
    backlogRecommendations,
    collaborativeRecs,
    loadingRecommendations,
    profile,
    profileLoading,

    // Trending
    trending,
    loadingTrending,
  };
}
