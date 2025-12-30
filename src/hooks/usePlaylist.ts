import { useState, useEffect } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { Game } from "../types";

const STORE_KEY = "user_playlist_queue";

export function usePlaylist(allGames: Game[]) {
    const [queueIds, setQueueIds] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    // Carregar fila salva
    useEffect(() => {
        async function loadQueue() {
            try {
                const store = await Store.load(".settings.dat");
                const saved = await store.get<string[]>(STORE_KEY);
                if (saved) {
                    setQueueIds(saved);
                }
            } catch (e) {
                console.error("Erro ao carregar playlist:", e);
            } finally {
                setIsLoading(false);
            }
        }
        loadQueue();
    }, []);

    // Salvar sempre que mudar
    const saveQueue = async (newQueue: string[]) => {
        setQueueIds(newQueue);
        try {
            const store = await Store.load(".settings.dat");
            await store.set(STORE_KEY, newQueue);
            await store.save();
        } catch (e) {
            console.error("Erro ao salvar playlist:", e);
        }
    };

    // Ações da fila

    const addToPlaylist = (gameId: string) => {
        if (!queueIds.includes(gameId)) {
            saveQueue([...queueIds, gameId]);
        }
    };

    const removeFromPlaylist = (gameId: string) => {
        saveQueue(queueIds.filter((id) => id !== gameId));
    };

    const moveUp = (index: number) => {
        if (index === 0) return;
        const newQueue = [...queueIds];
        [newQueue[index - 1], newQueue[index]] = [newQueue[index], newQueue[index - 1]];
        saveQueue(newQueue);
    };

    const moveDown = (index: number) => {
        if (index === queueIds.length - 1) return;
        const newQueue = [...queueIds];
        [newQueue[index + 1], newQueue[index]] = [newQueue[index], newQueue[index + 1]];
        saveQueue(newQueue);
    };

    // Hidrata os IDs transformando em objetos Game completos
    // Mantém a ordem da fila!
    const playlistGames = queueIds
        .map((id) => allGames.find((g) => g.id === id))
        .filter((g): g is Game => !!g); // Remove undefined se algum jogo foi deletado da biblioteca

    return {
        playlistGames,
        isLoading,
        addToPlaylist,
        removeFromPlaylist,
        moveUp,
        moveDown,
        isInPlaylist: (id: string) => queueIds.includes(id),
    };
}