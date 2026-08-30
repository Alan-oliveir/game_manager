import { useEffect, useMemo, useState } from 'react';

import { AchievementDetail, AchievementPlatform } from '@/types';

export type AchievementSortField = 'unlock_time' | 'rarity_percent';

export interface AchievementSort {
  field: AchievementSortField;
  direction: 'asc' | 'desc';
}

const PAGE_SIZE = 30;
const DEFAULT_SORT: AchievementSort = {
  field: 'unlock_time',
  direction: 'desc',
};

export function useAchievementFilters(achievements: AchievementDetail[]) {
  const [platformFilter, setPlatformFilter] = useState<AchievementPlatform[]>(
    []
  );
  const [sort, setSort] = useState<AchievementSort>(DEFAULT_SORT);
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE);

  const availablePlatforms = useMemo(
    () => Array.from(new Set(achievements.map(a => a.source))),
    [achievements]
  );

  const filteredAndSorted = useMemo(() => {
    const filtered =
      platformFilter.length === 0
        ? achievements
        : achievements.filter(a => platformFilter.includes(a.source));

    const arr = [...filtered];
    const dir = sort.direction === 'asc' ? 1 : -1;

    if (sort.field === 'rarity_percent') {
      arr.sort((a, b) => {
        if (a.rarity_percent == null && b.rarity_percent == null) return 0;

        if (a.rarity_percent == null) return 1;

        if (b.rarity_percent == null) return -1;

        return dir * (a.rarity_percent - b.rarity_percent);
      });
    } else {
      arr.sort((a, b) => dir * (a.unlock_time - b.unlock_time));
    }

    return arr;
  }, [achievements, platformFilter, sort]);

  useEffect(() => {
    setVisibleCount(PAGE_SIZE);
  }, [platformFilter, sort]);

  const visibleAchievements = useMemo(
    () => filteredAndSorted.slice(0, visibleCount),
    [filteredAndSorted, visibleCount]
  );

  const togglePlatform = (platform: AchievementPlatform) => {
    setPlatformFilter(prev =>
      prev.includes(platform)
        ? prev.filter(p => p !== platform)
        : [...prev, platform]
    );
  };

  return {
    platformFilter,
    togglePlatform,
    availablePlatforms,
    sort,
    setSort,
    visibleAchievements,
    hasMore: visibleCount < filteredAndSorted.length,
    loadMore: () => setVisibleCount(v => v + PAGE_SIZE),
    totalFiltered: filteredAndSorted.length,
  };
}
