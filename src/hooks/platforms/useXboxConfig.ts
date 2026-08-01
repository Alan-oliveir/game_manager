import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos instalados via Xbox App / Microsoft Store
 * (Gaming Services). Detecção totalmente automática (via marcador `.GamingRoot` em
 * cada drive) — sem login e sem pasta a configurar.
 */
export function useXboxConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();

  const { isImporting: isImportingXbox, run: importXboxGames } =
    usePlatformImportTrigger(() => platformsService.importXboxGames(), {
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
