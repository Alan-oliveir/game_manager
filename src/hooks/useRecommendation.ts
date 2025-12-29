import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { UserProfile } from "../types";

interface Genre {
    name: string;
}

export function useRecommendation() {
    const [profile, setProfile] = useState<UserProfile | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        async function loadProfile() {
            try {
                const data = await invoke<UserProfile>("get_user_profile");
                // Opcional: Log para ver o perfil no console e debugar
                console.log("Perfil de Usuário Carregado:", data);
                setProfile(data);
            } catch (error) {
                console.error("Falha ao carregar perfil de recomendação:", error);
            } finally {
                setLoading(false);
            }
        }
        loadProfile();
    }, []);

    /**
     * Calcula uma pontuação de afinidade para um jogo baseada nos gêneros dele.
     * Quanto maior o número, mais recomendado é o jogo.
     */
    const calculateAffinity = (gameGenres: Genre[]) => {
        if (!profile || !gameGenres) return 0;

        let totalScore = 0;

        gameGenres.forEach((g) => {
            // Busca se o gênero do jogo existe no perfil do usuário (case insensitive)
            const userGenre = profile.top_genres.find(
                (ug) => ug.name.toLowerCase() === g.name.toLowerCase()
            );

            if (userGenre) {
                totalScore += userGenre.score;
            }
        });

        return totalScore;
    };

    return { profile, loading, calculateAffinity };
}