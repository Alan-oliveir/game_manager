import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Game, RawgGame } from "../types";
import { useRecommendation } from "./useRecommendation";
import { trendingService } from "../services/trendingService";

export function useHome() {
    const [library, setLibrary] = useState<Game[]>([]);
    const [trending, setTrending] = useState<RawgGame[]>([]);
    const [loading, setLoading] = useState(true);

    const { profile, calculateAffinity, loading: profileLoading } = useRecommendation();

    useEffect(() => {
        async function fetchData() {
            try {
                // Busca Biblioteca Local
                const games = await invoke<Game[]>("get_games");
                setLibrary(games);

                // Busca API Key e Trending usando o serviço/backend
                try {
                    // Usa o comando do backend 'get_secret' (via service) em vez de ler o arquivo direto
                    const apiKey = await trendingService.getApiKey();

                    if (apiKey && apiKey.trim() !== "") {
                        // Reutiliza o serviço de Trending
                        const trendingResult = await trendingService.getTrending(apiKey);
                        setTrending(trendingResult);
                    } else {
                        console.warn("API Key da RAWG não encontrada ou vazia.");
                    }
                } catch (e) {
                    console.warn("Não foi possível carregar o Trending para a Home:", e);
                }

            } catch (error) {
                console.error("Erro ao carregar Home:", error);
            } finally {
                setLoading(false);
            }
        }
        fetchData();
    }, []);

    // Lógica de negócio para a Home

    // 1. Stats
    const totalGames = library.length;
    const totalPlaytime = library.reduce((acc, g) => acc + g.playtime, 0);
    const totalFavorites = library.filter(g => g.favorite).length;

    // 2. Continue Jogando (Entre 1 minuto e 5 horas de jogo)
    const continuePlaying = library
        .filter(g => g.playtime > 0 && g.playtime < 300)
        .sort((a, b) => b.playtime - a.playtime)
        .slice(0, 5);

    // 3. Recomendado para Você
    let backlogRecommendations: Game[] = [];

    if (profile) {
        backlogRecommendations = library
            .filter(g => g.playtime === 0) // Nunca jogados
            .sort((a, b) => {
                // Converte string de generos para array antes de calcular
                const genresA = a.genre ? a.genre.split(',').map(name => ({ name: name.trim() })) : [];
                const genresB = b.genre ? b.genre.split(',').map(name => ({ name: name.trim() })) : [];

                return calculateAffinity(genresB) - calculateAffinity(genresA);
            })
            .slice(0, 8);
    }

    // 4. Mais Jogados
    const mostPlayed = [...library]
        .sort((a, b) => b.playtime - a.playtime)
        .slice(0, 3);

    // 5. Distribuição de Gêneros
    const genreStats = library.reduce((acc, game) => {
        if (game.genre) {
            const genres = game.genre.split(',').map(s => s.trim());
            genres.forEach(g => {
                if (g !== "Desconhecido") {
                    acc[g] = (acc[g] || 0) + 1;
                }
            });
        }
        return acc;
    }, {} as Record<string, number>);

    const topGenres = Object.entries(genreStats)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 5);

    return {
        library,
        trending,
        loading: loading || profileLoading,
        stats: { totalGames, totalPlaytime, totalFavorites },
        continuePlaying,
        backlogRecommendations,
        mostPlayed,
        topGenres,
        profile
    };
}