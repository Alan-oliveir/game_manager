import { invoke } from '@tauri-apps/api/core';
import { Blocks, Frown } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ContentError, ContentLoading, ModCard } from '@/components';
import { debugError } from '@/lib/debug.ts';
import { TrendingModsResult } from '@/types';
import { Game } from '@/types/game';

interface GameModsProps {
  game: Game;
}

function ModsNoNexusMatch() {
  const { t } = useTranslation('game_detail');

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
      <Frown className="text-muted-foreground/50 h-8 w-8" />
      <p className="text-foreground text-sm font-medium">
        {t('addons_no_nexus_match_title')}
      </p>
      <p className="text-muted-foreground max-w-xs text-xs">
        {t('addons_no_nexus_match_description')}
      </p>
    </div>
  );
}

function ModsEmpty() {
  const { t } = useTranslation('game_detail');

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
      <Blocks className="text-muted-foreground/50 h-8 w-8" />
      <p className="text-foreground text-sm font-medium">
        {t('addons_mods_empty_title')}
      </p>
    </div>
  );
}

export function GameMods({ game }: GameModsProps) {
  const { t } = useTranslation('game_detail');
  const [result, setResult] = useState<TrendingModsResult | null>(null);
  const [status, setStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle');
  const [errorMsg, setErrorMsg] = useState('');
  const visibleMods = result?.mods.slice(0, 6) ?? [];

  const load = async () => {
    setStatus('loading');
    setErrorMsg('');

    try {
      const data = await invoke<TrendingModsResult>('get_trending_mods', {
        gameId: game.id,
      });
      setResult(data);
      setStatus('success');
    } catch (err) {
      debugError('Nexus mods error:', err);
      const msg = typeof err === 'string' ? err : t('addons_unknown_error');
      setErrorMsg(msg);
      setStatus('error');
    }
  };

  useEffect(() => {
    setResult(null);
    setStatus('idle');
  }, [game.id]);

  useEffect(() => {
    if (status === 'idle') load();
  }, [status]);

  if (status === 'loading')
    return <ContentLoading message={t('addons_mods_loading_message')} />;

  if (status === 'error')
    return (
      <ContentError message={errorMsg} onRetry={() => setStatus('idle')} />
    );

  if (status === 'success' && result?.availability === 'NoNexusMatch')
    return <ModsNoNexusMatch />;

  if (status === 'success' && result?.mods.length === 0) return <ModsEmpty />;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-foreground text-sm font-semibold">
          {t('addons_mods_header_title')}
        </h3>
        <span className="text-muted-foreground text-xs">
          {visibleMods.length} {t('addons_mods_count')}
        </span>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {visibleMods.map(mod => (
          <ModCard key={mod.modPageUrl} mod={mod} />
        ))}
      </div>
    </div>
  );
}
