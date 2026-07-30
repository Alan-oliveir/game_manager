import { useCallback, useMemo, useState } from 'react';

import { Game, Platform } from '@/types';

export type GridRow =
  | { type: 'header'; platform: Platform; count: number; collapsed: boolean }
  | { type: 'cards'; games: Game[] }; // slice de até columnCount jogos

export function useGroupedLibrary(
  games: Game[],
  groupByPlatform: boolean,
  columnCount: number
) {
  const [collapsedPlatforms, setCollapsedPlatforms] = useState<Set<string>>(
    new Set()
  );

  const rows = useMemo<GridRow[]>(() => {
    if (columnCount <= 0) return [];

    if (!groupByPlatform) {
      const result: GridRow[] = [];

      for (let i = 0; i < games.length; i += columnCount) {
        result.push({ type: 'cards', games: games.slice(i, i + columnCount) });
      }

      return result;
    }

    const groups = new Map<Platform, Game[]>();

    for (const game of games) {
      const key = game.platform; // PascalCase padronizado

      if (!groups.has(key)) groups.set(key, []);

      groups.get(key)!.push(game);
    }

    const sortedPlatforms = [...groups.keys()].sort();
    const result: GridRow[] = [];

    for (const platform of sortedPlatforms) {
      const platformGames = groups.get(platform)!;
      const collapsed = collapsedPlatforms.has(platform);
      result.push({
        type: 'header',
        platform,
        count: platformGames.length,
        collapsed,
      });

      if (!collapsed) {
        for (let i = 0; i < platformGames.length; i += columnCount) {
          result.push({
            type: 'cards',
            games: platformGames.slice(i, i + columnCount),
          });
        }
      }
    }

    return result;
  }, [games, groupByPlatform, columnCount, collapsedPlatforms]);

  const togglePlatform = useCallback((platform: Platform) => {
    setCollapsedPlatforms(prev => {
      const next = new Set(prev);

      if (next.has(platform)) {
        next.delete(platform);
      } else {
        next.add(platform);
      }

      return next;
    });
  }, []);

  return { rows, togglePlatform };
}
