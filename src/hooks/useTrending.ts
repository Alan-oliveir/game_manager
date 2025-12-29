import { useState, useEffect, useMemo } from "react";
import { RawgGame, Game } from "../types";
import { trendingService } from "../services/trendingService";

interface UseTrendingProps {
  userGames: Game[];
  cachedGames: RawgGame[];
  setCachedGames: (games: RawgGame[]) => void;
}

export function useTrending({
  userGames,
  cachedGames,
  setCachedGames,
}: UseTrendingProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedGenre, setSelectedGenre] = useState<string>("all");

  const fetchTrending = async () => {
    // Se já tem cache e não tem erro, usa o cache
    if (cachedGames.length > 0 && !error) {
      console.log("Usando cache para Trending.");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const apiKey = await trendingService.getApiKey();

      if (!apiKey || apiKey.trim() === "") {
        throw new Error("API Key da RAWG não configurada.");
      }

      console.log("Buscando jogos na RAWG...");
      const result = await trendingService.getTrending(apiKey);
      setCachedGames(result); // Atualiza o cache no App.tsx via prop
    } catch (err: any) {
      console.error("Erro no hook useTrending:", err);
      const msg = String(err);
      if (msg.includes("não configurada") || msg.includes("401")) {
        setError("API Key inválida ou ausente. Verifique as configurações.");
      } else {
        setError(`Erro ao buscar jogos: ${msg}`);
      }
    } finally {
      setLoading(false);
    }
  };

  // Carrega apenas na montagem inicial se não houver cache
  useEffect(() => {
    fetchTrending();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Lógica de Filtragem (Memoized para performance)
  const { filteredGames, allGenres } = useMemo(() => {
    // Remove jogos que o usuário já tem na biblioteca local
    const userGameNames = new Set(
      userGames.map((g) => g.name.toLowerCase().replace(/[^a-z0-9]/g, ""))
    );

    const available = cachedGames.filter((rawgGame) => {
      const rawgName = rawgGame.name.toLowerCase().replace(/[^a-z0-9]/g, "");
      return !userGameNames.has(rawgName);
    });

    // Extrai gêneros únicos para o filtro
    const genres = Array.from(
      new Set(cachedGames.flatMap((g) => g.genres.map((genre) => genre.name)))
    ).sort();

    // Aplica o filtro de gênero selecionado
    const filtered = available.filter((game) => {
      if (selectedGenre === "all") return true;
      return game.genres.some((g) => g.name === selectedGenre);
    });

    return { filteredGames: filtered, allGenres: genres };
  }, [cachedGames, userGames, selectedGenre]);

  return {
    games: filteredGames,
    allGenres,
    loading,
    error,
    selectedGenre,
    setSelectedGenre,
    retry: fetchTrending,
    addToWishlist: trendingService.addToWishlist,
  };
}
