import { invoke } from '@tauri-apps/api/core';
import { PackageX } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ContentError, ContentLoading, DlcCard } from '@/components';
import { Game, GameDlc } from '@/types';

interface GameDlcsProps {
  game: Game;
}

function DlcsEmpty() {
  const { t } = useTranslation('game_detail');

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
      <PackageX className="text-muted-foreground/50 h-8 w-8" />
      <p className="text-foreground text-sm font-medium">
        {t('addons_dlcs_empty_title')}
      </p>
    </div>
  );
}

export function GameDlcs({ game }: GameDlcsProps) {
  const { t } = useTranslation('game_detail');
  const [dlcs, setDlcs] = useState<GameDlc[]>([]);
  const [status, setStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle');
  const [errorMsg, setErrorMsg] = useState('');

  const load = async () => {
    setStatus('loading');
    setErrorMsg('');

    try {
      const data = await invoke<GameDlc[]>('get_game_dlcs', {
        gameId: game.id,
      });
      setDlcs(data);
      setStatus('success');
    } catch (err) {
      const msg = typeof err === 'string' ? err : t('addons_unknown_error');
      setErrorMsg(msg);
      setStatus('error');
    }
  };

  useEffect(() => {
    setDlcs([]);
    setStatus('idle');
  }, [game.id]);

  useEffect(() => {
    if (status === 'idle') load();
  }, [status]);

  if (status === 'loading')
    return <ContentLoading message={t('addons_dlcs_loading_message')} />;

  if (status === 'error')
    return (
      <ContentError message={errorMsg} onRetry={() => setStatus('idle')} />
    );

  if (status === 'success' && dlcs.length === 0) return <DlcsEmpty />;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-foreground text-sm font-semibold">
          {t('addons_dlcs_header_title')}
        </h3>
        <span className="text-muted-foreground text-xs">
          {dlcs.length} {t('addons_dlcs_count')}
        </span>
      </div>

      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        {dlcs.map(dlc => (
          <DlcCard key={dlc.igdbId} dlc={dlc} />
        ))}
      </div>
    </div>
  );
}
