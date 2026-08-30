import { Gamepad2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { useAchievementLeaderboards } from '@/hooks/achievements';
import { AchievementDetail, PLATFORM_STYLES } from '@/types';
import { Separator } from '@/ui/separator';

interface PlatformBreakdownProps {
  achievements: AchievementDetail[];
}

export function PlatformBreakdown({
  achievements,
}: Readonly<PlatformBreakdownProps>) {
  const { t } = useTranslation('common');
  const { platformBreakdown } = useAchievementLeaderboards(achievements);

  if (platformBreakdown.length === 0) return null;

  const total = achievements.length;

  return (
    <div className="bg-card border-border col-span-1 h-full rounded-xl border p-6">
      <div className="mb-3 flex items-center gap-2">
        <Gamepad2 size={20} className="text-primary" />
        <h2 className="text-lg font-semibold">
          {t('achievements_platform_breakdown_section')}
        </h2>
      </div>
      <Separator className="mb-3" />
      <div className="space-y-4">
        {platformBreakdown.map(({ platform, count }) => {
          const percent = Math.round((count / total) * 100);
          const library = PLATFORM_STYLES[platform];

          return (
            <div key={platform}>
              <div className="mb-1 flex justify-between text-xs">
                <span className="font-medium">{library.label}</span>
                <span className="text-muted-foreground">
                  {t('achievements_count_suffix', { count })}
                </span>
              </div>
              <div className="bg-secondary h-2 overflow-hidden rounded-full">
                <div
                  className="bg-primary/80 h-full transition-all duration-1000 ease-out"
                  style={{ width: `${percent}%` }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
