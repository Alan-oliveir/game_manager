import { useTranslation } from 'react-i18next';

import { formatHours } from '@/utils';

interface HltbBadgeProps {
  mainStory?: number;
  mainExtra?: number;
  completionist?: number;
  coopTime?: number;
}

interface HltbStat {
  key: string;
  labelKey: string;
  labelDefault: string;
  value: number;
}

// Azul usado pela própria HowLongToBeat, pra manter a badge reconhecível.
const HLTB_COLOR_CLASS = 'text-[#66c0f4] border-[#328ed6]/40 bg-[#328ed6]/10';

// grid-cols precisa ser uma classe estática (não interpolada) pro Tailwind
// conseguir gerar o CSS — por isso o lookup em vez de `grid-cols-${n}`.
const GRID_COLS_BY_COUNT: Record<number, string> = {
  1: 'grid-cols-1',
  2: 'grid-cols-2',
  3: 'grid-cols-3',
  4: 'grid-cols-4',
};

export function HltbBadge({
  mainStory,
  mainExtra,
  completionist,
  coopTime,
}: Readonly<HltbBadgeProps>) {
  const { t } = useTranslation('game_detail');

  const stats: HltbStat[] = (
    [
      {
        key: 'main',
        labelKey: 'Main Story',
        value: mainStory,
      },
      {
        key: 'extra',
        labelKey: 'Main + Sides',
        value: mainExtra,
      },
      {
        key: 'completionist',
        labelKey: 'Completionist',
        value: completionist,
      },
      {
        key: 'coop',
        labelKey: 'Co-op',
        value: coopTime,
      },
    ] as {
      key: string;
      labelKey: string;
      labelDefault: string;
      value?: number;
    }[]
  ).filter((s): s is HltbStat => !!s.value && s.value > 0);

  if (stats.length === 0) return null;

  return (
    <div
      className={`flex flex-col gap-2 rounded-lg border px-3 py-2 ${HLTB_COLOR_CLASS}`}
    >
      <div className="flex items-center gap-2">
        <span className="text-base font-bold tracking-wide uppercase">
          {t('sidebar_hltb_heading')}
        </span>
      </div>
      <div
        className={`grid gap-2 font-mono ${GRID_COLS_BY_COUNT[stats.length] ?? 'grid-cols-3'}`}
      >
        {stats.map(stat => (
          <div key={stat.key} className="flex flex-col">
            <span className="text-base font-bold">
              {formatHours(stat.value)}
            </span>
            <span className="text-[10px] leading-tight font-semibold tracking-wide uppercase opacity-80">
              {t(stat.labelKey)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
