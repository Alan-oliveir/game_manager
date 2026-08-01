import { useCallback, useState } from 'react';

import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

export type IndiegalaImportMode = 'installed' | 'full';

const MODE_STORAGE_KEY = 'indiegala_import_mode';

function readStoredMode(): IndiegalaImportMode {
  if (typeof localStorage === 'undefined') return 'installed';

  return localStorage.getItem(MODE_STORAGE_KEY) === 'full'
    ? 'full'
    : 'installed';
}

/**
 * Hook para gerenciar a importação de jogos da IndieGala (IGClient).
 * Detecção 100% automática — sem login. O usuário escolhe entre dois modos:
 * `installed` (só o que está instalado agora, via installed.json) e `full`
 * (biblioteca completa de posse via config.json, cruzada com instalados
 * pra marcar o que já está no disco). O modo escolhido é lembrado em
 * localStorage entre sessões, como steamRoot faz em useSteamConfig.
 */
export function useIndiegalaConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();
  const [mode, setModeState] = useState<IndiegalaImportMode>(readStoredMode);

  const setMode = useCallback((next: IndiegalaImportMode) => {
    setModeState(next);

    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(MODE_STORAGE_KEY, next);
    }
  }, []);

  const { isImporting: isImportingIndiegala, run: importIndiegalaGames } =
    usePlatformImportTrigger(
      () => platformsService.importIndiegalaGames(mode === 'full'),
      {
        platformLabel: 'Indiegala',
        setStatus,
        onLibraryUpdate,
      }
    );

  return {
    mode,
    setMode,
    loading: { importingIndiegala: isImportingIndiegala },
    status,
    actions: { importIndiegalaGames },
  };
}
