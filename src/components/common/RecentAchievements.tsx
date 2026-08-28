import { invoke } from '@tauri-apps/api/core';
import { formatDistanceToNow } from 'date-fns';
import { ptBR } from 'date-fns/locale';
import { ChevronRight, Medal, Trophy } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/button';
import { Skeleton } from '@/ui/skeleton';

type AchievementPlatform = 'steam' | 'epic' | 'gog' | 'xbox';

interface Achievement {
  source: AchievementPlatform;
  game_name: string;
  achievement_name: string;
  unlock_time: number;
  game_id: string;
}

const PLATFORM_STYLES: Record<
  AchievementPlatform,
  { label: string; className: string }
> = {
  steam: { label: 'Steam', className: 'bg-sky-500/10 text-sky-500' },
  epic: { label: 'Epic', className: 'bg-neutral-500/10 text-neutral-400' },
  gog: { label: 'GOG', className: 'bg-purple-500/10 text-purple-400' },
  xbox: { label: 'Xbox', className: 'bg-green-500/10 text-green-500' },
};

const PREVIEW_LIMIT = 3;

interface RecentAchievementsProps {
  onViewAll?: () => void;
}

export function RecentAchievements({
  onViewAll,
}: Readonly<RecentAchievementsProps>) {
  const { t } = useTranslation('common');
  const [achievements, setAchievements] = useState<Achievement[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<Achievement[]>('get_recent_achievements')
      .then(setAchievements)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-full" />
      </div>
    );
  }

  const header = (
    <div className="mb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
            <Trophy size={24} />
          </div>
          <div>
            <h2 className="text-2xl font-bold">{t('achievements_recent')}</h2>
          </div>
        </div>

        {onViewAll && achievements.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            className="gap-2"
            onClick={onViewAll}
          >
            <ChevronRight size={14} />
            {t('achievements_view_all')}
          </Button>
        )}
      </div>
    </div>
  );

  if (achievements.length === 0) {
    return (
      <div className="space-y-3">
        {header}
        <p className="text-muted-foreground text-sm">
          {t('achievements_empty')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {header}
      <div className="space-y-2">
        {achievements.slice(0, PREVIEW_LIMIT).map(ach => {
          const library = PLATFORM_STYLES[ach.source];

          return (
            <div
              key={`${ach.source}-${ach.game_id}-${ach.achievement_name}`}
              className="bg-card hover:bg-accent/5 flex items-center justify-between rounded-lg border p-3 transition-colors"
            >
              <div className="flex items-center gap-3 overflow-hidden">
                <div className="shrink-0 rounded-full bg-yellow-500/10 p-2 text-yellow-500">
                  <Medal size={16} />
                </div>
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold">
                    {ach.achievement_name}
                  </p>
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
                </div>
              </div>
              <span className="text-muted-foreground ml-2 shrink-0 text-[10px] whitespace-nowrap">
                {formatDistanceToNow(new Date(ach.unlock_time * 1000), {
                  addSuffix: true,
                  locale: ptBR,
                })}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
