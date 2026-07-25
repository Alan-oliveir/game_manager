import { ImageOff, Play } from 'lucide-react';
import { memo, ReactNode, useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ActionButton, CachedImage } from '@/components';
import { cn } from '@/lib/utils';

interface StandardGameCardProps {
  id: string;
  title: string;
  coverUrl?: string | null;
  subtitle?: string;
  platform?: string;
  badge?: ReactNode;
  rating?: number;
  onClick?: () => void;
  actions?: ReactNode;
  className?: string;
  onPlay?: () => void;
}

/**
 * Card padrão para exibição de jogos em grades (Início, Biblioteca, Favoritos, Em Alta e Lista de Desejos).
 * Aspect ratio fixo 3:4 (capa vertical de jogo).
 *
 * Features:
 * - Fallback automatico se imagem falhar: exibe gradiente + nome do jogo em maiusculas
 * - Overlay de acoes aparece no hover (botao Play + acoes customizadas)
 * - Suporta badge no canto superior esquerdo (ex: "Favorito", "Novo", "Oferta")
 * - Efeito hover: elevacao + zoom suave na imagem
 *
 * Memoizado: com centenas de cards na grade, o React.memo evita re-renderizar
 * um card quando o pai re-renderiza mas as props deste card específico não
 * mudaram.
 */
function StandardGameCard({
  id,
  title,
  coverUrl,
  subtitle,
  platform,
  badge,
  rating,
  onClick,
  actions,
  className,
  onPlay,
}: Readonly<StandardGameCardProps>) {
  const [imageError, setImageError] = useState(false);
  const { t } = useTranslation('library');

  const handleImageError = useCallback(() => setImageError(true), []);

  return (
    <div
      className={cn(
        'group bg-card border-border relative flex h-full flex-col overflow-hidden rounded-xl border transition-all hover:-translate-y-1 hover:shadow-lg',
        className
      )}
    >
      {onClick && (
        <button
          type="button"
          className="absolute inset-0 z-20 cursor-pointer"
          aria-label={t('game_card_open_details_aria_label', { title })}
          onClick={onClick}
        />
      )}

      <div className="bg-muted relative aspect-3/4 overflow-hidden">
        {coverUrl && !imageError ? (
          <CachedImage
            src={coverUrl}
            gameId={id}
            alt={title}
            className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
            onError={handleImageError}
          />
        ) : (
          <div className="from-secondary/50 via-muted to-background flex h-full w-full flex-col items-center justify-center bg-linear-to-br p-4 text-center">
            <ImageOff className="mb-3 h-10 w-10 opacity-20" />
            <span className="text-muted-foreground line-clamp-2 text-[10px] font-semibold tracking-widest uppercase">
              {title}
            </span>
          </div>
        )}

        {platform && (
          <div className="absolute top-2 right-2 z-10 flex items-center gap-1 rounded bg-black/60 px-1.5 py-0.5 text-[10px] font-medium text-white backdrop-blur-md">
            {platform}
          </div>
        )}

        <div className="pointer-events-none absolute inset-0 z-30 flex items-center justify-center gap-2 bg-black/60 p-4 opacity-0 transition-opacity group-hover:opacity-100">
          {onPlay && (
            <div className="pointer-events-auto">
              <ActionButton
                icon={Play}
                variant="glass"
                onClick={onPlay}
                tooltip={t('game_card_play_now_tooltip')}
              />
            </div>
          )}

          {actions && (
            <div className="pointer-events-auto flex items-center justify-center gap-2">
              {actions}
            </div>
          )}
        </div>

        {badge && (
          <div className="absolute top-2 left-2 z-40 rounded-full border border-purple-400/30 bg-purple-600 px-2 py-0.5 text-[10px] font-bold text-white shadow-lg backdrop-blur-md">
            {badge}
          </div>
        )}
      </div>

      <div className="flex flex-1 flex-col gap-1 p-3">
        <h3 className="line-clamp-1 text-sm font-semibold" title={title}>
          {title}
        </h3>
        <div className="mt-auto flex items-center justify-between">
          <span className="text-muted-foreground max-w-30 truncate text-xs">
            {subtitle}
          </span>
          {rating && (
            <span className="flex items-center gap-1 rounded bg-yellow-500/10 px-1.5 py-0.5 text-[10px] font-bold text-yellow-500">
              {'\u2605'} {rating}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export default memo(StandardGameCard);
