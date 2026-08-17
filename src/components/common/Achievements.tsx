import { invoke } from '@tauri-apps/api/core';
import { formatDistanceToNow } from 'date-fns';
import { ptBR } from 'date-fns/locale';
import { Medal, Trophy } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Skeleton } from '@/ui/skeleton';

type Library = 'steam' | 'epic' | 'gog' | 'xbox';

interface Achievement {
  library: Library;
  game_name: string;
  achievement_name: string;
  unlock_time: number; // Timestamp Unix
  game_id: string;
}

const PLATFORM_STYLES: Record<Library, { label: string; className: string }> = {
  steam: { label: 'Steam', className: 'bg-sky-500/10 text-sky-500' },
  epic: { label: 'Epic', className: 'bg-neutral-500/10 text-neutral-400' },
  gog: { label: 'GOG', className: 'bg-purple-500/10 text-purple-400' },
  xbox: { label: 'Xbox', className: 'bg-green-500/10 text-green-500' },
};

export default function Achievements() {
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

  if (achievements.length === 0) {
    return (
      <div className="space-y-3">
        <h3 className="mb-4 flex items-center gap-2 text-lg font-bold">
          <Trophy className="text-yellow-500" size={20} />
          {t('achievements_recent')}
        </h3>
        <p className="text-muted-foreground text-sm">
          {t('achievements_empty')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <h3 className="mb-4 flex items-center gap-2 text-lg font-bold">
        <Trophy className="text-yellow-500" size={20} />
        {t('achievements_recent')}
      </h3>

      <div className="space-y-2">
        {achievements.map(ach => {
          const library = PLATFORM_STYLES[ach.library];

          return (
            <div
              key={`${ach.library}-${ach.game_id}-${ach.achievement_name}`}
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
