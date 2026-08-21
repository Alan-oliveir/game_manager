// src/views/AchievementsView.tsx
import { invoke } from '@tauri-apps/api/core';
import { formatDistanceToNow } from 'date-fns';
import { ptBR } from 'date-fns/locale';
import { Loader2, Medal, Trophy } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

type Library = 'steam' | 'epic' | 'gog' | 'xbox';

interface AchievementDetail {
  library: Library;
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

const PLATFORM_STYLES: Record<Library, { label: string; className: string }> = {
  steam: { label: 'Steam', className: 'bg-sky-500/10 text-sky-500' },
  epic: { label: 'Epic', className: 'bg-neutral-500/10 text-neutral-400' },
  gog: { label: 'GOG', className: 'bg-purple-500/10 text-purple-400' },
  xbox: { label: 'Xbox', className: 'bg-green-500/10 text-green-500' },
};

export default function Achievements() {
  const { t } = useTranslation('common');
  const [achievements, setAchievements] = useState<AchievementDetail[]>([]);
  const [loading, setLoading] = useState(true);
  const [libraryFilter, setLibraryFilter] = useState<Library | 'all'>('all');

  useEffect(() => {
    invoke<AchievementDetail[]>('get_all_achievements')
      .then(setAchievements)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const filtered = useMemo(
    () =>
      libraryFilter === 'all'
        ? achievements
        : achievements.filter(a => a.library === libraryFilter),
    [achievements, libraryFilter]
  );

  const availableLibraries = useMemo(
    () => Array.from(new Set(achievements.map(a => a.library))),
    [achievements]
  );

  if (loading) {
    return (
      <div className="flex h-full flex-1 items-center justify-center">
        <Loader2 className="text-primary h-10 w-10 animate-spin" />
      </div>
    );
  }

  return (
    <div className="custom-scrollbar bg-background flex-1 overflow-y-auto p-8">
      <div className="mx-auto max-w-5xl space-y-6">
        <div className="flex items-center gap-2">
          <Trophy className="text-yellow-500" size={24} />
          <h1 className="text-2xl font-bold">{t('achievements_title')}</h1>
        </div>

        {availableLibraries.length > 1 && (
          <div className="flex flex-wrap gap-2">
            <FilterChip
              active={libraryFilter === 'all'}
              onClick={() => setLibraryFilter('all')}
              label={t('achievements_filter_all')}
            />
            {availableLibraries.map(lib => (
              <FilterChip
                key={lib}
                active={libraryFilter === lib}
                onClick={() => setLibraryFilter(lib)}
                label={PLATFORM_STYLES[lib].label}
              />
            ))}
          </div>
        )}

        {filtered.length === 0 ? (
          <p className="text-muted-foreground text-sm">
            {t('achievements_empty')}
          </p>
        ) : (
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
            {filtered.map(ach => {
              const library = PLATFORM_STYLES[ach.library];

              return (
                <div
                  key={`${ach.library}-${ach.game_id}-${ach.achievement_name}`}
                  className="bg-card hover:bg-accent/5 flex items-start gap-3 rounded-lg border p-4 transition-colors"
                >
                  {ach.icon_url ? (
                    <img
                      src={ach.icon_url}
                      alt=""
                      className="h-10 w-10 shrink-0 rounded"
                    />
                  ) : (
                    <div className="shrink-0 rounded-full bg-yellow-500/10 p-2 text-yellow-500">
                      <Medal size={18} />
                    </div>
                  )}

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between gap-2">
                      <p className="truncate text-sm font-semibold">
                        {ach.achievement_name}
                      </p>
                      {ach.rarity_percent != null && (
                        <span className="text-muted-foreground shrink-0 text-[10px]">
                          {ach.rarity_percent.toFixed(1)}%
                        </span>
                      )}
                    </div>

                    {ach.description && (
                      <p className="text-muted-foreground mt-0.5 line-clamp-2 text-xs">
                        {ach.description}
                      </p>
                    )}

                    <div className="mt-2 flex items-center justify-between gap-2">
                      <div className="flex items-center gap-1.5 overflow-hidden">
                        <span
                          className={`shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium ${library.className}`}
                        >
                          {library.label}
                        </span>
                        <p className="text-muted-foreground truncate text-xs">
                          {ach.game_name}
                        </p>
                      </div>
                      <span className="text-muted-foreground shrink-0 text-[10px] whitespace-nowrap">
                        {formatDistanceToNow(new Date(ach.unlock_time * 1000), {
                          addSuffix: true,
                          locale: ptBR,
                        })}
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function FilterChip({
  active,
  onClick,
  label,
}: Readonly<{ active: boolean; onClick: () => void; label: string }>) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
        active
          ? 'bg-primary text-primary-foreground border-primary'
          : 'text-muted-foreground hover:bg-accent/10 border-white/10'
      }`}
    >
      {label}
    </button>
  );
}
