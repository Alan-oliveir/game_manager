import { ArrowDownAZ, ArrowUpAZ } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { SortField, SortOption } from '@/hooks';
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

type SortDropdownProps = {
  sort: SortOption;
  onChange: (sort: SortOption) => void;
};

const SORT_FIELDS: { value: SortField; labelKey: string }[] = [
  { value: 'name', labelKey: 'header_sort_field_name' },
  { value: 'criticScore', labelKey: 'header_sort_field_critic_score' },
  { value: 'releaseDate', labelKey: 'header_sort_field_release_date' },
];

export function SortDropdown({ sort, onChange }: SortDropdownProps) {
  const { t } = useTranslation('common');

  const setField = (field: string) =>
    onChange({ ...sort, field: field as SortField });
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
          {sort.direction === 'asc'
            ? t('header_sort_ascending')
            : t('header_sort_descending')}
        </button>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
