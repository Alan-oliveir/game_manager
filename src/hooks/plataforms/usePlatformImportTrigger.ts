import { useCallback } from 'react';

import { parsePlatformError } from '@/errors/errorMessages';
import { usePlatformImportListener } from '@/hooks';
import { toast } from '@/utils/toast';

import type { ImportStatus } from './types';

interface UsePlatformImportTriggerOptions {
  platformLabel: string;
  setStatus: (status: ImportStatus) => void;
  onLibraryUpdate?: () => void;
}

/**
 * Substitui `usePlatformImportAction` para comandos migrados para `spawn_import`
 * no backend (retornam void imediatamente, resultado chega via evento).
 * Mantém a mesma interface externa (`isImporting`, `run`) para minimizar
 * mudanças nos call sites — só troca a fonte de verdade de Promise para evento.
 */
export function usePlatformImportTrigger<Args extends unknown[] = []>(
  importFn: (...args: Args) => Promise<void>,
  { platformLabel, setStatus, onLibraryUpdate }: UsePlatformImportTriggerOptions
) {
  const { isImporting } = usePlatformImportListener({
    platformLabel,
    setStatus,
    onLibraryUpdate,
  });

  const run = useCallback(
    async (...args: Args) => {
      try {
        await importFn(...args);
        // Sucesso/erro do processo em si chegam via import_complete/import_error,
        // tratados dentro de usePlatformImportListener — não duplicar aqui.
      } catch (e) {
        // Só captura falha síncrona (ex: comando nem chegou a spawnar).
        const errorMsg = parsePlatformError(e);
        setStatus({ type: 'error', message: errorMsg });
        toast.error(errorMsg);
      }
    },

    [importFn, platformLabel]
  );

  return { isImporting, run };
}
