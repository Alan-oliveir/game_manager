import { useTranslation } from 'react-i18next';

import { useProtonDbData } from '@/hooks';
import { Game, GameDetails } from '@/types';

export function ProtonSection({
  game,
  details,
}: {
  game: Game;
  details: GameDetails | null;
}) {
  const { t } = useTranslation('game_detail');
  const { data: protonDbData } = useProtonDbData(game.id, details?.steamAppId);

  // Helper para colorir a badge de ProtonDB de acordo com o tier
  const getTierColors = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'platinum':
        return 'border-blue-300 text-blue-400';
      case 'gold':
        return 'border-yellow-400 text-yellow-500';
      case 'silver':
        return 'border-gray-300 text-gray-300';
      case 'bronze':
        return 'border-orange-600 text-orange-600';
      case 'borked':
        return 'border-red-500 text-red-500';
      default:
        return 'border-border text-muted-foreground';
    }
  };

  return (
    protonDbData && (
      <div>
        <h3 className="text-muted-foreground mb-3 text-sm font-semibold tracking-widest uppercase">
          {t('extras_section_proton')}
        </h3>
        <div className="border-border/50 flex items-center justify-between rounded-lg border px-4 py-3">
          <div className="flex flex-col gap-1">
            <span className="text-foreground text-sm font-medium">
              {t('extras_proton_status')}
            </span>
            <span className="text-muted-foreground text-xs">
              {t('extras_proton_based_on', {
                count: protonDbData.total,
                confidence: protonDbData.confidence,
              })}
            </span>
          </div>
          <div
            className={`flex items-center justify-center rounded border px-3 py-1 text-xs font-semibold tracking-wider uppercase ${getTierColors(protonDbData.tier)}`}
          >
            {protonDbData.tier}
          </div>
        </div>
      </div>
    )
  );
}
