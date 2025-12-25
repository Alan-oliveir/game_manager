import { Clock, Heart, Star } from "lucide-react";
import { Game } from "../types";

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
  // Mock data para exemplo visual
  const mockGames: Game[] =
    games.length === 0
      ? [
          {
            id: "1",
            name: "God of War Ragnarök",
            genre: "Ação/Aventura",
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
        ]
      : games;

  return (
    <div className="flex-1 overflow-y-auto p-6">
      {/* Section Title */}
      <div className="mb-6">
        <h2 className="text-3xl font-bold text-foreground mb-2">
          Minha Biblioteca
        </h2>
        <p className="text-muted-foreground">
          {mockGames.length} {mockGames.length === 1 ? "jogo" : "jogos"} na sua
          coleção
        </p>
      </div>

      {/* Games Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6">
        {mockGames.map((game) => (
          <div
            key={game.id}
            className="group relative bg-card rounded-lg overflow-hidden border border-border
                     hover:scale-105 hover:shadow-2xl transition-all duration-300 cursor-pointer"
            onClick={() => onGameClick(game)}
          >
            {/* Cover Image */}
            <div className="relative aspect-[2/3] overflow-hidden bg-muted">
              {game.cover_url ? (
                <img
                  src={game.cover_url}
                  alt={game.name}
                  className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                />
              ) : (
                /* Fallback para quando não tem capa: Um gradiente sutil */
                <div className="w-full h-full bg-gradient-to-br from-sidebar-accent via-muted to-background flex flex-col items-center justify-center p-4 text-center">
                  <span className="text-4xl font-bold opacity-20 mb-2">
                    {game.name.charAt(0).toUpperCase()}
                  </span>
                  <span className="text-xs text-muted-foreground font-medium uppercase tracking-wider">
                    Sem Capa
                  </span>
                </div>
              )}

              {/* Favorite Button */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleFavorite(game.id);
                }}
                className="absolute top-2 right-2 p-2 bg-black/60 backdrop-blur-sm rounded-full
                         hover:bg-black/80 transition-colors z-10"
              >
                <Heart
                  size={20}
                  className={
                    game.favorite ? "fill-red-500 text-red-500" : "text-white"
                  }
                />
              </button>

              {/* Overlay on Hover */}
              <div
                className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent
                            opacity-0 group-hover:opacity-100 transition-opacity duration-300
                            flex flex-col justify-end p-4"
              >
                {/* Stats */}
                <div className="space-y-2 text-white text-sm">
                  {game.rating && (
                    <div className="flex items-center gap-1">
                      <Star
                        size={16}
                        className="fill-yellow-400 text-yellow-400"
                      />
                      <span>{game.rating}/5</span>
                    </div>
                  )}
                  {game.playtime > 0 && (
                    <div className="flex items-center gap-1">
                      <Clock size={16} />
                      <span>{game.playtime}h jogadas</span>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Game Info */}
            <div className="p-4">
              <h3 className="font-semibold text-card-foreground text-lg mb-2 line-clamp-1">
                {game.name}
              </h3>
              <div className="flex gap-2 flex-wrap">
                {game.genre && (
                  <span className="px-2 py-1 bg-primary/10 text-primary text-xs rounded-md font-medium">
                    {game.genre}
                  </span>
                )}
                {game.platform && (
                  <span className="px-2 py-1 bg-secondary/10 text-secondary text-xs rounded-md font-medium">
                    {game.platform}
                  </span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Empty State */}
      {games.length === 0 && (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="w-24 h-24 bg-muted rounded-full flex items-center justify-center mb-4">
            <Library size={48} className="text-muted-foreground" />
          </div>
          <h3 className="text-xl font-semibold text-foreground mb-2">
            Nenhum jogo na biblioteca
          </h3>
          <p className="text-muted-foreground mb-6">
            Adicione seu primeiro jogo para começar sua coleção!
          </p>
        </div>
      )}
    </div>
  );
}

function Library({ size, className }: { size: number; className?: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="m16 6 4 14" />
      <path d="M12 6v14" />
      <path d="M8 8v12" />
      <path d="M4 4v16" />
    </svg>
  );
}
