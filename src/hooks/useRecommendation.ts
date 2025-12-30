import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { UserProfile } from "../types";

interface Genre {
    name: string;
}

interface UseRecommendationProps {
    profileCache?: UserProfile | null;
    setProfileCache?: (profile: UserProfile) => void;
}

export function useRecommendation({ profileCache, setProfileCache }: UseRecommendationProps = {}) {
    // Inicia com o cache se existir
    const [profile, setProfile] = useState<UserProfile | null>(profileCache || null);
    const [loading, setLoading] = useState(!profileCache);

    useEffect(() => {
        if (profileCache) {
            setLoading(false);
            return;
        }

        async function loadProfile() {
            setLoading(true);
            try {
                const data = await invoke<UserProfile>("get_user_profile");
                setProfile(data);

                if (setProfileCache) {
                    setProfileCache(data);
                }
            } catch (error) {
                console.error("Falha ao carregar perfil:", error);
            } finally {
                setLoading(false);
            }
        }
        loadProfile();
    }, [profileCache]);

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