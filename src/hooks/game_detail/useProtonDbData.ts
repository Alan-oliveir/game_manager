import { invoke } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { useCallback, useEffect, useRef, useState } from 'react';

import { ProtonDbSummary } from '@/types/game_detail';

type ProtonDbStatus =
  | 'idle'
  | 'loading'
  | 'success'
  | 'no_steam_id'
  | 'not_linux'
  | 'error';

interface UseProtonDbDataReturn {
  data: ProtonDbSummary | null;
  status: ProtonDbStatus;
  retry: () => void;
}

export function useProtonDbData(
  gameId: string,
  steamAppId: string | null | undefined
): UseProtonDbDataReturn {
  const [data, setData] = useState<ProtonDbSummary | null>(null);
  const [status, setStatus] = useState<ProtonDbStatus>('idle');
  const abortRef = useRef(false);

  useEffect(() => {
    abortRef.current = false;

    return () => {
      abortRef.current = true;
    };
  }, [gameId]);

  const load = useCallback(async () => {
    // Checagem extra: só executa se o SO for Linux
    if (type() !== 'linux') {
      setStatus('not_linux');

      return;
    }

    if (!steamAppId) {
      setStatus('no_steam_id');

      return;
    }

    setStatus('loading');

    try {
      const summary = await invoke<ProtonDbSummary | null>(
        'fetch_protondb_data',
        { steamAppId }
      );

      if (abortRef.current) return;

      if (!summary) {
        // Se vier nulo (sem dados suficientes no ProtonDB)
        setStatus('success');
        setData(null);

        return;
      }

      setData(summary);
      setStatus('success');
    } catch {
      if (!abortRef.current) setStatus('error');
    }
  }, [steamAppId]);

  useEffect(() => {
    setData(null);
    setStatus('idle');
  }, [gameId]);

  useEffect(() => {
    if (status === 'idle') load();
  }, [status, load]);

  useEffect(() => {
    if (steamAppId && status === 'no_steam_id') load();
  }, [steamAppId, status, load]);

  return { data, status, retry: () => setStatus('idle') };
}
