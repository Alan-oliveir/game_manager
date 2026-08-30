import { useMemo } from 'react';

import { AchievementDetail } from '@/types';

const RARITY_TIERS = [
  {
    key: 'legendary',
    label: 'Lendária',
    max: 5,
    points: 100,
    textClass: 'text-orange-400',
    borderClass: 'border-l-orange-400',
  },
  {
    key: 'rare',
    label: 'Rara',
    max: 20,
    points: 40,
    textClass: 'text-purple-400',
    borderClass: 'border-l-purple-400',
  },
  {
    key: 'uncommon',
    label: 'Incomum',
    max: 50,
    points: 15,
    textClass: 'text-blue-400',
    borderClass: 'border-l-blue-400',
  },
  {
    key: 'common',
    label: 'Comum',
    max: Infinity,
    points: 5,
    textClass: 'text-muted-foreground',
    borderClass: 'border-l-white/10',
  },
] as const;

const LEVEL_TIERS = [
  {
    label: 'Platina',
    min: 6000,
    bgClass: 'bg-cyan-500/10',
    textClass: 'text-cyan-300',
  },
  {
    label: 'Ouro',
    min: 2000,
    bgClass: 'bg-yellow-500/10',
    textClass: 'text-yellow-400',
  },
  {
    label: 'Prata',
    min: 500,
    bgClass: 'bg-slate-400/10',
    textClass: 'text-slate-300',
  },
  {
    label: 'Bronze',
    min: 0,
    bgClass: 'bg-orange-700/10',
    textClass: 'text-orange-400',
  },
] as const;

export function getTier(rarityPercent: number | null) {
  const pct = rarityPercent ?? 100;

  return (
    RARITY_TIERS.find(t => pct < t.max) ?? RARITY_TIERS[RARITY_TIERS.length - 1]
  );
}

function getLevel(totalScore: number) {
  return (
    LEVEL_TIERS.find(l => totalScore >= l.min) ??
    LEVEL_TIERS[LEVEL_TIERS.length - 1]
  );
}

export function useAchievementStats(achievements: AchievementDetail[]) {
  return useMemo(() => {
    if (achievements.length === 0) return null;

    let totalScore = 0;
    let rarest: AchievementDetail | null = null;
    const perGame = new Map<string, { name: string; count: number }>();

    for (const ach of achievements) {
      totalScore += getTier(ach.rarity_percent).points;

      if (
        ach.rarity_percent != null &&
        (rarest === null ||
          ach.rarity_percent < (rarest.rarity_percent ?? Infinity))
      ) {
        rarest = ach;
      }

      const key = `${ach.source}-${ach.game_id}`;
      const entry = perGame.get(key);

      if (entry) entry.count += 1;
      else perGame.set(key, { name: ach.game_name, count: 1 });
    }

    let favoriteGame: { name: string; count: number } | null = null;

    for (const entry of perGame.values()) {
      if (!favoriteGame || entry.count > favoriteGame.count)
        favoriteGame = entry;
    }

    return {
      total: achievements.length,
      totalScore,
      level: getLevel(totalScore),
      rarest,
      favoriteGame,
    };
  }, [achievements]);
}
