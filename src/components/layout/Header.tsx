import {
  Gamepad2,
  LayoutGrid,
  Moon,
  Search,
  Settings,
  Store,
  Sun,
} from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Kofi, ViewFiltersDropdown } from '@/components';
import { QuickSettings } from '@/dialogs/QuickSettings';
import {
  useHeaderState,
  useRecommendationAnalysis,
  useTheme,
  type ViewFilters,
} from '@/hooks';
import { Game } from '@/types';
import { Button } from '@/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/tooltip';
import { openExternalLink } from '@/utils/openLink.ts';
import { PlataformsConfig } from '@/windows/PlatformsConfig';

interface HeaderProps {
  onAddGame: () => void;
  searchTerm: string;
  onSearchChange: (term: string) => void;
  activeSection: string;
  viewFilters: ViewFilters;
  onViewFiltersChange: (filters: ViewFilters) => void;
  groupByPlatform: boolean;
  onToggleGroupByPlatform: () => void;
  onCheckUpdates: () => void;
  onLibraryUpdate: () => void;
  userGames: Game[];
}

export default function Header({
  onAddGame,
  searchTerm,
  onSearchChange,
  activeSection,
  viewFilters,
  onViewFiltersChange,
  groupByPlatform,
  onToggleGroupByPlatform,
  onCheckUpdates,
  onLibraryUpdate,
  userGames,
}: Readonly<HeaderProps>) {
  const { t } = useTranslation('common');

  const { isDark, toggleTheme } = useTheme();
  const { isSearchable, isFilterable, searchPlaceholder, searchAriaLabel } =
    useHeaderState(activeSection);

  const [isQuickSettingsOpen, setIsQuickSettingsOpen] = useState(false);
  const [isStoresConfigOpen, setIsStoresConfigOpen] = useState(false);

  const { analysisStatus, generateRecommendationAnalysis } =
    useRecommendationAnalysis();

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 border-border sticky top-0 z-50 flex h-16 items-center justify-between gap-2 border-b px-3 backdrop-blur md:px-4">
      {/* Search Bar */}
      <div className="mr-2 max-w-xl flex-1 transition-opacity duration-200">
        <div className="group relative">
          <Search
            className={`absolute top-1/2 left-3 -translate-y-1/2 transition-colors ${
              isSearchable
                ? 'text-muted-foreground group-focus-within:text-primary'
                : 'text-muted-foreground/40'
            }`}
            size={18}
          />
          <input
            type="text"
            disabled={!isSearchable}
            placeholder={searchPlaceholder}
            aria-label={searchAriaLabel}
            value={isSearchable ? searchTerm : ''}
            onChange={e => onSearchChange(e.target.value)}
            className={`h-9 w-full rounded-md border py-2 pr-4 pl-9 text-sm transition-all ${
              isSearchable
                ? 'bg-muted/50 hover:bg-muted focus:bg-background focus:border-primary focus:ring-primary/20 text-foreground placeholder:text-muted-foreground border-transparent focus:ring-1 focus:outline-none'
                : 'bg-muted/20 text-muted-foreground placeholder:text-muted-foreground/40 cursor-not-allowed border-transparent'
            } `}
          />
        </div>
      </div>

      {analysisStatus && (
        <div className="text-muted-foreground animate-pulse text-xs">
          {analysisStatus}
        </div>
      )}

      <div className="ml-auto flex items-center gap-2">
        <Button
          onClick={onAddGame}
          size="sm"
          variant="outline"
          className="shrink-0 px-3 md:px-4"
          title={t('header_add_game_title')}
        >
          <Gamepad2 size={18} />
          <span className="ml-1 hidden md:inline">
            {t('header_add_game_button')}
          </span>
        </Button>

        <Button
          onClick={() => setIsStoresConfigOpen(true)}
          size="sm"
          variant="outline"
          className="shrink-0 px-3 md:px-4"
          title={t('header_store_settings_title')}
        >
          <Store size={18} />
          <span className="ml-1 hidden md:inline">
            {t('header_store_settings_button')}
          </span>
        </Button>

        {/* Filter toggle (adulto/duplicatas/não-instalados) */}
        {isFilterable && (
          <ViewFiltersDropdown
            filters={viewFilters}
            onChange={onViewFiltersChange}
          />
        )}

        {/* Toggle de agrupar por plataforma — modo de view, não filtro de conteúdo */}
        {isFilterable && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={groupByPlatform ? 'secondary' : 'ghost'}
                size="icon"
                onClick={onToggleGroupByPlatform}
                className="shrink-0"
              >
                <LayoutGrid size={18} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p>{t('header_group_by_platform_title')}</p>
            </TooltipContent>
          </Tooltip>
        )}

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              onClick={() =>
                openExternalLink('https://ko-fi.com/alandeogoncalves')
              }
              variant="ghost"
              size="icon"
              className="text-muted-foreground shrink-0 hover:bg-blue-500/10 hover:text-blue-500"
            >
              <Kofi size={18} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t('header_support_kofi_title')}</p>
          </TooltipContent>
        </Tooltip>

        <Button
          onClick={() => setIsQuickSettingsOpen(true)}
          variant="ghost"
          size="icon"
          className="text-muted-foreground hover:text-foreground shrink-0"
          title={t('header_quick_settings_title')}
        >
          <Settings size={18} />
        </Button>

        <Button
          onClick={toggleTheme}
          variant="ghost"
          size="icon"
          className="text-muted-foreground hover:text-foreground shrink-0"
          title={t('header_toggle_theme_title')}
        >
          {isDark ? <Sun size={18} /> : <Moon size={18} />}
        </Button>
      </div>

      <QuickSettings
        open={isQuickSettingsOpen}
        onClose={() => setIsQuickSettingsOpen(false)}
        onGenerateReport={generateRecommendationAnalysis}
        onCheckUpdates={onCheckUpdates}
        userGames={userGames}
      />

      <PlataformsConfig
        isOpen={isStoresConfigOpen}
        onClose={() => setIsStoresConfigOpen(false)}
        onLibraryUpdate={onLibraryUpdate}
      />
    </header>
  );
}
