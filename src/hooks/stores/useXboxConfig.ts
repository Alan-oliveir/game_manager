import { useLibraryImportTrigger, useLibraryStatus } from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos instalados via Xbox App / Microsoft Store
 * (Gaming Services). Detecção totalmente automática (via marcador `.GamingRoot` em
 * cada drive) — sem login e sem pasta a configurar.
 */
export function useXboxConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = useLibraryStatus();

  const { isImporting: isImportingXbox, run: importXboxGames } =
    useLibraryImportTrigger(() => storesService.importXboxGames(), {
      platformLabel: 'Xbox',
      setStatus,
      onLibraryUpdate,
    });

  return {
    loading: {
      importingXbox: isImportingXbox,
    },
    status,
    actions: { importXboxGames },
  };
}
