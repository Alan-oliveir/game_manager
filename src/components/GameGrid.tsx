import { Game } from "../types";
import { Library } from "lucide-react";
import GameCard from "./GameCard";

interface GameGridProps {
  games: Game[];
  onToggleFavorite: (id: string) => void;
  onGameClick: (game: Game) => void;
}

export default function GameGrid({
  games,
  onToggleFavorite,
  onGameClick,
}: GameGridProps) {
  // Mantendo seu Mock Data
  const mockGames: Game[] =
    games.length === 0
      ? [
          {
            id: "1",
            name: "God of War Ragnarök",
            genre: "Ação",
            platform: "PC",
            cover_url:
              "https://images.unsplash.com/photo-1552820728-8b83bb6b773f?w=400&h=600&fit=crop",
            playtime: 45,
            rating: 5,
            favorite: true,
          },
          {
            id: "2",
            name: "Elden Ring",
            genre: "RPG",
            platform: "PC",
            cover_url:
              "https://images.unsplash.com/photo-1538481199705-c710c4e965fc?w=400&h=600&fit=crop",
            playtime: 120,
            rating: 5,
            favorite: false,
          },
          {
            id: "3",
            name: "Cyberpunk 2077",
            genre: "RPG",
            platform: "PC",
            cover_url:
              "https://images.unsplash.com/photo-1542751371-adc38448a05e?w=400&h=600&fit=crop",
            playtime: 80,
            rating: 4,
            favorite: true,
          },
          {
            id: "4",
            name: "Hades II",
            genre: "Roguelike",
            platform: "Steam",
            playtime: 12,
            rating: 5,
            favorite: false,
          },
          {
            id: "5",
            name: "Stardew Valley",
            genre: "Simulação",
            platform: "Steam",
            playtime: 200,
            rating: 5,
            favorite: true,
          },
          {
            id: "6",
            name: "Baldur's Gate 3",
            genre: "RPG",
            platform: "GOG",
            playtime: 150,
            rating: 5,
            favorite: false,
          },
        ]
      : games;

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-foreground">Minha Biblioteca</h2>
        <p className="text-sm text-muted-foreground">
          {mockGames.length} jogos na coleção
        </p>
      </div>

      {/* GRID AJUSTADO:
         - grid-cols-1: Mobile (começa com 1 card grande)
         - sm:grid-cols-2: Tablets pequenos
         - lg:grid-cols-3: Laptop pequeno
         - xl:grid-cols-4: Laptop padrão
         - 2xl:grid-cols-5: Monitores grandes/Full HD (Seu objetivo)
         - gap-6: Mais espaço entre os cards
      */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6">
        {mockGames.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            onToggleFavorite={onToggleFavorite}
            onClick={onGameClick}
          />
        ))}
      </div>

      {games.length === 0 && mockGames === games && (
        <div className="flex flex-col items-center justify-center py-20 text-center opacity-60">
          <Library size={64} className="mb-4 text-muted-foreground" />
          <h3 className="text-lg font-medium">Sua biblioteca está vazia</h3>
        </div>
      )}
    </div>
  );
}
