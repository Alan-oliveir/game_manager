import GameGrid from "../components/GameGrid";
import { Game } from "../types";
import { Heart } from "lucide-react";

interface FavoritesProps {
    games: Game[];
    searchTerm: string;
    onToggleFavorite: (id: string) => void;
    onGameClick: (game: Game) => void;
    onDeleteGame: (id: string) => void;
    onEditGame: (game: Game) => void;
}

export default function Favorites({ games, searchTerm, ...actions }: FavoritesProps) {
    const favorites = games.filter(g => g.favorite);

    const displayedGames = favorites.filter((game) => {
        if (!searchTerm) return true;
        const term = searchTerm.toLowerCase();
        return game.name.toLowerCase().includes(term);
    });

    if (favorites.length === 0) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground">
                <Heart className="w-16 h-16 mb-4 opacity-20" />
                <p>Você ainda não tem favoritos.</p>
            </div>
        )
    }

    return <GameGrid games={displayedGames} {...actions} />;
}