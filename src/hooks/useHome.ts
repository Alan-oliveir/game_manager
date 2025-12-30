import { useState, useEffect } from "react";
import { RawgGame, Game, UserProfile } from "../types";
import { useRecommendation } from "./useRecommendation";
import { trendingService } from "../services/trendingService";

interface UseHomeProps {
  games: Game[];
  trendingCache: RawgGame[];
  setTrendingCache: (games: RawgGame[]) => void;
  profileCache: UserProfile | null;
  setProfileCache: (profile: UserProfile) => void;
}

export function useHome({
  games: library,
  trendingCache,
  setTrendingCache,
  profileCache,
  setProfileCache,
}: UseHomeProps) {
  const [trending, setTrending] = useState<RawgGame[]>(trendingCache);
  const [loadingTrending, setLoadingTrending] = useState(false);
  const {
    profile,
    calculateAffinity,
    loading: profileLoading,
  } = useRecommendation({
    profileCache,
    setProfileCache,
  });

  // Busca Trending se o cache estiver vazio
  useEffect(() => {
    async function fetchTrendingIfNeeded() {
      if (trendingCache.length > 0) {
        setTrending(trendingCache);
        setLoadingTrending(false);
        return;
      }

      setLoadingTrending(true);
      try {
        const apiKey = await trendingService.getApiKey();
        if (apiKey && apiKey.trim() !== "") {
          const result = await trendingService.getTrending(apiKey);
          setTrending(result);
          setTrendingCache(result);
        }
      } catch (e) {
        console.warn("Home: Falha ao buscar trending", e);
      } finally {
        setLoadingTrending(false);
      }
    }
    fetchTrendingIfNeeded();
  }, [trendingCache]); // Roda se o cache mudar ou montar

  // Lógica de Negócios (Síncrona - Instantânea)

  // 1. Stats
  const totalGames = library.length;
  const totalPlaytime = library.reduce((acc, g) => acc + g.playtime, 0);
  const totalFavorites = library.filter((g) => g.favorite).length;

  // 2. Continue Jogando
  const continuePlaying = library
    .filter((g) => g.playtime > 0 && g.playtime < 50)
    .sort((a, b) => b.playtime - a.playtime)
    .slice(0, 5);

  // 3. Recomendado (Backlog)
  let backlogRecommendations: Game[] = [];
  if (profile) {
    backlogRecommendations = library
      .filter((g) => g.playtime === 0)
      .sort((a, b) => {
        const genresA = a.genre
          ? a.genre.split(",").map((n) => ({ name: n.trim() }))
          : [];
        const genresB = b.genre
          ? b.genre.split(",").map((n) => ({ name: n.trim() }))
          : [];
        return calculateAffinity(genresB) - calculateAffinity(genresA);
      })
      .slice(0, 5);
  }

  // 4. Mais Jogados
  const mostPlayed = [...library]
    .sort((a, b) => b.playtime - a.playtime)
    .slice(0, 3);

  // 5. Gêneros
  const genreStats = library.reduce((acc, game) => {
    if (game.genre) {
      game.genre.split(",").forEach((g) => {
        const clean = g.trim();
        if (clean !== "Desconhecido") acc[clean] = (acc[clean] || 0) + 1;
      });
    }
    return acc;
  }, {} as Record<string, number>);

  const topGenres = Object.entries(genreStats)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 6);

  return {
    stats: { totalGames, totalPlaytime, totalFavorites },
    continuePlaying,
    backlogRecommendations,
    mostPlayed,
    topGenres,
    trending,
    profile,
    // Mostra loading se estiver carregando perfil ou trending crítico
    loading: profileLoading || (loadingTrending && trending.length === 0),
  };
}
