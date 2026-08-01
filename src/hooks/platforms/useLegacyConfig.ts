import {
  useLocalStoragePlatformPath,
  usePlatformImportTrigger,
  usePlatformStatus,
} from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos da Legacy Games.
 * Aceita opcionalmente um `appStatePath` manual para o app-state.json,
 * usado quando a detecção automática (inclusive via Wine, no Linux) falha.
 */
export function useLegacyConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();
  const [appStatePath, setAppStatePath] = useLocalStoragePlatformPath(
    'legacy_app_state_path'
  );

  const { isImporting: isImportingLegacy, run: importLegacyGames } =
    usePlatformImportTrigger(
      (appStatePath?: string) =>
        platformsService.importLegacyGames(appStatePath),
      {
        platformLabel: 'LegacyGames',
        setStatus,
        onLibraryUpdate,
      }
    );

  return {
    appStatePath,
    setAppStatePath,
    loading: { importingLegacy: isImportingLegacy },
    status,
    actions: { importLegacyGames },
  };
}
