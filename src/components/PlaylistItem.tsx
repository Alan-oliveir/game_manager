import {
  ArrowDown,
  ArrowUp,
  GripVertical,
  ImageOff,
  Play,
  Trash2,
} from "lucide-react";
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
      className="group flex items-center gap-4 p-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-md animate-in fade-in slide-in-from-left-4 duration-300 cursor-grab active:cursor-grabbing"
      onClick={onClick}
    >
      {/* BLOCO DE CONTROLE (Grip + Setas) */}
      <div className="flex items-center gap-0 mr-1">
        <GripVertical size={20} className="text-muted-foreground/40" />

        {/* Coluna de Setas */}
        <div className="flex flex-col items-center justify-center gap-0.5 text-muted-foreground">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 rounded-full hover:text-primary disabled:opacity-20"
            onClick={(e) => {
              e.stopPropagation();
              onMoveUp();
            }}
            disabled={index === 0}
            title="Mover para cima"
          >
            <ArrowUp size={14} />
          </Button>

          <span className="text-[12px] font-mono font-bold w-6 text-center select-none">
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
            title="Mover para baixo"
          >
            <ArrowDown size={14} />
          </Button>
        </div>
      </div>

      {/* Capa Pequena */}
      <div className="relative h-16 w-12 shrink-0 rounded overflow-hidden bg-muted cursor-pointer group/img shadow-sm">
        {game.cover_url ? (
          <img
            src={game.cover_url}
            alt=""
            className="h-full w-full object-cover"
            draggable={false}
          />
        ) : (
          <div className="h-full w-full flex items-center justify-center text-[8px]">
            <ImageOff className="opacity-50" />
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

      {/* Info Principal */}
      <div className="flex-1 min-w-0 cursor-pointer select-none">
        <h4 className="font-semibold truncate group-hover:text-primary transition-colors text-sm">
          {game.name}
        </h4>
        <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
          <span className="bg-secondary/50 px-1.5 py-0.5 rounded text-secondary-foreground border border-border/50">
            {game.genre || "Geral"}
          </span>
          <span>•</span>
          <span>
            {game.playtime > 0 ? `${Math.floor(game.playtime / 60)}h` : "0h"}
          </span>
        </div>
      </div>

      {/* 4. Botões de Ação */}
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          className="h-7 px-2 flex items-center gap-1 bg-primary/90 hover:bg-primary text-primary-foreground shadow-sm"
          onClick={(e) => {
            e.stopPropagation();
            launchGame(game);
          }}
          title="Jogar Agora"
        >
          <Play size={12} className="fill-current" />{" "}
          <span className="text-xs font-bold">Play</span>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
          onClick={(e) => {
            e.stopPropagation();
            onRemove();
          }}
          title="Remover da fila"
        >
          <Trash2 size={16} />
        </Button>
      </div>
    </div>
  );
}
