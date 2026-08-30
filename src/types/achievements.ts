export type AchievementPlatform = 'steam' | 'epic' | 'gog' | 'xbox';

export interface DashboardAchievement {
  source: AchievementPlatform;
  game_name: string;
  achievement_name: string;
  unlock_time: number;
  game_id: string;
}

export interface AchievementDetail {
  source: AchievementPlatform;
  game_id: string;
  game_name: string;
  achievement_name: string;
  description: string | null;
  icon_url: string | null;
  rarity_percent: number | null;
  rarity_slug: string | null;
  category: string | null;
  unlock_time: number;
}

export const PLATFORM_STYLES: Record<
  AchievementPlatform,
  { label: string; className: string }
> = {
  steam: { label: 'Steam', className: 'bg-sky-500/10 text-sky-500' },
  epic: { label: 'Epic', className: 'bg-neutral-500/10 text-neutral-400' },
  gog: { label: 'GOG', className: 'bg-purple-500/10 text-purple-400' },
  xbox: { label: 'Xbox', className: 'bg-green-500/10 text-green-500' },
};
