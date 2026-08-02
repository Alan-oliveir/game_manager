import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { toast } from '@/utils/toast';

interface EnrichProgressPayload {
  current: number;
  total_found: number;
  last_game: string;
  status: string;
  platform: string | null;
}

interface EnrichCompletePayload {
  platform: string | null;
  message: string;
}

/**
 * Toast de progresso por plataforma para o enriquecimento automático pós-import
 * (enrich_newly_imported). Ignora eventos sem `platform` (update_metadata,
 * fill_missing_metadata), que já têm feedback próprio na tela de Configurações.
 */
export function useEnrichmentProgressNotifications() {
  const { t } = useTranslation('settings');
  const toastIds = useRef<Map<string, string | number>>(new Map());

  useEffect(() => {
    const unlistenProgress = listen<EnrichProgressPayload>(
      'enrich_progress',
      event => {
        const { platform, current, total_found, last_game } = event.payload;

        if (!platform) return;

        const message = t('enriching_progress', {
          game: last_game,
          current,
          total: total_found,
        });
        const existingId = toastIds.current.get(platform);

        const id = toast.loading(
          message,
          existingId ? { id: existingId } : undefined
        );
        toastIds.current.set(platform, id);
      }
    );

    const unlistenComplete = listen<EnrichCompletePayload>(
      'enrich_complete',
      event => {
        const { platform, message } = event.payload;

        if (!platform) return;

        const existingId = toastIds.current.get(platform);
        toast.success(message, existingId ? { id: existingId } : undefined);
        toastIds.current.delete(platform);
      }
    );

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
    };
  }, [t]);
}
