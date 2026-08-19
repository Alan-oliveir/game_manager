import {
  ArrowDown,
  ArrowUp,
  GripVertical,
  ImageOff,
  Play,
  Trash2,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Game } from '@/types';
import { Button } from '@/ui/button';

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

/**
 * Item individual da fila de jogos (Playlist).
 * Suporta drag & drop e reordenação manual via setas.
 *
 * Layout horizontal compacto:
 * - Grip icon (indicador de drag)
 * - Setas de reordenação + número da posição
 * - Miniatura da capa com overlay de Play
 * - Info do jogo (nome, gênero, tempo jogado)
 * - Botão Play principal + botão Remover
 *
 * Cursor muda para 'grab' no hover e 'grabbing' ao arrastar.
 * Botões de seta desabilitam automaticamente nos extremos (primeiro/último).
 *
 * @param game - Dados completos do jogo
 * @param index - Posição atual na fila (0-based)
 * @param total - Total de itens na fila (para desabilitar seta baixo no último)
 * @param onMoveUp - Callback para mover uma posição acima
 * @param onMoveDown - Callback para mover uma posição abaixo
 * @param onRemove - Callback para remover da fila
 * @param onPlay - Callback do overlay da capa (lança o jogo)
 * @param onClick - Callback ao clicar no card (abre detalhes)
 */
export function PlaylistCard({
  game,
  index,
  total,
  onMoveUp,
  onMoveDown,
  onRemove,
  onPlay,
  onClick,
}: Readonly<PlaylistItemProps>) {
  const { t } = useTranslation('playlist');

  return (
    <div
      className="group bg-card border-border hover:border-primary/50 animate-in fade-in slide-in-from-left-4 flex cursor-grab items-center gap-3 rounded-xl border p-2.5 transition-all duration-300 hover:shadow-md active:cursor-grabbing lg:gap-4 lg:p-3"
      onClick={onClick}
    >
      {/* Bloco de controle (Grip + Setas) */}
      <div className="mr-1 flex items-center gap-0">
        <GripVertical
          size={18}
          className="text-muted-foreground/40 lg:h-5 lg:w-5"
        />

        {/* Botões de mover na fila */}
        <div className="text-muted-foreground flex flex-col items-center justify-center gap-0.5">
          <Button
            variant="ghost"
            size="icon"
            className="hover:text-primary h-6 w-6 rounded-full disabled:opacity-20"
            onClick={e => {
              e.stopPropagation();
              e.preventDefault();
              onMoveUp();
            }}
            onMouseDown={e => e.stopPropagation()}
            onTouchStart={e => e.stopPropagation()}
            disabled={index === 0}
            title={t('playlist_card_move_up_title')}
          >
            <ArrowUp size={13} className="lg:h-3.5 lg:w-3.5" />
          </Button>

          <span className="w-6 text-center font-mono text-[11px] font-bold select-none lg:text-xs">
            {index + 1}
          </span>

          <Button
            variant="ghost"
            size="icon"
            className="hover:text-primary h-6 w-6 rounded-full disabled:opacity-20"
            onClick={e => {
              e.stopPropagation();
              e.preventDefault();
              onMoveDown();
            }}
            onMouseDown={e => e.stopPropagation()}
            onTouchStart={e => e.stopPropagation()}
            disabled={index === total - 1}
            title={t('playlist_card_move_down_title')}
          >
            <ArrowDown size={13} className="lg:h-3.5 lg:w-3.5" />
          </Button>
        </div>
      </div>

      {/* Cover do jogo */}
      <div className="bg-muted group/img relative h-14 w-10 shrink-0 cursor-pointer overflow-hidden rounded shadow-sm lg:h-16 lg:w-12">
        {game.coverUrl ? (
          <img
            src={game.coverUrl}
            alt=""
            className="h-full w-full object-cover"
            draggable={false}
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center text-[8px]">
            <ImageOff className="h-4 w-4 opacity-50" />
          </div>
        )}
        <div
          className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover/img:opacity-100"
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onPlay();
          }}
          onMouseDown={e => e.stopPropagation()}
          onTouchStart={e => e.stopPropagation()}
        >
          <Play size={15} className="fill-white text-white lg:h-4 lg:w-4" />
        </div>
      </div>

      {/* Info Principal */}
      <div className="min-w-0 flex-1 cursor-pointer select-none">
        <h4 className="group-hover:text-primary truncate text-sm font-semibold transition-colors">
          {game.name}
        </h4>
        <div className="text-muted-foreground mt-1 flex items-center gap-2 text-xs">
          <span className="bg-secondary/50 text-secondary-foreground border-border/50 max-w-25 truncate rounded border px-1.5 py-0.5 lg:max-w-none">
            {game.genres?.[0] || t('playlist_card_default_genre')}
          </span>
          <span>•</span>
          <span>
            {(game.playtime ?? 0) > 0
              ? t('playlist_card_playtime_text', {
                  hours: Math.floor((game.playtime ?? 0) / 60),
                })
              : t('playlist_card_playtime_text', { hours: 0 })}
          </span>
        </div>
      </div>

      {/* Botões de Ação */}
      <div className="flex items-center gap-1.5 lg:gap-2">
        <Button
          size="sm"
          className="bg-primary/90 hover:bg-primary text-primary-foreground flex h-7 items-center gap-1 px-2 shadow-sm"
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onPlay();
          }}
          onMouseDown={e => e.stopPropagation()}
          onTouchStart={e => e.stopPropagation()}
          title={t('playlist_card_play_now_title')}
        >
          <Play size={11} className="fill-current lg:h-3 lg:w-3" />
          <span className="text-xs font-bold">
            {t('playlist_card_play_button_label')}
          </span>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 h-8 w-8"
          onClick={e => {
            e.stopPropagation();
            e.preventDefault();
            onRemove();
          }}
          onMouseDown={e => e.stopPropagation()}
          onTouchStart={e => e.stopPropagation()}
          title={t('playlist_card_remove_title')}
        >
          <Trash2 size={15} className="lg:h-4 lg:w-4" />
        </Button>
      </div>
    </div>
  );
}
