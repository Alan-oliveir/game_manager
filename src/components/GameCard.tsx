import { Game } from "../types";
import { Clock, Heart, Star, ImageOff } from "lucide-react";

interface GameCardProps {
  game: Game;
  onToggleFavorite: (id: string) => void;
  onClick: (game: Game) => void;
}

export default function GameCard({
  game,
  onToggleFavorite,
  onClick,
}: GameCardProps) {
  return (
    <div
      className="group relative bg-card rounded-xl overflow-hidden border border-border
                 hover:scale-[1.02] hover:shadow-2xl transition-all duration-300 cursor-pointer flex flex-col h-full"
      onClick={() => onClick(game)}
    >
      {/* Imagem da Capa */}
      <div className="relative aspect-[2/3] overflow-hidden bg-muted">
        {game.cover_url ? (
          <img
            src={game.cover_url}
            alt={game.name}
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
            loading="lazy"
          />
        ) : (
          /* Fallback Visual */
          <div className="w-full h-full bg-gradient-to-br from-sidebar-accent via-muted to-background flex flex-col items-center justify-center p-4 text-center">
            <ImageOff className="w-10 h-10 opacity-20 mb-3" />
            <span className="text-xs text-muted-foreground font-semibold uppercase tracking-widest">
              {game.name.slice(0, 20)}
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

        {/* Overlay de Status (Hover) */}
        <div
          className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/20 to-transparent
                        opacity-0 group-hover:opacity-100 transition-opacity duration-300
                        flex flex-col justify-end p-4"
        >
          <div className="space-y-1.5 text-white text-sm font-medium">
            {game.rating && (
              <div className="flex items-center gap-2">
                <Star size={16} className="text-yellow-400 fill-yellow-400" />
                <span>{game.rating}/5</span>
              </div>
            )}
            {game.playtime > 0 && (
              <div className="flex items-center gap-2">
                <Clock size={16} className="text-blue-300" />
                <span>{game.playtime}h jogadas</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Título e Tags */}
      <div className="p-4 flex flex-col gap-2 flex-1">
        <h3
          className="font-bold text-card-foreground text-base leading-tight line-clamp-1"
          title={game.name}
        >
          {game.name}
        </h3>

        <div className="flex gap-2 flex-wrap mt-auto pt-1">
          {game.genre && (
            <span className="px-2 py-1 bg-primary/10 text-primary text-xs rounded-md font-medium whitespace-nowrap">
              {game.genre}
            </span>
          )}
          {game.platform && (
            <span className="px-2 py-1 bg-secondary text-secondary-foreground text-xs rounded-md font-medium whitespace-nowrap">
              {game.platform}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
