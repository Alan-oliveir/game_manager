import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos do Ubisoft Game Launcher.
 * Detecção automática via %LOCALAPPDATA%\Ubisoft Game Launcher.
 */
export function useUbisoftConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();

  const { isImporting: isImportingUbisoft, run: importUbisoftGames } =
    usePlatformImportTrigger(() => platformsService.importUbisoftGames(), {
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
