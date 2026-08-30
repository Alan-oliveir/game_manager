import { formatDistanceToNow } from 'date-fns';
import { ptBR } from 'date-fns/locale';
import { Medal } from 'lucide-react';

import { getTier } from '@/hooks/achievements';
import { AchievementDetail, PLATFORM_STYLES } from '@/types';

interface AchievementCardProps {
  achievement: AchievementDetail;
}

export function AchievementCard({
  achievement: ach,
}: Readonly<AchievementCardProps>) {
  const library = PLATFORM_STYLES[ach.source];
  const tier = getTier(ach.rarity_percent);

  return (
    <div
      className={`bg-card hover:bg-accent/5 flex items-start gap-3 rounded-lg border border-l-4 p-4 transition-colors ${tier.borderClass}`}
    >
      {ach.icon_url ? (
        <img
          src={ach.icon_url}
          alt=""
          className="h-14 w-14 shrink-0 rounded-md object-cover"
        />
      ) : (
        <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-md bg-yellow-500/10 text-yellow-500">
          <Medal size={22} />
        </div>
      )}

      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between gap-2">
          <p className="truncate text-sm font-semibold">
            {ach.achievement_name}
          </p>
          {ach.rarity_percent != null && (
            <span
              className={`shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[10px] font-medium whitespace-nowrap ${tier.textClass}`}
            >
              {tier.label} · {ach.rarity_percent.toFixed(1)}%
            </span>
          )}
        </div>

        {ach.description && (
          <p className="text-muted-foreground mt-0.5 line-clamp-2 text-xs">
            {ach.description}
          </p>
        )}

        <div className="mt-2 flex items-center justify-between gap-2">
          <div className="flex items-center gap-1.5 overflow-hidden">
            <span
              className={`shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium ${library.className}`}
            >
              {library.label}
            </span>
            <p className="text-muted-foreground truncate text-xs">
              {ach.game_name}
            </p>
          </div>
          <span className="text-muted-foreground shrink-0 text-[10px] whitespace-nowrap">
            {formatDistanceToNow(new Date(ach.unlock_time * 1000), {
              addSuffix: true,
              locale: ptBR,
            })}
          </span>
        </div>
      </div>
    </div>
  );
}
