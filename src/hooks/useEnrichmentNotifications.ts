import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

import { toast } from '@/utils/toast';

interface EnrichCompletePayload {
  library: string | null;
  message: string;
}

/**
 * Escuta o evento global `enrich_complete`, emitido tanto pelo enriquecimento automático pós-import
 * (`enrich_newly_imported`, com `library` preenchido) quanto pelas varreduras gerais da biblioteca (`update_metadata`,
 * `fill_missing_metadata`, com `library: null`). Montado uma vez no nível raiz do app.
 */
export function useEnrichmentNotifications() {
  useEffect(() => {
    const unlisten = listen<EnrichCompletePayload>('enrich_complete', event => {
      toast.success(event.payload.message);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);
}
