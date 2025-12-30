import { ArrowDown, ArrowUp, Play, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Game } from "../types";
import { launchGame } from "@/utils/launcher.ts";

interface PlaylistItemProps {
  game: Game;
  index: number;
  total: number;
  onMoveUp: () => void;
  onMoveDown: () => void;
  onRemove: () => void;
  onPlay: () => void;
  onClick: () => void;
}

export default function PlaylistItem({
  game,
  index,
  total,
  onMoveUp,
  onMoveDown,
  onRemove,
  onPlay,
  onClick,
}: PlaylistItemProps) {
  return (
    <div
      className="group flex items-center gap-4 p-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-md animate-in fade-in slide-in-from-left-4 duration-300"
      onClick={onClick}
    >
      {/* 1. Ordem / Controle */}
      <div className="flex flex-col items-center gap-1 text-muted-foreground">
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 rounded-full hover:text-primary disabled:opacity-20"
          onClick={(e) => {
            e.stopPropagation();
            onMoveUp();
          }}
          disabled={index === 0}
        >
          <ArrowUp size={14} />
        </Button>
        <span className="text-xs font-mono font-bold w-6 text-center">
          {index + 1}
        </span>
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 rounded-full hover:text-primary disabled:opacity-20"
          onClick={(e) => {
            e.stopPropagation();
            onMoveDown();
          }}
          disabled={index === total - 1}
        >
          <ArrowDown size={14} />
        </Button>
      </div>

      {/* 2. Capa Pequena */}
      <div className="relative h-16 w-12 shrink-0 rounded overflow-hidden bg-muted cursor-pointer group/img">
        {game.cover_url ? (
          <img
            src={game.cover_url}
            alt=""
            className="h-full w-full object-cover"
          />
        ) : (
          <div className="h-full w-full flex items-center justify-center text-[8px]">
            Sem Capa
          </div>
        )}
        {/* Overlay Play Mini */}
        <div
          className="absolute inset-0 bg-black/50 opacity-0 group-hover/img:opacity-100 flex items-center justify-center transition-opacity"
          onClick={(e) => {
            e.stopPropagation();
            onPlay();
          }}
        >
          <Play size={16} className="fill-white text-white" />
        </div>
      </div>

      {/* 3. Info Principal */}
      <div className="flex-1 min-w-0 cursor-pointer">
        <h4 className="font-semibold truncate group-hover:text-primary transition-colors">
          {game.name}
        </h4>
        <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
          <span className="bg-secondary px-1.5 py-0.5 rounded text-secondary-foreground">
            {game.genre || "Geral"}
          </span>
          <span>•</span>
          <span>
            {game.playtime > 0
              ? `${Math.floor(game.playtime / 60)}h jogadas`
              : "Nunca jogado"}
          </span>
        </div>
      </div>

      {/* 4. Botões de Ação */}
      <Button
        size="sm"
        className="flex items-center gap-1 bg-accent text-white hover:bg-muted-foreground"
        onClick={(e) => {
          e.stopPropagation();
          launchGame(game);
        }}
        title="Jogar Agora"
      >
        <Play size={14} /> <span className="text-xs font-bold">Play</span>
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 opacity-0 group-hover:opacity-100 transition-opacity"
        onClick={(e) => {
          e.stopPropagation();
          onRemove();
        }}
        title="Remover da fila"
      >
        <Trash2 size={16} />
      </Button>
    </div>
  );
}
