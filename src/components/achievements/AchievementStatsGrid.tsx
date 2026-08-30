import { Award, Gamepad2, Gem, Trophy } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { StatCard } from '@/components/cards';
import { useAchievementStats } from '@/hooks/achievements';
import { AchievementDetail } from '@/types';

interface AchievementStatsGridProps {
  achievements: AchievementDetail[];
}

export function AchievementStatsGrid({
  achievements,
}: Readonly<AchievementStatsGridProps>) {
  const { t } = useTranslation('common');
  const stats = useAchievementStats(achievements);

  if (!stats) return null;

  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
      <StatCard
        icon={<Trophy size={20} />}
        color="text-yellow-500"
        bg="bg-yellow-500/10"
        label={t('achievements_stat_total')}
        value={stats.total}
        sublabel={t('achievements_stat_points', { points: stats.totalScore })}
      />
      <StatCard
        icon={<Gamepad2 size={20} />}
        color="text-blue-500"
        bg="bg-blue-500/10"
        label={t('achievements_stat_favorite_game')}
        value={stats.favoriteGame?.name ?? '—'}
        sublabel={
          stats.favoriteGame
            ? t('achievements_stat_favorite_count', {
                count: stats.favoriteGame.count,
              })
            : undefined
        }
      />
      <StatCard
        icon={<Gem size={20} />}
        color="text-purple-400"
        bg="bg-purple-500/10"
        label={t('achievements_stat_rarest')}
        value={stats.rarest?.achievement_name ?? '—'}
        sublabel={
          stats.rarest
            ? `${stats.rarest.rarity_percent!.toFixed(1)}% · ${stats.rarest.game_name}`
            : t('achievements_stat_rarest_empty')
        }
      />
      <StatCard
        icon={<Award size={20} />}
        color={stats.level.textClass}
        bg={stats.level.bgClass}
        label={t('achievements_stat_level')}
        value={stats.level.label}
        sublabel={t('achievements_stat_points', { points: stats.totalScore })}
      />
    </div>
  );
}
