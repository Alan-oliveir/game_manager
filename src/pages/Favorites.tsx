import GameGrid from "../components/GameGrid";
import { Game, GameActions } from "../types";
import { Heart } from "lucide-react";
import { useMemo } from "react";

interface FavoritesProps extends GameActions {
  games: Game[];
  searchTerm: string;
}

export default function Favorites({
  games,
  searchTerm,
  ...actions
}: FavoritesProps) {
  const displayedGames = useMemo(() => {
    const favorites = games.filter((g) => g.favorite);

    if (!searchTerm) return favorites;

    const term = searchTerm.toLowerCase();
    return favorites.filter(
      (game) =>
        game.name.toLowerCase().includes(term) ||
        (game.genre && game.genre.toLowerCase().includes(term)) ||
        (game.platform && game.platform.toLowerCase().includes(term))
    );
  }, [games, searchTerm]);

  if (
    displayedGames.length === 0 &&
    games.filter((g) => g.favorite).length === 0
  ) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground">
        <Heart className="w-16 h-16 mb-4 opacity-20" />
        <p>Você ainda não tem favoritos.</p>
      </div>
    );
  }

  return (
    <GameGrid
      games={displayedGames}
      title="Meus Favoritos"
      subtitle={`${displayedGames.length} jogo${
        displayedGames.length === 1 ? "" : "s"
      } amado${displayedGames.length === 1 ? "" : "s"}`}
      {...actions}
    />
  );
}
