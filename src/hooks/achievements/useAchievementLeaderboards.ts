import { useMemo } from 'react';

import { AchievementDetail, AchievementPlatform } from '@/types';

export interface TopGameEntry {
  key: string;
  gameId: string;
  gameName: string;
  source: AchievementPlatform;
  count: number;
}

export interface PlatformBreakdownEntry {
  platform: AchievementPlatform;
  count: number;
}

const TOP_GAMES_LIMIT = 5;

export function useAchievementLeaderboards(achievements: AchievementDetail[]) {
  return useMemo(() => {
    const gameMap = new Map<string, TopGameEntry>();
    const platformMap = new Map<AchievementPlatform, number>();

    for (const ach of achievements) {
      const key = `${ach.source}-${ach.game_id}`;
      const entry = gameMap.get(key);

      if (entry) entry.count += 1;
      else
        gameMap.set(key, {
          key,
          gameId: ach.game_id,
          gameName: ach.game_name,
          source: ach.source,
          count: 1,
        });

      platformMap.set(ach.source, (platformMap.get(ach.source) ?? 0) + 1);
    }

    const topGames = Array.from(gameMap.values())
      .sort((a, b) => b.count - a.count)
      .slice(0, TOP_GAMES_LIMIT);

    const platformBreakdown: PlatformBreakdownEntry[] = Array.from(
      platformMap.entries()
    )
      .map(([platform, count]) => ({ platform, count }))
      .sort((a, b) => b.count - a.count);

    return { topGames, platformBreakdown };
  }, [achievements]);
}
