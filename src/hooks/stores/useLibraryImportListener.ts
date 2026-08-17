import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';

import type { ImportStatus } from '@/types';
import { toast } from '@/utils/toast';

type ImportCompletePayload = [library: string, message: string];
type ImportErrorPayload = [library: string, error: string];

interface UseLibraryImportListenerOptions {
  libraryLabel: string;
  setStatus: (status: ImportStatus) => void;
  onLibraryUpdate?: () => void;
}

/**
 * Escuta os eventos `import_started` / `import_complete` / `import_error`
 * emitidos pelo backend para importações que rodam em background (spawn),
 * filtrando pelo nome da plataforma.
 */
export function useLibraryImportListener({
  libraryLabel,
  setStatus,
  onLibraryUpdate,
}: UseLibraryImportListenerOptions) {
  const [isImporting, setIsImporting] = useState(false);
  const onLibraryUpdateRef = useRef(onLibraryUpdate);
  onLibraryUpdateRef.current = onLibraryUpdate;

  useEffect(() => {
    const unlistenStarted = listen<string>('import_started', event => {
      if (event.payload !== libraryLabel) return;

      setIsImporting(true);
      setStatus({ type: null, message: '' });
    });

    const unlistenComplete = listen<ImportCompletePayload>(
      'import_complete',
      event => {
        const [library, message] = event.payload;

        if (library !== libraryLabel) return;

        setIsImporting(false);
        setStatus({ type: 'success', message });
        toast.success(message);
        onLibraryUpdateRef.current?.();
      }
    );

    const unlistenError = listen<ImportErrorPayload>('import_error', event => {
      const [library, error] = event.payload;

      if (library !== libraryLabel) return;

      setIsImporting(false);
      setStatus({ type: 'error', message: error });
      toast.error(error);
    });

    return () => {
      unlistenStarted.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [libraryLabel, setStatus]);

  return { isImporting };
}
