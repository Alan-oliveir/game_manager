import { useEffect, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { Game } from "../types";

const STORE_KEY = "user_playlist_queue";

export function usePlaylist(allGames: Game[]) {
  const [queueIds, setQueueIds] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

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
    [newQueue[index - 1], newQueue[index]] = [
      newQueue[index],
      newQueue[index - 1],
    ];
    saveQueue(newQueue);
  };

  const moveDown = (index: number) => {
    if (index === queueIds.length - 1) return;
    const newQueue = [...queueIds];
    [newQueue[index + 1], newQueue[index]] = [
      newQueue[index],
      newQueue[index + 1],
    ];
    saveQueue(newQueue);
  };

  const reorderPlaylist = (startIndex: number, endIndex: number) => {
    const newQueue = Array.from(queueIds);
    const [removed] = newQueue.splice(startIndex, 1);
    newQueue.splice(endIndex, 0, removed);
    saveQueue(newQueue);
  };

  const playlistGames = queueIds
    .map((id) => allGames.find((g) => g.id === id))
    .filter((g): g is Game => !!g);

  return {
    playlistGames,
    isLoading,
    addToPlaylist,
    removeFromPlaylist,
    moveUp,
    moveDown,
    reorderPlaylist,
    isInPlaylist: (id: string) => queueIds.includes(id),
  };
}
