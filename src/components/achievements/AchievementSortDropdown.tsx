import { ArrowDownAZ, ArrowUpAZ } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { AchievementSort, AchievementSortField } from '@/hooks/achievements';
import { Button } from '@/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/ui/dropdown-menu';

interface AchievementSortDropdownProps {
  sort: AchievementSort;
  onChange: (sort: AchievementSort) => void;
}

const SORT_FIELDS: { value: AchievementSortField; labelKey: string }[] = [
  { value: 'unlock_time', labelKey: 'achievements_sort_field_date' },
  { value: 'rarity_percent', labelKey: 'achievements_sort_field_rarity' },
];

export function AchievementSortDropdown({
  sort,
  onChange,
}: Readonly<AchievementSortDropdownProps>) {
  const { t } = useTranslation('common');

  const setField = (field: string) =>
    onChange({ ...sort, field: field as AchievementSortField });
  const toggleDirection = () =>
    onChange({ ...sort, direction: sort.direction === 'asc' ? 'desc' : 'asc' });

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" className="gap-2">
          {sort.direction === 'asc' ? (
            <ArrowUpAZ className="h-4 w-4" />
          ) : (
            <ArrowDownAZ className="h-4 w-4" />
          )}
          {t('header_sort_label')}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>{t('header_sort_field_label')}</DropdownMenuLabel>
        <DropdownMenuRadioGroup value={sort.field} onValueChange={setField}>
          {SORT_FIELDS.map(({ value, labelKey }) => (
            <DropdownMenuRadioItem key={value} value={value}>
              {t(labelKey)}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>

        <DropdownMenuSeparator />

        <button
          onClick={toggleDirection}
          className="hover:bg-accent flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
        >
          {sort.direction === 'asc' ? (
            <ArrowUpAZ className="h-4 w-4" />
          ) : (
            <ArrowDownAZ className="h-4 w-4" />
          )}
          {sort.field === 'rarity_percent'
            ? sort.direction === 'asc'
              ? t('achievements_sort_rarest_first')
              : t('achievements_sort_common_first')
            : sort.direction === 'desc'
              ? t('achievements_sort_newest_first')
              : t('achievements_sort_oldest_first')}
        </button>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
