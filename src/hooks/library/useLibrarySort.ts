import { useMemo } from 'react';

import { Game } from '@/types';

export type SortField = 'name' | 'criticScore' | 'releaseDate';
export type SortDirection = 'asc' | 'desc';

export interface SortOption {
  field: SortField;
  direction: SortDirection;
}

function compareNullable<T>(
  a: T | null | undefined,
  b: T | null | undefined,
  compare: (a: T, b: T) => number
): number {
  const aEmpty = a === null || a === undefined || a === '';
  const bEmpty = b === null || b === undefined || b === '';

  if (aEmpty && bEmpty) return 0;
  if (aEmpty) return 1;
  if (bEmpty) return -1;

  return compare(a as T, b as T);
}

export function useLibrarySort(games: Game[], sort: SortOption): Game[] {
  return useMemo(() => {
    return [...games].sort((a, b) => {
      let result: number;

      switch (sort.field) {
        case 'criticScore':
          result = compareNullable(
            a.criticScore,
            b.criticScore,
            (x, y) => x - y
          );
          break;
        case 'releaseDate':
          result = compareNullable(a.releaseDate, b.releaseDate, (x, y) =>
            x.localeCompare(y)
          );
          break;
        case 'name':
        default:
          result = a.name.localeCompare(b.name, undefined, {
            sensitivity: 'base',
          });
          break;
      }

      return sort.direction === 'desc' ? -result : result;
    });
  }, [games, sort.field, sort.direction]);
}
