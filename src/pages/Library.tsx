import GameGrid from "../components/GameGrid";
import {Game} from "../types";
import { useMemo } from "react";

interface LibraryProps {
    games: Game[];
    searchTerm: string;
    onToggleFavorite: (id: string) => void;
    onGameClick: (game: Game) => void;
    onDeleteGame: (id: string) => void;
    onEditGame: (game: Game) => void;
}

export default function Library({games, searchTerm, ...actions}: LibraryProps) {
    const displayedGames = useMemo(() => {
        if (!searchTerm) return games;
        const term = searchTerm.toLowerCase();
        return games.filter((game) =>
            game.name.toLowerCase().includes(term) ||
            (game.genre && game.genre.toLowerCase().includes(term)) ||
            (game.platform && game.platform.toLowerCase().includes(term))
        );
    }, [games, searchTerm]);

    return (
        <GameGrid games={displayedGames} {...actions} />
    );
}