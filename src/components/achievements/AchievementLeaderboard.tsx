import { Trophy } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { useAchievementLeaderboards } from '@/hooks/achievements';
import { AchievementDetail, PLATFORM_STYLES } from '@/types';
import { Separator } from '@/ui/separator';

interface AchievementLeaderboardProps {
  achievements: AchievementDetail[];
}

export function AchievementLeaderboard({
  achievements,
}: Readonly<AchievementLeaderboardProps>) {
  const { t } = useTranslation('common');
  const { topGames } = useAchievementLeaderboards(achievements);

  if (topGames.length === 0) return null;

  return (
    <div className="bg-card border-border rounded-xl border p-6 lg:col-span-2">
      <div className="mb-3 flex items-center gap-2">
        <Trophy size={20} className="text-primary" />
        <h2 className="text-lg font-semibold">
          {t('achievements_top_games_section')}
        </h2>
      </div>
      <Separator className="mb-3" />
      <div className="grid grid-cols-1 gap-4">
        {topGames.map((game, index) => {
          const library = PLATFORM_STYLES[game.source];

          return (
            <div key={game.key} className="flex w-full items-center gap-3 p-2">
              <div className="text-muted-foreground bg-muted flex w-6 justify-center rounded font-bold">
                {index + 1}
              </div>
              <div className="min-w-0 flex-1">
                <h4 className="truncate text-sm font-medium">
                  {game.gameName}
                </h4>
                <div className="mt-1 flex items-center justify-between gap-2">
                  <span
                    className={`shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium ${library.className}`}
                  >
                    {library.label}
                  </span>
                  <span className="bg-secondary rounded px-1.5 py-0.5 font-mono text-xs whitespace-nowrap">
                    {t('achievements_count_suffix', { count: game.count })}
                  </span>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
