import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Game } from '@/types/game';
import { GameDlcs, GameMods } from '@/windows';

type AddonsSubTab = 'mods' | 'dlcs';

interface GameAddonsProps {
  game: Game;
}

export function GameAddons({ game }: GameAddonsProps) {
  const { t } = useTranslation('game_detail');
  const [subTab, setSubTab] = useState<AddonsSubTab>('mods');

  return (
    <div className="space-y-4">
      <div className="border-border inline-flex gap-1 rounded-lg border p-1">
        <button
          onClick={() => setSubTab('mods')}
          className={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${
            subTab === 'mods'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          {t('addons_subtab_mods')}
        </button>
        <button
          onClick={() => setSubTab('dlcs')}
          className={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${
            subTab === 'dlcs'
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          {t('addons_subtab_dlcs')}
        </button>
      </div>

      {subTab === 'mods' ? <GameMods game={game} /> : <GameDlcs game={game} />}
    </div>
  );
}
