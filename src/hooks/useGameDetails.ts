import { useState, useEffect } from "react";
import { Game, GameDetails, GamePlatformLink } from "../types";
import { detailsService } from "../services/detailsService";

export function useGameDetails(selectedGame: Game | null, allGames: Game[]) {
  const [details, setDetails] = useState<GameDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [siblings, setSiblings] = useState<GamePlatformLink[]>([]);

  useEffect(() => {
    if (!selectedGame) {
      setDetails(null);
      return;
    }

    // Identifica o mesmo jogo em outras plataformas (pelo nome)
    const related = allGames
      .filter(
        (g) =>
          g.name.toLowerCase() === selectedGame.name.toLowerCase() &&
          g.id !== selectedGame.id // Exclui o próprio jogo atual
      )
      .map((g) => ({ id: g.id, platform: g.platform || "Outra" }));

    setSiblings(related);

    // Busca dados na nuvem
    const fetchRemote = async () => {
      setLoading(true);
      try {
        const data = await detailsService.getGameDetails(selectedGame.name);
        setDetails(data);
      } catch (err) {
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    fetchRemote();
  }, [selectedGame, allGames]);

  return { details, loading, siblings };
}
