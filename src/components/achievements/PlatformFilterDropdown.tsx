import { Filter } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { AchievementPlatform, PLATFORM_STYLES } from '@/types';
import { Button } from '@/ui/button';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/ui/dropdown-menu';

interface PlatformFilterDropdownProps {
  availablePlatforms: AchievementPlatform[];
  selected: AchievementPlatform[];
  onToggle: (platform: AchievementPlatform) => void;
}

export function PlatformFilterDropdown({
  availablePlatforms,
  selected,
  onToggle,
}: Readonly<PlatformFilterDropdownProps>) {
  const { t } = useTranslation('common');

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="secondary"
          size="sm"
          className="w-40 gap-2 font-medium"
        >
          <Filter size={16} />
          {t('achievements_filter_platforms_button')}
          {selected.length > 0 && (
            <span className="bg-primary text-primary-foreground ml-1 flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-bold shadow-sm">
              {selected.length}
            </span>
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-40">
        <DropdownMenuLabel>{t('platforms_label')}</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {availablePlatforms.map(platform => (
          <DropdownMenuCheckboxItem
            key={platform}
            checked={selected.includes(platform)}
            onCheckedChange={() => onToggle(platform)}
          >
            {PLATFORM_STYLES[platform].label}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
