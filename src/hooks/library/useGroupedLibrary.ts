import { useCallback, useMemo, useState } from 'react';

import { Game, Library } from '@/types';

export type GridRow =
  | { type: 'header'; library: Library; count: number; collapsed: boolean }
  | { type: 'cards'; games: Game[] }; // slice de até columnCount jogos

export function useGroupedLibrary(
  games: Game[],
  groupByLibrary: boolean,
  columnCount: number
) {
  const [collapsedLibraries, setCollapsedLibraries] = useState<Set<string>>(
    new Set()
  );

  const rows = useMemo<GridRow[]>(() => {
    if (columnCount <= 0) return [];

    if (!groupByLibrary) {
      const result: GridRow[] = [];

      for (let i = 0; i < games.length; i += columnCount) {
        result.push({ type: 'cards', games: games.slice(i, i + columnCount) });
      }

      return result;
    }

    const groups = new Map<Library, Game[]>();

    for (const game of games) {
      const key = game.library; // PascalCase padronizado

      if (!groups.has(key)) groups.set(key, []);

      groups.get(key)!.push(game);
    }

    const sortedLibraries = [...groups.keys()].sort();
    const result: GridRow[] = [];

    for (const library of sortedLibraries) {
      const libraryGames = groups.get(library)!;
      const collapsed = collapsedLibraries.has(library);
      result.push({
        type: 'header',
        library,
        count: libraryGames.length,
        collapsed,
      });

      if (!collapsed) {
        for (let i = 0; i < libraryGames.length; i += columnCount) {
          result.push({
            type: 'cards',
            games: libraryGames.slice(i, i + columnCount),
          });
        }
      }
    }

    return result;
  }, [games, groupByLibrary, columnCount, collapsedLibraries]);

  const toggleLibrary = useCallback((library: Library) => {
    setCollapsedLibraries(prev => {
      const next = new Set(prev);

      if (next.has(library)) {
        next.delete(library);
      } else {
        next.add(library);
      }

      return next;
    });
  }, []);

  return { rows, toggleLibrary };
}
