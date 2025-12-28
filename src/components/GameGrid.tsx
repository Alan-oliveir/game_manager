import { Game } from "../types";
import { Library } from "lucide-react";
import GameCard from "./GameCard";

interface GameGridProps {
  games: Game[];
  title?: string;
  subtitle?: string;
  onToggleFavorite: (id: string) => void;
  onGameClick: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  onEditGame: (game: Game) => void;
}

export default function GameGrid({
  games,
  title = "Minha Biblioteca",
  subtitle,
  onToggleFavorite,
  onGameClick,
  onDeleteGame,
  onEditGame,
}: GameGridProps) {
  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-foreground">{title}</h2>
        <p className="text-sm text-muted-foreground">
          {subtitle ||
            `${games.length} ${
              games.length === 1 ? "jogo" : "jogos"
            } encontrado${games.length === 1 ? "" : "s"}`}
        </p>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6">
        {games.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            onToggleFavorite={onToggleFavorite}
            onClick={onGameClick}
            onDelete={onDeleteGame}
            onEdit={onEditGame}
          />
        ))}
      </div>

      {/* Estado Vazio */}
      {games.length === 0 && (
        <div className="flex flex-col items-center justify-center py-20 text-center opacity-60">
          <div className="w-24 h-24 bg-muted rounded-full flex items-center justify-center mb-4">
            <Library size={48} className="text-muted-foreground" />
          </div>
          <h3 className="text-lg font-medium">Nenhum jogo encontrado</h3>
          <p className="text-sm text-muted-foreground max-w-xs mx-auto mt-2">
            Tente buscar por outro termo ou adicione um novo jogo à sua coleção.
          </p>
        </div>
      )}
    </div>
  );
}
