import {
  useLibraryImportTrigger,
  useLibraryStatus,
  useLocalStorageLibraryPath,
} from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos da Legacy Games.
 * Aceita opcionalmente um `appStatePath` manual para o app-state.json,
 * usado quando a detecção automática (inclusive via Wine, no Linux) falha.
 */
export function useLegacyConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = useLibraryStatus();
  const [appStatePath, setAppStatePath] = useLocalStorageLibraryPath(
    'legacy_app_state_path'
  );

  const { isImporting: isImportingLegacy, run: importLegacyGames } =
    useLibraryImportTrigger(
      (appStatePath?: string) => storesService.importLegacyGames(appStatePath),
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
