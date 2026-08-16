import { ImageOff, Pencil, Star, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Game, LibraryDisplayNames } from '@/types';
import { Badge } from '@/ui/badge';

interface GameHeaderProps {
  game: Game;
  onClose: () => void;
  onEditToggle: () => void;
  isEditing: boolean;
}

export function GameHeader({
  game,
  onClose,
  onEditToggle,
  isEditing,
}: GameHeaderProps) {
  const { t } = useTranslation('game_detail');

  return (
    <div className="bg-muted relative h-44 w-full shrink-0 overflow-hidden">
      {/* --- GRUPO DE AÇÕES (Topo Direito) --- */}
      <div className="absolute top-4 right-4 z-50 flex items-center gap-2">
        <button
          onClick={onEditToggle}
          className={`rounded-full p-2 backdrop-blur-md transition-all ${
            isEditing
              ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
              : 'bg-black/40 text-white hover:bg-black/70'
          }`}
          title={isEditing ? t('header_cancel_edit') : t('header_edit_details')}
        >
          <Pencil size={18} />
        </button>

        <button
          onClick={onClose}
          className="rounded-full bg-black/40 p-2 text-white backdrop-blur-md transition-all hover:bg-black/70"
          title={t('header_close')}
        >
          <X size={18} />
        </button>
      </div>

      {/* Imagem de Fundo (Blur) */}
      {game.coverUrl && (
        <div
          className="absolute inset-0 scale-105 bg-cover bg-center opacity-60 blur-3xl"
          style={{ backgroundImage: `url(${game.coverUrl})` }}
        />
      )}

      {/* Gradiente de Leitura */}
      <div className="absolute inset-0 bg-linear-to-b from-black/30 via-transparent to-black/90" />

      {/* --- CONTEÚDO (Alinhado embaixo) --- */}
      <div className="absolute bottom-0 left-0 z-10 flex w-full items-center gap-4 p-6">
        {/* CAPA */}
        {game.coverUrl ? (
          <img
            src={game.coverUrl}
            alt=""
            className="bg-muted h-32 w-24 shrink-0 rounded-lg object-cover shadow-2xl"
          />
        ) : (
          <div className="bg-muted flex h-32 w-24 shrink-0 items-center justify-center rounded-lg shadow-2xl">
            <ImageOff className="h-10 w-10 opacity-50" />
          </div>
        )}

        {/* COLUNA DE TEXTO */}
        <div className="flex min-h-32 flex-1 flex-col justify-center gap-3">
          {/* TÍTULO */}
          <h1 className="line-clamp-2 text-5xl leading-tight font-black tracking-tight text-white drop-shadow-xl">
            {game.name}
          </h1>

          {/* BADGES */}
          <div className="flex items-center gap-3">
            <Badge className="border-white/10 bg-white/10 px-3 py-1 text-sm font-semibold text-white backdrop-blur-md hover:bg-white/20">
              {LibraryDisplayNames[game.platform] || 'PC'}
            </Badge>

            {game.installed && (
              <Badge className="border-green-500/30 bg-green-500/20 px-3 py-1 text-sm font-semibold text-green-300 backdrop-blur-md">
                {t('header_installed')}
              </Badge>
            )}

            {game.userRating && (
              <div className="flex items-center gap-1.5 rounded-full border border-yellow-500/30 bg-black/50 px-3 py-1 text-sm font-bold text-yellow-400 backdrop-blur-md">
                <Star size={12} className="fill-yellow-400 text-yellow-400" />
                {game.userRating}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
