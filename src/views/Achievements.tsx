import { invoke } from '@tauri-apps/api/core';
import { ChartBar, Clock, Loader2, Trophy } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  AchievementCard,
  AchievementLeaderboard,
  AchievementSortDropdown,
  AchievementStatsGrid,
  PlatformBreakdown,
  PlatformFilterDropdown,
} from '@/components/achievements';
import { useAchievementFilters } from '@/hooks/achievements';
import { AchievementDetail } from '@/types';
import { Button } from '@/ui/button';
import { Separator } from '@/ui/separator';

const CACHE_TTL_MS = 5 * 60 * 1000;

interface AchievementsProps {
  cachedAchievements: AchievementDetail[];
  setCachedAchievements: (list: AchievementDetail[]) => void;
  cachedFetchedAt: number | null;
  setCachedFetchedAt: (value: number | null) => void;
}

export default function Achievements({
  cachedAchievements,
  setCachedAchievements,
  cachedFetchedAt,
  setCachedFetchedAt,
}: Readonly<AchievementsProps>) {
  const { t } = useTranslation('common');
  const [loading, setLoading] = useState(cachedAchievements.length === 0);

  useEffect(() => {
    const isStale =
      cachedFetchedAt == null || Date.now() - cachedFetchedAt > CACHE_TTL_MS;

    if (!isStale && cachedAchievements.length > 0) {
      setLoading(false);

      return;
    }

    setLoading(true);
    invoke<AchievementDetail[]>('get_all_achievements')
      .then(data => {
        setCachedAchievements(data);
        setCachedFetchedAt(Date.now());
      })
      .catch(console.error)
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const achievements = cachedAchievements;

  const {
    platformFilter,
    togglePlatform,
    availablePlatforms,
    sort,
    setSort,
    visibleAchievements,
    hasMore,
    loadMore,
    totalFiltered,
  } = useAchievementFilters(achievements);

  return (
    <div className="bg-background flex h-full flex-1 flex-col overflow-hidden">
      <div className="border-border/40 shrink-0 border-b px-5 pt-5 pb-3 lg:px-8 lg:pt-6 lg:pb-4">
        <div className="mb-2 flex items-center gap-2.5 lg:gap-3">
          <div className="rounded-lg bg-purple-500/10 p-2 text-purple-500">
            <Trophy size={24} className="lg:h-6 lg:w-6" />
          </div>
          <div>
            <h1 className="text-xl font-bold lg:text-2xl">
              {t('achievements_title')}
            </h1>
            <p className="text-muted-foreground text-sm">
              {t('achievements_description', { count: achievements.length })}
            </p>
          </div>
        </div>
      </div>

      <div className="custom-scrollbar flex-1 overflow-y-auto p-5 lg:p-8">
        <div className="mx-auto max-w-7xl space-y-6">
          {loading ? (
            <div className="flex h-64 items-center justify-center">
              <Loader2 className="text-primary h-10 w-10 animate-spin" />
            </div>
          ) : (
            <>
              {achievements.length > 0 && (
                <div>
                  <div className="mb-6 flex items-center gap-2">
                    <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                      <Clock size={24} />
                    </div>
                    <h2 className="text-2xl font-bold">
                      {t('achievement_summary_section')}
                    </h2>
                  </div>
                  <AchievementStatsGrid achievements={achievements} />
                </div>
              )}

              <Separator />

              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center gap-2">
                  <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                    <Clock size={24} />
                  </div>
                  <h2 className="text-2xl font-bold">
                    {t('achievement_list_section')}
                  </h2>
                </div>

                <div className="flex gap-2">
                  {availablePlatforms.length > 1 && (
                    <PlatformFilterDropdown
                      availablePlatforms={availablePlatforms}
                      selected={platformFilter}
                      onToggle={togglePlatform}
                    />
                  )}
                  <AchievementSortDropdown sort={sort} onChange={setSort} />
                </div>
              </div>

              {totalFiltered === 0 ? (
                <p className="text-muted-foreground text-sm">
                  {t('achievements_empty')}
                </p>
              ) : (
                <>
                  <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
                    {visibleAchievements.map(ach => (
                      <AchievementCard
                        key={`${ach.source}-${ach.game_id}-${ach.achievement_name}`}
                        achievement={ach}
                      />
                    ))}
                  </div>

                  {hasMore && (
                    <div className="flex justify-end pt-2">
                      <Button variant="outline" onClick={loadMore}>
                        {t('achievements_load_more')}
                      </Button>
                    </div>
                  )}

                  {achievements.length > 0 && (
                    <>
                      <Separator />
                      <div className="mb-6 flex items-center gap-2">
                        <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                          <ChartBar size={24} />
                        </div>
                        <h2 className="text-2xl font-bold">
                          {t('achievements_statistics_section')}
                        </h2>
                      </div>

                      <div className="grid grid-cols-2 gap-4 lg:grid-cols-3 lg:gap-8">
                        <AchievementLeaderboard achievements={achievements} />
                        <PlatformBreakdown achievements={achievements} />
                      </div>
                    </>
                  )}
                </>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
