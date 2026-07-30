import { Filter } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { type ViewFilters } from '@/hooks';
import { Badge } from '@/ui/badge';
import { Button } from '@/ui/button';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@/ui/dropdown-menu';

type ViewFiltersDropdownProps = {
  filters: ViewFilters;
  onChange: (filters: ViewFilters) => void;
};

const FILTER_OPTIONS: { key: keyof ViewFilters; labelKey: string }[] = [
  { key: 'hideDuplicates', labelKey: 'header_filters_hide_duplicates' },
  { key: 'hideAdult', labelKey: 'header_filters_hide_adult' },
  { key: 'hideNotInstalled', labelKey: 'header_filters_hide_not_installed' },
];

export function ViewFiltersDropdown({
  filters,
  onChange,
}: ViewFiltersDropdownProps) {
  const { t } = useTranslation('common');

  const toggle = (key: keyof ViewFilters) => {
    onChange({ ...filters, [key]: !filters[key] });
  };

  const activeCount = Object.values(filters).filter(Boolean).length;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" className="gap-2">
          <Filter className="h-4 w-4" />
          {t('header_filters_label')}
          {activeCount > 0 && (
            <Badge variant="secondary" className="ml-1">
              {activeCount}
            </Badge>
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {FILTER_OPTIONS.map(({ key, labelKey }) => (
          <DropdownMenuCheckboxItem
            key={key}
            checked={filters[key]}
            onCheckedChange={() => toggle(key)}
          >
            {t(labelKey)}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
