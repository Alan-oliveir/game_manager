import GameGrid from "../components/GameGrid";
import {Game} from "../types";

interface LibraryProps {
    games: Game[];
    searchTerm: string;
    onToggleFavorite: (id: string) => void;
    onGameClick: (game: Game) => void;
    onDeleteGame: (id: string) => void;
    onEditGame: (game: Game) => void;
}

export default function Library({games, searchTerm, ...actions}: LibraryProps) {
    const displayedGames = games.filter((game) => {
        if (!searchTerm) return true;
        const term = searchTerm.toLowerCase();
        return (
            game.name.toLowerCase().includes(term) ||
            (game.genre && game.genre.toLowerCase().includes(term))
        );
    });

    return (
        <GameGrid games={displayedGames} {...actions} />
    );
}