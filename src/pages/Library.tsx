import GameGrid from "../components/GameGrid";
import { Game, GameActions } from "../types";
import { useMemo } from "react";

interface LibraryProps extends GameActions {
  games: Game[];
  searchTerm: string;
}

export default function Library({
  games,
  searchTerm,
  ...actions
}: LibraryProps) {
  const displayedGames = useMemo(() => {
    if (!searchTerm) return games;
    const term = searchTerm.toLowerCase();
    return games.filter(
      (game) =>
        game.name.toLowerCase().includes(term) ||
        (game.genre && game.genre.toLowerCase().includes(term)) ||
        (game.platform && game.platform.toLowerCase().includes(term))
    );
  }, [games, searchTerm]);

  return <GameGrid games={displayedGames} {...actions} />;
}
