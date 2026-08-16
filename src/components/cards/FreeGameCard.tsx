import { Calendar, ExternalLink } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Affinity } from '@/components';
import { cn } from '@/lib/utils';
import { Giveaway } from '@/types';
import { Badge } from '@/ui/badge';
import { getStoreColor, getStoreLabel } from '@/utils/store.ts';

interface FreeGameCardProps extends Giveaway {
  badge?: 'SÉRIE FAVORITA' | 'TOP PICK' | 'PARA VOCÊ';
  isFavSeries?: boolean;
  className?: string;
}

export function FreeGameCard({
  title,
  badge,
  className,
  image,
  worth,
  platforms,
  end_date: endDate,
  open_giveaway_url: url,
}: Readonly<FreeGameCardProps>) {
  const { t } = useTranslation('trending');

  const openLink = async () => {
    const { open } = await import('@tauri-apps/plugin-shell');
    open(url);
  };

  // Usa funções centralizadas
  const platformLabel = getStoreLabel(platforms);
  const platformColor = getStoreColor(platforms);

  const isValidDate =
    endDate && endDate !== 'N/A' && !Number.isNaN(new Date(endDate).getTime());

  const formattedDate = isValidDate
    ? new Date(endDate)
        .toLocaleDateString('pt-BR', {
          day: '2-digit',
          month: 'short',
        })
        .replace('.', '')
    : null;

  // Calcula dias restantes
  const daysLeft = isValidDate
    ? Math.ceil(
        (new Date(endDate).getTime() - Date.now()) / (1000 * 60 * 60 * 24)
      )
    : null;

  const badgeLabel =
    badge === 'SÉRIE FAVORITA'
      ? t('free_game_card_badge_favorite_series')
      : badge === 'TOP PICK'
        ? t('free_game_card_badge_top_pick')
        : badge === 'PARA VOCÊ'
          ? t('free_game_card_badge_for_you')
          : null;

  return (
    <div
      className={cn(
        'group border-border bg-card relative flex h-full cursor-pointer flex-col overflow-hidden rounded-xl border transition-all duration-300 hover:-translate-y-1 hover:shadow-lg',
        className
      )}
    >
      <button
        type="button"
        className="absolute inset-0 z-20"
        aria-label={t('free_game_card_open_giveaway', { title })}
        onClick={() => {
          void openLink();
        }}
      />

      {/* Container da Imagem com aspect ratio 16:9 */}
      <div className="relative aspect-video w-full overflow-hidden">
        <img
          src={image}
          alt={title}
          className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
          loading="lazy"
        />

        {/* Overlay gradiente */}
        <div className="absolute inset-0 bg-linear-to-t from-black/80 via-black/20 to-transparent opacity-60 transition-opacity" />

        {/* Badge Recomendação com Tooltip */}
        {badge && (
          <div className="absolute top-3 left-3">
            <Affinity badge={badge}>
              <Badge className="bg-purple-600 font-bold text-white">
                {badgeLabel}
              </Badge>
            </Affinity>
          </div>
        )}

        {/* Badge da Plataforma - Canto direito inferior */}
        <div className="absolute right-3 bottom-3">
          <Badge
            className={cn('text-xs font-semibold shadow-md', platformColor)}
          >
            {platformLabel}
          </Badge>
        </div>

        {/* Ícone de link externo no hover */}
        <div className="absolute top-3 right-3 opacity-0 transition-all duration-300 group-hover:opacity-100">
          <div className="flex items-center justify-center rounded-full bg-white/20 p-2 backdrop-blur-sm">
            <ExternalLink size={18} className="text-white" />
          </div>
        </div>
      </div>

      {/* Conteúdo */}
      <div className="relative flex flex-1 flex-col gap-2 p-3">
        <h3 className="text-foreground line-clamp-2 text-sm leading-tight font-semibold transition-colors">
          {title}
        </h3>

        {/* Footer com preço e data */}
        <div className="mt-auto flex items-center justify-between gap-1.5">
          {/* Preço original */}
          {worth && worth !== 'N/A' && (
            <div className="flex items-center gap-1">
              <span className="text-muted-foreground text-xs line-through">
                {worth}
              </span>
              <span className="text-xs font-bold text-green-500">
                {t('free_game_card_discount_label')}
              </span>
            </div>
          )}

          {/* Data de expiração */}
          {formattedDate && (
            <div className="flex items-center gap-1 rounded-md bg-orange-500/10 px-2 py-0.5">
              <Calendar size={12} className="text-orange-500" />
              <span className="text-xs font-semibold text-orange-500">
                {daysLeft && daysLeft <= 3
                  ? t('free_game_card_days_left_short', { count: daysLeft })
                  : formattedDate}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
