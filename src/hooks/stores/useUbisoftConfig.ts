import { useLibraryImportTrigger, useLibraryStatus } from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos do Ubisoft Game Launcher.
 * Detecção automática via %LOCALAPPDATA%\Ubisoft Game Launcher.
 */
export function useUbisoftConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = useLibraryStatus();

  const { isImporting: isImportingUbisoft, run: importUbisoftGames } =
    useLibraryImportTrigger(() => storesService.importUbisoftGames(), {
      platformLabel: 'Ubisoft',
      setStatus,
      onLibraryUpdate,
    });

  return {
    loading: { importingUbisoft: isImportingUbisoft },
    status,
    actions: { importUbisoftGames },
  };
}
